# Style Guide

## Rust Style Conventions

### Formatting

Rust code is formatted with `cargo fmt` using the default settings. Run before every commit:

```bash
cargo fmt
cargo fmt --check  # CI check
```

### Naming

| Element | Convention | Example |
|---------|-----------|---------|
| Types (structs, enums, traits) | `PascalCase` | `HtmlNode`, `IrTree`, `SemanticIr` |
| Functions | `snake_case` | `parse_html()`, `build_ir()` |
| Methods | `snake_case` | `generator.generate()` |
| Variables | `snake_case` | `node_count`, `styled_nodes` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_DEPTH`, `DEFAULT_PASSES` |
| Modules | `snake_case` | `motarjim_parser`, `motarjim_css` |
| Crates | `snake_case` with hyphens in Cargo.toml | `motarjim-gen-flutter` |
| Type parameters | short `PascalCase` | `<T>`, `<E>`, `<N: AsRef<str>>` |
| Lifetimes | short `lowercase` | `'a`, `'src` |

### Documentation

- All public items must have doc comments (`///`).
- Use `#![deny(missing_docs)]` at the crate level.
- Doc comments explain *what* and *why*, not *how* (the code explains *how*).
- Include at least one code example in doc comments for public functions.
- Use backticks for code references within doc comments.
- Document panics, errors, and safety invariants explicitly.

```rust
/// Build an IR tree from a parsed document and computed styles.
///
/// The builder walks the document tree and produces an [`IrTree`] where
/// each node carries semantic, layout, and target hints.
///
/// # Errors
///
/// Returns a [`DiagnosticBag`] if the document contains unrecoverable
/// structural issues.
///
/// # Example
///
/// ```rust
/// let builder = IrBuilder::new();
/// let ir = builder.build(&doc, &styles, &diag);
/// ```
pub fn build(&self, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>, diag: &DiagnosticBag) -> IrTree;
```

### Linting

Workspace-wide lint rules (in `Cargo.toml`):

```toml
[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "allow", priority = -1 }
nursery = { level = "allow", priority = -1 }
unwrap_used = "allow"
panic = "deny"
missing_docs_in_private_items = "deny"
```

Per-crate attributes:

```rust
#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]
```

### Unsafe Code

`unsafe` is **forbidden** by default (`#![forbid(unsafe_code)]`). If unsafe is absolutely necessary:

1. Gate behind a `cfg(feature = "unsafe-optimizations")` feature flag
2. Provide a safe fallback when the feature is disabled
3. Document safety invariants with `// SAFETY:` comments
4. Get explicit approval from maintainers

### Error Handling

- Use `Result<T, Vec<Diagnostic>>` for compiler operations (not custom error types).
- Avoid panics. Use `.expect()` only in test code and infallible operations.
- Use `anyhow` only in binary crates (`motarjim-cli`), never in library crates.
- Library crates use `motarjim_diag` types for all error reporting.

### Ownership and Borrowing

- Prefer `&str` over `String` for function parameters.
- Prefer `&[T]` over `&Vec<T>` for slice parameters.
- Use `SmolStr` for small, frequently cloned strings (tag names, attribute names).
- Use `Arc` for shared ownership across threads.
- Use `Box<dyn Trait>` for trait objects.
- Avoid `Rc<RefCell<T>>` — prefer `Arc<Mutex<T>>` or restructuring.

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explicit_name() {
        // Arrange
        // Act
        // Assert
    }
}
```

- Use descriptive test names that explain the scenario.
- Follow Arrange-Act-Assert pattern with blank line separation.
- Use `proptest` for property-based tests.
- Use `insta` for snapshot/golden tests.
- Keep tests fast — avoid filesystem I/O (use `VirtualFileSystem`).

## TypeScript Style Conventions

### Formatting

TypeScript is formatted with Prettier:

```bash
npm run format
```

### Configuration

```json
{
  "compilerOptions": {
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "noImplicitReturns": true,
    "noFallthroughCasesInSwitch": true,
    "exactOptionalPropertyTypes": true
  }
}
```

### Naming

| Element | Convention | Example |
|---------|-----------|---------|
| Types/Interfaces | `PascalCase` | `HtmlNode`, `UiNode`, `NodeEmitter` |
| Functions | `camelCase` | `parseHtml()`, `detectSemantics()` |
| Variables | `camelCase` | `styledNodes`, `optimizedTree` |
| Constants | `UPPER_SNAKE_CASE` | `DEFAULT_PLATFORM` |
| Files | `kebab-case` | `css-analyzer.ts`, `generator-core.ts` |
| Packages | `kebab-case` | `@motarjim/playground-sdk` |

### Type System

- **No `any`**. Use `unknown` when the type is genuinely unknown, then narrow.
- Prefer interfaces over type aliases for object shapes.
- Use `readonly` for immutable properties.
- Use `as const` for literal types.
- Use `satisfies` operator for type validation without widening.

```typescript
// Good
interface CompileResult {
  readonly code: string;
  readonly diagnostics: readonly Diagnostic[];
}

