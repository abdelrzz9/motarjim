# motarjim

**HTML/CSS → Native UI Code compiler** for Flutter, Jetpack Compose, and SwiftUI.

Write once in HTML/CSS. Ship native code for every platform.

[![npm version](https://img.shields.io/badge/npm-0.1.0-blue)]()
[![license](https://img.shields.io/badge/license-MIT-green)]()
[![build](https://img.shields.io/badge/build-passing-brightgreen)]()
[![tests](https://img.shields.io/badge/tests-71%20passed-brightgreen)]()
[![benchmark](https://img.shields.io/badge/benchmark-98ms%20%2F%201000%20nodes-blue)]()

---

## Features

- **Local-first** — Zero cloud dependencies. Everything runs on your machine.
- **Multi-platform** — Generate Flutter (Dart), Jetpack Compose (Kotlin), or SwiftUI from the same HTML/CSS.
- **Compiler architecture** — Full pipeline: parse → analyze → optimize → generate. No runtime, no WebView.
- **71 tests** — Comprehensive test suite with snapshot coverage.
- **~100ms for 1000 nodes** — 5× headroom against the 500ms target.
- **Optional AI enhancement** — Ollama integration for smarter component detection.
- **TypeScript strict mode** — Zero `any`, zero errors.

## Architecture

```
HTML + CSS
    │
    ▼
┌─────────────┐
│   Parser    │ ──▶ HtmlNode AST
└─────────────┘
    │
    ▼
┌──────────────┐
│ CSS Analyzer │ ──▶ StyledNode tree
└──────────────┘
    │
    ▼
┌──────────────────┐
│Semantic Analyzer │ ──▶ SemanticHint[]
└──────────────────┘     (rule-based + optional AI)
    │
    ▼
┌──────────┐
│    IR    │ ──▶ UiNode (platform-neutral)
└──────────┘
    │
    ▼
┌──────────┐
│Optimizer │ ──▶ Optimized UiNode
└──────────┘
    │
    ▼
┌────────────────┐
│  Generator     │ ──▶ Flutter / Compose / SwiftUI
└────────────────┘
```

## Supported Targets

| Platform | Language | Widget Set |
|----------|----------|------------|
| Flutter | Dart | Material Design |
| Jetpack Compose | Kotlin | Material 3 |
| SwiftUI | Swift | iOS 17+ |

## Why This Exists

Mobile teams face a choice: write UI code three times (Flutter + Compose + SwiftUI) or use a cross-platform framework that adds a runtime layer.

html-native-engine offers a third path: write your UI structure in HTML/CSS, generate native code for each platform, and get the performance and idiomatic feel of hand-written platform code — without writing it three times.

## Quick Start

### Installation

```bash
# From npm (when published)
npm install -g html-native-engine

# From source
git clone https://github.com/abdelrzz9/motarjim.git
cd motarjim
npm install
```

### CLI Usage

```bash
# Convert to Flutter
html-native convert \
  --input page.html \
  --css styles.css \
  --target flutter \
  --output lib/generated.dart

# Convert to Compose
html-native convert \
  --input page.html \
  --css styles.css \
  --target compose \
  --output GeneratedView.kt

# Convert to SwiftUI
html-native convert \
  --input page.html \
  --css styles.css \
  --target swiftui \
  --output GeneratedView.swift
```

## Examples

### Input HTML

```html
<nav class="navbar">
  <h1>My App</h1>
</nav>
<section class="hero">
  <h1>Welcome</h1>
  <p>Build something great</p>
  <button>Get Started</button>
</section>
```

### Input CSS

```css
.navbar { background: #333; color: white; padding: 1rem; }
.hero { text-align: center; padding: 4rem; background: #1a1a2e; color: white; }
button { background: blue; color: white; border-radius: 8px; padding: 12px 24px; }
```

### Generated Flutter

```dart
import 'package:flutter/material.dart';

class GeneratedView extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        AppBar(title: Text("My App")),
        Column(
          children: [
            Text("Welcome"),
            Text("Build something great"),
            ElevatedButton(
              onPressed: () {},
              child: Text("Get Started"),
            ),
          ],
        ),
      ],
    );
  }
}
```

### Generated Compose

```kotlin
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.foundation.layout.*
import androidx.compose.ui.Modifier

@Composable
fun GeneratedView() {
    Column {
        TopAppBar(title = { Text("My App") })
        Column {
            Text(text = "Welcome")
            Text(text = "Build something great")
            Button(onClick = { }) {
                Text(text = "Get Started")
            }
        }
    }
}
```

### Generated SwiftUI

```swift
import SwiftUI

struct GeneratedView: View {
    var body: some View {
        VStack {
            Text("My App")
                .navigationTitle("My App")
            VStack {
                Text("Welcome")
                Text("Build something great")
                Button("Get Started") {
                    // action
                }
            }
        }
    }
}
```

## AI Enhancement

Optional Ollama integration for improved component detection:

```bash
html-native convert \
  --input page.html \
  --css styles.css \
  --target flutter \
  --ai-enhance
```

See [docs/ai-enhancement.md](docs/ai-enhancement.md) for setup instructions.

## Performance Benchmarks

| Metric | Value |
|--------|-------|
| Pipeline (1000 nodes) | **98ms** median |
| Target | 500ms |
| Headroom | **5.1×** |
| Generators (all 3) | +13ms |

See [docs/benchmarks.md](docs/benchmarks.md) for detailed results.

## Roadmap

- [x] Core pipeline (parser → CSS → semantic → IR → optimize → generate)
- [x] 3 platform generators (Flutter, Compose, SwiftUI)
- [x] Media query support
- [x] AI enhancement layer
- [x] Performance benchmarks (100ms / 1000 nodes)
- [ ] CSS value mapping (colors, padding, etc.)
- [ ] Responsive design generation
- [ ] Advanced CSS selectors
- [ ] Watch mode
- [ ] VS Code extension

See [docs/roadmap.md](docs/roadmap.md) for full roadmap.

## Documentation

- [Introduction](docs/introduction.md)
- [Architecture](docs/architecture.md)
- [Pipeline](docs/pipeline.md)
- [Parser](docs/parser.md)
- [CSS Analyzer](docs/css-analyzer.md)
- [Semantic Analyzer](docs/semantic-analyzer.md)
- [IR (Intermediate Representation)](docs/ir.md)
- [Optimizer](docs/optimizer.md)
- [Generator Core](docs/generator-core.md)
- [Flutter Generator](docs/flutter-generator.md)
- [Compose Generator](docs/compose-generator.md)
- [SwiftUI Generator](docs/swiftui-generator.md)
- [CLI Reference](docs/cli.md)
- [AI Enhancement](docs/ai-enhancement.md)
- [Benchmarks](docs/benchmarks.md)
- [Contributing](docs/contributing.md)
- [Troubleshooting](docs/troubleshooting.md)
- [FAQ](docs/faq.md)
- [Roadmap](docs/roadmap.md)

## Contributing

See [docs/contributing.md](docs/contributing.md) for setup, workflow, and coding standards.

## License

MIT
