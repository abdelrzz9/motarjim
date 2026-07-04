#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! CLI entry point for the Motarjim compiler.
//!
//! Provides argument parsing via clap and dispatches to the compiler pipeline.

use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use clap::{Parser, Subcommand};
use motarjim_config::{Config, OutputFormat};
use motarjim_core::{CompileOptions, Compiler};
use motarjim_diag::{Diagnostic, Severity};
use motarjim_fs::RealFileSystem;

/// The Motarjim compiler CLI.
#[derive(Parser, Debug)]
#[command(name = "motarjim", version, about, long_about = None)]
pub struct CliArgs {
    /// The subcommand to execute.
    #[command(subcommand)]
    pub command: Command,
}

/// Available subcommands for the Motarjim compiler.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Compile a single input file.
    Compile {
        /// Path to the input HTML file.
        input: PathBuf,
        /// Target platform (flutter, compose, swiftui).
        #[arg(short, long, default_value = "flutter")]
        platform: String,
        /// Enable minification.
        #[arg(short, long)]
        minify: bool,
        /// Enable source maps.
        #[arg(long)]
        source_maps: bool,
        /// Enable strict mode (warnings as errors).
        #[arg(short, long)]
        strict: bool,
        /// Output file path (optional; defaults to stdout).
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Watch a file for changes and recompile.
    Watch {
        /// Path to the input HTML file.
        input: PathBuf,
        /// Target platform (flutter, compose, swiftui).
        #[arg(short, long, default_value = "flutter")]
        platform: String,
    },
    /// Create a default motarjim config file in the current directory.
    Init,
    /// Type-check and lint input without generating output.
    Check {
        /// Path to the input HTML file.
        input: PathBuf,
    },
}

/// Run the Motarjim compiler with the given CLI arguments.
///
/// # Errors
/// Returns an error if compilation or I/O fails.
pub fn run(args: &CliArgs) -> Result<i32, Box<dyn Error>> {
    match &args.command {
        Command::Compile {
            input,
            platform,
            minify,
            source_maps,
            strict,
            output,
        } => cmd_compile(
            input,
            platform,
            *minify,
            *source_maps,
            *strict,
            output.as_ref(),
        ),
        Command::Watch { input, platform } => cmd_watch(input, platform),
        Command::Init => cmd_init(),
        Command::Check { input } => cmd_check(input),
    }
}

/// Parses a platform string into an `OutputFormat`.
fn parse_platform(s: &str) -> Result<OutputFormat, Box<dyn Error>> {
    match s.to_lowercase().as_str() {
        "flutter" | "dart" => Ok(OutputFormat::Dart),
        "compose" | "kotlin" => Ok(OutputFormat::Kotlin),
        "swiftui" | "swift" => Ok(OutputFormat::Swift),
        other => {
            Err(format!("Unknown platform '{other}'. Use flutter, compose, or swiftui").into())
        }
    }
}

/// Runs the compile command on the given input file.
fn cmd_compile(
    input: &Path,
    platform: &str,
    minify: bool,
    source_maps: bool,
    strict: bool,
    output: Option<&PathBuf>,
) -> Result<i32, Box<dyn Error>> {
    let platform = parse_platform(platform)?;
    let config = load_config();
    let fs = Arc::new(RealFileSystem::new());
    let compiler = Compiler::new(config, fs);

    let options = CompileOptions {
        platform,
        minify,
        source_maps,
        strict,
    };

    match compiler.compile_file(input, &options) {
        Ok(result) => {
            if let Some(out_path) = output {
                std::fs::write(out_path, &result.output)?;
                eprintln!("Written to {}", out_path.display());
            } else {
                print!("{}", result.output);
            }

            if !result.diagnostics.is_empty() {
                print_diagnostics(&result.diagnostics);
            }

            Ok(0)
        }
        Err(diags) => {
            print_diagnostics(&diags);
            Ok(1)
        }
    }
}

