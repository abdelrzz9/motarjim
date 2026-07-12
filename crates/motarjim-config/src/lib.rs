#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! Configuration loading and management for the Motarjim compiler.
//!
//! Handles loading configuration from JSON/TOML files, merging multiple
//! config sources (CLI args, project config, defaults), and resolving
//! platform-specific settings.
//!
//! # Example
//!
//! ```rust
//! use motarjim_config::{Config, PlatformConfig};
//!
//! let config = Config::default();
//! assert!(!config.platforms.is_empty());
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Top-level compiler configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Platform configurations keyed by platform name.
    pub platforms: HashMap<String, PlatformConfig>,
    /// Global compiler settings.
    pub global: GlobalConfig,
}

/// Platform-specific configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformConfig {
    /// The output format for this platform.
    pub format: OutputFormat,
    /// Output directory path.
    pub output_dir: PathBuf,
    /// Whether to minify the output.
    pub minify: bool,
    /// Whether to generate source maps.
    pub source_maps: bool,
    /// Custom platform options.
    pub options: HashMap<String, String>,
}

/// Output format options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Dart/Flutter code generation.
    Dart,
    /// Kotlin/Compose code generation.
    Kotlin,
    /// `SwiftUI` code generation.
    Swift,
}

/// Global compiler settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalConfig {
    /// Enable verbose output.
    pub verbose: bool,
    /// Enable strict mode (warnings as errors).
    pub strict: bool,
    /// Maximum number of parallel tasks.
    pub max_parallel: usize,
    /// Cache directory path.
    pub cache_dir: Option<PathBuf>,
    /// Whether to enable incremental compilation.
    pub incremental: bool,
    /// Viewport width for media query evaluation (default 1920).
    pub viewport_width: u32,
    /// Viewport height for media query evaluation (default 1080).
    pub viewport_height: u32,
    /// Preferred color scheme for `prefers-color-scheme` media feature (default "light").
    pub prefers_color_scheme: String,
    /// Custom global options.
    pub options: HashMap<String, String>,
}

