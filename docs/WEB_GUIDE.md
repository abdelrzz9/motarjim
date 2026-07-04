# Web App Guide

## Architecture

motarjim's web presence consists of a single React application at `apps/web/` which serves as the interactive playground. There is no separate website application — the playground is the web face of motarjim.

```
motarjim/
└── apps/
    └── web/                  # Web playground (Vite + React)
        ├── src/
        │   ├── App.tsx
        │   ├── main.tsx
        │   ├── components/
        │   ├── features/
        │   ├── hooks/
        │   ├── services/
        │   └── stores/
        ├── public/
        ├── index.html
        ├── vite.config.ts
        ├── tsconfig.json
        └── package.json
```

The web app communicates with the Rust compiler engine through WebAssembly (WASM) bindings via `motarjim-wasm`. There is no server-side compilation — all compilation happens in the browser.

## Tech Stack

| Technology | Usage |
|------------|-------|
| **TypeScript** | 5.4+, strict mode |
| **React** | 18.3+ UI framework |
| **Vite** | Build tool |
| **Zustand** | State management |
| **React Router** | Client-side routing |
| **WASM** | Rust compiler via `motarjim-wasm` |

## Project Structure

### `apps/web`

The interactive web playground where users can write HTML/CSS and see generated native code in real-time.

```
apps/web/
├── src/
│   ├── App.tsx               # Root component
│   ├── main.tsx              # Entry point
│   ├── index.css             # Global styles
│   ├── components/           # Reusable UI components
│   ├── features/             # Feature-specific components
│   ├── hooks/                # Custom React hooks
│   ├── services/             # WASM compiler bridge, storage
│   ├── stores/               # Zustand stores
│   ├── utils/                # Utility functions
│   └── vite-env.d.ts         # Vite type declarations
├── public/                   # Static assets
├── index.html                # HTML entry point
├── vite.config.ts
├── tsconfig.json
├── tsconfig.node.json
└── package.json
```

## Development Workflow

### Setup

```bash
cd motarjim
npm install
```

### Run the Playground

```bash
npm run dev
# Opens at http://localhost:5173
```

### Build for Production

```bash
npm run build
# Output in apps/web/dist/
```

### Linting and Type Checking

```bash
npm run lint          # ESLint
npm run format        # Prettier
```

## Building for Production

```bash
npm run build
# Output: apps/web/dist/
# Deploy as static site
```

The build output is static HTML/CSS/JS. Serve with any static file server (nginx, Vercel, Netlify, Cloudflare Pages, etc.).

## Deployment

### Static Hosting (Recommended)

The playground is a static site. Deploy to:
- **Vercel**: Zero-config deployment
- **Netlify**: Connect repo, set build command, set publish directory
- **GitHub Pages**: Deploy from `gh-pages` branch
- **Cloudflare Pages**: Connect repo, set build configuration

## Architecture Decisions

### Why React?

The playground uses React 18 for component composition, state management via Zustand, and broad ecosystem support. Monaco editor integration (`@monaco-editor/react`) provides the code editing experience.

### Why No Server-Side Rendering?

The compiler runs entirely in the browser via WASM. This eliminates:
- Server costs for compilation
- Network latency for each compilation
- Data privacy concerns (user HTML never leaves the browser)
- Deployment complexity
