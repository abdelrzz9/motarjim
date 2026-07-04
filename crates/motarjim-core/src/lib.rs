#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]
#![allow(unexpected_cfgs)]

//! Main compilation pipeline for the Motarjim compiler.
//!
//! Orchestrates all phases: parse HTML/CSS input, resolve styles, build IR,
//! optimize IR, and generate platform code.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Cancellation token support for long-running operations.
///
/// Provides [`CancelToken`](cancellation::CancelToken) and
/// [`Cancelled`](cancellation::Cancelled) for cooperative cancellation
/// throughout the compilation pipeline.
///
/// This module is only available when the `cancellation` feature is enabled.
#[cfg(feature = "cancellation")]
pub mod cancellation;

#[cfg(feature = "events")]
pub mod event;

#[cfg(feature = "query-system")]
pub mod query;

use motarjim_ast::ir::IrTree;
use motarjim_ast::style::ComputedStyle;
use motarjim_ast::{Document, NodeId};
use motarjim_cache::ArtifactCache;
use motarjim_config::{Config, OutputFormat};
use motarjim_css::StyleResolver;
use motarjim_diag::{Diagnostic, Severity};
#[cfg(feature = "cancellation")]
use motarjim_diag::DiagnosticCode;
use motarjim_fs::FileSystem;
#[cfg(not(feature = "plugin-system"))]
use motarjim_gen_compose::ComposeGenerator;
#[cfg(not(feature = "plugin-system"))]
use motarjim_gen_flutter::FlutterGenerator;
#[cfg(not(feature = "plugin-system"))]
use motarjim_gen_swiftui::SwiftUIGenerator;
use motarjim_incremental::IncrementalEngine;
use motarjim_ir::IrBuilder;
use motarjim_optimizer::{register_default_passes, PassManager};
use motarjim_parser::{CssParser, HtmlParser};
use motarjim_profiling::ProfilingSession;

#[cfg(feature = "plugin-system")]
pub mod plugin;

#[cfg(feature = "dag")]
pub mod dag;

/// Compilation statistics.
#[derive(Debug, Clone, Default)]
pub struct CompileStats {
    /// Total number of HTML nodes parsed.
    pub nodes_parsed: usize,
    /// Number of CSS rules processed.
    pub css_rules: usize,
    /// Number of IR nodes built.
    pub ir_nodes: usize,
    /// Number of optimization passes applied.
    pub optimizations_applied: usize,
    /// Number of nodes removed by optimization.
    pub nodes_removed: usize,
    /// Total number of diagnostics emitted.
    pub diagnostics_count: usize,
    /// Number of errors.
    pub error_count: usize,
    /// Compilation time in milliseconds.
    pub time_ms: u64,
}

/// Options for a single compilation.
#[derive(Debug, Clone)]
pub struct CompileOptions {
    /// Target output platform.
    pub platform: OutputFormat,
    /// Whether to minify the output.
    pub minify: bool,
    /// Whether to generate source maps.
    pub source_maps: bool,
    /// Enable strict mode (warnings as errors).
    pub strict: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            platform: OutputFormat::Dart,
            minify: false,
            source_maps: false,
            strict: false,
        }
    }
}

/// The result of a single compilation.
#[derive(Debug, Clone)]
pub struct CompileResult {
    /// The generated platform code.
    pub output: String,
    /// The parsed HTML AST.
    pub ast: Document,
    /// The built IR tree.
    pub ir: IrTree,
    /// All diagnostics collected during compilation.
    pub diagnostics: Vec<Diagnostic>,
    /// Compilation statistics.
    pub stats: CompileStats,
}

/// Describes a single compilation target.
#[derive(Debug, Clone)]
pub struct CompileTarget {
    /// Path to the input file.
    pub input_path: String,
    /// Compilation options.
    pub options: CompileOptions,
}

