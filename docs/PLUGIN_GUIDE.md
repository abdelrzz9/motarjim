# Plugin Development Guide

## Plugin Architecture

motarjim's plugin system allows third-party developers to add new platform generators without modifying the compiler core. The system is built on two traits in `motarjim-core::plugin`:

```
┌─────────────────────────────────────────────┐
│            Plugin System Architecture        │
│                                             │
│  ┌──────────────────┐                       │
│  │   Generator      │  Trait for platform   │
│  │   Trait          │  code generators      │
│  └────────┬─────────┘                       │
│           │                                  │
│           │ implements                       │
│           ▼                                  │
│  ┌──────────────────┐                       │
│  │  FlutterGenerator│  Built-in generators  │
│  │  ComposeGenerator│  (included by default) │
│  │  SwiftUIGenerator│                       │
│  └──────────────────┘                       │
│           │                                  │
│           │ implements                       │
│           ▼                                  │
│  ┌──────────────────┐                       │
│  │  Plugin Trait    │  Groups generators +  │
│  │                  │  future capabilities   │
│  └────────┬─────────┘                       │
│           │                                  │
│           ▼                                  │
│  ┌──────────────────┐                       │
│  │  PluginRegistry  │  Registration hub     │
│  └──────────────────┘                       │
└─────────────────────────────────────────────┘
```

All plugin-related types are gated behind the `plugin-system` Cargo feature.

## Creating a Plugin

### Step 1: Create a new crate

```bash
cargo new motarjim-gen-react-native --lib
cd motarjim-gen-react-native
```

### Step 2: Add dependencies

```toml
# Cargo.toml
[package]
name = "motarjim-gen-react-native"
version = "0.1.0"
edition = "2021"

[dependencies]
motarjim-ast = { git = "https://github.com/motarjim/motarjim.git" }
motarjim-diag = { git = "https://github.com/motarjim/motarjim.git" }
motarjim-ir = { git = "https://github.com/motarjim/motarjim.git" }
motarjim-formatter = { git = "https://github.com/motarjim/motarjim.git" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### Step 3: Implement the Generator trait

```rust
// src/lib.rs
use motarjim_ast::ir::IrTree;
use motarjim_diag::Diagnostic;
use motarjim_formatter::CodeWriter;

pub struct ReactNativeGenerator;

impl motarjim_core::plugin::Generator for ReactNativeGenerator {
    fn name(&self) -> &'static str {
        "react-native"
    }

    fn generate(
        &self,
        ir: &IrTree,
        options: &motarjim_core::plugin::GenerateOptions,
    ) -> Result<String, Vec<Diagnostic>> {
        let mut writer = CodeWriter::new(CodeWriter::default_options());

        // Emit imports
        writer.write_line("import React from 'react';");
        writer.write_line("import { View, Text, Pressable } from 'react-native';");
        writer.newline();

        // Emit component
        writer.write_line("export default function GeneratedView() {");
        writer.indent();
        writer.write_line("return (");
        writer.indent();

        // Walk IR tree and emit
        emit_node(&mut writer, ir.root(), 0);

        writer.dedent();
        writer.write_line(");");
        writer.dedent();
        writer.write_line("}");

        Ok(writer.to_string())
    }
}

fn emit_node(writer: &mut CodeWriter, node: &IrNode, depth: usize) {
    match &node.semantic {
        SemanticIr::Text => {
            let text = escape_js_string(&node.computed_style.content_text);
            writer.write_line(&format!("<Text>{text}</Text>"));
        }
        SemanticIr::Button => {
            writer.write_line("<Pressable onPress={() => {}}>");
            writer.indent();
            for child_id in &node.children {
                // Recursively emit children
            }
            writer.dedent();
            writer.write_line("</Pressable>");
        }
        SemanticIr::Container | SemanticIr::Root => {
            writer.write_line("<View>");
            writer.indent();
            for child_id in &node.children {
                // Recursively emit children
            }
            writer.dedent();
            writer.write_line("</View>");
        }
        // ... handle other semantic types
        _ => {}
    }
}

fn escape_js_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\'', "\\'")
}
```

### Step 4: Implement the Plugin trait (optional)

To group your generator with future capabilities:

```rust
pub struct ReactNativePlugin;

