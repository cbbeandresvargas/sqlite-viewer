//! Capa de acceso a SQLite: independiente de la UI.

use std::path::Path;

use rusqlite::{types::ValueRef, Connection, OpenFlags};

#[derive(Default)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub enum ExecOutcome {
    /// Resultado con filas; `bool` indica si se truncó por el límite.
    Rows(QueryResult, bool),
    /// Sentencia sin resultados (INSERT/UPDATE/DDL...): filas afectadas.
    Affected(usize),
}

pub struct Database {
    conn: Connection,
}

/// Escapa un identificador SQL entre comillas dobles.
fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

fn value_to_string(v: ValueRef<'_>) -> String {
    const MAX_CELL: usize = 200; // evita cargar textos/blobs enormes en la UI
    match v {
        ValueRef::Null => "NULL".into(),
        ValueRef::Integer(i) => i.to_string(),
        ValueRef::Real(f) => f.to_string(),
        ValueRef::Text(t) => {
            let s = String::from_utf8_lossy(t);
            if s.chars().count() > MAX_CELL {
                let mut cut: String = s.chars().take(MAX_CELL).collect();
                cut.push('…');
                cut
            } else {
                s.replace('\n', " ")
            }
        }
        ValueRef::Blob(b) => format!("<BLOB {} bytes>", b.len()),
    }
}

impl Database {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        // Valida que realmente sea una base SQLite.
        conn.query_row("SELECT count(*) FROM sqlite_master", [], |r| r.get::<_, i64>(0))?;
        // Caché pequeña para equipos con poca RAM (~2 MB).
        let _ = conn.execute_batch("PRAGMA cache_size = -2000;");
        Ok(Self { conn })
    }

    pub fn list_tables(&self) -> rusqlite::Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT name FROM sqlite_master WHERE type IN ('table','view') \
             AND name NOT LIKE 'sqlite_%' ORDER BY type, name",
        )?;
        let names = stmt.query_map([], |r| r.get::<_, String>(0))?.collect();
        names
    }

    pub fn count_rows(&self, table: &str) -> rusqlite::Result<i64> {
        self.conn.query_row(&format!("SELECT count(*) FROM {}", quote_ident(table)), [], |r| r.get(0))
    }

    pub fn table_page(&self, table: &str, limit: i64, offset: i64) -> rusqlite::Result<QueryResult> {
        let sql = format!("SELECT * FROM {} LIMIT ?1 OFFSET ?2", quote_ident(table));
        let mut stmt = self.conn.prepare(&sql)?;
        let (res, _) = collect(&mut stmt, rusqlite::params![limit, offset], usize::MAX)?;
        Ok(res)
    }

    pub fn execute(&self, sql: &str, max_rows: usize) -> rusqlite::Result<ExecOutcome> {
        let sql = sql.trim();
        let mut stmt = self.conn.prepare(sql)?;
        if stmt.column_count() > 0 {
            let (res, truncated) = collect(&mut stmt, [], max_rows)?;
            Ok(ExecOutcome::Rows(res, truncated))
        } else {
            drop(stmt);
            // execute_batch permite varias sentencias separadas por ';'
            let before = self.conn.total_changes();
            self.conn.execute_batch(sql)?;
            Ok(ExecOutcome::Affected((self.conn.total_changes() - before) as usize))
        }
    }
}

fn collect<P: rusqlite::Params>(
    stmt: &mut rusqlite::Statement<'_>,
    params: P,
    max_rows: usize,
) -> rusqlite::Result<(QueryResult, bool)> {
    let columns: Vec<String> = stmt.column_names().into_iter().map(String::from).collect();
    let n = columns.len();
    let mut rows_out = Vec::new();
    let mut truncated = false;
    let mut rows = stmt.query(params)?;
    while let Some(row) = rows.next()? {
        if rows_out.len() >= max_rows {
            truncated = true;
            break;
        }
        let mut r = Vec::with_capacity(n);
        for i in 0..n {
            r.push(value_to_string(row.get_ref(i)?));
        }
        rows_out.push(r);
    }
    Ok((QueryResult { columns, rows: rows_out }, truncated))
}