/// The main compiler orchestrating the full pipeline.
pub struct Compiler {
    /// Compiler configuration.
    config: Config,
    /// Abstract filesystem for I/O operations.
    fs: Arc<dyn FileSystem>,
    /// Manager for optimization passes.
    pass_manager: PassManager,
    /// Session for collecting profiling data.
    profiling_session: ProfilingSession,
    /// Optional artifact cache.
    cache: Option<ArtifactCache>,
    /// Optional incremental compilation engine.
    incremental: Option<IncrementalEngine>,
    /// Event bus for lifecycle events.
    #[cfg(feature = "events")]
    event_bus: event::EventBus,
    /// Registry of code generators (available when `plugin-system` feature is enabled).
    #[cfg(feature = "plugin-system")]
    generator_registry: plugin::GeneratorRegistry,
    /// Token used to signal and check for cancellation of the current compilation.
    #[cfg(feature = "cancellation")]
    cancel_token: cancellation::CancelToken,
}

impl std::fmt::Debug for Compiler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("Compiler");
        d.field("config", &self.config)
            .field("pass_manager", &format_args!("PassManager({})", self.pass_manager.len()))
            .field("profiling_session", &self.profiling_session.label())
            .field("cache", &self.cache)
            .field("incremental", &self.incremental);
        #[cfg(feature = "events")]
        {
            d.field("event_bus", &format_args!("EventBus({})", self.event_bus.handler_count()));
        }
        #[cfg(feature = "plugin-system")]
        {
            d.field("generator_registry", &format_args!("GeneratorRegistry({})", self.generator_registry.len()));
        }
        #[cfg(feature = "cancellation")]
        {
            d.field("cancel_token", &self.cancel_token.is_cancelled());
        }
        d.finish()
    }
}

impl Compiler {
    /// Create a new compiler with the given configuration and filesystem.
    #[must_use]
    pub fn new(config: Config, fs: Arc<dyn FileSystem>) -> Self {
        let mut pass_manager = PassManager::new();
        register_default_passes(&mut pass_manager);

        let cache = config
            .global
            .cache_dir
            .clone()
            .map(ArtifactCache::new);

        let incremental = if config.global.incremental {
            let dir = config
                .global
                .cache_dir
                .clone()
                .unwrap_or_else(|| std::path::PathBuf::from(".motarjim/cache"));
            Some(IncrementalEngine::new(dir.join("incremental")))
        } else {
            None
        };

        #[cfg(feature = "plugin-system")]
        let mut generator_registry = plugin::GeneratorRegistry::new();
        #[cfg(feature = "plugin-system")]
        plugin::register_builtin_generators(&mut generator_registry);

        Self {
            config,
            fs,
            pass_manager,
            profiling_session: ProfilingSession::new("compilation"),
            cache,
            incremental,
            #[cfg(feature = "events")]
            event_bus: event::EventBus::new(),
            #[cfg(feature = "plugin-system")]
            generator_registry,
            #[cfg(feature = "cancellation")]
            cancel_token: cancellation::CancelToken::new(),
        }
    }

