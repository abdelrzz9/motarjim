# Web apps and VS Code extension

motarjim keeps user-facing TypeScript projects in `apps/` and reusable TypeScript surfaces in `packages/`.

## Apps

- `apps/playground` is the restored Vite playground. It contains the split editor, output panel, pipeline visualization, command palette, notifications, status bar, and keyboard helpers that previously lived under `web/`.
- `apps/website` is the documentation and marketing website shell. It is intentionally small and links users toward the docs and local playground.

Run the apps from the workspace root:

```bash
npm install
npm run start:playground
npm run start:website
```

The playground defaults to <http://localhost:3000>. The website defaults to <http://localhost:3001>.

## Packages

- `packages/vscode-extension` contains the VS Code extension entry point and command contribution for opening the local playground.
- `packages/playground-sdk` contains shared playground request/target types.
- `packages/website-sdk` contains website navigation metadata used by website surfaces.

`packages/` is intentionally tracked by Git so these TypeScript workspaces can be developed alongside the Rust crates.
