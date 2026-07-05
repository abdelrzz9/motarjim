//! Plugin system for the Motarjim compiler.
//!
//! Provides a pluggable generator architecture: generators are registered
//! via the [`Generator`] trait and can be composed through the [`Plugin`]
//! system. Built-in generators (Flutter, Compose, SwiftUI) are included
//! as default plugins.
//!
//! This module is gated behind the `plugin-system` feature flag.

use motarjim_ast::ir::IrTree;
use motarjim_diag::Diagnostic;

/// Options passed to generators when generating platform code.
#[derive(Debug, Clone)]
pub struct GenerateOptions {
    /// Whether to minify the generated code.
    pub minify: bool,
    /// Whether to generate source maps.
    pub source_maps: bool,
}

/// Trait implemented by all generators (Flutter, Compose, SwiftUI, etc.).
///
/// # Example
///
/// ```rust
/// # use motarjim_core::plugin::{Generator, GenerateOptions};
/// # use motarjim_ast::ir::IrTree;
/// # use motarjim_diag::Diagnostic;
/// struct MyGenerator;
///
/// impl Generator for MyGenerator {
///     fn name(&self) -> &'static str {
///         "my-generator"
///     }
///
///     fn generate(&self, _ir: &IrTree, _options: &GenerateOptions) -> Result<String, Vec<Diagnostic>> {
///         Ok(String::new())
///     }
/// }
/// ```
pub trait Generator: Send + Sync {
    /// The unique name of this generator (e.g., `"flutter"`, `"compose"`).
    fn name(&self) -> &'static str;

    /// Generate platform code from the given IR tree.
    ///
    /// # Errors
    /// Returns a vector of [`Diagnostic`]s if generation fails.
    fn generate(&self, ir: &IrTree, options: &GenerateOptions) -> Result<String, Vec<Diagnostic>>;
}

/// Registry of all registered generators.
///
/// Generators are keyed by their [`name`](Generator::name) and can be
/// looked up at code-generation time.
pub struct GeneratorRegistry {
    /// Registered generators, in registration order.
    generators: Vec<Box<dyn Generator>>,
}

impl GeneratorRegistry {
    /// Creates a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            generators: Vec::new(),
        }
    }

    /// Registers a generator with this registry.
    pub fn register(&mut self, gen: Box<dyn Generator>) {
        self.generators.push(gen);
    }

    /// Looks up a generator by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&dyn Generator> {
        self.generators
            .iter()
            .find(|g| g.name() == name)
            .map(Box::as_ref)
    }

    /// Returns a reference to all registered generators.
    #[must_use]
    pub fn all(&self) -> &[Box<dyn Generator>] {
        &self.generators
    }

    /// Returns the number of registered generators.
    #[must_use]
    pub fn len(&self) -> usize {
        self.generators.len()
    }

    /// Returns `true` if no generators are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.generators.is_empty()
    }

    /// Consumes the registry and returns the contained generators.
    #[must_use]
    pub fn into_vec(self) -> Vec<Box<dyn Generator>> {
        self.generators
    }
}

impl Default for GeneratorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait implemented by all plugins.
///
/// A plugin can register one or more generators (and in the future, other
/// capabilities) into a [`PluginRegistry`].
pub trait Plugin: Send + Sync {
    /// The unique name of this plugin.
    fn name(&self) -> &'static str;

    /// Registers this plugin's generators with the given registry.
    fn register(&self, registry: &mut PluginRegistry);
}

/// Registry for plugins and generators.
///
/// Serves as the integration point where plugins register their generators.
pub struct PluginRegistry {
    /// The generator registry plugins populate.
    generators: GeneratorRegistry,
}

impl PluginRegistry {
    /// Creates a new empty plugin registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            generators: GeneratorRegistry::new(),
        }
    }

    /// Registers a generator with the inner generator registry.
    pub fn register_generator(&mut self, gen: Box<dyn Generator>) {
        self.generators.register(gen);
    }

    /// Returns a reference to the inner generator registry.
    #[must_use]
    pub fn generator_registry(&self) -> &GeneratorRegistry {
        &self.generators
    }

    /// Returns a mutable reference to the inner generator registry.
    #[must_use]
    pub fn generator_registry_mut(&mut self) -> &mut GeneratorRegistry {
        &mut self.generators
    }

    /// Registers a plugin, allowing it to add its generators.
    pub fn register_plugin(&mut self, plugin: &dyn Plugin) {
        plugin.register(self);
    }

    /// Consumes the registry and returns the registered generators.
    #[must_use]
    pub fn into_generators(self) -> Vec<Box<dyn Generator>> {
        self.generators.into_vec()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Built-in generator plugins
// ---------------------------------------------------------------------------

/// Built-in plugin wrapping the Flutter/Dart generator.
pub struct FlutterGeneratorPlugin;

/// Built-in plugin wrapping the Jetpack Compose/Kotlin generator.
pub struct ComposeGeneratorPlugin;

/// Built-in plugin wrapping the SwiftUI generator.
pub struct SwiftUIGeneratorPlugin;

impl Generator for FlutterGeneratorPlugin {
    fn name(&self) -> &'static str {
        "flutter"
    }

    fn generate(&self, ir: &IrTree, _options: &GenerateOptions) -> Result<String, Vec<Diagnostic>> {
        let gen = motarjim_gen_flutter::FlutterGenerator::new();
        Ok(gen.generate(ir))
    }
}

impl Generator for ComposeGeneratorPlugin {
    fn name(&self) -> &'static str {
        "compose"
    }

    fn generate(&self, ir: &IrTree, _options: &GenerateOptions) -> Result<String, Vec<Diagnostic>> {
        let gen = motarjim_gen_compose::ComposeGenerator::new();
        Ok(gen.generate(ir))
    }
}