    /// Compile an HTML/CSS input string.
    ///
    /// # Errors
    /// Returns a vector of [`Diagnostic`]s if compilation fails.
    pub fn compile(
        &self,
        input: &str,
        options: &CompileOptions,
    ) -> Result<CompileResult, Vec<Diagnostic>> {
        let mut profiling = ProfilingSession::new("compile");

        // Phase 1: Parse HTML
        #[cfg(feature = "events")]
        self.emit_event(event::CompilerEvent::BeforeParse {
            source: input.to_string(),
        });
        let mut html_timer = profiling.start_phase("parse_html");
        let mut html_parser = HtmlParser::new(input);
        let ast = match html_parser.parse() {
            Ok(doc) => {
                #[cfg(feature = "events")]
                self.emit_event(event::CompilerEvent::AfterParse {
                    result: Ok(doc.clone()),
                });
                doc
            }
            Err(diags) => {
                #[cfg(feature = "events")]
                self.emit_event(event::CompilerEvent::AfterParse {
                    result: Err(diags.clone()),
                });
                return Err(diags);
            }
        };
        profiling.record_phase("parse_html", html_timer.stop());

        #[cfg(feature = "cancellation")]
        self.cancel_token
            .check()
            .map_err(|e| vec![Diagnostic::new(Severity::Error, DiagnosticCode::new(700, "Compilation cancelled"), e.message)])?;

        // Phase 2: Parse CSS from <style> tags
        let mut css_timer = profiling.start_phase("parse_css");
        let css_source = extract_css_from_html(input);
        let stylesheet = css_source.as_ref().and_then(|css_text| {
            let mut css_parser = CssParser::new(css_text);
            css_parser.parse().ok()
        });
        profiling.record_phase("parse_css", css_timer.stop());

        #[cfg(feature = "cancellation")]
        self.cancel_token
            .check()
            .map_err(|e| vec![Diagnostic::new(Severity::Error, DiagnosticCode::new(700, "Compilation cancelled"), e.message)])?;

        // Phase 3: Resolve styles
        #[cfg(feature = "events")]
        let sheet = stylesheet.clone().unwrap_or_else(|| motarjim_ast::css::CssStylesheet {
            rules: vec![],
            source_path: None,
        });
        #[cfg(feature = "events")]
        self.emit_event(event::CompilerEvent::BeforeStyle {
            document: ast.clone(),
            stylesheet: sheet,
        });
        let mut style_timer = profiling.start_phase("resolve_styles");
        let mut resolver = StyleResolver::new();
        if let Some(ref sheet) = stylesheet {
            resolver.add_stylesheet(sheet.clone());
        }
        let style_map: HashMap<NodeId, ComputedStyle> = ast
            .nodes
            .iter()
            .map(|node| {
                let style = node
                    .element
                    .as_ref()
                    .map(|el| resolver.resolve(el).style)
                    .unwrap_or_default();
                (node.id, style)
            })
            .collect();
        #[cfg(feature = "events")]
        {
            let styled_doc = motarjim_ast::StyledDocument {
                nodes: ast
                    .nodes
                    .iter()
                    .map(|node| motarjim_ast::StyledNode {
                        node: node.clone(),
                        computed_style: style_map.get(&node.id).cloned().unwrap_or_default(),
                    })
                    .collect(),
                root_id: ast.root_id,
            };
            self.emit_event(event::CompilerEvent::AfterStyle {
                result: Ok(styled_doc),
            });
        }
        profiling.record_phase("resolve_styles", style_timer.stop());

        #[cfg(feature = "cancellation")]
        self.cancel_token
            .check()
            .map_err(|e| vec![Diagnostic::new(Severity::Error, DiagnosticCode::new(700, "Compilation cancelled"), e.message)])?;

        // Phase 4: Build IR
        #[cfg(feature = "events")]
        {
            let semantic_doc = motarjim_ast::SemanticDocument {
                nodes: ast.nodes.clone(),
                root_id: ast.root_id,
            };
            self.emit_event(event::CompilerEvent::BeforeIr {
                semantic: semantic_doc,
            });
        }
        let mut ir_timer = profiling.start_phase("build_ir");
        let ir_builder = IrBuilder::new();
        let ir_diag = motarjim_diag::DiagnosticBag::new();
        let ir = ir_builder.build(&ast, &style_map, &ir_diag);
        #[cfg(feature = "events")]
        self.emit_event(event::CompilerEvent::AfterIr {
            result: Ok(ir.clone()),
        });
        profiling.record_phase("build_ir", ir_timer.stop());

        #[cfg(feature = "cancellation")]
        self.cancel_token
            .check()
            .map_err(|e| vec![Diagnostic::new(Severity::Error, DiagnosticCode::new(700, "Compilation cancelled"), e.message)])?;

        // Phase 5: Optimize IR
        let mut opt_timer = profiling.start_phase("optimize_ir");
        let mut mutable_ir = ir;
        let opt_results = self.pass_manager.run_all(&mut mutable_ir);
        let nodes_removed: usize = opt_results.iter().map(|r| r.nodes_removed).sum();
        profiling.record_phase("optimize_ir", opt_timer.stop());

        #[cfg(feature = "cancellation")]
        self.cancel_token
            .check()
            .map_err(|e| vec![Diagnostic::new(Severity::Error, DiagnosticCode::new(700, "Compilation cancelled"), e.message)])?;

        // Phase 6: Generate platform code
        #[cfg(any(feature = "events", feature = "plugin-system"))]
        let target_str = match options.platform {
            motarjim_config::OutputFormat::Dart => "flutter",
            motarjim_config::OutputFormat::Kotlin => "compose",
            motarjim_config::OutputFormat::Swift => "swiftui",
        };
        #[cfg(feature = "events")]
        self.emit_event(event::CompilerEvent::BeforeGenerate {
            tree: mutable_ir.clone(),
            target: target_str.to_string(),
        });
        let mut gen_timer = profiling.start_phase("generate");
        #[cfg(feature = "plugin-system")]
        let output = {
            let gen_options = plugin::GenerateOptions {
                minify: options.minify,
                source_maps: options.source_maps,
            };
            self.generator_registry
                .get(target_str)
                .and_then(|g| g.generate(&mutable_ir, &gen_options).ok())
                .unwrap_or_default()
        };
        #[cfg(not(feature = "plugin-system"))]
        let output = generate_for_platform(&mutable_ir, options.platform, options.minify);
        #[cfg(feature = "events")]
        self.emit_event(event::CompilerEvent::AfterGenerate {
            result: Ok(output.clone()),
            target: target_str.to_string(),
        });
        profiling.record_phase("generate", gen_timer.stop());

        // Check options for strict mode
        let all_diagnostics = Vec::new();

        let error_count = 0;
        let total_ms = profiling
            .phases()
            .values()
            .map(|d| d.as_millis() as u64)
            .sum();

        let stats = CompileStats {
            nodes_parsed: ast.nodes.len(),
            css_rules: stylesheet.map_or(0, |s| s.rules.len()),
            ir_nodes: mutable_ir.nodes.len(),
            optimizations_applied: opt_results.len(),
            nodes_removed,
            diagnostics_count: all_diagnostics.len(),
            error_count,
            time_ms: total_ms,
        };

        if options.strict && !all_diagnostics.is_empty() {
            return Err(all_diagnostics);
        }

        Ok(CompileResult {
            output,
            ast,
            ir: mutable_ir,
            diagnostics: all_diagnostics,
            stats,
        })
    }

