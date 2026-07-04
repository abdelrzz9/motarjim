$ErrorActionPreference = "Stop"
$RootDir = Split-Path -Parent $PSScriptRoot

Write-Host "🧪 Running Motarjim Tests" -ForegroundColor Cyan
Write-Host "=========================" -ForegroundColor Cyan

# Rust tests
Write-Host "`n🦀 Running Rust tests..." -ForegroundColor Yellow
Set-Location $RootDir
cargo test
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Rust tests failed" -ForegroundColor Red
    exit 1
}
Write-Host "✅ Rust tests passed" -ForegroundColor Green

# Clippy
Write-Host "`n🔍 Running Clippy..." -ForegroundColor Yellow
cargo clippy --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Clippy failed" -ForegroundColor Red
    exit 1
}
Write-Host "✅ Clippy passed" -ForegroundColor Green

# Format check
Write-Host "`n📐 Checking formatting..." -ForegroundColor Yellow
cargo fmt --check
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Formatting check failed" -ForegroundColor Red
    exit 1
}
Write-Host "✅ Formatting check passed" -ForegroundColor Green

Write-Host "`n🎉 All tests passed!" -ForegroundColor Green
