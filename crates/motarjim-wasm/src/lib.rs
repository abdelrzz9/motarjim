//! WASM bindings for the Motarjim compiler.
//!
//! This crate provides JavaScript bindings for the compiler via wasm-bindgen,
//! enabling the compiler to run in web browsers.

#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

use motarjim_core::{CompileOptions, Compiler};
use wasm_bindgen::prelude::*;

/// WASM wrapper for the Motarjim compiler.
#[wasm_bindgen]
pub struct WasmCompiler {
    /// The inner compiler instance.
    inner: Compiler,
    /// Compiler configuration.
    #[allow(dead_code)]
    config: motarjim_config::Config,
}

impl Default for WasmCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl WasmCompiler {
    /// Create a new compiler instance.
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new() -> Self {
        let config = motarjim_config::Config::new();
        let fs = std::sync::Arc::new(motarjim_fs::RealFileSystem);
        let inner = Compiler::new(config.clone(), fs);
        Self { inner, config }
    }

    /// Compile HTML to the specified platform.
    /// Returns JSON string with result or error.
    #[wasm_bindgen]
    pub fn compile(&self, html: &str, css: Option<String>, platform: &str) -> String {
        let options = CompileOptions {
            platform: match platform {
                "flutter" => motarjim_config::OutputFormat::Dart,
                "compose" => motarjim_config::OutputFormat::Kotlin,
                "swiftui" => motarjim_config::OutputFormat::Swift,
                _ => motarjim_config::OutputFormat::Dart,
            },
            ..Default::default()
        };

        // Build combined input with style tag
        let input = if let Some(css_text) = &css {
            format!("<style>{css_text}</style>\n{html}")
        } else {
            html.to_string()
        };

        match self.inner.compile(&input, &options) {
            Ok(result) => {
                #[derive(serde::Serialize)]
                struct Success {
                    success: bool,
                    code: String,
                    stats: serde_json::Value,
                }
                serde_json::to_string(&Success {
                    success: true,
                    code: result.output,
                    stats: serde_json::json!({
                        "nodes": result.stats.nodes_parsed,
                        "css_rules": result.stats.css_rules,
                        "time_ms": result.stats.time_ms,
                    }),
                })
                .unwrap_or_else(|e| {
                    format!("{{\"success\":false,\"error\":\"Serialize error: {e}\"}}")
                })
            }
            Err(diagnostics) => {
                let errors: Vec<String> = diagnostics
                    .iter()
                    .map(|d| format!("{:?}: {}", d.severity, d.message))
                    .collect();
                serde_json::json!({
                    "success": false,
                    "errors": errors,
                })
                .to_string()
            }
        }
    }

    /// Get version info.
    #[wasm_bindgen]
    #[must_use]
    pub fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}