impl Generator for SwiftUIGeneratorPlugin {
    fn name(&self) -> &'static str {
        "swiftui"
    }

    fn generate(&self, ir: &IrTree, _options: &GenerateOptions) -> Result<String, Vec<Diagnostic>> {
        let gen = motarjim_gen_swiftui::SwiftUIGenerator::new();
        Ok(gen.generate(ir))
    }
}

/// Register the three built-in generators into a [`GeneratorRegistry`].
pub fn register_builtin_generators(registry: &mut GeneratorRegistry) {
    registry.register(Box::new(FlutterGeneratorPlugin));
    registry.register(Box::new(ComposeGeneratorPlugin));
    registry.register(Box::new(SwiftUIGeneratorPlugin));
}

#[cfg(test)]
mod tests {
    use super::*;
    use motarjim_ast::ir::{IrNode, IrTree, LayoutIr, SemanticIr, TargetIr};
    use motarjim_ast::NodeId;
    use motarjim_ast_html::ComputedStyle;
    use smol_str::SmolStr;

    fn make_node(
        id: u32,
        semantic: SemanticIr,
        layout: LayoutIr,
        children: Vec<u32>,
        parent: Option<u32>,
    ) -> IrNode {
        IrNode {
            id: NodeId(id),
            semantic,
            layout,
            target: TargetIr::Generic {
                platform: SmolStr::new_inline("test"),
                node: SmolStr::new_inline("Test"),
            },
            computed_style: ComputedStyle::default(),
            children: children.into_iter().map(NodeId).collect(),
            parent: parent.map(NodeId),
        }
    }

    fn make_tree(nodes: Vec<IrNode>, root_id: u32) -> IrTree {
        IrTree {
            nodes,
            root_id: NodeId(root_id),
            target_hints: vec![],
        }
    }

    #[test]
    fn test_generator_registry_register_and_get() {
        let mut registry = GeneratorRegistry::new();
        assert!(registry.is_empty());

        registry.register(Box::new(FlutterGeneratorPlugin));
        let gen = registry.get("flutter");
        assert!(gen.is_some());
        assert_eq!(gen.unwrap().name(), "flutter");
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
    }

    #[test]
    fn test_generator_registry_get_unknown() {
        let registry = GeneratorRegistry::new();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_generator_registry_all() {
        let mut registry = GeneratorRegistry::new();
        registry.register(Box::new(FlutterGeneratorPlugin));
        registry.register(Box::new(ComposeGeneratorPlugin));
        let all = registry.all();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_generator_registry_len_and_empty() {
        let mut registry = GeneratorRegistry::new();
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());

        registry.register(Box::new(FlutterGeneratorPlugin));
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
    }

    #[test]
    fn test_plugin_registration() {
        struct TestPlugin;

        impl Plugin for TestPlugin {
            fn name(&self) -> &'static str {
                "test"
            }

            fn register(&self, registry: &mut PluginRegistry) {
                registry.register_generator(Box::new(FlutterGeneratorPlugin));
            }
        }

        let mut plugin_registry = PluginRegistry::new();
        plugin_registry.register_plugin(&TestPlugin);
        assert!(plugin_registry
            .generator_registry()
            .get("flutter")
            .is_some());
    }

    #[test]
    fn test_builtin_flutter_generator() {
        let tree = make_tree(
            vec![make_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                vec![],
                None,
            )],
            0,
        );
        let options = GenerateOptions {
            minify: false,
            source_maps: false,
        };

        let plugin = FlutterGeneratorPlugin;
        let result = plugin.generate(&tree, &options);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("import 'package:flutter/material.dart';"));
    }

    #[test]
    fn test_builtin_compose_generator() {
        let tree = make_tree(
            vec![make_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                vec![],
                None,
            )],
            0,
        );
        let options = GenerateOptions {
            minify: false,
            source_maps: false,
        };

        let plugin = ComposeGeneratorPlugin;
        let result = plugin.generate(&tree, &options);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("import androidx.compose"));
    }

    #[test]
    fn test_builtin_swiftui_generator() {
        let tree = make_tree(
            vec![make_node(
                0,
                SemanticIr::Root,
                LayoutIr::FlexColumn,
                vec![],
                None,
            )],
            0,
        );
        let options = GenerateOptions {
            minify: false,
            source_maps: false,
        };

        let plugin = SwiftUIGeneratorPlugin;
        let result = plugin.generate(&tree, &options);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("import SwiftUI"));
    }

    #[test]
    fn test_register_builtin_generators() {
        let mut registry = GeneratorRegistry::new();
        register_builtin_generators(&mut registry);
        assert_eq!(registry.len(), 3);
        assert!(registry.get("flutter").is_some());
        assert!(registry.get("compose").is_some());
        assert!(registry.get("swiftui").is_some());
    }

    #[test]
    fn test_unknown_generator_name() {
        let registry = GeneratorRegistry::new();
        assert!(registry.get("react-native").is_none());
    }

    #[test]
    fn test_plugin_registry_into_generators() {
        let mut plugin_registry = PluginRegistry::new();
        plugin_registry.register_generator(Box::new(FlutterGeneratorPlugin));
        plugin_registry.register_generator(Box::new(ComposeGeneratorPlugin));

        let generators = plugin_registry.into_generators();
        assert_eq!(generators.len(), 2);
    }

    #[test]
    fn test_generator_registry_default() {
        let registry = GeneratorRegistry::default();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_plugin_registry_default() {
        let registry = PluginRegistry::default();
        assert!(registry.generator_registry().is_empty());
    }
}
