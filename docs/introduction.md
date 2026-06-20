# Introduction

**html-native-engine** is a local-first compiler that converts HTML and CSS into native UI code for three mobile platforms:

- **Flutter** (Dart)
- **Jetpack Compose** (Kotlin)
- **SwiftUI** (Swift)

## Philosophy

Write your UI once in HTML/CSS — the language of the web — and generate platform-native code that feels hand-written. No runtime, no interpretation layer, no WebView. The output is idiomatic code for each platform that you can open in Android Studio, Xcode, or your editor of choice.

## Why Local-First?

Unlike cloud-based converters, html-native-engine runs entirely on your machine:

- **Zero data leaves your computer.** Your HTML and CSS never touch a remote server.
- **No API keys, no accounts.** Install and run.
- **Offline capable.** Works without internet (except the optional Ollama AI enhancement).
- **CI/CD friendly.** Run it in GitHub Actions, GitLab CI, or any pipeline.

## What It Is Not

- **Not a runtime or framework.** It generates static source files. You compile them with the standard platform toolchain.
- **Not a design tool exporter.** It takes hand-written or tool-generated HTML/CSS.
- **Not pixel-perfect.** CSS is a rich layout language; native UI frameworks have different primitives. The output is functionally equivalent, not pixel-identical.

## Who It's For

- **Mobile engineers** who want to prototype in HTML and ship native code.
- **Design-to-code workflows** where designs are exported as HTML/CSS and need native implementations.
- **Cross-platform tooling** that targets mobile UIs from web-standard inputs.
- **Compiler engineers** interested in source-to-source translation for UI languages.

## Key Features

- **Full pipeline:** parser → CSS analyzer → semantic analyzer → IR → optimizer → code generator
- **103 tests** covering parsing, CSS matching, semantic detection, IR conversion, optimization, code generation, and CLI commands
- **12 snapshot tests** verifying generated output across nav, card, hero, and form patterns
- **100ms pipeline** for 1000+ HTML nodes — 5× headroom against the 500ms target
- **7 CLI commands:** convert, init, watch, batch, validate, explain, new (templates)
- **Smart defaults:** auto-detect input, CSS, target from output extension
- **Watch mode:** auto-regenerate on HTML/CSS changes via chokidar
- **Media query support** in the CSS analyzer (parsed into the IR, generator consumption in progress)
- **Optional Ollama AI enhancement** for improved semantic component detection
- **TypeScript strict mode** throughout — zero `any` in production code
