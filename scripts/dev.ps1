param(
    [switch]$SkipBuild,
    [switch]$SkipWasm
)

$ErrorActionPreference = "Stop"
$RootDir = Split-Path -Parent $PSScriptRoot

Write-Host "🚀 Motarjim Development Environment" -ForegroundColor Cyan
Write-Host "==================================" -ForegroundColor Cyan

# Check prerequisites
function Check-Command($cmd) {
    if (!(Get-Command $cmd -ErrorAction SilentlyContinue)) {
        Write-Host "❌ Missing prerequisite: $cmd" -ForegroundColor Red
        exit 1
    }
    Write-Host "✅ Found $cmd" -ForegroundColor Green
}

Check-Command "cargo"
Check-Command "node"
Check-Command "wasm-pack"

# Build WASM
if (-not $SkipWasm) {
    Write-Host "`n📦 Building WASM..." -ForegroundColor Yellow
    Set-Location "$RootDir/crates/motarjim-wasm"
    wasm-pack build --target web --out-dir "$RootDir/apps/web/public/wasm"
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ WASM build failed" -ForegroundColor Red
        exit 1
    }
    Write-Host "✅ WASM built successfully" -ForegroundColor Green
}

# Build Rust
if (-not $SkipBuild) {
    Write-Host "`n🦀 Building Rust crates..." -ForegroundColor Yellow
    Set-Location $RootDir
    cargo build
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Rust build failed" -ForegroundColor Red
        exit 1
    }
    Write-Host "✅ Rust crates built successfully" -ForegroundColor Green
}

# Install web dependencies
Write-Host "`n📦 Installing web dependencies..." -ForegroundColor Yellow
Set-Location "$RootDir/apps/web"
npm install
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ npm install failed" -ForegroundColor Red
    exit 1
}
Write-Host "✅ Web dependencies installed" -ForegroundColor Green

# Start web dev server
Write-Host "`n🌐 Starting web dev server..." -ForegroundColor Yellow
Write-Host "   Running at http://localhost:5173" -ForegroundColor Cyan
npm run dev
