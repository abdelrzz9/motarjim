# Release Guide

## Version Numbering

motarjim follows [Semantic Versioning 2.0.0](https://semver.org/):

- **MAJOR** (1.0.0, 2.0.0) — Breaking changes to the public API or generated output format
- **MINOR** (0.1.0, 0.2.0) — New features, new generators, new platforms
- **PATCH** (0.1.1, 0.1.2) — Bug fixes, performance improvements, documentation

Current version: `0.1.0`

### Version Strategy

- The workspace root `motarjim` crate is not published (`publish = false`).
- Individual crates use the same version (kept in sync via `workspace.package`).
- Breaking changes to a single crate may not require a major version bump for the whole workspace.

## Release Process

### 1. Prepare the Release Branch

```bash
git checkout main
git pull origin main
git checkout -b release/v0.2.0
```

### 2. Update Versions

Update all crate versions:

```bash
# Using cargo-edit
cargo install cargo-edit
cargo set-version 0.2.0 --workspace

# Or manually edit the [workspace.package] section in root Cargo.toml
# version = "0.2.0"
```

### 3. Update Changelog

Move changes from `[Unreleased]` to a new version section in `CHANGELOG.md`:

```markdown
## [0.2.0] - 2026-07-04

### Added

- New feature description
- ...

### Fixed

- Bug fix description
- ...

### Changed

- Breaking change description with migration notes
- ...

## [0.1.0] - 2026-06-01
```

### 4. Update Documentation

- Update version references in `README.md`, `CLI_GUIDE.md`, etc.
- Verify all examples work with the new version.
- Update any screenshot/images if output changed.

### 5. Run Full Test Suite

```bash
# Rust workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check

# Benchmarks (verify no regressions)
cargo bench --workspace

# Fuzz tests (quick check)
cargo fuzz run html_parser -- -runs=10000
```

### 6. Create Release Commit

```bash
git add -A
git commit -m "chore: release v0.2.0"
git push origin release/v0.2.0
```

### 7. Open and Merge Release PR

Open a pull request from `release/v0.2.0` to `main`. The PR should:

- Summarize all changes since the last release
- Note any breaking changes and migration steps
- List new contributors

After review and CI passes, merge to `main`.

### 8. Tag the Release

```bash
git checkout main
git pull origin main
git tag -a v0.2.0 -m "v0.2.0"
git push origin v0.2.0
```

### 9. Create GitHub Release

Go to [GitHub Releases](https://github.com/motarjim/motarjim/releases/new):

- Select the tag `v0.2.0`
- Title: `v0.2.0`
- Description: Copy the changelog entry for this version
- Attach any binaries (optional)

## Changelog Generation

### Manual Updates

Update `CHANGELOG.md` as changes are merged:

```markdown
## [Unreleased]

### Added

- Feature description (#PR-number)

### Fixed

- Bug description (#PR-number)

### Changed

- Breaking change description with migration notes (#PR-number)
```

### Automatic Generation (Future)

Planned: Use `git-cliff` or similar tool to generate changelogs from conventional commits:

```bash
cargo install git-cliff
git cliff --output CHANGELOG.md
```

## Publishing to crates.io

### Prerequisites

- Account on [crates.io](https://crates.io)
- API token: `cargo login`
- All crates pass `cargo publish --dry-run`

### Publishing Order

Crates must be published in dependency order (leaf crates first):

```bash
# Layer 1: No workspace dependencies
cargo publish -p motarjim-diag
cargo publish -p motarjim-ast
cargo publish -p motarjim-profiling
cargo publish -p motarjim-fs

# Layer 2: Depends on Layer 1
cargo publish -p motarjim-config
cargo publish -p motarjim-serialize
cargo publish -p motarjim-lexer
cargo publish -p motarjim-formatter

# Layer 3: Depends on Layer 2
cargo publish -p motarjim-parser
cargo publish -p motarjim-selectors
cargo publish -p motarjim-css
cargo publish -p motarjim-ir

# Layer 4: Depends on Layer 3
cargo publish -p motarjim-optimizer
cargo publish -p motarjim-gen-flutter
cargo publish -p motarjim-gen-compose
cargo publish -p motarjim-gen-swiftui

# Layer 5: Depends on Layer 4
cargo publish -p motarjim-cache
cargo publish -p motarjim-incremental

# Layer 6: Depends on Layer 5
cargo publish -p motarjim-core

# Layer 7: Depends on Layer 6
cargo publish -p motarjim-cli
cargo publish -p motarjim-lsp
cargo publish -p motarjim-wasm
cargo publish -p motarjim-ffi
```

### Publishing Script

```bash
#!/bin/bash
# publish.sh - Publish all crates in dependency order

set -e

CRATES=(
  motarjim-diag
  motarjim-ast
  motarjim-profiling
  motarjim-fs
  motarjim-config
  motarjim-serialize
  motarjim-lexer
  motarjim-formatter
  motarjim-parser
  motarjim-selectors
  motarjim-css
  motarjim-ir
  motarjim-optimizer
  motarjim-gen-flutter
  motarjim-gen-compose
  motarjim-gen-swiftui
  motarjim-cache
  motarjim-incremental
  motarjim-core
  motarjim-cli
  motarjim-lsp
  motarjim-wasm
  motarjim-ffi
)

for crate in "${CRATES[@]}"; do
  echo "Publishing $crate..."
  cargo publish -p "$crate"
  echo "Waiting for crates.io index to update..."
  sleep 30
done

echo "All crates published!"
```

### Dry Run

Before publishing, verify everything works:

```bash
cargo publish -p motarjim-diag --dry-run
cargo publish -p motarjim-ast --dry-run
# ... etc
```

## Creating GitHub Releases

### Manual Process

1. Go to [GitHub Releases](https://github.com/motarjim/motarjim/releases/new)
2. Select the version tag
3. Title: `v0.2.0`
4. Paste the changelog entry
5. Attach binaries (if applicable):
   - `motarjim-x86_64-linux.tar.gz`
   - `motarjim-x86_64-macos.tar.gz`
   - `motarjim-x86_64-windows.zip`
6. Publish release

### Automated with GitHub CLI

```bash
gh release create v0.2.0 \
  --title "v0.2.0" \
  --notes "Release notes from changelog" \
  ./target/release/motarjim
```

### Building Release Binaries

```bash
# Linux
cargo build --release -p motarjim-cli
tar czf motarjim-x86_64-linux.tar.gz -C target/release motarjim

# macOS (cross-compile from Linux)
cargo build --release --target x86_64-apple-darwin -p motarjim-cli
tar czf motarjim-x86_64-macos.tar.gz -C target/x86_64-apple-darwin/release motarjim

# Windows (cross-compile from Linux)
cargo build --release --target x86_64-pc-windows-gnu -p motarjim-cli
zip motarjim-x86_64-windows.zip target/x86_64-pc-windows-gnu/release/motarjim.exe
```

## Post-Release

### Announce the Release

- [ ] Post on [GitHub Discussions](https://github.com/motarjim/motarjim/discussions)
- [ ] Tweet/post on social media (if project has presence)
- [ ] Notify community channels

### Update Development Version

After the release, bump the version for ongoing development:

```bash
cargo set-version 0.3.0-dev --workspace
```

Add new `[Unreleased]` section to `CHANGELOG.md`.

## Release Checklist

- [ ] All CI tests pass
- [ ] All benchmarks pass with no regressions
- [ ] Changelog is complete and accurate
- [ ] Version numbers updated across workspace
- [ ] Documentation updated for new version
- [ ] Breaking changes documented with migration guide
- [ ] Release branch reviewed and merged
- [ ] Git tag created and pushed
- [ ] GitHub release created
- [ ] Crates published to crates.io
- [ ] Release announced