    /// Compile a file by reading it from the filesystem.
    ///
    /// # Errors
    /// Returns a vector of [`Diagnostic`]s if compilation fails.
    pub fn compile_file(
        &self,
        path: &Path,
        options: &CompileOptions,
    ) -> Result<CompileResult, Vec<Diagnostic>> {
        let entry = self.fs.read(path).map_err(|e| {
            vec![Diagnostic::new(
                Severity::Error,
                motarjim_diag::codes::CONFIG_FILE_NOT_FOUND,
                format!("Failed to read {}: {e}", path.display()),
            )]
        })?;
        self.compile(&entry.content, options)
    }

    /// Compile multiple targets and return results for each.
    #[must_use]
    pub fn compile_all(&self, targets: &[CompileTarget]) -> Vec<Result<CompileResult, Vec<Diagnostic>>> {
        targets
            .iter()
            .map(|target| {
                let path = Path::new(&target.input_path);
                self.compile_file(path, &target.options)
            })
            .collect()
    }

    /// Returns a reference to the compiler's configuration.
    #[must_use]
    pub const fn config(&self) -> &Config {
        &self.config
    }

    /// Returns a reference to the profiling session.
    #[must_use]
    pub const fn profiling_session(&self) -> &ProfilingSession {
        &self.profiling_session
    }

    /// Returns a clone of the cancellation token for sharing with phases.
    #[cfg(feature = "cancellation")]
    #[must_use]
    pub fn cancel_token(&self) -> cancellation::CancelToken {
        self.cancel_token.clone()
    }

    /// Request cancellation of the current compilation.
    #[cfg(feature = "cancellation")]
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    /// Register a third-party generator with the compiler.
    ///
    /// This is only available when the `plugin-system` feature is enabled.
    #[cfg(feature = "plugin-system")]
    pub fn register_generator(&mut self, gen: Box<dyn plugin::Generator>) {
        self.generator_registry.register(gen);
    }

    /// Register a plugin with the compiler.
    ///
    /// The plugin can register one or more generators.
    /// This is only available when the `plugin-system` feature is enabled.
    #[cfg(feature = "plugin-system")]
    pub fn register_plugin(&mut self, plugin: &dyn plugin::Plugin) {
        let mut registry = plugin::PluginRegistry::new();
        plugin.register(&mut registry);
        for gen in registry.into_generators() {
            self.generator_registry.register(gen);
        }
    }

    /// Emit a compiler lifecycle event, ignoring handler errors.
    #[cfg(feature = "events")]
    fn emit_event(&self, event: event::CompilerEvent) {
        let _ = self.event_bus.emit(&event);
    }

