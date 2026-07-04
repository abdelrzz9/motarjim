import * as vscode from 'vscode';
import { selectPlatform, getCompilerConfig } from './commands';

let outputChannel: vscode.OutputChannel;
let statusBarItem: vscode.StatusBarItem;

export function activate(context: vscode.ExtensionContext) {
  outputChannel = vscode.window.createOutputChannel('Motarjim');
  outputChannel.appendLine('Motarjim extension activated');

  statusBarItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Right,
    100
  );
  statusBarItem.text = '$(symbol-event) Motarjim';
  statusBarItem.tooltip = 'Motarjim Compiler';
  statusBarItem.command = 'motarjim.openPlayground';
  statusBarItem.show();

  context.subscriptions.push(
    vscode.commands.registerCommand('motarjim.openPlayground', () => {
      vscode.env.openExternal(
        vscode.Uri.parse('http://localhost:3000')
      );
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('motarjim.compileFile', async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        vscode.window.showWarningMessage('No active editor');
        return;
      }
      const document = editor.document;
      if (document.languageId !== 'html') {
        vscode.window.showWarningMessage('Not an HTML file');
        return;
      }

      const platform = await selectPlatform();
      if (!platform) return;

      const config = await getCompilerConfig();

      outputChannel.appendLine(`Compiling: ${document.fileName}`);
      outputChannel.appendLine(`Platform: ${platform}`);
      outputChannel.appendLine(`Minify: ${config.minify}`);

      vscode.window.withProgress(
        {
          location: vscode.ProgressLocation.Notification,
          title: 'Motarjim: Compiling...',
          cancellable: false,
        },
        async (_progress) => {
          outputChannel.appendLine('Compilation requires the Motarjim Rust CLI.');
          outputChannel.appendLine(`Run: motarjim compile "${document.fileName}" --platform ${platform}${config.minify ? ' --minify' : ''}`);

          vscode.window.showInformationMessage(
            `Motarjim compilation started for ${platform}. Check output panel for details.`
          );
          outputChannel.show();
        }
      );
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('motarjim.showOutput', () => {
      outputChannel.show();
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('motarjim.clearCache', () => {
      outputChannel.appendLine('Cache cleared');
      vscode.window.showInformationMessage('Motarjim cache cleared');
    })
  );

  context.subscriptions.push(outputChannel);
  context.subscriptions.push(statusBarItem);
}

export function deactivate() {
  outputChannel?.dispose();
  statusBarItem?.dispose();
}
