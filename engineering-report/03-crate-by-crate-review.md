# Crate-by-Crate Review

## Foundation Layer

### `motarjim-span` (283 LOC)

| Aspect | Assessment |
|--------|------------|
| **Purpose** | Source positions, spans, file abstraction |
| **Maturity** | 8/10 |
| **Public API** | `SourceLocation` (1-based line/col + byte offset), `SourceSpan` (start+end), `SourceFile` (path, content, line starts, snippet extraction) |
| **Quality** | Clean. `snippet()` and `context()` for pretty error display. Binary search for line lookups. |
| **Issues** | `From<Range<usize>>` sets default line/col = 1/1 (caller must resolve). No end-of-file sentinel span. |
| **Tests** | **0 tests** in crate |
| **Missing** | Span merging across files, span for EOF, path normalization |

---

### `motarjim-errors` (370 LOC)

| Aspect | Assessment |
|--------|------------|
| **Purpose** | Diagnostic data model |
| **Maturity** | 8/10 |
| **Public API** | `Diagnostic` (builder pattern: severity, code, message, span, suggestions, notes, hint, docs_url), `DiagnosticBag` (Vec wrapper with push/push_error/push_warning), `Severity` (5 levels), `DiagnosticCode` |
| **Quality** | Excellent builder pattern. Clean `DiagnosticBag` with convenience methods. |
| **Issues** | No tests in crate. `DiagnosticCode::message` is `&'static str` only (no dynamic messages). |
| **Missing** | Child diagnostics, diagnostic phases/phases, auto-fix suggestions with replacements |

---

### `motarjim-diag` (402 LOC)

| Aspect | Assessment |
|--------|------------|
| **Purpose** | Predefined error codes + pretty printer |
| **Maturity** | 7/10 |
| **Public API** | `codes` module (22 error codes E0001-E0712), `DiagnosticEmitter` (feature-gated behind `color`) |
| **Quality** | ANSI-colored terminal output with source snippets. Rustc-style formatting. |
| **Issues** | Only 22 codes for a full compiler. No category organization beyond numeric range. Emitter always writes to stdout (no configurable output). |
| **Tests** | 12 tests (8 for codes, 4 for emitter) |
| **Missing** | JSON diagnostic output, file-based suppression, diagnostic groups |

---

## AST Layer

### `motarjim-ast-html` (1,118 LOC)

| Aspect | Assessment |
|--------|------------|
| **Purpose** | Arena-based HTML AST + ComputedStyle |
| **Maturity** | 7/10 |
| **Public API** | `HtmlNode`, `Document` (flat Vec<HtmlNode> with NodeId indices), `ComputedStyle` (50+ fields), `EdgeValues`, `Background`, `Border`, `SemanticIr` (41 variants), `DisplayType`, `PositionType`, `FlexDirection`, etc. |
| **Quality** | Well-structured. Strong enums for CSS properties. Serialization behind feature flag. |
| **Issues** | `ComputedStyle` uses `Option<String>` for many fields instead of typed values. No `top`/`right`/`bottom`/`left` fields. |
| **Tests** | 12 tests across 3 files |

---

### `motarjim-ast-css` (1,056 LOC)

| Aspect | Assessment |
|--------|------------|
| **Purpose** | CSS AST types |
| **Maturity** | 8/10 |
| **Public API** | `CssStylesheet`, `CssRule` (10 variants: Style, Media, FontFace, Keyframes, Import, Charset, Namespace, Supports, Page, Other), `Selector` (with `matches()` and `specificity()`), `SimpleSelector` (7 variants), `CssValue` (16 variants), `CssUnit` (28 units) |
| **Quality** | Comprehensive. `CssNumber` handles NaN via `to_bits()`. Good Display/FromStr round-trip for units. |
| **Issues** | Duplicate selector types with `motarjim-selectors`. `CssValue::Color` is unstructured (r, g, b, a fields + color_space string). |
| **Tests** | 13 tests across 3 files |

---

### `motarjim-ast-ir` (471 LOC)

| Aspect | Assessment |
|--------|------------|
| **Purpose** | IR types |
| **Maturity** | 7/10 |
| **Public API** | `IrNode`, `IrTree`, `SemanticIr` (41 variants), `LayoutIr` (17 variants), `TargetIr` (4 variants), `TargetHint`, `HintType`, `Breakpoint`, `ResponsiveVariant`, `LayoutConstraints` |
| **Quality** | Clean types. `SmallVec<[NodeId; 4]>` for children. SmolStr for strings. |
| **Issues** | `LayoutStrategy` is dead duplicate enum (14 variants, never used). `LayoutConstraints` never instantiated outside tests. |
| **Tests** | Limited (tested via motarjim-ir tests) |