    /// Returns a reference to the event bus.
    #[cfg(feature = "events")]
    #[must_use]
    pub fn event_bus(&self) -> &event::EventBus {
        &self.event_bus
    }

    /// Returns a mutable reference to the event bus.
    #[cfg(feature = "events")]
    pub fn event_bus_mut(&mut self) -> &mut event::EventBus {
        &mut self.event_bus
    }
}

/// Extract CSS source from HTML input by looking for `<style>` tags.
fn extract_css_from_html(input: &str) -> Option<String> {
    let mut css_parts: Vec<String> = Vec::new();
    let mut remaining = input;

    while let Some(start) = remaining.find("<style") {
        let after_tag = if let Some(gt) = remaining[start..].find('>') {
            &remaining[start + gt + 1..]
        } else {
            break;
        };

        if let Some(end) = after_tag.find("</style>") {
            css_parts.push(after_tag[..end].to_string());
            remaining = &after_tag[end + 8..];
        } else {
            break;
        }
    }

    if css_parts.is_empty() {
        None
    } else {
        Some(css_parts.join("\n"))
    }
}

/// Generate platform code from the IR tree.
#[cfg(not(feature = "plugin-system"))]
fn generate_for_platform(ir: &IrTree, platform: OutputFormat, _minify: bool) -> String {
    match platform {
        OutputFormat::Dart => {
            let gen = FlutterGenerator::new();
            gen.generate(ir)
        }
        OutputFormat::Kotlin => {
            let gen = ComposeGenerator::new();
            gen.generate(ir)
        }
        OutputFormat::Swift => {
            let gen = SwiftUIGenerator::new();
            gen.generate(ir)
        }
    }
}

/// A pipeline that exposes individual compilation phases for advanced usage.
#[derive(Debug)]
pub struct Pipeline<'a> {
    /// Reference to the compiler driving the pipeline.
    compiler: &'a Compiler,
}

impl<'a> Pipeline<'a> {
    /// Create a new pipeline from a compiler reference.
    #[must_use]
    pub const fn new(compiler: &'a Compiler) -> Self {
        Self { compiler }
    }

    /// Parse HTML input into a document.
    ///
    /// # Errors
    /// Returns diagnostics if parsing fails.
    pub fn parse_html(&self, input: &str) -> Result<Document, Vec<Diagnostic>> {
        let mut parser = HtmlParser::new(input);
        parser.parse()
    }

    /// Parse CSS input into a stylesheet.
    ///
    /// # Errors
    /// Returns diagnostics if parsing fails.
    pub fn parse_css(&self, input: &str) -> Result<motarjim_ast::css::CssStylesheet, Vec<Diagnostic>> {
        let mut parser = CssParser::new(input);
        parser.parse()
    }

    /// Build an IR tree from a document and computed styles.
    #[must_use]
    pub fn build_ir(
        &self,
        doc: &Document,
        styles: &HashMap<NodeId, ComputedStyle>,
    ) -> IrTree {
        let builder = IrBuilder::new();
        let diag = motarjim_diag::DiagnosticBag::new();
        builder.build(doc, styles, &diag)
    }

    /// Run optimization passes on the IR tree.
    pub fn optimize(&self, ir: &mut IrTree) {
        self.compiler.pass_manager.run_all(ir);
    }