/// Runs the watch command (not yet implemented).
fn cmd_watch(_input: &Path, _platform: &str) -> Result<i32, Box<dyn Error>> {
    eprintln!("Watch mode is not yet implemented");
    eprintln!("Use `motarjim compile <input>` for one-shot compilation");
    Ok(1)
}

/// Initialises a `motarjim.json` config file in the current directory.
fn cmd_init() -> Result<i32, Box<dyn Error>> {
    let config_path = Path::new("motarjim.json");
    if config_path.exists() {
        eprintln!("motarjim.json already exists");
        return Ok(1);
    }

    let default_config = r#"{
  "platforms": {
    "flutter": {
      "format": "dart",
      "output_dir": "output/flutter",
      "minify": false,
      "source_maps": false
    },
    "compose": {
      "format": "kotlin",
      "output_dir": "output/compose",
      "minify": false,
      "source_maps": false
    },
    "swiftui": {
      "format": "swift",
      "output_dir": "output/swiftui",
      "minify": false,
      "source_maps": false
    }
  },
  "global": {
    "verbose": false,
    "strict": false,
    "max_parallel": 4,
    "incremental": true
  }
}
"#;
    std::fs::write(config_path, default_config)?;
    eprintln!("Created motarjim.json");
    Ok(0)
}

/// Runs the check command, compiling with strict mode enabled.
///
/// JavaScript inputs (`.js`, `.mjs`) are routed to the `motarjim-js`
/// parser and semantic analyzer instead of the HTML/CSS compiler pipeline.
fn cmd_check(input: &Path) -> Result<i32, Box<dyn Error>> {
    if is_javascript_file(input) {
        return cmd_check_js(input);
    }

    let config = load_config();
    let fs = Arc::new(RealFileSystem::new());
    let compiler = Compiler::new(config, fs);

    let options = CompileOptions {
        strict: true,
        ..CompileOptions::default()
    };

    match compiler.compile_file(input, &options) {
        Ok(result) => {
            if !result.diagnostics.is_empty() {
                print_diagnostics(&result.diagnostics);
            }
            eprintln!("Check complete: {} diagnostics", result.diagnostics.len());
            Ok(0)
        }
        Err(diags) => {
            print_diagnostics(&diags);
            Ok(1)
        }
    }
}

/// Returns `true` if `path`'s extension marks it as JavaScript source.
fn is_javascript_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("js" | "mjs" | "jsx")
    )
}

/// Parses and semantically analyzes a JavaScript file, printing diagnostics.
fn cmd_check_js(input: &Path) -> Result<i32, Box<dyn Error>> {
    let source = std::fs::read_to_string(input)?;
    let mut parser = motarjim_js::JsParser::new(&source);
    match parser.parse() {
        Ok(program) => {
            let diagnostics = motarjim_js::SemanticAnalyzer::new().analyze(&program);
            if !diagnostics.is_empty() {
                print_diagnostics(&diagnostics);
            }
            eprintln!("Check complete: {} diagnostics", diagnostics.len());
            Ok(0)
        }
        Err(diagnostics) => {
            print_diagnostics(&diagnostics);
            Ok(1)
        }
    }
}

/// Loads compiler configuration from `motarjim.json` or `motarjim.toml`.
fn load_config() -> Config {
    let config_path = Path::new("motarjim.json");
    if config_path.exists() {
        Config::from_json_file(config_path).unwrap_or_default()
    } else if Path::new("motarjim.toml").exists() {
        let content = std::fs::read_to_string("motarjim.toml").unwrap_or_default();
        Config::from_toml(&content).unwrap_or_default()
    } else {
        Config::default()
    }
}

