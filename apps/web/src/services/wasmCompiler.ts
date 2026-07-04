import type { CompileRequest, CompileResult } from './types';

let wasmModule: Record<string, unknown> | null = null;
let wasmAvailable: boolean | null = null;

function loadScript(url: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const existing = document.querySelector(`script[src="${url}"]`);
    if (existing) { resolve(); return; }
    const script = document.createElement('script');
    script.src = url;
    script.type = 'module';
    script.onload = () => resolve();
    script.onerror = () => reject(new Error(`Failed to load ${url}`));
    document.head.appendChild(script);
  });
}

async function loadWasm(): Promise<Record<string, unknown> | null> {
  if (wasmModule) return wasmModule;
  if (wasmAvailable === false) return null;
  try {
    await loadScript('/wasm/motarjim_wasm.js');
    if (window.motarjim_wasm_init) await window.motarjim_wasm_init();
    const WasmCtor = window.WasmCompiler;
    if (!WasmCtor) throw new Error('WasmCompiler not found');
    wasmModule = new WasmCtor() as unknown as Record<string, unknown>;
    return wasmModule;
  } catch {
    console.warn('WASM compiler not available, using fallback compiler');
    wasmAvailable = false;
    return null;
  }
}

function generateFlutterCode(html: string, _css: string): string {
  const hasContainer = html.includes('class="container"') || html.includes("class='container'");
  const indent = '  ';
  const lines: string[] = [
    "import 'package:flutter/material.dart';",
    '',
    'void main() => runApp(const MyApp());',
    '',
    'class MyApp extends StatelessWidget {',
    `${indent}const MyApp({super.key});`,
    '',
    `${indent}@override`,
    `${indent}Widget build(BuildContext context) {`,
    `${indent}${indent}return MaterialApp(`,
    `${indent}${indent}${indent}title: 'Motarjim',`,
    `${indent}${indent}${indent}theme: ThemeData(`,
    `${indent}${indent}${indent}${indent}colorSchemeSeed: const Color(0xFF6366F1),`,
    `${indent}${indent}${indent}${indent}useMaterial3: true,`,
    `${indent}${indent}${indent}),`,
    `${indent}${indent}${indent}home: const HomePage(),`,
    `${indent}${indent});`,
    `${indent}}`,
    '}',
    '',
    'class HomePage extends StatelessWidget {',
    `${indent}const HomePage({super.key});`,
    '',
    `${indent}@override`,
    `${indent}Widget build(BuildContext context) {`,
    `${indent}${indent}return Scaffold(`,
    `${indent}${indent}${indent}body: ${hasContainer ? 'Container(\n' +
      indent + indent + indent + indent + 'padding: const EdgeInsets.all(20),\n' +
      indent + indent + indent + indent + 'child: Column(\n' +
      indent + indent + indent + indent + indent + 'crossAxisAlignment: CrossAxisAlignment.start,\n' +
      indent + indent + indent + indent + indent + 'children: [\n' +
      indent + indent + indent + indent + indent + indent + "const Text('Hello, Motarjim!',\n" +
      indent + indent + indent + indent + indent + indent + indent + "style: TextStyle(fontSize: 32, fontWeight: FontWeight.bold)),\n" +
      indent + indent + indent + indent + indent + indent + "const SizedBox(height: 8),\n" +
      indent + indent + indent + indent + indent + indent + "const Text('Start editing to see the compiled output.'),\n" +
      indent + indent + indent + indent + indent + indent + '],\n' +
      indent + indent + indent + indent + indent + '),\n' +
      indent + indent + indent + indent + ')' : 'const Center(child: Text(\'Motarjim\'))'}`,
    `${indent}${indent}${indent});`,
    `${indent}}`,
    '}',
  ];
  return lines.join('\n');
}

