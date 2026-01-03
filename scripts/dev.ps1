# PowerShell script for faster development on Windows
# Usage: .\scripts\dev.ps1

param(
    [string]$Command = "check"
)

switch ($Command) {
    "check" {
        Write-Host "Running cargo check..." -ForegroundColor Green
        cargo check
    }
    "test" {
        Write-Host "Running tests..." -ForegroundColor Green
        cargo test --lib
    }
    "run" {
        Write-Host "Running app..." -ForegroundColor Green
        cargo run --release
    }
    "watch" {
        Write-Host "Watching for changes..." -ForegroundColor Green
        if (Get-Command cargo-watch -ErrorAction SilentlyContinue) {
            cargo watch -x check
        } else {
            Write-Host "cargo-watch not installed. Install with: cargo install cargo-watch" -ForegroundColor Yellow
            Write-Host "Falling back to manual check..." -ForegroundColor Yellow
            while ($true) {
                cargo check
                Start-Sleep -Seconds 2
            }
        }
    }
    "clean" {
        Write-Host "Cleaning..." -ForegroundColor Green
        cargo clean
    }
    default {
        Write-Host "Usage: .\scripts\dev.ps1 [check|test|run|watch|clean]" -ForegroundColor Yellow
        Write-Host "  check  - Quick compile check (fastest)" -ForegroundColor Cyan
        Write-Host "  test   - Run tests" -ForegroundColor Cyan
        Write-Host "  run    - Run the app" -ForegroundColor Cyan
        Write-Host "  watch  - Watch for changes and check" -ForegroundColor Cyan
        Write-Host "  clean  - Clean build artifacts" -ForegroundColor Cyan
    }
}

