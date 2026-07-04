use std::process;

use clap::Parser;
use motarjim_cli::{run, CliArgs};

fn main() {
    let args = CliArgs::parse();
    let exit_code = match run(&args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Error: {e}");
            1
        }
    };
    process::exit(exit_code);
}
