#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

echo "🚀 Motarjim Development Environment"
echo "=================================="

check_command() {
    if ! command -v "$1" &> /dev/null; then
        echo "❌ Missing prerequisite: $1"
        exit 1
    fi
    echo "✅ Found $1"
}

check_command cargo
check_command node
check_command wasm-pack

echo ""
echo "📦 Building WASM..."
cd "$ROOT_DIR/crates/motarjim-wasm"
wasm-pack build --target web --out-dir "$ROOT_DIR/apps/web/public/wasm"

echo ""
echo "🦀 Building Rust crates..."
cd "$ROOT_DIR"
cargo build

echo ""
echo "📦 Installing web dependencies..."
cd "$ROOT_DIR/apps/web"
npm install

echo ""
echo "🌐 Starting web dev server..."
echo "   Running at http://localhost:5173"
npm run dev
