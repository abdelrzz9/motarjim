# Testing Guide

## Testing Philosophy

motarjim follows a multi-layered testing strategy inspired by the compiler infrastructure community:

1. **Unit tests** — Every crate has fine-grained unit tests covering individual functions and modules. Target: 95% line coverage.
2. **Integration tests** — End-to-end tests that exercise the full compilation pipeline from HTML/CSS input to generated code.
3. **Golden tests** — Generated output is compared against stored "golden" files. Changes are reviewed and updated explicitly.
4. **Property-based tests** — Random inputs test invariants and edge cases beyond hand-written tests.
5. **Fuzz tests** — Long-running fuzzing finds crash-inducing inputs and regressions.
6. **Benchmarks** — Performance regression detection via Criterion benchmarks.

## Running Tests

### Rust Tests

```bash
# All tests (default)
cargo test --workspace

# A specific crate
cargo test -p motarjim-parser

# A specific test name
cargo test -p motarjim-core compile_simple

# With release profile (slower compilation, faster test execution)
cargo test --release

# With nextest (parallel, per-test output)
cargo install cargo-nextest
cargo nextest run

# With test coverage
cargo install cargo-tarpaulin
cargo tarpaulin --workspace
```

### Golden Tests

Golden tests compare generated output against expected files stored in `crates/motarjim-test-utils/tests/golden/`.

```bash
# Run golden tests
cargo test golden

# Update golden files (when output intentionally changes)
UPDATE_EXPECT=1 cargo test golden

# Review golden changes
git diff crates/motarjim-test-utils/tests/golden/
```

### Fuzz Tests

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Run fuzz targets
cargo fuzz run html_parser   # Fuzz HTML parser with random byte sequences
cargo fuzz run css_parser    # Fuzz CSS parser
cargo fuzz run html_lexer    # Fuzz HTML lexer
cargo fuzz run css_lexer     # Fuzz CSS lexer
cargo fuzz run selector_parser  # Fuzz selector parser

# Run for specific number of iterations
cargo fuzz run html_parser -- -runs=100000

# Minimize a crashing input
cargo fuzz minimize html_parser crash-xxx
```

### Benchmarks

```bash
# All benchmarks
cargo bench --workspace

# Specific benchmark
cargo bench -p motarjim-parser

# Compare against baseline
cargo bench --workspace -- --baseline main
```

## Writing Tests

### Unit Tests (Rust)

Tests live in a `#[cfg(test)] mod tests` block at the bottom of each source file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_div() {
        let mut parser = HtmlParser::new("<div>hello</div>");
        let doc = parser.parse().expect("parse should succeed");
        assert_eq!(doc.nodes.len(), 2); // root + div
        assert_eq!(doc.nodes[1].tag_name, "div");
    }

    #[test]
    fn test_empty_input() {
        let mut parser = HtmlParser::new("");
        let result = parser.parse();
        assert!(result.is_ok());
    }
}
```

### Integration Tests

Integration tests live in `tests/` directories at crate root and test the public API:

```rust
// crates/motarjim-core/tests/compile_test.rs
#[test]
fn test_end_to_end_navigation_bar() {
    let result = compile(CompilerOptions {
        html: SourceFile::from_path("tests/html/nav-bar.html"),
        css: Some(SourceFile::from_path("tests/css/nav-bar.css")),
        target: Target::Flutter,
        ..Default::default()
    });
    assert!(result.diagnostics.is_empty());
    assert!(result.code.dart.unwrap().contains("AppBar"));
}
```

### Golden Tests

Golden tests live in `crates/motarjim-test-utils/tests/golden_test.rs`:

```rust
#[test]
fn golden_flutter_card() {
    let input_html = include_str!("../tests/golden/html/card.html");
    let input_css = include_str!("../tests/golden/css/card.css");

    let compiler = test_compiler();
    let result = compiler.compile(&format!("<style>{input_css}</style>\n{input_html}"), &default_options()).unwrap();

    insta::assert_snapshot!("flutter_card", result.output);
}
```

### Property-Based Tests

Use `proptest` for property-based testing:

```rust
// crates/motarjim-parser/src/proptests.rs
proptest! {
    #[test]
    fn html_roundtrip(html in html_strategy()) {
        let mut parser = HtmlParser::new(&html);
        let doc = parser.parse().expect("parse should not panic");
        // Verify structural invariants
        prop_assert!(doc.nodes.iter().all(|n| n.id.0 > 0));
        prop_assert!(doc.root_id.0 > 0);
    }

    #[test]
    fn css_does_not_panic(css in css_strategy()) {
        let mut parser = CssParser::new(&css);
        let _ = parser.parse(); // Must not panic
    }
}
```

## Test Structure

### Directory Layout

```
motarjim/
├── crates/
│   ├── motarjim-parser/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── html.rs
│   │   │   ├── css.rs
│   │   │   ├── proptests.rs     # Property-based tests
│   │   │   └── ...
│   │   ├── tests/
│   │   │   └── integration.rs    # Integration tests
│   │   ├── benches/
│   │   │   └── parser_bench.rs  # Criterion benchmarks
│   │   └── examples/
│   │       └── basics.rs        # Runnable examples
│   └── motarjim-test-utils/
│       └── tests/
│           └── golden_test.rs    # Golden comparison tests
├── fuzz/
│   ├── Cargo.toml
│   └── fuzz_targets/
│       ├── html_parser.rs
│       ├── css_parser.rs
│       ├── html_lexer.rs
│       ├── css_lexer.rs
│       └── selector_parser.rs
```

### Per-Crate Test Coverage Targets

| Crate | Unit Tests | Integration | Coverage Target |
|-------|-----------|-------------|-----------------|
| `motarjim-lexer` | Tokenize HTML/CSS | — | 95% |
| `motarjim-parser` | Parse tokens → AST | Round-trip | 95% |
| `motarjim-selectors` | Parse, specificity, match | — | 95% |
| `motarjim-css` | Parse, cascade, computed | Style resolution | 90% |
| `motarjim-ir` | Build IR from styled nodes | — | 90% |
| `motarjim-optimizer` | Each pass individually | Pipeline | 95% |
| `motarjim-gen-flutter` | Widget emission | Golden output | 90% |
| `motarjim-gen-compose` | Widget emission | Golden output | 90% |
| `motarjim-gen-swiftui` | Widget emission | Golden output | 90% |
| `motarjim-core` | — | End-to-end | 85% |
| `motarjim-cli` | Argument parsing | File I/O | 90% |

## Fixtures and Golden Files

### Golden File Structure

```
crates/motarjim-test-utils/tests/golden/
├── html/
│   ├── simple-div.html
│   ├── nested-elements.html
│   ├── form-with-inputs.html
│   ├── navigation-bar.html
│   ├── card-grid.html
│   ├── hero-section.html
│   ├── ecommerce-product.html
│   ├── dashboard-layout.html
│   └── blog-article.html
├── css/
│   ├── simple-rules.css
│   ├── cascade-specificity.css
│   ├── media-queries.css
│   ├── flexbox.css
│   └── responsive.css
├── output/
│   ├── flutter/
│   ├── compose/
│   └── swiftui/
└── ir/
    └── *.json
