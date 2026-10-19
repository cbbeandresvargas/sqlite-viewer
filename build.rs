fn main() {
    // Estilo oscuro fijo para los widgets estándar (ScrollView, TextEdit) en Linux y Windows.
    let config = slint_build::CompilerConfiguration::new().with_style("fluent-dark".into());
    slint_build::compile_with_config("ui/appwindow.slint", config)
        .expect("Error compilando la UI de Slint");
}
