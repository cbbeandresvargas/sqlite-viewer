# Compila SQLite Viewer en Windows (PowerShell).
# Requisitos: Rust (https://rustup.rs) + "Visual Studio Build Tools" con
# la carga de trabajo "Desarrollo de escritorio con C++".
# Uso:  powershell -ExecutionPolicy Bypass -File build.ps1 [-x86]
param([switch]$x86)
$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

$target = if ($x86) { "i686-pc-windows-msvc" } else { "x86_64-pc-windows-msvc" }
rustup target add $target
cargo build --release --target $target
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

New-Item -ItemType Directory -Force -Path dist | Out-Null
Copy-Item "target\$target\release\sqlite-viewer.exe" dist\ -Force
Write-Host "OK -> dist\sqlite-viewer.exe" -ForegroundColor Green