```

### Updating Golden Files

When generated output changes intentionally:

```bash
UPDATE_EXPECT=1 cargo test golden
```

This updates all golden files. Always review the diff before committing:

```bash
git diff crates/motarjim-test-utils/tests/golden/
```

### Adding New Golden Tests

1. Add input HTML to `crates/motarjim-test-utils/tests/golden/html/`
2. Add CSS to `crates/motarjim-test-utils/tests/golden/css/` (if needed)
3. Create a test function in `crates/motarjim-test-utils/tests/golden_test.rs`
4. Run with `UPDATE_EXPECT=1` to generate initial golden output
5. Verify the output manually
6. Commit both the test and golden files

## Benchmarking

### Criterion Benchmarks

Benchmarks live in `benches/` directories within each crate:

```rust
// crates/motarjim-parser/benches/parser_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_parse_small(c: &mut Criterion) {
    let html = "<div>Hello</div>";
    c.bench_function("parse-small-html", |b| {
        b.iter(|| {
            let mut parser = HtmlParser::new(black_box(html));
            parser.parse().unwrap();
        })
    });
}
```

### Running Benchmarks

```bash
# All benchmarks
cargo bench --workspace

# Compare with previous run
cargo bench --workspace -- --baseline main

# HTML report
# Open target/criterion/report/index.html
```

### Benchmark Categories

| Benchmark | What It Measures |
|-----------|-----------------|
| `parse_small` | 50-node HTML document |
| `parse_medium` | 500-node HTML document |
| `parse_large` | 5000-node HTML document |
| `cascade_small` | 50 nodes, 10 CSS rules |
| `cascade_medium` | 500 nodes, 100 CSS rules |
| `build_ir_small` | 50 styled nodes → IR |
| `optimize_small` | IR optimization passes |
| `generate_flutter` | Full IR → Dart generation |
| `pipeline_small` | End-to-end small page |
| `pipeline_large` | End-to-end large page |

### Performance Regression Policy

- **Soft limit**: 10% regression triggers a warning in PR comments
- **Hard limit**: 25% regression blocks the PR
- **Exceptions**: Performance regressions that enable new features or fix correctness bugs are evaluated case-by-case

## Fuzz Testing

### Targets

| Target | Description |
|--------|-------------|
| `html_parser` | Fuzz the HTML parser with random byte sequences |
| `css_parser` | Fuzz the CSS parser |
| `html_lexer` | Fuzz the HTML tokenizer |
| `css_lexer` | Fuzz the CSS tokenizer |
| `selector_parser` | Fuzz the CSS selector parser |

### Running

```bash
# Run forever (default)
cargo fuzz run html_parser

# Run with corpus
cargo fuzz run html_parser fuzz/corpus/html_parser

# Run specific number of iterations
cargo fuzz run html_parser -- -runs=100000
```

### Adding a New Fuzz Target

1. Create a new file in `fuzz/fuzz_targets/`
2. Add it to `fuzz/Cargo.toml`
3. Implement the `fuzz_target!` macro:

```rust
// fuzz/fuzz_targets/js_lexer.rs
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let mut lexer = motarjim_js::JsLexer::new(s);
        let _ = lexer.tokenize();
    }
});
```

## CI Pipeline

### GitHub Actions

```yaml
name: Test
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Test
        run: cargo test --workspace

      - name: Clippy
        run: cargo clippy --workspace -- -D warnings

      - name: Format
        run: cargo fmt --check

      - name: Bench
        run: cargo bench --workspace
```

### Pre-Merge Checklist

- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` is clean
- [ ] `cargo fmt --check` passes
- [ ] New features have tests (unit + integration)
- [ ] Golden files are updated if output changed
- [ ] Performance benchmarks don't regress beyond limits