### `motarjim-ast` (39 LOC)

| Aspect | Assessment |
|--------|------------|
| **Purpose** | Facade re-exporting all ast-* crates |
| **Maturity** | 10/10 |
| **Quality** | Perfect for its role. |

---

## Parsing Layer

### `motarjim-lexer` (874 LOC)

| Aspect | Assessment |
|--------|------------|
| **Purpose** | HTML and CSS tokenizers |
| **Maturity** | 5/10 |
| **Public API** | `Cursor<'a>` (char-by-char with line/col tracking), `Token<T>` (kind, span, raw), `HtmlTokenizer`, `CssTokenizer`, `HtmlTokenKind` (11 variants), `CssTokenKind` (30 variants) |
| **Quality** | Clean `Cursor` abstraction. CSS tokenizer is thorough. |
| **Issues** | HTML tokenizer does NOT produce `AttrName`/`AttrValue` tokens — attributes are embedded in `OpenTagStart` raw string, forcing parser to re-scan. No CDATA handling, no character references. |
| **Tests** | 20 unit tests + 2 proptests |
| **Benchmarks** | Criterion benchmark exists |

---

### `motarjim-html` (2,118 LOC)

| Aspect | Assessment |
|--------|------------|
| **Purpose** | html5ever-based HTML parser |
| **Maturity** | 8/10 |
| **Public API** | `HtmlParser::parse()` → tree-based `Document`/`Fragment`, `Node`, `NodeKind` (5 variants), `ParseError`, `DiagnosticBag` |
| **Quality** | Excellent. Full HTML5 spec compliance via html5ever. Clean converter isolation (only converter module touches html5ever types). |
| **Issues** | Produces tree-based AST incompatible with arena-based AST in `motarjim-ast-html`. Not wired into main pipeline. |
| **Tests** | 34 tests across 3 files |

---

### `motarjim-parser` (3,658 LOC)

| Aspect | Assessment |
|--------|------------|
| **Purpose** | Custom HTML parser + LightningCSS wrapper |
| **Maturity** | 6/10 |
| **Public API** | `HtmlParser`, `CssParser`, `css::parse_css` |
| **Quality** | CSS parser is well-structured (thin wrapper + 1,589 LOC converter). HTML parser works around lexer limitations. |
| **Issues** | HTML parser re-scans raw tag text for attributes. Validation pass has `#[allow(dead_code)]`. |
| **Tests** | 62+ tests (12 HTML, 50 CSS) + 3 proptests |
| **Benchmarks** | Criterion benchmark exists |

---

### `motarjim-js` (6,972 LOC) — Largest Crate

| Aspect | Assessment |
|--------|------------|
| **Purpose** | ECMAScript frontend (lexer, parser, semantic analysis, transforms, DOM events) |
| **Maturity** | 7/10 (crate), 2/10 (pipeline integration) |
| **Public API** | `JsLexer`, `JsParser`, `SemanticAnalyzer`, `Visitor`/`VisitorMut`/`Fold` traits, `find_dom_event_bindings`, `run_transforms` |
| **Quality** | Full Pratt parser with 11 precedence levels. Comprehensive scope tracking (const reassignment, captures, imports). |
| **Issues** | Only 2 unit tests. DOM event bindings extracted but not consumed by IR/generators. Many runtime "not supported" errors (generators, decorators, import.meta). |
| **Tests** | **2 tests** only |
| **Benchmarks** | Criterion benchmark exists |

---

## CSS Engine Layer

### `motarjim-selectors` (1,068 LOC)

| Aspect | Assessment |
|--------|------------|
| **Purpose** | Selector parser + specificity calculator |
| **Maturity** | 8/10 |
| **Public API** | `Selector` (List/Compound/Complex/Simple), `SimpleSelector` (7 variants), `Combinator`, `PseudoClass` (16 variants including :not(), :is(), :where(), :has(), :nth-child()), `PseudoElement` (6 variants), `Specificity` |
| **Quality** | Full recursive-descent parser. Rich pseudo-class support. Clean display trait. |
| **Issues** | Not wired into CSS engine matching (engine uses simplified matching that ignores combinators). |
| **Tests** | Property-based tests with proptest |