impl motarjim_core::plugin::Plugin for ReactNativePlugin {
    fn name(&self) -> &'static str {
        "react-native"
    }

    fn register(&self, registry: &mut motarjim_core::plugin::PluginRegistry) {
        registry.register_generator(Box::new(ReactNativeGenerator));
    }
}
```

### Step 5: Register with the compiler

When `plugin-system` feature is enabled, register generators via the `Compiler`:

```rust
use motarjim_core::{CompileOptions, Compiler};
use motarjim_config::Config;
use motarjim_fs::RealFileSystem;
use std::sync::Arc;

fn main() {
    let config = Config::new();
    let fs = Arc::new(RealFileSystem::new());
    let mut compiler = Compiler::new(config, fs);

    // Register third-party generator
    compiler.register_generator(Box::new(
        motarjim_gen_react_native::ReactNativeGenerator
    ));

    // Or via plugin
    compiler.register_plugin(&motarjim_gen_react_native::ReactNativePlugin);

    let options = CompileOptions {
        platform: motarjim_config::OutputFormat::Dart, // Still needed for default
        ..Default::default()
    };

    // The compiler now supports "react-native" as a target
    let result = compiler.compile("<div>Hello</div>", &options);
}
```

## Plugin API Reference

### `Generator` Trait

```rust
/// Trait implemented by all platform code generators.
pub trait Generator: Send + Sync {
    /// Unique name (e.g., "flutter", "react-native").
    fn name(&self) -> &'static str;

    /// Generate platform code from the IR tree.
    fn generate(
        &self,
        ir: &IrTree,
        options: &GenerateOptions,
    ) -> Result<String, Vec<Diagnostic>>;
}
```

### `GenerateOptions`

```rust
pub struct GenerateOptions {
    pub minify: bool,
    pub source_maps: bool,
}
```

### `GeneratorRegistry`

```rust
impl GeneratorRegistry {
    pub fn new() -> Self;
    pub fn register(&mut self, gen: Box<dyn Generator>);
    pub fn get(&self, name: &str) -> Option<&dyn Generator>;
    pub fn all(&self) -> &[Box<dyn Generator>];
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

### `Plugin` Trait

```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn register(&self, registry: &mut PluginRegistry);
}
```

### `PluginRegistry`

```rust
impl PluginRegistry {
    pub fn new() -> Self;
    pub fn register_generator(&mut self, gen: Box<dyn Generator>);
    pub fn register_plugin(&mut self, plugin: &dyn Plugin);
    pub fn generator_registry(&self) -> &GeneratorRegistry;
    pub fn into_generators(self) -> Vec<Box<dyn Generator>>;
}
```

### Compiler Plugin Registration

```rust
impl Compiler {
    /// Register a generator with the compiler.
    /// Available only with the `plugin-system` feature.
    pub fn register_generator(&mut self, gen: Box<dyn Generator>);

    /// Register a plugin with the compiler.
    /// Available only with the `plugin-system` feature.
    pub fn register_plugin(&mut self, plugin: &dyn Plugin);
}
```

## Registering Generators

### Without Plugin System (Default)

When `plugin-system` is disabled, built-in generators are selected via match statement in `generate_for_platform()`. Third-party generators cannot be added without modifying `motarjim-core`.

### With Plugin System

When `plugin-system` feature is enabled:

```toml
# Cargo.toml
[dependencies]
motarjim-core = { git = "https://github.com/motarjim/motarjim.git", features = ["plugin-system"] }
```

The compiler uses the `GeneratorRegistry` for dispatch. Register any number of generators via `Compiler::register_generator()` or `Compiler::register_plugin()`.

## Building and Testing Plugins

### Build

```bash
cargo build -p motarjim-gen-react-native
```

### Test with Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use motarjim_ast::ir::{IrNode, IrTree, LayoutIr, SemanticIr, TargetIr};
    use motarjim_ast::style::ComputedStyle;
    use motarjim_ast::NodeId;

    #[test]
    fn test_generates_text() {
        let tree = make_single_node_tree(SemanticIr::Text);
        let gen = ReactNativeGenerator;
        let result = gen.generate(&tree, &GenerateOptions {
            minify: false,
            source_maps: false,
        });
        assert!(result.is_ok());
        assert!(result.unwrap().contains("<Text>"));
    }