/// Prints diagnostics to stderr in a human-readable format.
fn print_diagnostics(diags: &[Diagnostic]) {
    for diag in diags {
        let severity = match diag.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
            Severity::Hint => "hint",
            Severity::Note => "note",
        };
        eprintln!("{severity}[E{:04}]: {}", diag.code.number, diag.message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_parse_platform_dart() {
        assert_eq!(parse_platform("flutter").unwrap(), OutputFormat::Dart);
        assert_eq!(parse_platform("dart").unwrap(), OutputFormat::Dart);
    }

    #[test]
    fn test_parse_platform_compose() {
        assert_eq!(parse_platform("compose").unwrap(), OutputFormat::Kotlin);
        assert_eq!(parse_platform("kotlin").unwrap(), OutputFormat::Kotlin);
    }

    #[test]
    fn test_parse_platform_swiftui() {
        assert_eq!(parse_platform("swiftui").unwrap(), OutputFormat::Swift);
        assert_eq!(parse_platform("swift").unwrap(), OutputFormat::Swift);
    }

    #[test]
    fn test_parse_platform_unknown() {
        assert!(parse_platform("unknown").is_err());
    }

    #[test]
    fn test_cli_compile_command() {
        let args = CliArgs::try_parse_from(&[
            "motarjim",
            "compile",
            "input.html",
            "--platform",
            "flutter",
        ]);
        assert!(args.is_ok());
        let args = args.unwrap();
        match &args.command {
            Command::Compile {
                input, platform, ..
            } => {
                assert_eq!(input, &PathBuf::from("input.html"));
                assert_eq!(platform, "flutter");
            }
            _ => panic!("Expected Compile command"),
        }
    }

    #[test]
    fn test_cli_init_command() {
        let args = CliArgs::try_parse_from(&["motarjim", "init"]);
        assert!(args.is_ok());
        assert!(matches!(args.unwrap().command, Command::Init));
    }

    #[test]
    fn test_cli_check_command() {
        let args = CliArgs::try_parse_from(&["motarjim", "check", "input.html"]);
        assert!(args.is_ok());
        match &args.unwrap().command {
            Command::Check { input } => {
                assert_eq!(input, &PathBuf::from("input.html"));
            }
            _ => panic!("Expected Check command"),
        }
    }

    #[test]
    fn test_is_javascript_file() {
        assert!(is_javascript_file(Path::new("app.js")));
        assert!(is_javascript_file(Path::new("app.mjs")));
        assert!(is_javascript_file(Path::new("app.jsx")));
        assert!(!is_javascript_file(Path::new("index.html")));
    }

    #[test]
    fn test_cmd_check_js_reports_semantic_diagnostics() {
        let dir =
            std::env::temp_dir().join(format!("motarjim-cli-check-js-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let file = dir.join("demo.js");
        std::fs::write(&file, "const x = 1; x = 2;").expect("write temp file");

        let exit_code = cmd_check_js(&file).expect("cmd_check_js should not error");
        assert_eq!(exit_code, 0);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_cli_watch_command() {
        let args =
            CliArgs::try_parse_from(&["motarjim", "watch", "input.html", "--platform", "swiftui"]);
        assert!(args.is_ok());
        match &args.unwrap().command {
            Command::Watch { input, platform } => {
                assert_eq!(input, &PathBuf::from("input.html"));
                assert_eq!(platform, "swiftui");
            }
            _ => panic!("Expected Watch command"),
        }
    }

    #[test]
    fn test_cli_compile_with_flags() {
        let args = CliArgs::try_parse_from(&[
            "motarjim",
            "compile",
            "in.html",
            "--platform",
            "compose",
            "--minify",
            "--source-maps",
            "--strict",
            "--output",
            "out.txt",
        ]);
        assert!(args.is_ok());
        let args = args.unwrap();
        match args.command {
            Command::Compile {
                input,
                platform,
                minify,
                source_maps,
                strict,
                output,
            } => {
                assert_eq!(input, PathBuf::from("in.html"));
                assert_eq!(platform, "compose");
                assert!(minify);
                assert!(source_maps);
                assert!(strict);
                assert_eq!(output, Some(PathBuf::from("out.txt")));
            }
            _ => panic!("Expected Compile command"),
        }
    }

    #[test]
    fn test_cli_app_version() {
        // Verify the app builds its clap definition without panic
        let _ = CliArgs::command();
    }
}