---

### `motarjim-css` (1,839 LOC)

| Aspect | Assessment |
|--------|------------|
| **Purpose** | CSS cascade, selector matching, computed style |
| **Maturity** | 6/10 |
| **Public API** | `Cascade`, `StyleResolver`, `ComputedValues`, value parsers (`parse_color`, `parse_length`, `parse_font_weight`) |
| **Quality** | Correct specificity, !important, source order, parent inheritance. Parallel resolution via rayon. |
| **Issues** | Combinator traversal NOT implemented. Pseudo-classes always match. Media/supports never evaluated. `var()` unresolved. Positioning offsets missing. |
| **Tests** | 445 LOC of tests |
| **Benchmarks** | Criterion benchmark (cascade + value parsing) |

---

## IR Layer

### `motarjim-ir` (1,140 LOC)

| Aspect | Assessment |
|--------|------------|
| **Purpose** | IR builder + 4 inference passes |
| **Maturity** | 6/10 |
| **Public API** | `IrBuilder`, `SemanticAnalyzer`, `LayoutInferrer`, `ResponsiveInferrer` (STUB), `AccessibilityInferrer` |
| **Quality** | Clean 4-pass architecture. Comprehensive semantic mapping (HTML tags, ARIA roles). |
| **Issues** | Responsive is stub. TargetIr not populated. Diagnostics parameter ignored. |
| **Tests** | ~30 tests, Criterion benchmark |
| **Missing** | Responsive breakpoints, aria-labelledby resolution, text direction |

---

## Optimization Layer

### `motarjim-optimizer` (2,009 LOC)

| Aspect | Assessment |
|--------|------------|
| **Purpose** | 6 optimization passes via PassManager |
| **Maturity** | 7/10 |
| **Public API** | `PassManager`, `Pass` trait, `PassResult`, `PassContext`, `PassCost`, `register_default_passes` |
| **Quality** | Thread-safe via atomics. Clean trait design. Comprehensive helper utilities. |
| **Issues** | Dependency declarations unused. No parallel execution. `memory_freed` always 0. No optimization levels. |
| **Tests** | 40+ tests, Criterion benchmark |

---

## Output Layer

### `motarjim-formatter` (393 LOC)

| Aspect | Assessment |
|--------|------------|
| **Purpose** | CodeWriter + platform-specific helpers |
| **Maturity** | 9/10 |
| **Public API** | `CodeWriter` (indent, write, write_line, block, write_block, write_stmt), plus `dart`, `kotlin`, `swift` platform modules |
| **Quality** | Clean, well-tested, ergonomic. Line-start tracking prevents leading whitespace. |
| **Issues** | Platform modules unused by generators (generators hand-code output via generic CodeWriter) |
| **Tests** | 13 tests |

### `motarjim-gen-flutter` (1,033 LOC)

| Aspect | Assessment |
|--------|------------|
| **Maturity** | 5/10 |
| **Generated Code** | Valid Dart, hardcoded values. 34/41 semantic variants handled. |
| **CSS Mapping** | Only padding, margin, color, justify-content, align-items |
| **Bugs** | `emit_table_cell` writes `TableRow(` instead of cell widget |
| **Tests** | 14 tests (no snapshot tests) |

### `motarjim-gen-compose` (904 LOC)

| Aspect | Assessment |
|--------|------------|
| **Maturity** | 5/10 |
| **Generated Code** | Valid Kotlin, hardcoded values. Best CSS mapping of the three. |
| **CSS Mapping** | padding, margin, color, width, height, justify-content, align-items |
| **Bugs** | Lists use `listOf(1)` fake data; images require non-standard `coil` library |
| **Tests** | 12 tests (no snapshot tests) |

### `motarjim-gen-swiftui` (783 LOC)

| Aspect | Assessment |
|--------|------------|
| **Maturity** | 4/10 |
| **Generated Code** | Several bugs produce invalid Swift |
| **CSS Mapping** | Only padding and color |
| **Bugs** | `emit_hstack` alignment syntax wrong; `emit_dialog` modifier orphan; `emit_nav_bar` modifier chain invalid |
| **Tests** | 11 tests (no snapshot tests) |

---

## Infrastructure Layer

### `motarjim-fs` (301 LOC)

