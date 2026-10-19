// Oculta la consola en Windows (release).
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod db;

use std::cell::RefCell;
use std::rc::Rc;

use slint::{ModelRc, SharedString, VecModel};

slint::include_modules!();

const PAGE_SIZE: i64 = 50;
/// Límite de filas para consultas manuales (protege la RAM en equipos modestos).
const MAX_QUERY_ROWS: usize = 1000;

#[derive(Default)]
struct State {
    db: Option<db::Database>,
    tables: Vec<String>,
    active: Option<usize>,
    page: i64,
    total_rows: i64,
}

fn to_model_strings(v: Vec<String>) -> ModelRc<SharedString> {
    ModelRc::new(VecModel::from(v.into_iter().map(SharedString::from).collect::<Vec<_>>()))
}

fn to_model_rows(rows: Vec<Vec<String>>) -> ModelRc<ModelRc<SharedString>> {
    ModelRc::new(VecModel::from(rows.into_iter().map(to_model_strings).collect::<Vec<_>>()))
}

fn set_status(ui: &AppWindow, msg: impl Into<SharedString>, error: bool) {
    ui.set_status(msg.into());
    ui.set_status_error(error);
}

fn show_result(ui: &AppWindow, res: db::QueryResult) {
    ui.set_columns(to_model_strings(res.columns));
    ui.set_rows(to_model_rows(res.rows));
}

fn refresh_tables(ui: &AppWindow, st: &mut State) {
    st.tables = st.db.as_ref().and_then(|d| d.list_tables().ok()).unwrap_or_default();
    ui.set_tables(to_model_strings(st.tables.clone()));
}

fn load_page(ui: &AppWindow, st: &mut State) {
    let (Some(db), Some(idx)) = (st.db.as_ref(), st.active) else { return };
    let Some(table) = st.tables.get(idx).cloned() else { return };
    let total_pages = ((st.total_rows + PAGE_SIZE - 1) / PAGE_SIZE).max(1);
    let offset = st.page * PAGE_SIZE;
    match db.table_page(&table, PAGE_SIZE, offset) {
        Ok(res) => {
            let n = res.rows.len() as i64;
            show_result(ui, res);
            ui.set_page(st.page as i32);
            ui.set_total_pages(total_pages as i32);
            ui.set_paging_enabled(true);
            ui.set_header_title(table.into());
            ui.set_header_info(format!("{} filas en total", st.total_rows).into());
            let from = if n == 0 { 0 } else { offset + 1 };
            set_status(
                ui,
                format!("Filas {from}–{} · LIMIT {PAGE_SIZE} OFFSET {offset}", offset + n),
                false,
            );
        }
        Err(e) => set_status(ui, format!("Error: {e}"), true),
    }
}

fn open_db(ui: &AppWindow, state: &Rc<RefCell<State>>, path: &std::path::Path) {
    match db::Database::open(path) {
        Ok(d) => {
            let mut st = state.borrow_mut();
            *st = State { db: Some(d), ..Default::default() };
            refresh_tables(ui, &mut st);
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string());
            ui.set_file_name(name.into());
            ui.set_active_table(-1);
            ui.set_paging_enabled(false);
            ui.set_header_title("Selecciona una tabla".into());
            ui.set_header_info("".into());
            show_result(ui, db::QueryResult::default());
            set_status(ui, format!("Abierto: {} ({} tablas)", path.display(), st.tables.len()), false);
        }
        Err(e) => set_status(ui, format!("No se pudo abrir {}: {e}", path.display()), true),
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    let state = Rc::new(RefCell::new(State::default()));

    // --- Abrir archivo (diálogo nativo) ---
    {
        let weak = ui.as_weak();
        let state = state.clone();
        ui.on_open_file(move || {
            let Some(ui) = weak.upgrade() else { return };
            let Some(path) = rfd::FileDialog::new()
                .set_title("Abrir base de datos SQLite")
                .add_filter("SQLite", &["db", "sqlite", "sqlite3", "db3", "s3db"])
                .add_filter("Todos", &["*"])
                .pick_file()
            else {
                return;
            };
            open_db(&ui, &state, &path);
        });
    }

    // --- Abrir archivo pasado por línea de comandos ("Abrir con…", arrastrar al .exe) ---
    if let Some(arg) = std::env::args_os().nth(1) {
        open_db(&ui, &state, std::path::Path::new(&arg));
    }

    // --- Seleccionar tabla ---
    {
        let weak = ui.as_weak();
        let state = state.clone();
        ui.on_select_table(move |idx| {
            let Some(ui) = weak.upgrade() else { return };
            let mut st = state.borrow_mut();
            let idx = idx as usize;
            let Some(table) = st.tables.get(idx).cloned() else { return };
            st.active = Some(idx);
            st.page = 0;
            st.total_rows = st.db.as_ref().and_then(|d| d.count_rows(&table).ok()).unwrap_or(0);
            ui.set_active_table(idx as i32);
            load_page(&ui, &mut st);
        });
    }

    // --- Paginación ---
    {
        let weak = ui.as_weak();
        let state = state.clone();
        ui.on_prev_page(move || {
            let Some(ui) = weak.upgrade() else { return };
            let mut st = state.borrow_mut();
            if st.page > 0 {
                st.page -= 1;
                load_page(&ui, &mut st);
            }
        });
    }
    {
        let weak = ui.as_weak();
        let state = state.clone();
        ui.on_next_page(move || {
            let Some(ui) = weak.upgrade() else { return };
            let mut st = state.borrow_mut();
            if (st.page + 1) * PAGE_SIZE < st.total_rows {
                st.page += 1;
                load_page(&ui, &mut st);
            }
        });
    }

    // --- Consulta SQL manual ---
    {
        let weak = ui.as_weak();
        let state = state.clone();
        ui.on_run_sql(move |sql| {
            let Some(ui) = weak.upgrade() else { return };
            let mut st = state.borrow_mut();
            let Some(db) = st.db.as_ref() else {
                set_status(&ui, "Primero abre una base de datos", true);
                return;
            };
            let start = std::time::Instant::now();
            let result = db.execute(sql.as_str(), MAX_QUERY_ROWS);
            let ms = start.elapsed().as_secs_f64() * 1000.0;
            match result {
                Ok(db::ExecOutcome::Rows(res, truncated)) => {
                    let n = res.rows.len();
                    show_result(&ui, res);
                    st.active = None;
                    ui.set_paging_enabled(false);
                    ui.set_active_table(-1);
                    ui.set_header_title("Resultado de consulta".into());
                    let extra = if truncated { format!(" (limitado a {MAX_QUERY_ROWS})") } else { String::new() };
                    ui.set_header_info(format!("{n} filas{extra}").into());
                    set_status(&ui, format!("OK · {n} filas · {ms:.1} ms"), false);
                }
                Ok(db::ExecOutcome::Affected(n)) => {
                    refresh_tables(&ui, &mut st);
                    set_status(&ui, format!("OK · {n} filas afectadas · {ms:.1} ms"), false);
                }
                Err(e) => set_status(&ui, format!("Error SQL: {e}"), true),
            }
        });
    }

    ui.run()
}
