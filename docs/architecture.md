# Architecture

## Overview

html-native-engine follows a classic compiler architecture with discrete, composable stages. Each stage transforms the input into a progressively more platform-independent representation, culminating in native UI code generation.

```
HTML + CSS
    │
    ▼
┌─────────────┐
│   Parser    │  HtmlNode AST
└─────────────┘
    │
    ▼
┌──────────────┐
│ CSS Analyzer │  ResolvedStyles per node
└──────────────┘
    │
    ▼
┌──────────────────┐
│Semantic Analyzer │  SemanticHint[] (component types)
└──────────────────┘
    │
    ▼
┌──────────┐
│    IR    │  UiNode platform-neutral tree
└──────────┘
    │
    ▼
┌──────────┐
│Optimizer │  Optimized UiNode tree
└──────────┘
    │
    ▼
┌────────────────┐
│Generator-Core  │  Shared traversal + formatting
└────────────────┘
    │
    ┌────┬────┬────┐
    ▼    ▼    ▼    ▼
 Flutter Compose SwiftUI  (platform code)
```

## Package Structure

```
html-native-engine/
├── packages/
│   ├── shared/                 # Core types shared by all packages
│   │   └── index.ts            # HtmlNode, StyledNode, UiNode, ResolvedStyles, SemanticHint
│   ├── parser/                 # HTML → HtmlNode AST
│   │   └── index.ts            # parseHtml(), parseFragment()
│   ├── css-analyzer/           # CSS → ResolvedStyles per node
│   │   └── index.ts            # parseCss(), matchSelector(), resolveStyles(), applyStyles()
│   ├── semantic-analyzer/      # Rule-based component detection
│   │   ├── index.ts            # detectSemantics()
│   │   └── ai.ts               # createAiDetector() (optional Ollama integration)
│   ├── ir/                     # Platform-neutral intermediate representation
│   │   └── index.ts            # styledNodeToIr(), createIrNode()
│   ├── optimizer/              # IR tree optimization passes
│   │   └── index.ts            # optimize(), defaultPasses
│   ├── generator-core/         # Shared code generation infrastructure
│   │   └── index.ts            # walkTree(), NodeEmitter, countNodes(), escapeString()
│   ├── generators/
│   │   ├── flutter/            # Dart code generation
│   │   ├── compose/            # Kotlin Compose code generation
│   │   └── swiftui/            # Swift code generation
│   └── cli/                    # Command-line interface
│       ├── index.ts            # Entry point
│       ├── bin.ts              # Binary shebang
│       └── src/
│           ├── cli.ts          # Commander program setup (7 commands)
│           ├── types.ts        # Shared type definitions
│           ├── config/         # Config loader + smart defaults
│           ├── commands/       # convert, init, watch, batch, validate, explain, new
│           ├── services/       # Pipeline, wizard, validation, templates, error formatting
│           └── ui/             # Progress spinners, output formatting
├── benchmarks/
│   ├── fixtures/               # Reusable benchmark input files
│   ├── run.ts                  # Benchmark runner
│   └── RESULTS.md              # Latest benchmark results
├── tests/                      # Test suite (71 tests, 12 snapshots)
└── docs/                       # Documentation
```

## Data Flow

### 1. Parsing (HTML → HtmlNode)

