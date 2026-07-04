import * as vscode from 'vscode';

export async function selectPlatform(): Promise<string | undefined> {
  const config = vscode.workspace.getConfiguration('motarjim');
  const defaultPlatform = config.get<string>('defaultPlatform', 'flutter');

  const platforms = [
    {
      label: 'Flutter (Dart)',
      description: 'Generate Flutter widget code',
      detail: 'Target platform: Flutter',
      value: 'flutter',
    },
    {
      label: 'Jetpack Compose (Kotlin)',
      description: 'Generate Jetpack Compose code',
      detail: 'Target platform: Compose',
      value: 'compose',
    },
    {
      label: 'SwiftUI',
      description: 'Generate SwiftUI code',
      detail: 'Target platform: SwiftUI',
      value: 'swiftui',
    },
  ];

  const preselected = platforms.findIndex((p) => p.value === defaultPlatform);

  const selected = await vscode.window.showQuickPick(platforms, {
    placeHolder: 'Select target platform',
    canPickMany: false,
    matchOnDescription: true,
    matchOnDetail: true,
    title: 'Motarjim: Select Target Platform',
    value: defaultPlatform,
  });

  if (preselected >= 0) {
    platforms[preselected].label = `$(check) ${platforms[preselected].label}`;
  }

  return selected?.value;
}

export async function getCompilerConfig(): Promise<{
  platform: string;
  minify: boolean;
}> {
  const config = vscode.workspace.getConfiguration('motarjim');
  const platform = config.get<string>('defaultPlatform', 'flutter');
  const minify = config.get<boolean>('minifyOutput', false);
  return { platform, minify };
}
