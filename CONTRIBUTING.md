# Contributing

## Development Environment Setup

### Prerequisites

- **Rust** 1.75+ (install via [rustup](https://rustup.rs/))
- **Cargo tools**: `cargo-fuzz`, `cargo-nextest` (optional but recommended)

### Clone and Build

```bash
git clone https://github.com/abdelrzz9/motarjim.git
cd motarjim

# Build the entire Rust workspace
cargo build --workspace

# Build the release binary
cargo build --release -p motarjim-cli

# Verify everything works
cargo test --workspace
```

### Editor Setup

We recommend VS Code with:
- [rust-analyzer](https://rust-analyzer.github.io/) for Rust development
- [EditorConfig](https://editorconfig.org/) for consistent formatting (`.editorconfig` is in the repo root)

## Building

### Rust Workspace

```bash
# Build all crates
cargo build --workspace

# Build a specific crate (faster)
cargo build -p motarjim-core

# Build with release optimizations
cargo build --release -p motarjim-cli
```

## Testing

### Rust Tests

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p motarjim-parser

# Run tests matching a name pattern
cargo test -p motarjim-core compile

# Run with release profile (slower compilation, faster execution)
cargo test --release

# Run with nextest (if installed)
cargo nextest run
```

### Linting and Formatting

```bash
# Rust
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

### Benchmarking

```bash
# Rust benchmarks (Criterion)
cargo bench --workspace
```

### Fuzz Testing

```bash
# Install cargo-fuzz if not already installed
cargo install cargo-fuzz

# Run fuzz targets
cargo fuzz run html_parser -- -runs=100000
cargo fuzz run css_parser
cargo fuzz run selector_parser
```

## Code Style and Conventions

### Rust

- **`#![deny(missing_docs)]`** on all public items. Every public function, struct, trait, and module must have a doc comment.
- **`#![forbid(unsafe_code)]`** unless benchmark-proven necessary. `unsafe` requires explicit justification and a `// SAFETY:` comment.
- **`clippy::all`** enforced in CI. Fix all warnings before committing.
- **Conventional formatting**: `cargo fmt` (use nightly channel for `imports_granularity`).
- **Naming**: `snake_case` for functions/variables, `PascalCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants.
- **Error handling**: Use `Result<T, Vec<Diagnostic>>` for compiler operations. Avoid panics (gated by `#[deny(panic)]` workspace lint).
- **Ownership**: Prefer borrowed references (`&str`, `&[T]`) over owned values. Use `SmolStr` for small strings, `Arc` for shared ownership.
- **Testing**: Unit tests in each crate's `tests` module. Integration tests in `tests/` at crate root. Use `proptest` for property-based testing.

### Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types:
- `feat` — New feature
- `fix` — Bug fix
- `refactor` — Code change that neither fixes nor adds
- `test` — Adding or updating tests
- `docs` — Documentation changes
- `bench` — Benchmark changes
- `chore` — Maintenance, build, dependencies

Examples:
```
feat(parser): add support for custom elements
fix(generator-flutter): emit valid Dart property names
refactor(motarjim-core): extract cancellation logic
test(motarjim-selectors): add nth-child specificity tests
docs(architecture): update pipeline diagram
bench: add 5000-node compilation benchmark
```

### Branch Naming

- `feature/<description>` — New features
- `fix/<description>` — Bug fixes
- `refactor/<description>` — Refactoring
- `docs/<description>` — Documentation

## Pull Request Process

1. **Create a feature branch** from `main`:
   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make changes** with clear, conventional commit messages.

3. **Run all checks locally**:
   ```bash
   cargo test --workspace
   cargo clippy --workspace -- -D warnings
   cargo fmt --check
   ```

4. **Benchmark** if performance-sensitive:
   ```bash
   cargo bench --workspace
   ```

5. **Open a PR** with a description of:
   - What the change does
   - Why it's needed (link to issue if applicable)
   - How it was tested
   - Any breaking changes or migration steps

6. **Address review feedback**. All conversations must be resolved before merge.

7. **Squash merge** into `main`. The squashed commit message becomes the canonical record.

## Issue Reporting

### Bug Reports

Include:

- **Environment**: OS, Rust version (`rustc --version`), cargo version
- **Input**: Minimal HTML/CSS that reproduces the issue
- **Expected behavior**: What should happen
- **Actual behavior**: What happens instead (include error output)
- **Additional context**: Screenshots, stack traces, related issues

### Feature Requests

Include:

- **Use case**: What you're trying to accomplish
- **Proposed solution**: How you envision the feature working
- **Alternatives**: What workarounds you've considered
- **Priority**: How important this is to your workflow

## Code Review Guidelines

### For Authors

- Keep PRs focused. One feature/fix per PR. Large PRs should be split into logical chunks.
- Respond to review comments within 3 business days.
- Update the PR description if the scope changes during review.

### For Reviewers

- Be constructive and specific. "This approach won't scale" is less helpful than "This naive O(n²) matching will be slow for 10K nodes; consider using a hashmap lookup."
- Approve when the code is correct, tested, and maintainable — not when it matches your personal style.
- Flag `unsafe` code, performance regressions, and missing tests as blocking items.

## Documentation

- Update `docs/` for any behavior changes.
- Add doc comments (`///`) to all new public API surfaces.
- Update `CHANGELOG.md` for notable changes (under `[Unreleased]`).
- Run `cargo doc --no-deps --workspace` to verify documentation builds without warnings.

## Getting Help

- Open a [GitHub Discussion](https://github.com/abdelrzz9/motarjim/discussions)
- Check existing [issues](https://github.com/abdelrzz9/motarjim/issues)

## Good First Issues

Areas where contributions are especially welcome:

1. **Wiring `motarjim-js` DOM events into the IR/generators** — The seam exists (`find_dom_event_bindings`) but nothing downstream consumes it yet.
2. **CSS value mapping** — Colors, padding/margin shorthands, typography values per platform.
3. **Documentation** — Updating stale docs against the current Rust implementation.
4. **`motarjim watch`** — File watching and incremental recompilation.
5. **Fuzz targets** — Adding `motarjim-js` fuzz targets to match the existing HTML/CSS targets.
