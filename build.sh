#!/usr/bin/env bash
# Compila SQLite Viewer en Linux (y opcionalmente Windows vía MinGW).
# Uso:
#   ./build.sh            -> binario Linux en dist/
#   ./build.sh --deps     -> instala dependencias del sistema (Debian/Ubuntu) y compila
#   ./build.sh --windows  -> además genera dist/sqlite-viewer.exe (requiere mingw-w64)
#   ./build.sh --deb      -> además genera el paquete dist/sqlite-viewer_*.deb
set -euo pipefail
cd "$(dirname "$0")"

if [[ " $* " == *" --deps "* ]]; then
  sudo apt-get update
  sudo apt-get install -y build-essential pkg-config libfontconfig1-dev \
    libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev libx11-dev \
    xdg-desktop-portal xdg-desktop-portal-gtk
fi

mkdir -p dist
cargo build --release
cp target/release/sqlite-viewer dist/
echo "✔ Linux:   dist/sqlite-viewer ($(du -h dist/sqlite-viewer | cut -f1))"

if [[ " $* " == *" --windows "* ]]; then
  if ! command -v x86_64-w64-mingw32-gcc >/dev/null; then
    echo "Instalando mingw-w64..."; sudo apt-get install -y mingw-w64
  fi
  rustup target add x86_64-pc-windows-gnu
  cargo build --release --target x86_64-pc-windows-gnu
  cp target/x86_64-pc-windows-gnu/release/sqlite-viewer.exe dist/
  echo "✔ Windows: dist/sqlite-viewer.exe"
fi

if [[ " $* " == *" --deb "* ]]; then
  command -v cargo-deb >/dev/null || cargo install cargo-deb --locked
  cargo deb --no-build --output dist/
  echo "✔ Paquete: $(ls dist/*.deb)"
  echo "  Instalar con: sudo apt install ./$(ls dist/*.deb | tail -1)"
fi
