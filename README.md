# motarjim

<p align="center">
  <img src="motarjim.png" alt="motarjim logo" width="200">
</p>

<p align="center">
  <strong>HTML/CSS → Native UI Code compiler</strong><br>
  Write once in HTML/CSS. Ship native code for every platform.
</p>

<p align="center">
  <a href="#"><img src="https://img.shields.io/badge/version-0.1.0-blue" alt="version"></a>
  <a href="#"><img src="https://img.shields.io/badge/license-MIT-green" alt="license"></a>
  <a href="#"><img src="https://img.shields.io/badge/build-passing-brightgreen" alt="build"></a>
  <a href="#"><img src="https://img.shields.io/badge/tests-493%20passed-brightgreen" alt="tests"></a>
  <a href="#"><img src="https://img.shields.io/badge/coverage-87%25-yellow" alt="coverage"></a>
  <a href="#"><img src="https://img.shields.io/badge/rustc-1.75%2B-orange" alt="rustc"></a>
</p>

---

## Quick Start

```bash
git clone https://github.com/motarjim/motarjim.git
cd motarjim
cargo build --release -p motarjim-cli
./target/release/motarjim compile index.html --platform flutter
```

## Key Features

- **Local-first** — Zero cloud dependencies. Everything runs on your machine.
- **Multi-platform** — Generate Flutter (Dart), Jetpack Compose (Kotlin), or SwiftUI from the same HTML/CSS.
- **Rust engine** — The Rust workspace under `crates/` is the single source of truth: parse → analyze → optimize → generate, no runtime, no WebView.
- **JavaScript front end** — `motarjim-js` parses variables, functions, arrow functions, template literals, imports/exports, and extracts DOM event bindings.
- **493+ tests** across the Rust workspace, plus fuzz targets and Criterion benchmarks per parser.
- **Diagnostics with error codes** — Rust-style `E0001`-`E0799` diagnostics with severities, spans, and notes.
- **Plugin system** — Extensible generator architecture for third-party platform targets.
- **LSP support** — Language server protocol implementation for IDE integration.
- **Incremental compilation** — Query-based caching and dependency tracking for fast rebuilds.

## Architecture

<p align="center">
  <img src="architecture_mono.png" alt="motarjim Compiler Architecture" width="100%">
</p>

See [ARCHITECTURE.md](ARCHITECTURE.md) for the full architecture document.

### Pipeline Stages

| # | Stage | Description |
|---|-------|-------------|
| 1 | **Parse** | Tokenize and parse HTML/CSS into typed ASTs |
| 2 | **Style** | Resolve CSS cascade, match selectors, compute styles |
| 3 | **Analyze** | Semantic inference, layout detection, accessibility analysis |
| 4 | **IR** | Build platform-agnostic `IrNode` tree (SemanticIR / LayoutIR / TargetIR) |
| 5 | **Optimize** | Merge text nodes, flatten containers, prune unused props, deduplicate styles |
| 6 | **Generate** | Walk IR tree and emit Flutter / Compose / SwiftUI code |

### Supported Targets

| Platform | Language | Widget Set |
|----------|----------|------------|
| Flutter | Dart | Material Design |
| Jetpack Compose | Kotlin | Material 3 |
| SwiftUI | Swift | iOS 17+ |

## Installation

### From Source

```bash
# Prerequisites: Rust 1.75+, Cargo
git clone https://github.com/motarjim/motarjim.git
cd motarjim
cargo build --release -p motarjim-cli

# The binary is at ./target/release/motarjim
# Optionally add to PATH:
cp ./target/release/motarjim ~/.local/bin/
```

### From crates.io (when published)

```bash
cargo install motarjim-cli
```

### Docker

```bash
docker build -t motarjim .
docker run --rm -v $(pwd):/work motarjim compile /work/index.html --platform flutter
```

## Usage Examples

### Basic Compilation

```bash
# Compile to Flutter
motarjim compile index.html --platform flutter

# Compile to Jetpack Compose
motarjim compile index.html --platform compose

# Compile to SwiftUI
motarjim compile index.html --platform swiftui
```

### With Output File

```bash
motarjim compile index.html --platform swiftui --output ContentView.swift
```

### With Options

```bash
motarjim compile index.html --platform compose --minify --source-maps --strict
```

### Initialize a Project

```bash
motarjim init
```

### Check for Diagnostics

```bash
motarjim check index.html
motarjim check app.js    # JavaScript analysis with motarjim-js
```

### Input

```html
<nav class="navbar">
  <h1>My App</h1>
</nav>
<section class="hero">
  <h1>Welcome</h1>
  <p>Build something great</p>
  <button>Get Started</button>
</section>
```

```css
.navbar { background: #333; color: white; padding: 1rem; }
.hero { text-align: center; padding: 4rem; background: #1a1a2e; color: white; }
button { background: blue; color: white; border-radius: 8px; padding: 12px 24px; }
```

### Generated Flutter

```dart
import 'package:flutter/material.dart';

class GeneratedView extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        AppBar(title: Text("My App")),
        Column(
          children: [
            Text("Welcome"),
            Text("Build something great"),
            ElevatedButton(
              onPressed: () {},
              child: Text("Get Started"),
            ),
          ],
        ),
      ],
    );
  }
}
```

### Generated Compose

```kotlin
import androidx.compose.material3.*
import androidx.compose.runtime.*

@Composable
fun GeneratedView() {
    Column {
        TopAppBar(title = { Text("My App") })
        Column {
            Text(text = "Welcome")
            Text(text = "Build something great")
            Button(onClick = { }) {
                Text(text = "Get Started")
            }
        }
    }
}
```

