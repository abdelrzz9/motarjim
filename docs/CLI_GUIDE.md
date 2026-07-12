# CLI Guide

## Installation

### From Source

```bash
git clone https://github.com/your-org/motarjim
cd motarjim
cargo build --release

# Binary at: ./target/release/motarjim
```

### From crates.io

```bash
cargo install motarjim
```

## Commands

### `motarjim build`

Build the site from source content.

```bash
motarjim build [OPTIONS]
```

Builds all content files in the input directory, applies templates, resolves CSS styles, and generates output files.

**Options:**

| Option | Short | Description |
|--------|-------|-------------|
| `--config <PATH>` | `-c` | Path to config file (default: `motarjim.toml`) |
| `--output <DIR>` | `-o` | Output directory (overrides config) |
| `--input <DIR>` | `-i` | Input content directory (overrides config) |
| `--strict` | `-s` | Treat warnings as errors |
| `--viewport-width <PX>` | | Viewport width for media query evaluation (default: 1024) |
| `--viewport-height <PX>` | | Viewport height for media query evaluation (default: 768) |
| `--color-scheme <MODE>` | | Preferred color scheme: `light` or `dark` (default: `light`) |

**Examples:**

```bash
# Build with defaults
motarjim build

# Build with custom config
motarjim build --config my-site.toml

# Build with dark mode media queries
motarjim build --color-scheme dark

# Build for mobile viewport
motarjim build --viewport-width 375 --viewport-height 812
```

### `motarjim serve`

Start the development server with live reload.

```bash
motarjim serve [OPTIONS]
```

**Options:**

| Option | Short | Description |
|--------|-------|-------------|
| `--port <PORT>` | `-p` | Port to listen on (default: 8080) |
| `--host <HOST>` | | Host to bind to (default: `127.0.0.1`) |
| `--no-livereload` | | Disable live reload injection |
| `--open` | `-o` | Open browser on start |

**Examples:**

```bash
# Start dev server
motarjim serve

# Custom port
motarjim serve --port 3000

# Accessible from network
motarjim serve --host 0.0.0.0 --port 80
```

### `motarjim init`

Create a default `motarjim.toml` configuration file in the current directory.

```bash
motarjim init
```

Creates:

```toml
[site]
title = "My Site"
base_url = "https://example.com"
author = "Your Name"

[build]
output_dir = "public"
input_dir = "content"

[build.viewport]
width = 1024
height = 768
prefers_color_scheme = "light"

[feeds.rss]
enabled = true
title = "My Site RSS Feed"

[feeds.atom]
enabled = true
```

### `motarjim check`

Validate content and CSS without generating output.

```bash
motarjim check <INPUT> [OPTIONS]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `INPUT` | Path to input file or directory |

Runs the full pipeline in strict mode and reports diagnostics (parse errors, CSS cascade issues, missing templates).

**Examples:**

```bash
# Check a single file
motarjim check content/index.md

# Check all content
motarjim check content/
```

## Configuration File Reference

### `motarjim.toml`

```toml
[site]
title = "My Site"
base_url = "https://example.com"
author = "Your Name"
language = "en"

[build]
output_dir = "public"
input_dir = "content"

[build.viewport]
width = 1024
height = 768
prefers_color_scheme = "light"

[build.formats]
html = true
rss = true
atom = true
json = false
plaintext = false

[feeds.rss]
enabled = true
title = "My Site RSS Feed"
description = "Latest content"
count = 20

[feeds.atom]
enabled = true
title = "My Site Atom Feed"
description = "Latest content"
count = 20

[server]
port = 8080
host = "127.0.0.1"
live_reload = true
```

### Configuration Precedence

```
CLI arguments > Config file (motarjim.toml) > Defaults
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `MOTARJIM_CONFIG` | Path to config file | `motarjim.toml` |
| `MOTARJIM_STRICT` | Enable strict mode | `false` |

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | Error (config, parse, or build failure) |

## Common Workflows

### Create and Build a New Site

```bash
# Initialize
motarjim init my-site
cd my-site

# Add content
cat > content/index.md <<EOF
---
title: Home
---
# Welcome

This is my site built with **Motarjim**.
EOF

# Build
motarjim build

# Serve
motarjim serve --open
```

### Using CSS Variables and Media Queries

```html
<style>
  :root {
    --primary-color: #333;
  }
  body {
    color: var(--primary-color);
    font-size: calc(16px * 1.2);
  }
  @media (min-width: 768px) {
    .sidebar { display: block; }
  }
</style>
```

Build with specific viewport:

```bash
motarjim build --viewport-width 768 --color-scheme dark
```

## Troubleshooting

### "motarjim: command not found"

The binary isn't in your PATH. Either:
- Install via cargo: `cargo install motarjim`
- Use the full path: `./target/release/motarjim`

### Build produces no output

- Check that `input_dir` contains `.md` or `.html` files
- Verify the config file is valid TOML
- Run with `--strict` to see diagnostics

### Media queries not working as expected

- Set viewport dimensions explicitly with `--viewport-width` and `--viewport-height`
- The default viewport is 1024x768 (desktop)
- Use `--color-scheme dark` to test dark mode styles
