use std::sync::Arc;

use motarjim_config::Config;
use motarjim_core::{CompileOptions, Compiler};
use motarjim_fs::VirtualFileSystem;

fn main() {
    let config = Config::new();
    let fs = Arc::new(VirtualFileSystem::new());
    let compiler = Compiler::new(config, fs);

    let html = r#"<!DOCTYPE html>
<html>
  <body>
    <h1>Hello, Motarjim!</h1>
    <p class="intro">This is a test.</p>
  </body>
</html>"#;

    let options = CompileOptions::default();
    match compiler.compile(html, &options) {
        Ok(result) => {
            println!("Compilation succeeded!");
            println!("  Nodes parsed:    {}", result.stats.nodes_parsed);
            println!("  IR nodes:        {}", result.stats.ir_nodes);
            println!("  Optimizations:   {}", result.stats.optimizations_applied);
            println!("  Time:            {}ms", result.stats.time_ms);
            println!("\nGenerated output:\n{}", result.output);
        }
        Err(diags) => {
            for d in &diags {
                eprintln!("Error: {:?} — {}", d.code(), d.message());
            }
        }
    }
}
