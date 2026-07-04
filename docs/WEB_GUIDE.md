# Web App Guide

## Architecture

motarjim's web presence consists of two TypeScript applications and three shared packages, all managed as npm workspaces under `apps/` and `packages/`.

```
motarjim/
├── apps/
│   ├── playground/          # Interactive compiler playground (Vite)
│   └── website/             # Documentation & marketing website (Vite)
├── packages/
│   ├── vscode-extension/    # VS Code extension (separate guide)
│   ├── playground-sdk/      # Shared playground types & API client
│   └── website-sdk/         # Shared website navigation metadata
└── package.json             # Workspace root
```

The web apps communicate with the Rust compiler engine through WebAssembly (WASM) bindings via `motarjim-wasm`. There is no server-side compilation — all compilation happens in the browser.

## Tech Stack

| Technology | Usage |
|------------|-------|
| **TypeScript** | 5.4+, strict mode |
| **Vite** | Build tool for both apps |
| **Vitest** | Test runner |
| **ESLint** | TypeScript linting |
| **Prettier** | Code formatting |
| **Zod** | Runtime schema validation |
| **WASM** | Rust compiler via `motarjim-wasm` |

## Project Structure

### `apps/playground`

The interactive web playground where users can write HTML/CSS and see generated native code in real-time.

```
apps/playground/
├── src/
│   ├── index.html           # Entry point
│   ├── main.ts              # App initialization
│   ├── App.ts               # Root component
│   ├── components/
│   │   ├── Editor/          # Split-panel editor
│   │   │   ├── HtmlPanel.ts
│   │   │   ├── CssPanel.ts
│   │   │   └── OutputPanel.ts
│   │   ├── Pipeline/        # Pipeline visualizer
│   │   │   └── StageBar.ts
│   │   ├── Toolbar/         # Platform switcher, actions
│   │   │   ├── PlatformSwitcher.ts
│   │   │   └── ActionBar.ts
│   │   └── StatusBar/       # Compile stats
│   │       └── CompileStats.ts
│   ├── services/
│   │   ├── compiler.ts      # WASM compiler bridge
│   │   ├── storage.ts       # Draft persistence
│   │   └── samples.ts       # Example HTML/CSS samples
│   └── styles/
│       └── theme.css        # Theme variables
├── vite.config.ts
└── package.json
```

### `apps/website`

The documentation and marketing website.

```
apps/website/
├── src/
│   ├── index.html
│   ├── main.ts
│   ├── pages/
│   │   ├── Home.ts
│   │   ├── Docs.ts
│   │   └── Playground.ts
│   └── styles/
│       └── main.css
├── vite.config.ts
└── package.json
```

### `packages/playground-sdk`

Shared types and API definitions used by the playground and any consumer of the WASM compiler.

```
packages/playground-sdk/
├── src/
│   ├── index.ts
│   ├── types.ts             # CompileRequest, CompileResponse, etc.
│   └── client.ts            # WASM compiler client wrapper
└── package.json
```

### `packages/website-sdk`

Shared website metadata (navigation structure, SEO metadata, etc.).

```
packages/website-sdk/
├── src/
│   ├── index.ts
│   └── navigation.ts        # Site nav structure
└── package.json
```

## Component Library

The playground uses vanilla TypeScript with no UI framework. Components are organized by feature:

| Component | Description |
|-----------|-------------|
| `Editor/HtmlPanel` | HTML code editor with syntax highlighting (planned: Monaco editor) |
| `Editor/CssPanel` | CSS code editor |
| `Editor/OutputPanel` | Generated code output with syntax highlighting and copy button |
| `Pipeline/StageBar` | Animated progress bar showing compilation stages |
| `Toolbar/PlatformSwitcher` | Toggle between Flutter, Compose, SwiftUI |
| `Toolbar/ActionBar` | Paste, format, upload file, clear, load sample buttons |
| `StatusBar/CompileStats` | Shows compile time, node count, components, generated lines |

## State Management

State is managed with a simple reactive store pattern (no external state library):

```typescript
// services/compiler.ts
type CompileState = {
  html: string;
  css: string;
  platform: 'flutter' | 'compose' | 'swiftui';
  output: string;
  stats: CompileStats | null;
  errors: CompileError[];
  isCompiling: boolean;
};

// Events
type CompileEvent =
  | { type: 'COMPILE_START' }
  | { type: 'COMPILE_SUCCESS'; output: string; stats: CompileStats }
  | { type: 'COMPILE_ERROR'; errors: CompileError[] }
  | { type: 'CHANGE_HTML'; html: string }
  | { type: 'CHANGE_CSS'; css: string }
  | { type: 'CHANGE_PLATFORM'; platform: string };
```

## API Reference

### WASM Compiler Bridge

The playground communicates with the Rust compiler through the `motarjim-wasm` crate's `WasmCompiler` class:

```typescript
// packages/playground-sdk/src/client.ts
interface CompileRequest {
  html: string;
  css?: string;
  platform: 'flutter' | 'compose' | 'swiftui';
}

interface CompileResponse {
  success: boolean;
  code: string;
  stats?: {
    nodes: number;
    css_rules: number;
    time_ms: number;
  };
  errors?: string[];
}

// Usage
const compiler = new WasmCompiler();
const result = compiler.compile(html, css, platform);
```

### Compiler API Methods

```typescript
class WasmCompiler {
  constructor();
  compile(html: string, css: string | null, platform: string): string;
  // Returns JSON string of CompileResponse
  static version(): string;
}
```

## Development Workflow

### Setup

```bash
cd motarjim
npm install
```

### Run the Playground

```bash
npm run start:playground
# Opens at http://localhost:3000
```

### Run the Website

```bash
npm run start:website
# Opens at http://localhost:3001
```

### Build for Production

```bash
npm run build:playground
npm run build:website
# Output in apps/playground/dist/ and apps/website/dist/
```

### Testing

```bash
npm test              # Run all tests
npm run test:watch    # Watch mode
```

### Linting and Type Checking

```bash
npm run lint          # ESLint
npm run typecheck     # tsc --noEmit
npm run format        # Prettier
```

## Building for Production

### Playground

```bash
npm run build:playground
# Output: apps/playground/dist/
# Deploy as static site
```

### Website

```bash
npm run build:website
# Output: apps/website/dist/
# Deploy as static site
```

Both build outputs are static HTML/CSS/JS. Serve with any static file server (nginx, Vercel, Netlify, Cloudflare Pages, etc.).

## Deployment

### Static Hosting (Recommended)

Both playground and website are static sites. Deploy to:

- **Netlify**: Connect repo, set build command, set publish directory
- **Vercel**: Zero-config deployment
- **GitHub Pages**: Deploy from `gh-pages` branch
- **Cloudflare Pages**: Connect repo, set build configuration

### Docker

```bash
# Build the playground
npm run build:playground

# Serve with nginx
docker run -v $(pwd)/apps/playground/dist:/usr/share/nginx/html:ro -p 8080:80 nginx:alpine
```

## Architecture Decisions

### Why No Server-Side Rendering?

The compiler runs entirely in the browser via WASM. This eliminates:
- Server costs for compilation
- Network latency for each compilation
- Data privacy concerns (user HTML never leaves the browser)
- Deployment complexity

### Why Vanilla TypeScript Over a Framework?

The playground is intentionally lightweight. Its state management needs are simple (one input → one output). A framework like React or Vue would add complexity without proportional benefit. Framework migration is straightforward if the app grows.

### Why Separate SDK Packages?

`playground-sdk` and `website-sdk` are published independently so external consumers (VS Code extension, third-party tools) can reuse the types and API client without depending on the full playground application.
