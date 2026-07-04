# WASM Guide

## Architecture

The `motarjim-wasm` crate provides WebAssembly bindings for the motarjim compiler, enabling it to run directly in web browsers. The same `motarjim-core` compiler engine powers all targets — CLI, LSP, WASM — with no duplicated logic.

```
┌─────────────────────────────────────────┐
│            Browser (JavaScript)         │
│                                         │
│  ┌───────────────────────────────────┐  │
│  │       WasmCompiler (JS class)     │  │
│  └──────────────┬────────────────────┘  │
│                 │ call                    │
│                 ▼                        │
│  ┌───────────────────────────────────┐  │
│  │       WASM Module (.wasm)         │  │
│  │  ┌─────────────────────────────┐  │  │
│  │  │   wasm-bindgen glue        │  │  │
│  │  └──────────┬──────────────────┘  │  │
│  │             │                      │  │
│  │             ▼                      │  │
│  │  ┌─────────────────────────────┐  │  │
│  │  │   motarjim-core (in WASM)  │  │  │
│  │  │   - Parse HTML/CSS         │  │  │
│  │  │   - Resolve styles         │  │  │
│  │  │   - Build IR               │  │  │
│  │  │   - Optimize               │  │  │
│  │  │   - Generate platform code │  │  │
│  │  └─────────────────────────────┘  │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

### Crate Structure

**Crate:** `crates/motarjim-wasm`
**Dependencies:** `motarjim-core`, `motarjim-config`, `wasm-bindgen`, `js-sys`, `serde_json`

The WASM crate is a thin wrapper that:
1. Creates a `Compiler` instance with appropriate configuration
2. Exposes a `compile()` method accepting HTML/CSS strings
3. Returns results as JSON strings (passing complex types across the WASM boundary)
4. Handles error serialization automatically

## JavaScript API Reference

### `WasmCompiler`

The main compiler class, exported as an ES module.

```typescript
class WasmCompiler {
  constructor();

  compile(html: string, css?: string | null, platform?: string): string;

  static version(): string;
}
```

#### `constructor()`

Creates a new compiler instance with default configuration.

```typescript
const compiler = new WasmCompiler();
```

#### `compile(html, css?, platform?)`

Compiles HTML/CSS to the target platform.

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `html` | `string` | — | HTML source code |
| `css` | `string` \| `null` | `null` | CSS source code (optional; can also be in `<style>` tags) |
| `platform` | `string` | `"flutter"` | Target platform: `"flutter"`, `"compose"`, `"swiftui"` |

**Returns:** JSON string — parse with `JSON.parse()` to get the response object.

**Success response:**

```json
{
  "success": true,
  "code": "import 'package:flutter/material.dart';\n\nclass GeneratedView extends StatelessWidget {\n  ...\n}",
  "stats": {
    "nodes": 42,
    "css_rules": 8,
    "time_ms": 12
  }
}
```

**Error response:**

```json
{
  "success": false,
  "errors": [
    "error[E0001]: Unexpected token at line 3:14"
  ]
}
```

**Examples:**

```javascript
const compiler = new WasmCompiler();

// Basic usage
const result = JSON.parse(compiler.compile(
  '<div>Hello World</div>',
  'div { color: blue; }',
  'flutter'
));
console.log(result.code);

// Compile to Compose
const composeResult = JSON.parse(compiler.compile(
  '<button>Click</button>',
  'button { padding: 12px; }',
  'compose'
));

// Compile to SwiftUI
const swiftResult = JSON.parse(compiler.compile(
  '<h1>Title</h1>',
  null,
  'swiftui'
));

// Error handling
const errorResult = JSON.parse(compiler.compile(
  '<div><p>Missing closing tag',
  null,
  'flutter'
));
if (errorResult.success) {
  // Use result.code
} else {
  console.error('Compilation failed:', errorResult.errors);
}
```

#### `static version()`

Returns the compiler version string.

```javascript
console.log(WasmCompiler.version()); // "0.1.0"
```

### TypeScript Types

```typescript
interface CompileSuccess {
  success: true;
  code: string;
  stats: {
    nodes: number;
    css_rules: number;
    time_ms: number;
  };
}

interface CompileError {
  success: false;
  errors: string[];
}

type CompileResult = CompileSuccess | CompileError;
```

## Building from Source

### Prerequisites

- Rust 1.75+
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/): `cargo install wasm-pack`
- Node.js 18+ (for integration testing)

### Build

```bash
cd crates/motarjim-wasm

# Build for browser (ES module)
wasm-pack build --target web

# Build for Node.js (CommonJS)
wasm-pack build --target nodejs

