//! AST validation pass for Motarjim CSS.
//!
//! Walks the converted CSTree and performs structural validation checks
//! that aren't caught by Lightning CSS (e.g. declaration ordering, duplicate
//! selectors, or missing required properties in at-rules).

#![allow(dead_code)]

use motarjim_ast::css::{
    AtRule, CharsetRule, CssRule, CssStylesheet, FontFaceRule, ImportRule, KeyframesRule,
    NamespaceRule, PageRule, StyleRule,
};
use motarjim_diag::{Diagnostic, DiagnosticCode, Severity};

/// Validates a `CssStylesheet` and returns a list of diagnostics.
///
/// This runs after AST conversion and catches patterns that are technically
/// valid CSS but may indicate user mistakes or problematic code.
pub fn validate(sheet: &CssStylesheet) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for rule in &sheet.rules {
        validate_rule(rule, &mut diagnostics);
    }

    diagnostics
}

fn code(number: u32, msg: &'static str) -> DiagnosticCode {
    DiagnosticCode::new(number, msg).with_prefix("CSS")
}

fn warn(code: DiagnosticCode, msg: impl Into<String>) -> Diagnostic {
    Diagnostic::new(Severity::Warning, code, msg)
}

fn error(code: DiagnosticCode, msg: impl Into<String>) -> Diagnostic {
    Diagnostic::new(Severity::Error, code, msg)
}

fn validate_rule(rule: &CssRule, diagnostics: &mut Vec<Diagnostic>) {
    match rule {
        CssRule::Style(style) => validate_style_rule(style, diagnostics),
        CssRule::Media(media) => {
            for child in &media.rules {
                validate_rule(child, diagnostics);
            }
        }
        CssRule::Supports(supports) => {
            for child in &supports.rules {
                validate_rule(child, diagnostics);
            }
        }
        CssRule::Keyframes(keyframes) => validate_keyframes_rule(keyframes, diagnostics),
        CssRule::FontFace(font_face) => validate_font_face_rule(font_face, diagnostics),
        CssRule::Page(page) => validate_page_rule(page, diagnostics),
        CssRule::Import(import) => validate_import_rule(import, diagnostics),
        CssRule::Charset(charset) => validate_charset_rule(charset, diagnostics),
        CssRule::Namespace(namespace) => validate_namespace_rule(namespace, diagnostics),
        CssRule::Other(at_rule) => validate_at_rule(at_rule, diagnostics),
    }
}

fn validate_style_rule(rule: &StyleRule, diagnostics: &mut Vec<Diagnostic>) {
    if rule.selectors.is_empty() {
        diagnostics.push(
            warn(
                code(1, "style-rule-no-selectors"),
                "Style rule has no selectors",
            )
            .with_note("style rule will be ignored"),
        );
    }
}

fn validate_keyframes_rule(rule: &KeyframesRule, diagnostics: &mut Vec<Diagnostic>) {
    if rule.keyframes.is_empty() {
        diagnostics.push(
            warn(
                code(2, "keyframes-empty"),
                "@keyframes rule has no keyframe blocks",
            )
            .with_note("keyframe rule will have no effect"),
        );
    }

    for (i, keyframe) in rule.keyframes.iter().enumerate() {
        if keyframe.selectors.is_empty() {
            diagnostics.push(
                warn(
                    code(3, "keyframe-no-selectors"),
                    format!("Keyframe at index {i} has no selectors"),
                )
                .with_note("keyframe block will be skipped"),
            );
        }
    }
}

fn validate_font_face_rule(rule: &FontFaceRule, diagnostics: &mut Vec<Diagnostic>) {
    let has_font_family = rule
        .declarations
        .iter()
        .any(|d| d.property.as_str() == "font-family");
    let has_src = rule
        .declarations
        .iter()
        .any(|d| d.property.as_str() == "src");

    if !has_font_family {
        diagnostics.push(
            warn(
                code(4, "fontface-missing-font-family"),
                "@font-face rule is missing 'font-family' property",
            )
            .with_note("font-face rule requires both font-family and src"),
        );
    }

    if !has_src {
        diagnostics.push(
            warn(
                code(5, "fontface-missing-src"),
                "@font-face rule is missing 'src' property",
            )
            .with_note("font-face rule requires both font-family and src"),
        );
    }
}

fn validate_page_rule(rule: &PageRule, diagnostics: &mut Vec<Diagnostic>) {
    if rule.declarations.is_empty() {
        diagnostics.push(
            warn(code(6, "page-empty"), "@page rule has no declarations")
                .with_note("page rule will have no effect"),
        );
    }
}

fn validate_import_rule(rule: &ImportRule, diagnostics: &mut Vec<Diagnostic>) {
    if rule.url.is_empty() {
        diagnostics.push(
            error(code(7, "import-empty-url"), "@import rule has an empty URL")
                .with_note("import rule url cannot be empty"),
        );
    }
}

fn validate_charset_rule(rule: &CharsetRule, diagnostics: &mut Vec<Diagnostic>) {
    if rule.encoding.is_empty() {
        diagnostics.push(
            error(
                code(8, "charset-empty-encoding"),
                "@charset rule has an empty encoding",
            )
            .with_note("charset encoding cannot be empty"),
        );
    }
}

fn validate_namespace_rule(rule: &NamespaceRule, diagnostics: &mut Vec<Diagnostic>) {
    if rule.url.is_empty() {
        diagnostics.push(
            error(
                code(9, "namespace-empty-url"),
                "@namespace rule has an empty URL",
            )
            .with_note("namespace URL cannot be empty"),
        );
    }
}

fn validate_at_rule(rule: &AtRule, diagnostics: &mut Vec<Diagnostic>) {
    if rule.name.is_empty() {
        diagnostics.push(
            warn(code(10, "atrule-empty-name"), "At-rule has an empty name")
                .with_note("at-rule name is required"),
        );
    }
}
