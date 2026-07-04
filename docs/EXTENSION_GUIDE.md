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
cd packages/vscode-extension
npm install
npm run package

# Install the VSIX
code --install-extension motarjim-vscode-extension-0.1.0.vsix
```

### From Source

```bash
cd packages/vscode-extension
npm install
npm run compile
code .
# Press F5 to run the extension in development mode
```

## Configuration

The extension has no required configuration. Optional settings (when available):

| Setting | Description | Default |
|---------|-------------|---------|
| `motarjim.playgroundUrl` | URL of the web playground | `http://localhost:3000` |
| `motarjim.compilerPath` | Path to the motarjim CLI binary | Auto-detected |

## Commands

| Command | ID | Description |
|---------|-----|-------------|
| `motarjim: Open Playground` | `motarjim.openPlayground` | Opens the motarjim web playground |

### How to Use

1. Open the Command Palette (`Ctrl+Shift+P` or `Cmd+Shift+P` on macOS)
2. Type "motarjim: Open Playground"
3. Press Enter — the playground opens in your default browser

## Keybindings

| Keybinding | Command | When |
|-----------|---------|------|
| (none yet) | `motarjim.openPlayground` | — |

## Development

### Extension Structure

```
packages/vscode-extension/
├── package.json         # Extension manifest
├── src/
│   ├── extension.ts     # Extension entry point
│   └── ...              # Additional handlers
└── tsconfig.json        # TypeScript config
```

### Current Implementation

The extension currently registers a single command (`motarjim.openPlayground`) that opens the web playground URL. The playground runs separately (via `npm run start:playground`) and the extension simply opens it in the browser.

### Building

```bash
cd packages/vscode-extension
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
cd packages/vscode-extension
code .
# Press F5 to open Extension Development Host
# Ctrl+Shift+P → "motarjim: Open Playground"
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
cd packages/vscode-extension
vsce package
# Creates motarjim-vscode-extension-<version>.vsix
```

### Publish

```bash
vsce publish
# Increments version automatically if not specified
```

### Version Bumping

```bash
# Patch version (0.1.0 → 0.1.1)
vsce publish patch

# Minor version (0.1.0 → 0.2.0)
vsce publish minor

# Major version (0.1.0 → 1.0.0)
vsce publish major
```

## Extension Development Guidelines

### Adding New Commands

1. Define the command in `package.json` under `contributes.commands`
2. Register the command in `src/extension.ts`:
   ```typescript
   context.subscriptions.push(
     vscode.commands.registerCommand('motarjim.newCommand', () => {
       // Command implementation
     })
   );
   ```
3. Add keybinding in `package.json` under `contributes.keybindings` (optional)

### Testing

```bash
npm test
```

Tests use VS Code's test API. Run in the Extension Development Host.

### Debugging

- Set breakpoints in `src/extension.ts`
- Press F5 to start the Extension Development Host
- Use the VS Code debug console for logging

## Troubleshooting

### "Command not found"
- Ensure the extension is enabled: Extensions panel → search "motarjim" → Enable
- Reload VS Code window after installation

### Playground doesn't open
- Verify the playground server is running: `npm run start:playground`
- Check `motarjim.playgroundUrl` setting
- Check VS Code's output panel for error messages