// Bad
interface CompileResult {
  code: string;
  diagnostics: any[];
}
```

### Imports

- Use named imports, not default imports:
  ```typescript
  import { parseHtml } from '@motarjim/parser';
  ```
- Use `import type` for type-only imports:
  ```typescript
  import type { HtmlNode } from '@motarjim/parser';
  ```
- No barrel files (`index.ts` that re-exports everything). Import directly from the module.

### Asynchronous Code

Use `async/await` consistently. Avoid raw promises and callbacks:

```typescript
// Good
async function compile(input: string): Promise<CompileResult> {
  const result = await compiler.compile(input);
  return result;
}

// Avoid
function compile(input: string): Promise<CompileResult> {
  return compiler.compile(input).then(r => r);
}
```

## Naming Conventions

### File Names

- Rust source: `snake_case.rs` (e.g., `html_parser.rs`, `computed_style.rs`)
- TypeScript source: `kebab-case.ts` (e.g., `css-analyzer.ts`, `generator-core.ts`)
- Test files: `<name>.test.ts` for Vitest, `<name>_test.rs` for Rust integration tests
- Benchmark files: `bench_<name>.rs` for Criterion

### Directory Names

- Rust crates: `motarjim-<name>` (hyphenated, matches Cargo.toml package name)
- TypeScript packages: `@motarjim/<name>` in npm scope, directory: `<name>` or `<name>-sdk`
- Source: `src/`
- Tests: `tests/`
- Benchmarks: `benches/`
- Examples: `examples/`

### Git Branches

```
feature/<description>
fix/<description>
refactor/<description>
docs/<description>
bench/<description>
```

## Documentation Conventions

### Code Comments

- Prefer self-documenting code over comments.
- Use comments to explain *why*, not *what*.
- `// TODO:` for planned but unimplemented work.
- `// HACK:` for workarounds that should be revisited.
- `// NOTE:` for important context.
- `// SAFETY:` before unsafe blocks explaining safety invariants.

### Markdown Documentation

- Use `#` for title, `##` for sections, `###` for subsections.
- Code blocks specify language: ` ```rust`, ` ```typescript`, ` ```bash`, ` ```json`.
- Use tables for structured data.
- Use relative links for cross-references: `[ARCHITECTURE.md](../ARCHITECTURE.md)`.
- One sentence per line in markdown source for cleaner diffs.

### Doc Comments (Rust)

```rust
/// Short description (one line).
///
/// Detailed explanation. Multiple paragraphs if needed.
///
/// # Examples
///
/// ```rust
/// // Example code here
/// ```
///
/// # Errors
///
/// Describes error conditions.
///
/// # Panics
///
/// Describes panicking conditions (rare).
///
/// # Safety
///
/// For unsafe functions, documents safety invariants.
```

## Testing Conventions

### Test Structure

- Unit tests: `#[cfg(test)] mod tests` at bottom of each source file.
- Integration tests: Separate file in `tests/` directory.
- Property-based tests: Separate file named `proptests.rs` in `src/`.
- Golden tests: In `motarjim-test-utils/tests/golden_test.rs`.
- Fuzz targets: In `fuzz/fuzz_targets/`.

### Test Naming

```rust
#[test]
fn test_<method>_<scenario>() {
    // e.g.,
    // test_parse_empty_input
    // test_build_ir_with_nested_elements
    // test_generate_flutter_card_component
}
```

### What to Test

1. **Happy path** — Normal expected input produces correct output.
2. **Edge cases** — Empty input, single element, maximum nesting, special characters.
3. **Error handling** — Invalid input produces appropriate diagnostics, not panics.
4. **Invariants** — Properties that must always hold (e.g., tree structure, diagnostic codes).
5. **Round-trips** — Parse → serialize → parse produces the same AST.
6. **Performance** — Critical paths have benchmarks with regression limits.
