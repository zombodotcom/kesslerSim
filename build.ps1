# Build script for Windows (PowerShell)
$ErrorActionPreference = "Stop"

Write-Host "Cleaning previous build artifacts..." -ForegroundColor Cyan
if (Test-Path "pkg") { Remove-Item -Recurse -Force "pkg" -ErrorAction SilentlyContinue }
if (Test-Path "dist") { Remove-Item -Recurse -Force "dist" -ErrorAction SilentlyContinue }
if (Test-Path "target\wasm32-unknown-unknown") { Remove-Item -Recurse -Force "target\wasm32-unknown-unknown" -ErrorAction SilentlyContinue }

Write-Host "Installing wasm-pack..." -ForegroundColor Cyan
if (-not (Get-Command wasm-pack -ErrorAction SilentlyContinue)) {
    Write-Host "wasm-pack not found. Please install it from: https://rustwasm.github.io/wasm-pack/installer/" -ForegroundColor Yellow
    exit 1
}

Write-Host "Building WASM package..." -ForegroundColor Cyan
wasm-pack build --target web --out-dir pkg

Write-Host "Creating dist directory..." -ForegroundColor Cyan
New-Item -ItemType Directory -Force -Path "dist" | Out-Null

Write-Host "Copying HTML and assets..." -ForegroundColor Cyan
Copy-Item "index.html" "dist\"
if (Test-Path "assets") {
    Copy-Item -Recurse "assets" "dist\"
}

Write-Host "Copying the built WASM files..." -ForegroundColor Cyan
Copy-Item -Recurse "pkg" "dist\"

Write-Host "Build complete! Output in dist/" -ForegroundColor Green
