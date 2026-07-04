# CLI Guide

## Installation

### From Source

```bash
git clone https://github.com/motarjim/motarjim.git
cd motarjim
cargo build --release -p motarjim-cli

# Binary at: ./target/release/motarjim
```

### From crates.io

```bash
cargo install motarjim-cli
```

### Docker

```bash
docker build -t motarjim .
docker run --rm -v $(pwd):/work motarjim compile /work/index.html --platform flutter
```

## Commands

### `motarjim compile`

Compile HTML/CSS to a target platform.

```bash
motarjim compile <INPUT> [OPTIONS]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `INPUT` | Path to input HTML file |

**Options:**

| Option | Short | Description |
|--------|-------|-------------|
| `--platform <PLATFORM>` | `-p` | Target platform: `flutter`, `compose`, `swiftui` (default: `flutter`) |
| `--output <PATH>` | `-o` | Write output to file instead of stdout |
| `--minify` | `-m` | Minify generated code |
| `--source-maps` | | Generate source maps |
| `--strict` | `-s` | Treat warnings as errors |

**Examples:**

```bash
# Basic compilation to stdout
motarjim compile index.html

# Specify target platform
motarjim compile index.html --platform swiftui

# Write to output file
motarjim compile index.html --platform compose --output app/GeneratedView.kt

# Full options
motarjim compile index.html --platform flutter --minify --source-maps --strict --output lib/generated.dart

# Compile with inline CSS
motarjim compile index.html --platform flutter
# (CSS is extracted from <style> tags in the HTML)
```

### `motarjim watch`

Watch files for changes and recompile automatically.

```bash
motarjim watch <INPUT> [OPTIONS]
```

**Note:** Watch mode is a stub and not yet fully implemented. Use `motarjim compile` for one-shot compilation.

### `motarjim init`

Create a default `motarjim.json` configuration file in the current directory.

```bash
motarjim init
```

Creates:

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

### `motarjim check`

Type-check and lint input without generating output.

```bash
motarjim check <INPUT>
```

For HTML/CSS files, runs the full compiler pipeline in strict mode and reports diagnostics. For JavaScript files (`.js`, `.mjs`, `.jsx`), routes to the `motarjim-js` parser and semantic analyzer.

**Examples:**

```bash
motarjim check index.html
motarjim check app.js
motarjim check component.mjs
```

## Configuration File Reference

### `motarjim.json`

```json
{
  "platforms": {
    "<name>": {
      "format": "dart | kotlin | swift",
      "output_dir": "path/to/output",
      "minify": false,
      "source_maps": false,
      "options": {
        "key": "value"
      }
    }
  },
  "global": {
    "verbose": false,
    "strict": false,
    "max_parallel": 4,
    "cache_dir": ".motarjim/cache",
    "incremental": true,
    "options": {
        "key": "value"
    }
  }
}
```

### `motarjim.toml`

```toml
[platforms.flutter]
format = "dart"
output_dir = "output/flutter"
minify = false
source_maps = false

[platforms.compose]
format = "kotlin"
output_dir = "output/compose"
minify = false
source_maps = false

[platforms.swiftui]
format = "swift"
output_dir = "output/swiftui"
minify = false
source_maps = false

[global]
verbose = false
strict = false
max_parallel = 4
incremental = true
```

### Configuration Precedence

```
CLI arguments > Config file (motarjim.json / motarjim.toml) > Defaults
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `MOTARJIM_CACHE_DIR` | Override cache directory | `.motarjim/cache` |
| `MOTARJIM_MAX_PARALLEL` | Max parallel compilation tasks | `4` |
| `MOTARJIM_STRICT` | Enable strict mode | `false` |
| `MOTARJIM_VERBOSE` | Enable verbose output | `false` |
| `RUST_BACKTRACE` | Rust panic backtrace (for debugging) | `0` |
| `RUST_LOG` | Rust log level (for tracing) | `error` |

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | Compilation error (file not found, parse failure, diagnostics) |

## Common Workflows

### Basic HTML to Flutter

```bash
cat > index.html <<EOF
<div class="container">
  <h1>Hello World</h1>
  <p>Built with motarjim</p>
</div>
EOF

motarjim compile index.html --platform flutter --output lib/main.dart
```

### HTML with External CSS

Create an HTML file with a `<style>` tag or pass CSS inline — motarjim extracts CSS from `<style>` tags automatically.

```html
<html>
<style>
  .card { padding: 16px; border-radius: 8px; background: white; }
  h1 { color: #333; font-size: 24px; }
</style>
<body>
  <div class="card">
    <h1>Card Title</h1>
  </div>
</body>
</html>
```

```bash
motarjim compile card.html --platform compose --output CardView.kt
```

### Multi-Platform Output

```bash
# Parse once conceptually, generate per platform
motarjim compile index.html --platform flutter --output lib/generated.dart
motarjim compile index.html --platform compose --output app/generated.kt
motarjim compile index.html --platform swiftui --output GeneratedView.swift
```

### JavaScript Analysis

```bash
cat > app.js <<EOF
const button = document.getElementById("submit");
button.addEventListener("click", () => {
  console.log("Clicked!");
});
const count = 1;
count = 2;  // Error: const reassignment
EOF

motarjim check app.js
# Output:
# error[E0712]: Assignment to a const binding
```

### CI/CD Integration

```yaml
# .github/workflows/compile.yml
name: Compile UI
on: [push]
jobs:
  compile:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Compile Flutter UI
        run: |
          cargo build --release -p motarjim-cli
          ./target/release/motarjim compile index.html --platform flutter --output lib/generated.dart
```

## Troubleshooting

### "motarjim: command not found"

The binary hasn't been added to your PATH. Either:
- Copy it: `cp ./target/release/motarjim ~/.local/bin/`
- Use the full path: `./target/release/motarjim`
- Install via cargo: `cargo install motarjim-cli`

### Compilation produces empty output

- Check that the HTML has content inside `<body>`
- Verify the file is readable
- Run with `--strict` to see diagnostics

### "Unknown platform" error

Use one of: `flutter`, `compose`, `swiftui` (or `dart`, `kotlin`, `swift`).

### "File not found"

Ensure the input file path is correct. motarjim reads from the filesystem directly — it does NOT resolve HTML imports.