| Aspect | Assessment |
|--------|------------|
| **Maturity** | 6/10 |
| **API** | `FileSystem` trait (read, exists, list, write, canonicalize), `RealFileSystem`, `VirtualFileSystem`, `FileWatcher` (stub) |
| **Issues** | `VirtualFileSystem.write()` has mutation bug (clones map but doesn't persist). FileWatcher is a stub. |

### `motarjim-config` (480 LOC)

| Aspect | Assessment |
|--------|------------|
| **Maturity** | 6/10 |
| **API** | `Config`, `PlatformConfig`, `GlobalConfig`, `ConfigBuilder`. JSON + TOML support. |
| **Issues** | Manual JSON walking instead of serde Deserialize. TOML via JSON round-trip. No YAML. No env var overrides. |

### `motarjim-session` (680 LOC)

| Aspect | Assessment |
|--------|------------|
| **Maturity** | 7/10 |
| **API** | `Session` (config, diagnostics, source_map, file_system, cache, incremental, profiling, cancellation) |
| **Issues** | Cancellation feature-gated. SourceMap has no path normalization. |

### `motarjim-cache` (372 LOC)

| Aspect | Assessment |
|--------|------------|
| **Maturity** | 6/10 |
| **API** | `ArtifactCache`, `CacheKey` (source_hash + platform + config_hash). Content-addressable disk cache. |
| **Issues** | `ChecksumMismatch` error variant never produced. Not wired into pipeline. |

### `motarjim-incremental` (380 LOC)

| Aspect | Assessment |
|--------|------------|
| **Maturity** | 6/10 |
| **API** | `IncrementalEngine` (file-change tracking via CSV state). `FileState`, `FileChange`. |
| **Issues** | CSV format fragile (no comma escaping). Not wired into pipeline. |

### `motarjim-profiling` (417 LOC)

| Aspect | Assessment |
|--------|------------|
| **Maturity** | 7/10 |
| **API** | `PhaseTimer` (pause/resume), `ProfilingSession`, `TelemetryBus`, `TelemetrySubscriber` trait |
| **Issues** | `trace` and `flamegraph` features are unimplemented placeholders. |

### `motarjim-serialize` (250 LOC)

| Aspect | Assessment |
|--------|------------|
| **Maturity** | 5/10 |
| **API** | JSON serialization for IrTree, config. Binary module is JSON in disguise (placeholder for MessagePack). |
| **Issues** | Not wired into pipeline |

---

## Integration Layer

### `motarjim-core` (3,913 LOC)

| Aspect | Assessment |
|--------|------------|
| **Maturity** | 7/10 |
| **API** | `Compiler` (compile, compile_file, compile_all), `Pipeline`, `CompileResult`, `CompileOptions`, plus feature-gated modules: `dag`, `plugin`, `event`, `query`, `cancellation` |
| **Quality** | Well-designed facade. Event system, plugin system, DAG scheduler, query cache are all production-quality designs. |
| **Issues** | Most features disabled by default. No incremental/cache wiring. |

### `motarjim-cli` (447 LOC)

| Aspect | Assessment |
|--------|------------|
| **Maturity** | 5/10 |
| **API** | 4 commands: compile, watch (stub), init, check. clap-based argument parsing. |
| **Issues** | Watch is stub. Single file input only. Config silently falls back on parse error. No optimization flags. |

### `motarjim-lsp` (389 LOC)

| Aspect | Assessment |
|--------|------------|
| **Maturity** | 5/10 |
| **API** | `Backend` (tower-lsp LanguageServer). Working diagnostics. Stubs: completion, hover, definition, semantic-tokens, code-actions. |
| **Issues** | Hover returns static string. Go-to-definition returns `Ok(None)`. Semantic tokens return empty. |

### `motarjim-ffi` (348 LOC)

| Aspect | Assessment |
|--------|------------|
| **Maturity** | 7/10 |
| **API** | 4 extern "C" functions: new, free, compile, free_string. Standard opaque-pointer pattern. |
| **Quality** | Good safety documentation. Null-pointer checks. JSON error output. |
| **Tests** | 9 tests |

### `motarjim-wasm` (104 LOC)

| Aspect | Assessment |
|--------|------------|
| **Maturity** | 6/10 |
| **API** | `WasmCompiler::new()`, `compile(html, css, platform) -> String`, `version() -> String` |
| **Issues** | Unknown platforms silently default to Dart (no error). No TypeScript types bundled. No tests. |