function generateComposeCode(_html: string): string {
  const indent = '  ';
  const lines: string[] = [
    'import android.os.Bundle',
    'import androidx.activity.ComponentActivity',
    'import androidx.activity.compose.setContent',
    'import androidx.compose.foundation.layout.*',
    'import androidx.compose.material3.*',
    'import androidx.compose.runtime.Composable',
    'import androidx.compose.ui.Alignment',
    'import androidx.compose.ui.Modifier',
    'import androidx.compose.ui.unit.dp',
    'import androidx.compose.ui.unit.sp',
    '',
    'class MainActivity : ComponentActivity() {',
    `${indent}override fun onCreate(savedInstanceState: Bundle?) {`,
    `${indent}${indent}super.onCreate(savedInstanceState)`,
    `${indent}${indent}setContent {`,
    `${indent}${indent}${indent}MotarjimTheme {`,
    `${indent}${indent}${indent}${indent}Surface(modifier = Modifier.fillMaxSize()) {`,
    `${indent}${indent}${indent}${indent}${indent}MainScreen()`,
    `${indent}${indent}${indent}${indent}}`,
    `${indent}${indent}${indent}}`,
    `${indent}${indent}}`,
    `${indent}}`,
    '}',
    '',
    '@OptIn(ExperimentalMaterial3Api::class)',
    '@Composable',
    'fun MainScreen() {',
    `${indent}Scaffold(`,
    `${indent}${indent}topBar = { TopAppBar(title = { Text("Motarjim") }) }`,
    `${indent}) { padding ->`,
    `${indent}${indent}Column(`,
    `${indent}${indent}${indent}modifier = Modifier`,
    `${indent}${indent}${indent}${indent}.fillMaxSize()`,
    `${indent}${indent}${indent}${indent}.padding(padding)`,
    `${indent}${indent}${indent}${indent}.padding(20.dp),`,
    `${indent}${indent}${indent}${indent}verticalArrangement = Arrangement.Top`,
    `${indent}${indent}${indent}) {`,
    `${indent}${indent}${indent}${indent}Text("Hello, Motarjim!", fontSize = 24.sp, fontWeight = FontWeight.Bold)`,
    `${indent}${indent}${indent}${indent}Spacer(modifier = Modifier.height(8.dp))`,
    `${indent}${indent}${indent}${indent}Text("Start editing to see the compiled output.")`,
    `${indent}${indent}${indent}}`,
    `${indent}${indent}}`,
    `${indent}}`,
    '}',
    '',
    '@Composable',
    'fun MotarjimTheme(content: @Composable () -> Unit) {',
    `${indent}MaterialTheme(`,
    `${indent}${indent}colorScheme = lightColorScheme(`,
    `${indent}${indent}${indent}primary = androidx.compose.ui.graphics.Color(0xFF6366F1)`,
    `${indent}${indent})`,
    `${indent}) { content() }`,
    '}',
  ];
  return lines.join('\n');
}

function generateSwiftUICode(_html: string): string {
  const indent = '  ';
  const lines: string[] = [
    'import SwiftUI',
    '',
    '@main',
    'struct MotarjimApp: App {',
    `${indent}var body: some Scene {`,
    `${indent}${indent}WindowGroup {`,
    `${indent}${indent}${indent}ContentView()`,
    `${indent}${indent}}`,
    `${indent}}`,
    '}',
    '',
    'struct ContentView: View {',
    `${indent}var body: some View {`,
    `${indent}${indent}NavigationView {`,
    `${indent}${indent}${indent}VStack(alignment: .leading, spacing: 8) {`,
    `${indent}${indent}${indent}${indent}Text("Hello, Motarjim!")`,
    `${indent}${indent}${indent}${indent}${indent}.font(.largeTitle)`,
    `${indent}${indent}${indent}${indent}${indent}.fontWeight(.bold)`,
    `${indent}${indent}${indent}${indent}Text("Start editing to see the compiled output.")`,
    `${indent}${indent}${indent}${indent}${indent}.font(.body)`,
    `${indent}${indent}${indent}${indent}${indent}.foregroundColor(.secondary)`,
    `${indent}${indent}${indent}}`,
    `${indent}${indent}${indent}${indent}.padding(20)`,
    `${indent}${indent}${indent}${indent}.navigationTitle("Motarjim")`,
    `${indent}${indent}${indent}}`,
    `${indent}${indent}}`,
    `${indent}}`,
    '}',
  ];
  return lines.join('\n');
}

function fallbackCompile(request: CompileRequest): CompileResult {
  const start = performance.now();
  let code = '';

  switch (request.platform) {
    case 'flutter':
      code = generateFlutterCode(request.html, request.css || '');
      break;
    case 'compose':
      code = generateComposeCode(request.html);
      break;
    case 'swiftui':
      code = generateSwiftUICode(request.html);
      break;
  }

  const elapsed = Math.round(performance.now() - start);

  return {
    success: true,
    code,
    diagnostics: [{
      severity: 'info',
      code: 'W0001',
      message: 'Running in fallback mode. Build the WASM module for full compilation support.',
      suggestions: ['Run wasm-pack build in the crates/motarjim-wasm directory'],
      notes: [],
    }],
    stats: {
      nodes_parsed: request.html.split(/<\//).length - 1,
      css_rules: request.css ? (request.css.match(/[^{}]+(?=\{)/g) || []).length : 0,
      ir_nodes: 0,
      time_ms: elapsed,
    },
  };
}

export async function compile(request: CompileRequest): Promise<CompileResult> {
  const compiler = await loadWasm();
  if (compiler) {
    try {
      const compileFn = compiler.compile as (html: string, css: string | null, platform: string) => string;
      const result = JSON.parse(
        compileFn(request.html, request.css || null, request.platform)
      ) as CompileResult;
      return result;
    } catch {
      console.warn('WASM compile failed, using fallback compiler');
      wasmAvailable = false;
      wasmModule = null;
    }
  }
  return fallbackCompile(request);
}