    fn make_single_node_tree(semantic: SemanticIr) -> IrTree {
        IrTree {
            nodes: vec![IrNode {
                id: NodeId(0),
                semantic,
                layout: LayoutIr::Inline,
                target: TargetIr::Generic {
                    platform: smol_str::SmolStr::new_inline("test"),
                    node: smol_str::SmolStr::new_inline("Test"),
                },
                computed_style: ComputedStyle::default(),
                children: vec![],
                parent: None,
            }],
            root_id: NodeId(0),
            target_hints: vec![],
        }
    }
}
```

### Integration Test with Full Pipeline

```rust
#[test]
fn test_plugin_end_to_end() {
    let config = Config::new();
    let fs = Arc::new(VirtualFileSystem::new());
    let mut compiler = Compiler::new(config, fs);

    compiler.register_generator(Box::new(ReactNativeGenerator));

    let result = compiler.compile(
        "<button>Click</button>",
        &CompileOptions {
            platform: OutputFormat::Dart, // Platform is irrelevant with plugins
            ..Default::default()
        },
    );

    assert!(result.is_ok());
    let output = result.unwrap().output;
    assert!(output.contains("<Pressable"));
    assert!(output.contains("onPress={() => {}}"));
}
```

## IR Reference for Plugin Authors

### `IrTree` Structure

```rust
pub struct IrTree {
    pub nodes: Vec<IrNode>,
    pub root_id: NodeId,
    pub target_hints: Vec<String>,
}
```

### `IrNode` Fields

```rust
pub struct IrNode {
    pub id: NodeId,
    pub semantic: SemanticIr,       // Semantic role (Button, Text, Card, etc.)
    pub layout: LayoutIr,           // Layout strategy (FlexColumn, Grid, etc.)
    pub target: TargetIr,           // Platform hints (for built-in generators)
    pub computed_style: ComputedStyle, // Resolved CSS styles
    pub children: Vec<NodeId>,      // Child node IDs (reference nodes in the tree)
    pub parent: Option<NodeId>,     // Parent node ID
}
```

### `SemanticIr` Variants

```rust
pub enum SemanticIr {
    Root,
    Container,
    Text,
    Button,
    Row,
    Column,
    Card,
    Image,
    TextField,
    NavigationBar,
    AppBar,
    ScrollView,
    Form,
    Footer,
    List,
    ListItem,
    Icon,
    Divider,
    Spacer,
    Dialog,
    Modal,
    Tabs,
    Section,
    Article,
    Header,
    Link,
    Grid,
    Stack,
    Unknown,
}
```

### `LayoutIr` Variants

```rust
pub enum LayoutIr {
    Inline,
    FlexColumn,
    FlexRow,
    Grid,
    Stack,
    Absolute,
    Scrollable,
    None,
}
```

### `ComputedStyle` Access

```rust
impl ComputedStyle {
    pub fn get(&self, property: &str) -> Option<&str>;
    pub fn color(&self) -> Option<Color>;
    pub fn font_size(&self) -> Option<Length>;
    pub fn padding(&self) -> Option<Rect>;
    pub fn margin(&self) -> Option<Rect>;
    pub fn border_radius(&self) -> Option<Length>;
    pub fn background(&self) -> Option<Background>;
    pub fn content_text(&self) -> String;
}
```

## Example Plugins

### React Native Generator (Conceptual)

A React Native generator maps `SemanticIr` to React Native components:

```rust
fn map_to_rn_component(semantic: &SemanticIr) -> &'static str {
    match semantic {
        SemanticIr::Container | SemanticIr::Root => "View",
        SemanticIr::Text => "Text",
        SemanticIr::Button => "Pressable",
        SemanticIr::Image => "Image",
        SemanticIr::TextField => "TextInput",
        SemanticIr::ScrollView => "ScrollView",
        SemanticIr::List => "FlatList",
        _ => "View",
    }
}
```

## Best Practices

1. **Use `motarjim-formatter`'s `CodeWriter`** for consistent indentation and formatting. Each generator inherits consistent code style.
2. **Generate valid syntax** — Verify output with the target platform's compiler/linter.
3. **Handle all `SemanticIr` variants** — Use `SemanticIr::Unknown` as a fallback, but handle as many types as practical.
4. **Respect `GenerateOptions`** — Support `minify` and `source_maps` where applicable.
5. **Report diagnostics** — Return `Vec<Diagnostic>` with clear messages for unsupported features or errors.
6. **Test with golden files** — Store expected output and compare.
7. **Feature gate your generator** — Make it optional behind a Cargo feature flag.