    /// Generate platform code from the IR tree.
    #[must_use]
    pub fn generate(&self, ir: &IrTree, platform: OutputFormat) -> String {
        #[cfg(feature = "plugin-system")]
        {
            let target_str = match platform {
                OutputFormat::Dart => "flutter",
                OutputFormat::Kotlin => "compose",
                OutputFormat::Swift => "swiftui",
            };
            let options = plugin::GenerateOptions {
                minify: false,
                source_maps: false,
            };
            return self
                .compiler
                .generator_registry
                .get(target_str)
                .and_then(|g| g.generate(ir, &options).ok())
                .unwrap_or_default();
        }
        #[cfg(not(feature = "plugin-system"))]
        {
            generate_for_platform(ir, platform, false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use motarjim_fs::VirtualFileSystem;

    fn default_config() -> Config {
        Config::new()
    }

    fn test_compiler() -> Compiler {
        let fs = Arc::new(VirtualFileSystem::new());
        Compiler::new(default_config(), fs)
    }

    #[test]
    fn test_compile_simple_html() {
        let compiler = test_compiler();
        let options = CompileOptions::default();
        let html = "<div>Hello</div>";
        let result = compiler.compile(html, &options).unwrap();
        assert!(!result.output.is_empty());
        assert!(result.stats.nodes_parsed > 0);
    }

    #[test]
    fn test_compile_with_css() {
        let compiler = test_compiler();
        let options = CompileOptions::default();
        let html = r#"<html>
        <style>div { color: red; }</style>
        <div>Hello</div>
        </html>"#;
        let result = compiler.compile(html, &options).unwrap();
        assert!(!result.output.is_empty());
        assert!(result.stats.nodes_parsed > 0);
    }

    #[test]
    fn test_compile_to_compose() {
        let compiler = test_compiler();
        let options = CompileOptions {
            platform: OutputFormat::Kotlin,
            minify: false,
            source_maps: false,
            strict: false,
        };
        let html = "<div>Hello</div>";
        let result = compiler.compile(html, &options).unwrap();
        assert!(result.output.contains("import androidx.compose"));
    }

    #[test]
    fn test_compile_to_swiftui() {
        let compiler = test_compiler();
        let options = CompileOptions {
            platform: OutputFormat::Swift,
            minify: false,
            source_maps: false,
            strict: false,
        };
        let html = "<div>Hello</div>";
        let result = compiler.compile(html, &options).unwrap();
        assert!(result.output.contains("import SwiftUI"));
    }

    #[test]
    fn test_compile_empty_input() {
        let compiler = test_compiler();
        let options = CompileOptions::default();
        let result = compiler.compile("", &options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_file() {
        let mut vfs = VirtualFileSystem::new();
        vfs.add_file("test.html", "<div>File test</div>");
        let fs = Arc::new(vfs);
        let compiler = Compiler::new(default_config(), fs);
        let options = CompileOptions::default();
        let result = compiler.compile_file(std::path::Path::new("test.html"), &options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_file_not_found() {
        let fs = Arc::new(VirtualFileSystem::new());
        let compiler = Compiler::new(default_config(), fs);
        let options = CompileOptions::default();
        let result = compiler.compile_file(std::path::Path::new("nonexistent.html"), &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_all() {
        let mut vfs = VirtualFileSystem::new();
        vfs.add_file("a.html", "<div>A</div>");
        vfs.add_file("b.html", "<div>B</div>");
        let fs = Arc::new(vfs);
        let compiler = Compiler::new(default_config(), fs);
        let targets = vec![
            CompileTarget {
                input_path: "a.html".to_string(),
                options: CompileOptions::default(),
            },
            CompileTarget {
                input_path: "b.html".to_string(),
                options: CompileOptions::default(),
            },
        ];
        let results = compiler.compile_all(&targets);
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
    }

    #[test]
    fn test_pipeline_parse_html() {
        let compiler = test_compiler();
        let pipeline = Pipeline::new(&compiler);
        let doc = pipeline.parse_html("<p>Test</p>").unwrap();
        assert!(doc.nodes.len() >= 2);
    }

    #[test]
    fn test_pipeline_parse_css() {
        let compiler = test_compiler();
        let pipeline = Pipeline::new(&compiler);
        let sheet = pipeline.parse_css("div { color: blue; }").unwrap();
        assert_eq!(sheet.rules.len(), 1);
    }

    #[test]
    fn test_extract_css_from_html() {
        let html = r#"<html><style>div { color: red; }</style></html>"#;
        let css = extract_css_from_html(html);
        assert!(css.is_some());
        assert!(css.unwrap().contains("color: red"));
    }

    #[test]
    fn test_compile_result_stats() {
        let compiler = test_compiler();
        let options = CompileOptions::default();
        let html = "<div><p>Nested</p></div>";
        let result = compiler.compile(html, &options).unwrap();
        assert!(result.stats.nodes_parsed > 1);
    }

    #[test]
    fn test_compile_options_default() {
        let opts = CompileOptions::default();
        assert_eq!(opts.platform, OutputFormat::Dart);
        assert!(!opts.minify);
        assert!(!opts.source_maps);
        assert!(!opts.strict);
    }
}
