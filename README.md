# SQLite Viewer (Rust + Slint)

Visor de bases de datos SQLite, nativo y ligero, para Linux y Windows. Usa unos 20–40 MB de RAM y no necesita GPU.

## Linux (Debian/Ubuntu)
```bash
chmod +x build.sh
./build.sh --deps        # primera vez: instala dependencias del sistema y compila
./build.sh               # siguientes veces
./build.sh --windows     # también genera el .exe de Windows (MinGW)
./dist/sqlite-viewer [archivo.db]
```
Desarrollo: `cargo run --release -- ruta/a/archivo.db`

## Windows
1. Instala Rust: https://rustup.rs
2. Instala "Visual Studio Build Tools" → "Desarrollo de escritorio con C++".
3. `powershell -ExecutionPolicy Bypass -File build.ps1` (usa `-x86` para equipos de 32 bits).
4. Ejecuta `dist\sqlite-viewer.exe`. El .exe es autónomo: no requiere DLLs ni redistribuibles.

## Uso
- **Abrir archivo SQLite**, o pasar la ruta como argumento ("Abrir con…" / arrastrar sobre el .exe).
- Al hacer clic en una tabla se cargan sus filas de 50 en 50 (Anterior / Siguiente).
- Panel **CONSULTA SQL**: escribe la sentencia y pulsa **▶ Ejecutar**. Las SELECT muestran como máximo 1000 filas.

## Paquete .deb (Debian/Ubuntu/Mint)
```bash
./build.sh --deb                                      # genera dist/sqlite-viewer_<versión>_amd64.deb
sudo apt install ./dist/sqlite-viewer_0.1.0-1_amd64.deb
sudo apt remove sqlite-viewer                         # desinstalar
```
El paquete instala el binario en `/usr/bin`, crea la entrada en el menú de aplicaciones con su icono y registra el programa para la opción "Abrir con…" de los archivos SQLite.