/// Configuration error types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// File not found.
    FileNotFound(String),
    /// Parse error with details.
    ParseError(String),
    /// Invalid configuration value.
    InvalidValue(String),
    /// Unsupported format.
    UnsupportedFormat(String),
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    /// Creates a default configuration.
    #[must_use]
    pub fn new() -> Self {
        let mut platforms = HashMap::new();
        platforms.insert(
            "flutter".to_string(),
            PlatformConfig {
                format: OutputFormat::Dart,
                output_dir: PathBuf::from("output/flutter"),
                minify: false,
                source_maps: false,
                options: HashMap::new(),
            },
        );
        platforms.insert(
            "compose".to_string(),
            PlatformConfig {
                format: OutputFormat::Kotlin,
                output_dir: PathBuf::from("output/compose"),
                minify: false,
                source_maps: false,
                options: HashMap::new(),
            },
        );
        platforms.insert(
            "swiftui".to_string(),
            PlatformConfig {
                format: OutputFormat::Swift,
                output_dir: PathBuf::from("output/swiftui"),
                minify: false,
                source_maps: false,
                options: HashMap::new(),
            },
        );
        Self {
            platforms,
            global: GlobalConfig {
                verbose: false,
                strict: false,
                max_parallel: 4,
                cache_dir: None,
                incremental: true,
                viewport_width: 1920,
                viewport_height: 1080,
                prefers_color_scheme: "light".to_string(),
                options: HashMap::new(),
            },
        }
    }

    /// Load configuration from a JSON file.
    ///
    /// # Errors
    /// Returns `ConfigError` if the file cannot be read or parsed.
    pub fn from_json_file(path: &Path) -> Result<Self, ConfigError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| ConfigError::FileNotFound(e.to_string()))?;
        Self::from_json(&content)
    }

    /// Parse configuration from a JSON string.
    ///
    /// # Errors
    /// Returns `ConfigError` if the JSON is invalid.
    pub fn from_json(json: &str) -> Result<Self, ConfigError> {
        let value: serde_json::Value =
            serde_json::from_str(json).map_err(|e| ConfigError::ParseError(e.to_string()))?;
        Self::from_value(&value)
    }

    /// Parse configuration from a TOML string.
    ///
    /// # Errors
    /// Returns `ConfigError` if the TOML is invalid.
    pub fn from_toml(toml_str: &str) -> Result<Self, ConfigError> {
        let toml_value: toml::Value =
            toml::from_str(toml_str).map_err(|e| ConfigError::ParseError(e.to_string()))?;
        let json_value =
            serde_json::to_value(toml_value).map_err(|e| ConfigError::ParseError(e.to_string()))?;
        Self::from_value(&json_value)
    }

    /// Deserializes a `Config` from a `serde_json::Value`.
    fn from_value(value: &serde_json::Value) -> Result<Self, ConfigError> {
        let mut config = Self::default();

        if let Some(obj) = value.as_object() {
            if let Some(platforms) = obj.get("platforms").and_then(|v| v.as_object()) {
                for (name, pcfg) in platforms {
                    if let Some(pobj) = pcfg.as_object() {
                        let format = match pobj
                            .get("format")
                            .and_then(|v| v.as_str())
                            .unwrap_or("dart")
                        {
                            "dart" | "flutter" => OutputFormat::Dart,
                            "kotlin" | "compose" => OutputFormat::Kotlin,
                            "swift" | "swiftui" => OutputFormat::Swift,
                            other => {
                                return Err(ConfigError::InvalidValue(format!(
                                    "unknown format: {other}"
                                )));
                            }
                        };
                        let output_dir = pobj
                            .get("output_dir")
                            .and_then(|v| v.as_str())
                            .map_or_else(|| PathBuf::from(format!("output/{name}")), PathBuf::from);
                        let minify = pobj
                            .get("minify")
                            .and_then(serde_json::Value::as_bool)
                            .unwrap_or(false);
                        let source_maps = pobj
                            .get("source_maps")
                            .and_then(serde_json::Value::as_bool)
                            .unwrap_or(false);
                        let options = pobj
                            .get("options")
                            .and_then(|v| v.as_object())
                            .map(|o| {
                                o.iter()
                                    .filter_map(|(k, v)| {
                                        v.as_str().map(|s| (k.clone(), s.to_string()))
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();

                        config.platforms.insert(
                            name.clone(),
                            PlatformConfig {
                                format,
                                output_dir,
                                minify,
                                source_maps,
                                options,
                            },
                        );
                    }
                }
            }

            if let Some(global) = obj.get("global").and_then(|v| v.as_object()) {
                if let Some(v) = global.get("verbose").and_then(serde_json::Value::as_bool) {
                    config.global.verbose = v;
                }
                if let Some(v) = global.get("strict").and_then(serde_json::Value::as_bool) {
                    config.global.strict = v;
                }
                if let Some(v) = global
                    .get("max_parallel")
                    .and_then(serde_json::Value::as_u64)
                {
                    config.global.max_parallel = v as usize;
                }
                if let Some(v) = global.get("cache_dir").and_then(|v| v.as_str()) {
                    config.global.cache_dir = Some(PathBuf::from(v));
                }
                if let Some(v) = global
                    .get("incremental")
                    .and_then(serde_json::Value::as_bool)
                {
                    config.global.incremental = v;
                }
                if let Some(v) = global.get("viewport_width").and_then(serde_json::Value::as_u64) {
                    config.global.viewport_width = v as u32;
                }
                if let Some(v) = global.get("viewport_height").and_then(serde_json::Value::as_u64) {
                    config.global.viewport_height = v as u32;
                }
                if let Some(v) = global.get("prefers_color_scheme").and_then(serde_json::Value::as_str) {
                    config.global.prefers_color_scheme = v.to_string();
                }
                if let Some(opts) = global.get("options").and_then(|v| v.as_object()) {
                    for (k, v) in opts {
                        if let Some(s) = v.as_str() {
                            config.global.options.insert(k.clone(), s.to_string());
                        }
                    }
                }
            }
        }

        Ok(config)
    }

    /// Merge another config into this one (other takes priority).
    pub fn merge(&mut self, other: Self) {
        self.platforms.extend(other.platforms);
        if other.global.verbose {
            self.global.verbose = true;
        }
        if other.global.strict {
            self.global.strict = true;
        }
        self.global.max_parallel = other.global.max_parallel;
        if let Some(cache_dir) = other.global.cache_dir {
            self.global.cache_dir = Some(cache_dir);
        }
        self.global.incremental = other.global.incremental;
        self.global.options.extend(other.global.options);
    }
}

/// Configuration builder for programmatic use.
#[derive(Debug, Clone, Default)]
pub struct ConfigBuilder {
    /// The underlying config being built.
    config: Config,
}

impl ConfigBuilder {
    /// Creates a new config builder with defaults.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    /// Sets the output format for a platform.
    #[must_use]
    pub fn platform_format(mut self, platform: &str, format: OutputFormat) -> Self {
        self.config
            .platforms
            .entry(platform.to_string())
            .or_insert_with(|| PlatformConfig {
                format,
                output_dir: PathBuf::from(format!("output/{platform}")),
                minify: false,
                source_maps: false,
                options: HashMap::new(),
            })
            .format = format;
        self
    }

    /// Sets verbose mode.
    #[must_use]
    pub const fn verbose(mut self, verbose: bool) -> Self {
        self.config.global.verbose = verbose;
        self
    }

    /// Sets strict mode.
    #[must_use]
    pub const fn strict(mut self, strict: bool) -> Self {
        self.config.global.strict = strict;
        self
    }

    /// Builds the configuration.
    #[must_use]
    pub fn build(self) -> Config {
        self.config
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dart => write!(f, "dart"),
            Self::Kotlin => write!(f, "kotlin"),
            Self::Swift => write!(f, "swift"),
        }
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(p) => write!(f, "config file not found: {p}"),
            Self::ParseError(e) => write!(f, "config parse error: {e}"),
            Self::InvalidValue(v) => write!(f, "invalid config value: {v}"),
            Self::UnsupportedFormat(fmt) => {
                write!(f, "unsupported config format: {fmt}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.platforms.len(), 3);
        assert!(config.platforms.contains_key("flutter"));
        assert!(config.platforms.contains_key("compose"));
        assert!(config.platforms.contains_key("swiftui"));
        assert!(!config.global.verbose);
    }

    #[test]
    fn test_from_json() {
        let json = r#"{
            "platforms": {
                "flutter": {
                    "format": "dart",
                    "output_dir": "custom/output",
                    "minify": true
                }
            },
            "global": {
                "verbose": true,
                "strict": true,
                "max_parallel": 8
            }
        }"#;
        let config = Config::from_json(json).unwrap();
        let flutter = config.platforms.get("flutter").unwrap();
        assert_eq!(flutter.format, OutputFormat::Dart);
        assert_eq!(flutter.output_dir, PathBuf::from("custom/output"));
        assert!(flutter.minify);
        assert!(config.global.verbose);
        assert!(config.global.strict);
        assert_eq!(config.global.max_parallel, 8);
    }

    #[test]
    fn test_from_toml() {
        let toml = r#"
[platforms.flutter]
format = "dart"
output_dir = "custom/output"
minify = true

[global]
verbose = true
strict = true
max_parallel = 8
"#;
        let config = Config::from_toml(toml).unwrap();
        let flutter = config.platforms.get("flutter").unwrap();
        assert_eq!(flutter.format, OutputFormat::Dart);
        assert!(flutter.minify);
        assert!(config.global.verbose);
    }

    #[test]
    fn test_invalid_format() {
        let json = r#"{"platforms": {"web": {"format": "unknown"}}}"#;
        let result = Config::from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_merge() {
        let mut base = Config::default();
        let mut override_cfg = Config::default();
        override_cfg.global.verbose = true;
        override_cfg.global.max_parallel = 16;
        base.merge(override_cfg);
        assert!(base.global.verbose);
        assert_eq!(base.global.max_parallel, 16);
    }

    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .platform_format("flutter", OutputFormat::Dart)
            .verbose(true)
            .strict(true)
            .build();
        assert!(config.global.verbose);
        assert!(config.global.strict);
    }

    #[test]
    fn test_json_file_not_found() {
        let result = Config::from_json_file(Path::new("/nonexistent/config.json"));
        assert!(matches!(result, Err(ConfigError::FileNotFound(_))));
    }

    #[test]
    fn test_invalid_json() {
        let result = Config::from_json("{invalid json}");
        assert!(matches!(result, Err(ConfigError::ParseError(_))));
    }

    #[test]
    fn test_global_options() {
        let json = r#"{
            "global": {
                "options": {
                    "theme": "dark",
                    "language": "en"
                }
            }
        }"#;
        let config = Config::from_json(json).unwrap();
        assert_eq!(config.global.options.get("theme").unwrap(), "dark");
        assert_eq!(config.global.options.get("language").unwrap(), "en");
    }
}
