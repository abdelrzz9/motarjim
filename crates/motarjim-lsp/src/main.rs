#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! Entry point for the Motarjim LSP server binary.

/// Start the language server.
#[tokio::main]
async fn main() {
    motarjim_lsp::start_server().await;
}