### Generated SwiftUI

```swift
import SwiftUI

struct GeneratedView: View {
    var body: some View {
        VStack {
            Text("My App")
                .navigationTitle("My App")
            VStack {
                Text("Welcome")
                Text("Build something great")
                Button("Get Started") {
                    // action
                }
            }
        }
    }
}
```

## Configuration

motarjim supports `motarjim.json` and `motarjim.toml` configuration files:

```json
{
  "platforms": {
    "flutter": {
      "format": "dart",
      "output_dir": "output/flutter",
      "minify": false,
      "source_maps": false
    },
    "compose": {
      "format": "kotlin",
      "output_dir": "output/compose",
      "minify": false,
      "source_maps": false
    },
    "swiftui": {
      "format": "swift",
      "output_dir": "output/swiftui",
      "minify": false,
      "source_maps": false
    }
  },
  "global": {
    "verbose": false,
    "strict": false,
    "max_parallel": 4,
    "incremental": true
  }
}
```

```toml
[platforms.flutter]
format = "dart"
output_dir = "output/flutter"
minify = false

[global]
verbose = false
strict = false
incremental = true
```

## Web Playground

motarjim ships with a Vite playground at `apps/playground`:

```bash
npm install
npm run start:playground
```

Open [http://localhost:3000](http://localhost:3000) for:
- Split-panel editor with HTML/CSS input and generated code output
- Pipeline visualizer with stage-by-stage progress
- Platform switcher (Flutter, Compose, SwiftUI)
- Dark/light theme support
- Auto-save drafts across sessions
- Error cards with detailed diagnostic information

## Development

### Prerequisites

- Rust 1.75+
- Node.js 18+
- npm 9+

### Build

```bash
# Rust workspace
cargo build --workspace

# TypeScript packages
npm install
npm run build
```

### Test

```bash
# Rust tests
cargo test --workspace

# TypeScript tests
npm test

# All tests with release profile
cargo test --release
```

### Lint

```bash
cargo clippy --workspace -- -D warnings
cargo fmt --check
npm run lint
```

### Benchmark

```bash
cargo bench --workspace
npm run bench
```

## Project Structure

```
motarjim/
├── crates/                 # Rust workspace (compiler engine)
│   ├── motarjim-core       # Compiler facade & pipeline orchestrator
│   ├── motarjim-parser     # HTML/CSS parser
│   ├── motarjim-lexer      # HTML/CSS tokenizer
│   ├── motarjim-css        # CSS engine (cascade, selectors, values)
│   ├── motarjim-selectors  # CSS selector engine
│   ├── motarjim-ir         # IR construction & inference
│   ├── motarjim-optimizer  # Optimization passes
│   ├── motarjim-js         # JavaScript front end
│   ├── motarjim-gen-flutter # Flutter code generator
│   ├── motarjim-gen-compose # Compose code generator
│   ├── motarjim-gen-swiftui # SwiftUI code generator
│   ├── motarjim-cli        # CLI application
│   ├── motarjim-lsp        # Language server
│   ├── motarjim-wasm       # WebAssembly bindings
│   ├── motarjim-diag       # Diagnostic system
│   ├── motarjim-ast        # AST type definitions
│   ├── motarjim-cache      # Compilation cache
│   ├── motarjim-config     # Configuration
│   ├── motarjim-fs         # Filesystem abstraction
│   ├── motarjim-formatter  # Code formatter
│   ├── motarjim-incremental # Incremental compilation
│   ├── motarjim-profiling  # Performance profiling
│   ├── motarjim-serialize  # Serialization
│   └── motarjim-ffi        # FFI bridge
├── packages/               # TypeScript packages
│   ├── vscode-extension    # VS Code extension
│   ├── playground-sdk      # Shared playground types
│   └── website-sdk         # Website metadata
├── apps/                   # TypeScript applications
│   ├── playground          # Web playground (Vite)
│   └── website             # Documentation website
├── fuzz/                   # Fuzz targets
├── docs/                   # Documentation
├── examples/               # Example inputs
├── scripts/                # Build scripts
└── xtask/                  # Cargo build tasks
```

## Documentation

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | Compiler architecture and design decisions |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Development setup and contribution guide |
| [docs/CLI_GUIDE.md](docs/CLI_GUIDE.md) | CLI commands, options, and configuration |
| [docs/WEB_GUIDE.md](docs/WEB_GUIDE.md) | Web playground and website development |
| [docs/WASM_GUIDE.md](docs/WASM_GUIDE.md) | WebAssembly bindings and browser usage |
| [docs/EXTENSION_GUIDE.md](docs/EXTENSION_GUIDE.md) | VS Code extension |
| [docs/TESTING_GUIDE.md](docs/TESTING_GUIDE.md) | Testing philosophy and practices |
| [docs/PLUGIN_GUIDE.md](docs/PLUGIN_GUIDE.md) | Plugin development for custom generators |
| [docs/STYLE_GUIDE.md](docs/STYLE_GUIDE.md) | Code style and conventions |
| [docs/RELEASE_GUIDE.md](docs/RELEASE_GUIDE.md) | Release process and publishing |
| [ROADMAP.md](ROADMAP.md) | Project roadmap and future plans |
| [SECURITY.md](SECURITY.md) | Security policy and vulnerability reporting |
| [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) | Community guidelines |

## Performance

| Metric | Value |
|--------|-------|
| Pipeline (1000 nodes) | **98ms** median |
| Target | 500ms |
| Headroom | **5.1×** |
| Generators (all 3) | +13ms |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, coding standards, and pull request process. All contributions are welcome!

## License

MIT
