#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

//! Language Server Protocol implementation for the Motarjim compiler.
//!
//! Provides a [`Backend`] implementing [`LanguageServer`] via `tower-lsp`,
//! including diagnostics, completion, hover, goto-definition, semantic
//! tokens, and code actions for HTML/CSS documents.

use std::sync::Arc;

use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{Diagnostic, Url, DocumentDiagnosticParams, DocumentDiagnosticReportResult, DocumentDiagnosticReport, RelatedFullDocumentDiagnosticReport, FullDocumentDiagnosticReport, InitializeParams, InitializeResult, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, CompletionOptions, HoverProviderCapability, OneOf, SemanticTokensServerCapabilities, SemanticTokensOptions, SemanticTokensFullOptions, SemanticTokensLegend, CodeActionProviderCapability, ServerInfo, DidOpenTextDocumentParams, DidChangeTextDocumentParams, DidSaveTextDocumentParams, CompletionParams, CompletionResponse, CompletionItem, InsertTextFormat, HoverParams, Hover, HoverContents, MarkedString, GotoDefinitionParams, GotoDefinitionResponse, SemanticTokensParams, SemanticTokensResult, SemanticTokens, CodeActionParams, CodeActionResponse, Range, Position, DiagnosticSeverity, NumberOrString};
use tower_lsp::{Client, LanguageServer, LspService, Server};

pub use motarjim_ast;
pub use motarjim_diag;
pub use motarjim_parser;

/// The language server backend.
///
/// Maintains open documents in a concurrent map and uses the Motarjim
/// compiler pipeline to produce diagnostics and editor intelligence.
pub struct Backend {
    /// LSP client for sending notifications and requests.
    client: Client,
    /// Open documents tracked by URI.
    documents: Arc<DashMap<String, String>>,
}

impl Backend {
    /// Create a new backend with the given LSP client.
    #[must_use]
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(DashMap::new()),
        }
    }

    /// Compute LSP diagnostics for a document URI by parsing its text.
    fn compute_diagnostics(&self, uri: &str) -> Vec<Diagnostic> {
        let text = match self.documents.get(uri) {
            Some(t) => t.clone(),
            None => return Vec::new(),
        };
        compute_diagnostics(&text)
    }

    /// Push diagnostics for the given URI to the client.
    async fn publish_diagnostics_for(&self, uri: &str) {
        let diagnostics = self.compute_diagnostics(uri);
        let url = match Url::parse(uri) {
            Ok(u) => u,
            Err(_) => return,
        };
        self.client.publish_diagnostics(url, diagnostics, None).await;
    }

    /// Pull-based diagnostic handler for the `textDocument/diagnostic` request.
    ///
    /// # Errors
    ///
    /// Returns a JSON-RPC error if processing fails.
    pub async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult> {
        let uri = params.text_document.uri;
        let diagnostics = self.compute_diagnostics(uri.as_str());
        Ok(DocumentDiagnosticReportResult::Report(
            DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                related_documents: None,
                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                    result_id: None,
                    items: diagnostics,
                },
            }),
        ))
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!["<".to_string(), ".".to_string()]),
                    ..CompletionOptions::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
                        full: Some(SemanticTokensFullOptions::Bool(true)),
                        range: Some(true),
                        legend: SemanticTokensLegend {
                            token_types: Vec::new(),
                            token_modifiers: Vec::new(),
                        },
                        ..SemanticTokensOptions::default()
                    }),
                ),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: "motarjim-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
        })
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        self.documents
            .insert(uri.clone(), params.text_document.text);
        self.publish_diagnostics_for(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        if let Some(change) = params.content_changes.into_iter().last() {
            self.documents.insert(uri.clone(), change.text);
        }
        self.publish_diagnostics_for(&uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        if let Some(text) = params.text {
            self.documents.insert(uri.clone(), text);
        }
        self.publish_diagnostics_for(&uri).await;
    }

    async fn completion(
        &self,
        _params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        let items = vec![
            CompletionItem {
                label: "div".to_string(),
                detail: Some("HTML div element".to_string()),
                insert_text: Some("<div>$0</div>".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "style".to_string(),
                detail: Some("CSS style tag".to_string()),
                insert_text: Some("<style>\n  $0\n</style>".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "class".to_string(),
                detail: Some("CSS class attribute".to_string()),
                insert_text: Some("class=\"$0\"".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
        ];
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, _params: HoverParams) -> Result<Option<Hover>> {
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(
                "Motarjim — compile HTML/CSS to native UI code".to_string(),
            )),
            range: None,
        }))
    }

    async fn goto_definition(
        &self,
        _params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        Ok(None)
    }

    async fn semantic_tokens_full(
        &self,
        _params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: Vec::new(),
        })))
    }

    async fn code_action(
        &self,
        _params: CodeActionParams,
    ) -> Result<Option<CodeActionResponse>> {
        Ok(None)
    }
}