The parser uses [parse5](https://github.com/inikulin/parse5) (the same engine as jsdom) to produce a typed AST. Each element becomes an `HtmlNode` with:

- `nodeId` — unique identifier
- `tagName` — lowercase tag name
- `attributes` — name/value pairs
- `children` — child nodes
- `value` — text content for `#text` nodes
- `sourceLocation` — line/column for error reporting

### 2. CSS Analysis (CSS + HtmlNode → StyledNode[])

The CSS analyzer uses [PostCSS](https://postcss.org/) to parse stylesheets into rules and media queries. Each HTML node is matched against CSS selectors (tag, class, id, universal) and its resolved styles are attached, producing `StyledNode[]`.

### 3. Semantic Analysis (StyledNode → SemanticHint[])

Rule-based heuristics examine tag names, class names, and computed styles to identify semantic components:

- `<nav>` → `NavigationBar`
- `.card` with shadow/rounded corners → `Card`
- `.hero` class → `HeroSection`
- `<footer>` → `Footer`
- `<form>` → `Form`

An optional Ollama AI detector merges AI-generated hints with rule-based results.

### 4. IR Conversion (StyledNode + SemanticHint → UiNode)

The Intermediate Representation is a platform-neutral tree of `UiNode` objects. Each node has:

- `type` — one of 38 possible types (Container, Text, Button, Card, etc.)
- `properties` — platform-independent style/attribute key-value pairs
- `children` — child UiNodes
- `value` — text content for text nodes
- `sourceHtmlTag` — original HTML tag name

CSS properties are converted to camelCase (e.g., `font-size` → `fontSize`, `border-radius` → `borderRadius`).

### 5. Optimization (UiNode → UiNode)

Three passes run sequentially:

1. **removeEmptyText** — deletes text nodes with no content
2. **mergeTextNodes** — merges adjacent text nodes
3. **flattenContainers** — removes redundant Container wrappers

Optimization happens exactly once at the pipeline level (CLI or `runPipeline()`). Generators expect pre-optimized IR.

### 6. Code Generation (UiNode → platform code)

The `generator-core` package provides:

- `NodeEmitter` interface — 13 methods that each platform implements
- `walkTree()` — recursive traversal that dispatches to emitter methods
- `countNodes()`, `escapeString()`, `findTextLabel()`, `getNonTextChildren()`

Each platform generator implements `NodeEmitter` with platform-specific syntax:

- **Flutter**: `Container`, `Column`, `Row`, `Text`, `ElevatedButton`, `Card`, etc.
- **Compose**: `Box`, `Column`, `Row`, `Text`, `Button`, `Card`, `TopAppBar`, etc.
- **SwiftUI**: `VStack`, `HStack`, `Text`, `Button`, `Form`, `.navigationTitle()`, etc.

## Design Decisions

### Why Not a Shared Boilerplate Wrapper?

Each platform's boilerplate (imports, class/struct/function declaration, closing syntax) is too syntax-divergent to benefit from a shared abstraction. A shared config object would be more complex than the three template strings it replaces.

### Why CamelCase Properties in IR?

CSS properties arrive in kebab-case (`font-size`). The IR converts them to camelCase (`fontSize`) during extraction. This is the natural form for Flutter/Dart properties, and both Kotlin and Swift also use camelCase. Each generator emits them directly.

### Why Single-Point Optimization?

Previously, each generator called `optimize()` internally, causing double-optimization bugs. Moving optimization to exactly one call site (the pipeline level) eliminated the issue and clarified ownership.

### Why generator-core Instead of a Shared Abstract Generator?

The three platform generators are similar-but-different. Forcing them into a shared abstract base class would require the abstraction to handle every difference, making it more complex than the sum of three independent generators. `generator-core` owns what's genuinely shared (traversal, escaping, counting) and leaves syntax decisions to each platform.

## Extension Points

1. **New platform generator** — implement `NodeEmitter`, add a `generate()` function
2. **New optimization pass** — add to `defaultPasses` array in optimizer
3. **New semantic rule** — add to `detectSemantics()` switch/case
4. **New CSS property** — add to `extractProps()` in IR
5. **New IR node type** — add to `UiNodeType` union, add case in `walkTree()`

## Future Scalability

- **Incremental compilation:** The pipeline is functional (input → output with no mutable state per call). The built-in watch mode (via chokidar) recompiles on file changes.
- **Source maps:** Each `UiNode` could carry source location data from `HtmlNode.sourceLocation`.
- **CSS cascade:** Current matching is flat (last rule wins). A proper specificity calculator would improve output quality.
- **Responsive design:** Media query data flows through the IR but generators don't consume it yet — a natural extension point.
