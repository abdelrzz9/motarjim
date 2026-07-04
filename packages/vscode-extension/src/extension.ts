import * as vscode from 'vscode';

export function activate(context: vscode.ExtensionContext) {
  const command = vscode.commands.registerCommand('motarjim.openPlayground', () => {
    vscode.env.openExternal(vscode.Uri.parse('http://localhost:3000'));
  });
  context.subscriptions.push(command);
}

export function deactivate() {}