# Build for bundler (webpack, Vite, etc.)
wasm-pack build --target bundler
```

### Build Output

```
crates/motarjim-wasm/
├── pkg/
│   ├── motarjim_wasm.js        # JS glue code
│   ├── motarjim_wasm_bg.wasm   # WASM binary (~1.5MB)
│   ├── motarjim_wasm.d.ts      # TypeScript types
│   └── package.json            # npm package manifest
└── src/
    └── lib.rs                  # WASM bindings source
```

### Optimization

```bash
# Build with LTO for maximum performance
RUSTFLAGS='-C target-cpu=native' wasm-pack build --target web --release

# Check binary size
du -sh pkg/*.wasm
```

## Integration Examples

### Vanilla JavaScript

```javascript
import init, { WasmCompiler } from './motarjim_wasm.js';

async function compile() {
  await init();

  const compiler = new WasmCompiler();
  const result = JSON.parse(compiler.compile(
    '<nav><h1>My App</h1></nav>',
    'nav { background: #333; }',
    'flutter'
  ));

  if (result.success) {
    document.getElementById('output').textContent = result.code;
  }
}
```

### React

```tsx
import React, { useEffect, useState } from 'react';
import init, { WasmCompiler } from 'motarjim-wasm';

interface CompileResult {
  success: boolean;
  code?: string;
  errors?: string[];
}

export function Compiler({ html, css, platform }: Props) {
  const [result, setResult] = useState<CompileResult | null>(null);

  useEffect(() => {
    async function compile() {
      await init();
      const compiler = new WasmCompiler();
      const res = JSON.parse(compiler.compile(html, css, platform));
      setResult(res);
    }
    compile();
  }, [html, css, platform]);

  if (!result) return <div>Compiling...</div>;
  if (!result.success) return <div>Error: {result.errors}</div>;
  return <pre>{result.code}</pre>;
}
```

### Vite (Web Playground)

```typescript
// vite.config.ts
import { defineConfig } from 'vite';

export default defineConfig({
  optimizeDeps: {
    exclude: ['motarjim-wasm'],  // Don't bundle WASM in optimize step
  },
  server: {
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
});
```

```typescript
// In app code
import init, { WasmCompiler } from 'motarjim-wasm/pkg/motarjim_wasm.js';

async function initCompiler() {
  await init();
  const compiler = new WasmCompiler();
  return compiler;
}
```

## Performance Considerations

### Compilation Speed

WASM compilation typically adds 50-150ms overhead for the initial `init()` call (loading and instantiating the WASM module). Subsequent compilations reuse the instantiated module and run at native speed.

| Scenario | Time |
|----------|------|
| WASM module load + instantiate | ~100ms (cached after first call) |
| Compile small page (50 nodes) | ~5ms |
| Compile medium page (500 nodes) | ~15ms |
| Compile large page (5000 nodes) | ~120ms |

### Bundle Size

| Asset | Size (uncompressed) | Size (gzip) |
|-------|---------------------|-------------|
| `motarjim_wasm_bg.wasm` | ~1.5 MB | ~400 KB |

### Optimization Tips

1. **Instantiate once**: Create a single `WasmCompiler` instance and reuse it.
2. **Lazy init**: Call `init()` on first user interaction, not on page load.
3. **Cache result**: If the input hasn't changed, don't re-compile.
4. **Use `SharedArrayBuffer`** if available for faster WASM memory access (requires proper COOP/COEP headers).

## Browser Compatibility

| Browser | WASM Support | Shared Memory |
|---------|-------------|---------------|
| Chrome 57+ | ✅ Full | ✅ (with COOP/COEP) |
| Firefox 52+ | ✅ Full | ✅ (with COOP/COEP) |
| Safari 11+ | ✅ Full | ⚠️ Partial |
| Edge 16+ | ✅ Full | ✅ (with COOP/COEP) |
| Node.js 8+ | ✅ Full | ✅ |

### COOP/COEP Headers

For `SharedArrayBuffer` support (optional, for performance), serve with:

```
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```

## Limitations

- **WASM module is ~1.5MB** — Consider loading it lazily or showing a loading indicator.
- **No filesystem access** — The WASM compiler works with in-memory strings. File I/O works via JS APIs before passing strings to the compiler.
- **Single-threaded** — WASM currently runs on the main thread. Compilation is fast enough that this is not a concern for typical use cases.
- **Platform-specific features** Some Flutter/Compose/SwiftUI platform APIs cannot be fully expressed through WASM (e.g., platform channels, native sensor access). Generated code is always valid source — platform-specific extensions should be added after generation.