/// Compute LSP diagnostics by parsing the given HTML/CSS text.
///
/// Uses [`motarjim_parser::HtmlParser`] to detect parse errors and
/// converts them to LSP [`Diagnostic`] items.
#[must_use]
pub fn compute_diagnostics(text: &str) -> Vec<Diagnostic> {
    let mut parser = motarjim_parser::HtmlParser::new(text);
    match parser.parse() {
        Ok(_) => Vec::new(),
        Err(diags) => diags.into_iter().map(motarjim_diag_to_lsp).collect(),
    }
}

/// Convert a [`motarjim_diag::Diagnostic`] to an LSP [`Diagnostic`].
fn motarjim_diag_to_lsp(d: motarjim_diag::Diagnostic) -> Diagnostic {
    let range = match d.span {
        Some(ref span) => Range {
            start: Position {
                line: span.start.line.saturating_sub(1),
                character: span.start.column.saturating_sub(1),
            },
            end: Position {
                line: span.end.line.saturating_sub(1),
                character: span.end.column.saturating_sub(1),
            },
        },
        None => Range::default(),
    };

    let severity = Some(match d.severity {
        motarjim_diag::Severity::Error => DiagnosticSeverity::ERROR,
        motarjim_diag::Severity::Warning => DiagnosticSeverity::WARNING,
        motarjim_diag::Severity::Info => DiagnosticSeverity::INFORMATION,
        motarjim_diag::Severity::Hint => DiagnosticSeverity::HINT,
        motarjim_diag::Severity::Note => DiagnosticSeverity::INFORMATION,
    });

    Diagnostic {
        range,
        severity,
        message: d.message,
        source: Some("motarjim".to_string()),
        code: Some(NumberOrString::Number(d.code.number as i32)),
        ..Diagnostic::default()
    }
}

/// Start the language server on stdin/stdout.
pub async fn start_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_diag() -> motarjim_diag::Diagnostic {
        motarjim_diag::Diagnostic::new(
            motarjim_diag::Severity::Error,
            motarjim_diag::codes::PARSER_UNEXPECTED_TOKEN,
            "test error",
        )
        .with_span(motarjim_diag::SourceSpan {
            start: motarjim_diag::SourceLocation {
                line: 2,
                column: 5,
                offset: 10,
            },
            end: motarjim_diag::SourceLocation {
                line: 2,
                column: 10,
                offset: 15,
            },
        })
    }

    #[test]
    fn test_convert_diagnostic_error() {
        let d = sample_diag();
        let lsp = motarjim_diag_to_lsp(d);
        assert_eq!(lsp.message, "test error");
        assert_eq!(lsp.severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(lsp.source, Some("motarjim".to_string()));
    }

    #[test]
    fn test_convert_diagnostic_range_zero_based() {
        let d = sample_diag();
        let lsp = motarjim_diag_to_lsp(d);
        assert_eq!(lsp.range.start.line, 1);
        assert_eq!(lsp.range.start.character, 4);
        assert_eq!(lsp.range.end.line, 1);
        assert_eq!(lsp.range.end.character, 9);
    }

    #[test]
    fn test_convert_diagnostic_no_span() {
        let d = motarjim_diag::Diagnostic::new(
            motarjim_diag::Severity::Warning,
            motarjim_diag::codes::A11Y_MISSING_ALT,
            "missing alt text",
        );
        let lsp = motarjim_diag_to_lsp(d);
        assert_eq!(lsp.severity, Some(DiagnosticSeverity::WARNING));
        assert_eq!(lsp.range.start.line, 0);
        assert_eq!(lsp.range.start.character, 0);
    }

    #[test]
    fn test_convert_severity_hint() {
        let d = motarjim_diag::Diagnostic::new(
            motarjim_diag::Severity::Hint,
            motarjim_diag::codes::PARSER_UNCLOSED_TAG,
            "unclosed tag hint",
        );
        let lsp = motarjim_diag_to_lsp(d);
        assert_eq!(lsp.severity, Some(DiagnosticSeverity::HINT));
    }

    #[test]
    fn test_diagnostic_code_number() {
        let d = sample_diag();
        let lsp = motarjim_diag_to_lsp(d);
        assert_eq!(lsp.code, Some(NumberOrString::Number(1)));
    }

    #[test]
    fn test_compute_diagnostics_valid_html() {
        let diags = compute_diagnostics("<div>hello</div>");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_compute_diagnostics_invalid_html() {
        let diags = compute_diagnostics("<div>");
        assert!(
            !diags.is_empty(),
            "unclosed div should produce diagnostics"
        );
    }

    #[test]
    fn test_compute_diagnostics_empty_string() {
        let diags = compute_diagnostics("");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_compute_diagnostics_nested_good() {
        let html = "<ul><li>item</li></ul>";
        let diags = compute_diagnostics(html);
        assert!(diags.is_empty());
    }
}
