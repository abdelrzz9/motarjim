# VS Code Extension Guide

## Features

The motarjim VS Code extension provides editor integration for the motarjim compiler:

- **Open Playground** — Launch the motarjim web playground directly from VS Code
- (Coming soon) Syntax highlighting for `.motarjim` files
- (Coming soon) LSP integration with diagnostics, completions, and hover information

## Installation

### From VS Code Marketplace

Search for "motarjim" in the VS Code Extensions panel (`Ctrl+Shift+X`) and click Install.

### From VSIX

```bash
# Build the extension
cd apps/vscode-extension
npm install
npm run package

# Install the VSIX
code --install-extension motarjim-vscode-extension-0.1.0.vsix
```

### From Source

```bash
cd apps/vscode-extension
npm install
npm run compile
code .
# Press F5 to run the extension in development mode
```

## Configuration

The extension has no required configuration. Optional settings (when available):

| Setting | Description | Default |
|---------|-------------|---------|
| `motarjim.compilerPath` | Path to the motarjim CLI binary | Auto-detected |

## Commands

| Command | ID | Description |
|---------|-----|-------------|
| `motarjim: Open Playground` | `motarjim.openPlayground` | Opens the motarjim web playground |

### How to Use

1. Open the Command Palette (`Ctrl+Shift+P` or `Cmd+Shift+P` on macOS)
2. Type "motarjim: Open Playground"
3. Press Enter — the playground opens in your default browser

## Development

### Extension Structure

```
apps/vscode-extension/
├── package.json         # Extension manifest
├── src/
│   ├── extension.ts     # Extension entry point
│   └── ...              # Additional handlers
└── tsconfig.json        # TypeScript config
```

### Building

```bash
cd apps/vscode-extension
npm install

# Compile TypeScript
npm run compile

# Watch mode during development
npm run watch

# Package into VSIX for distribution
npm install -g @vscode/vsce
vsce package
```

### Running in Development

```bash
cd apps/vscode-extension
code .
# Press F5 to open Extension Development Host
```

## Planned Features

### LSP Integration

The `motarjim-lsp` crate provides a Language Server Protocol implementation. The extension will connect to it for:

- **Diagnostics** — Real-time error/warning reporting as you type HTML/CSS
- **Completions** — CSS property/value autocompletion
- **Hover information** — Documentation on hover for CSS properties and HTML elements
- **Go to definition** — Navigate from CSS class references to their definitions
- **Document symbols** — Outline view for HTML structure and CSS rules
- **Semantic tokens** — Syntax highlighting powered by the compiler's lexer

### Preview Panel

An embedded webview panel showing generated code in real-time as you edit HTML/CSS files, similar to the web playground but inside VS Code.

### Snippets

Code snippets for common motarjim patterns:
- Basic HTML component template
- CSS style template
- Multi-platform compilation targets

## Publishing

### Prerequisites

- Install `vsce`: `npm install -g @vscode/vsce`
- Create a publisher on the [VS Code Marketplace](https://marketplace.visualstudio.com/vscode)
- Obtain a Personal Access Token

### Package

```bash
cd apps/vscode-extension
vsce package
# Creates motarjim-vscode-extension-<version>.vsix
```

### Publish

```bash
vsce publish
# Increments version automatically if not specified
```

## Troubleshooting

### "Command not found"
- Ensure the extension is enabled: Extensions panel → search "motarjim" → Enable
- Reload VS Code window after installation

### Playground doesn't open
- Verify the playground dev server is running: `npm run dev -w @motarjim/web`
- Check `motarjim.playgroundUrl` setting
- Check VS Code's output panel for error messages
