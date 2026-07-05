use smallvec::SmallVec;
use smol_str::SmolStr;

use motarjim_ast::css::{
    AtRule, CharsetRule, CssRule, CssStylesheet, Declaration, FontFaceRule, ImportRule, Keyframe,
    KeyframesRule, MediaCondition, MediaQuery, MediaRule, NamespaceRule, PageRule, StyleRule,
    SupportsRule,
};
use motarjim_ast_css::{AttributeOperator, PseudoClass, PseudoElement, Selector, SimpleSelector};
use motarjim_diag::codes;
use motarjim_diag::{Diagnostic, DiagnosticBag};
use motarjim_lexer::css::{CssTokenKind, CssTokenizer};
use motarjim_lexer::Token as LexerToken;

/// Parser for CSS source text.
///
/// Produces a [`CssStylesheet`] containing style rules and at-rules.
///
/// # Errors
///
/// Returns a vector of [`Diagnostic`]s if parsing fails.
///
/// # Example
///
/// ```rust
/// use motarjim_parser::CssParser;
///
/// let mut parser = CssParser::new("div { color: red; }");
/// let sheet = parser.parse();
/// assert!(sheet.is_ok());
/// ```
#[derive(Debug, Clone)]
pub struct CssParser {
    /// Tokenized CSS tokens.
    tokens: Vec<LexerToken<CssTokenKind>>,
    /// Current position in the token stream.
    pos: usize,
    /// Collected diagnostics during parsing.
    diagnostics: DiagnosticBag,
}

impl CssParser {
    /// Creates a new `CssParser` for the given source text.
    #[must_use]
    pub fn new(source: &str) -> Self {
        let mut tokenizer = CssTokenizer::new(source);
        let tokens = tokenizer.tokenize();
        Self {
            tokens,
            pos: 0,
            diagnostics: DiagnosticBag::new(),
        }
    }

    /// Parses the CSS source and returns a [`CssStylesheet`].
    ///
    /// # Errors
    ///
    /// Returns a vector of [`Diagnostic`]s if parsing encounters errors.
    pub fn parse(&mut self) -> Result<CssStylesheet, Vec<Diagnostic>> {
        let mut rules: Vec<CssRule> = Vec::new();

        while let Some(t) = self.peek_token() {
            let kind = t.kind;

            match kind {
                CssTokenKind::Eof => break,
                CssTokenKind::AtKeyword => {
                    if let Some(rule) = self.parse_at_rule() {
                        rules.push(rule);
                    }
                }
                CssTokenKind::CloseBrace => {
                    self.consume_token();
                }
                _ => {
                    if let Some(rule) = self.parse_style_rule() {
                        rules.push(rule);
                    } else {
                        self.consume_token();
                    }
                }
            }
        }

        if self.diagnostics.has_errors() {
            Err(self.diagnostics.clone().into_diagnostics())
        } else {
            Ok(CssStylesheet {
                rules,
                source_path: None,
            })
        }
    }

    /// Consumes and returns the next CSS token.
    fn consume_token(&mut self) -> Option<LexerToken<CssTokenKind>> {
        let token = self.tokens.get(self.pos)?;
        self.pos += 1;
        Some(token.clone())
    }

    /// Peeks at the next CSS token without consuming it.
    fn peek_token(&self) -> Option<&LexerToken<CssTokenKind>> {
        self.tokens.get(self.pos)
    }

    /// Expects a CSS token of the given kind; emits a diagnostic on mismatch.
    fn expect_token(&mut self, kind: CssTokenKind) -> bool {
        let token = match self.consume_token() {
            Some(t) => t,
            None => return false,
        };
        if token.kind == kind {
            true
        } else {
            self.diagnostics.push_error(
                codes::CSS_PARSE_ERROR,
                format!("Expected {:?}, found {:?}", kind, token.kind),
            );
            false
        }
    }

    /// Parses a CSS style rule (selectors + declarations).
    fn parse_style_rule(&mut self) -> Option<CssRule> {
        let selectors = self.parse_selectors()?;
        if selectors.is_empty() {
            return None;
        }

        if !self.expect_token(CssTokenKind::OpenBrace) {
            return None;
        }

        let declarations = self.parse_declarations();

        self.expect_token(CssTokenKind::CloseBrace);

        Some(CssRule::Style(StyleRule {
            selectors,
            declarations,
        }))
    }

    /// Parses CSS selectors from the token stream.
    fn parse_selectors(&mut self) -> Option<Vec<Selector>> {
        let mut selectors: Vec<Selector> = Vec::new();
        let mut simple_selectors: Vec<SimpleSelector> = Vec::new();

        while let Some(t) = self.peek_token() {
            let kind = t.kind;

            match kind {
                CssTokenKind::OpenBrace => break,
                CssTokenKind::Comma => {
                    self.consume_token();
                    if !simple_selectors.is_empty() {
                        selectors.push(Selector {
                            simple_selectors: std::mem::take(&mut simple_selectors),
                            combinators: Vec::new(),
                        });
                    }
                }
                CssTokenKind::Ident => {
                    let raw = self.consume_token()?.raw;
                    simple_selectors.push(SimpleSelector::Type(SmolStr::from(raw)));
                }
                CssTokenKind::Dot => {
                    self.consume_token();
                    let class_name = match self.peek_token() {
                        Some(t) if t.kind == CssTokenKind::Ident => {
                            self.consume_token()?.raw.clone()
                        }
                        _ => {
                            self.diagnostics.push_error(
                                codes::CSS_UNSUPPORTED_SELECTOR,
                                "Expected class name after '.'",
                            );
                            continue;
                        }
                    };
                    simple_selectors.push(SimpleSelector::Class(SmolStr::from(class_name)));
                }
                CssTokenKind::Hash => {
                    self.consume_token();
                    let id_name = match self.peek_token() {
                        Some(t) if t.kind == CssTokenKind::Ident => {
                            self.consume_token()?.raw.clone()
                        }
                        _ => {
                            self.diagnostics.push_error(
                                codes::CSS_UNSUPPORTED_SELECTOR,
                                "Expected id name after '#'",
                            );
                            continue;
                        }
                    };
                    simple_selectors.push(SimpleSelector::Id(SmolStr::from(id_name)));
                }
                CssTokenKind::Star => {
                    simple_selectors.push(SimpleSelector::Universal);
                    self.consume_token();
                }
                CssTokenKind::OpenBracket => {
                    if let Some(attr_sel) = self.parse_attribute_selector() {
                        simple_selectors.push(attr_sel);
                    }
                }
                CssTokenKind::Colon => {
                    self.consume_token();
                    let is_pseudo_element = self
                        .peek_token()
                        .is_some_and(|t| t.kind == CssTokenKind::Colon);
                    if is_pseudo_element {
                        self.consume_token();
                        if let Some(pe) = self.parse_pseudo_element() {
                            simple_selectors.push(pe);
                        }
                    } else if let Some(pc) = self.parse_pseudo_class() {
                        simple_selectors.push(pc);
                    }
                }
                _ => {
                    self.consume_token();
                }
            }
        }

        if !simple_selectors.is_empty() {
            selectors.push(Selector {
                simple_selectors: std::mem::take(&mut simple_selectors),
                combinators: Vec::new(),
            });
        }

        if selectors.is_empty() {
            None
        } else {
            Some(selectors)
        }
    }

    /// Parses an attribute selector `[name]` or `[name=value]`.
    fn parse_attribute_selector(&mut self) -> Option<SimpleSelector> {
        self.consume_token(); // consume '['
        let name = match self.peek_token() {
            Some(t) if t.kind == CssTokenKind::Ident => self.consume_token()?.raw,
            _ => {
                self.diagnostics
                    .push_error(codes::CSS_UNSUPPORTED_SELECTOR, "Expected attribute name");
                self.skip_to_bracket_close();
                return None;
            }
        };

        let operator = self.parse_attribute_operator();
        let value = if operator.is_some() {
            match self.peek_token() {
                Some(t)
                    if t.kind == CssTokenKind::Ident
                        || t.kind == CssTokenKind::String
                        || t.kind == CssTokenKind::Number =>
                {
                    Some(SmolStr::from(self.consume_token()?.raw))
                }
                _ => None,
            }
        } else {
            None
        };

        if let Some(t) = self.peek_token() {
            if t.kind == CssTokenKind::Ident && (t.raw == "i" || t.raw == "I") {
                self.consume_token();
            }
        }

        self.expect_token(CssTokenKind::CloseBracket);

        Some(SimpleSelector::Attribute {
            name: SmolStr::from(name),
            operator,
            value,
            case_sensitive: true,
        })
    }

    /// Parses an attribute operator (`=`, `~=`, `|=`, etc.).
    fn parse_attribute_operator(&mut self) -> Option<AttributeOperator> {
        let op = match self.peek_token()?.kind {
            CssTokenKind::Equals => {
                self.consume_token();
                AttributeOperator::Equals
            }
            _ => return None,
        };
        Some(op)
    }

    /// Parses a CSS pseudo-class selector.
    fn parse_pseudo_class(&mut self) -> Option<SimpleSelector> {
        let name = match self.peek_token() {
            Some(t) if t.kind == CssTokenKind::Ident => {
                self.consume_token()?.raw.to_ascii_lowercase()
            }
            _ => return None,
        };

        let pseudo = match name.as_str() {
            "hover" => PseudoClass::Hover,
            "active" => PseudoClass::Active,
            "focus" => PseudoClass::Focus,
            "visited" => PseudoClass::Visited,
            "link" => PseudoClass::Link,
            "disabled" => PseudoClass::Disabled,
            "enabled" => PseudoClass::Enabled,
            "checked" => PseudoClass::Checked,
            "first-child" => PseudoClass::FirstChild,
            "last-child" => PseudoClass::LastChild,
            "first-of-type" => PseudoClass::FirstOfType,
            "last-of-type" => PseudoClass::LastOfType,
            "root" => PseudoClass::Root,
            "empty" => PseudoClass::Empty,
            "target" => PseudoClass::Target,
            other => PseudoClass::Custom(SmolStr::from(other)),
        };

        Some(SimpleSelector::PseudoClass(pseudo))
    }

    /// Parses a CSS pseudo-element selector (`::before`, `::after`, etc.).
    fn parse_pseudo_element(&mut self) -> Option<SimpleSelector> {
        let name = match self.peek_token() {
            Some(t) if t.kind == CssTokenKind::Ident => {
                self.consume_token()?.raw.to_ascii_lowercase()
            }
            _ => return None,
        };

        let pseudo = match name.as_str() {
            "before" => PseudoElement::Before,
            "after" => PseudoElement::After,
            "first-line" => PseudoElement::FirstLine,
            "first-letter" => PseudoElement::FirstLetter,
            "placeholder" => PseudoElement::Placeholder,
            "selection" => PseudoElement::Selection,
            other => PseudoElement::Custom(SmolStr::from(other)),
        };

        Some(SimpleSelector::PseudoElement(pseudo))
    }

    /// Skips tokens until a closing bracket `]` or EOF is found.
    fn skip_to_bracket_close(&mut self) {
        while let Some(t) = self.peek_token() {
            if t.kind == CssTokenKind::CloseBracket || t.kind == CssTokenKind::Eof {
                if t.kind == CssTokenKind::CloseBracket {
                    self.consume_token();
                }
                break;
            }
            self.consume_token();
        }
    }

    /// Parses CSS declarations inside a block.
    fn parse_declarations(&mut self) -> SmallVec<[Declaration; 4]> {
        let mut decls = SmallVec::new();

        loop {
            // Skip stray semicolons
            while let Some(t) = self.peek_token() {
                if t.kind == CssTokenKind::Semicolon {
                    self.consume_token();
                } else {
                    break;
                }
            }

            let kind = match self.peek_token() {
                Some(t) => t.kind,
                None => break,
            };

            match kind {
                CssTokenKind::CloseBrace => break,
                CssTokenKind::Eof => break,
                CssTokenKind::AtKeyword => {
                    self.consume_token();
                }
                CssTokenKind::Ident => {
                    let property_token = match self.consume_token() {
                        Some(t) => t,
                        None => break,
                    };
                    let property = property_token.raw;

                    // Expect colon
                    match self.peek_token() {
                        Some(t) if t.kind == CssTokenKind::Colon => {
                            self.consume_token();
                        }
                        _ => {
                            self.diagnostics.push_error(
                                codes::CSS_PARSE_ERROR,
                                format!("Expected ':' after property '{property}'"),
                            );
                            continue;
                        }
                    }

                    // Parse value tokens until ';' or '}'
                    let value = self.collect_declaration_value();

                    // Check for !important
                    let important = if let Some(t) = self.peek_token() {
                        t.kind == CssTokenKind::Important
                    } else {
                        false
                    };
                    if important {
                        self.consume_token();
                    }

                    decls.push(Declaration {
                        property: SmolStr::from(property),
                        value: value.trim().to_string(),
                        important,
                    });
                }
                _ => {
                    self.consume_token();
                }
            }
        }

        decls
    }

    /// Collects the raw text of a declaration value until a delimiter is reached.
    fn collect_declaration_value(&mut self) -> String {
        let mut parts: Vec<String> = Vec::new();

        while let Some(t) = self.peek_token() {
            let kind = t.kind;

            match kind {
                CssTokenKind::Semicolon | CssTokenKind::CloseBrace | CssTokenKind::Eof => break,
                CssTokenKind::Important => break,
                _ => {
                    if let Some(t) = self.consume_token() {
                        parts.push(t.raw);
                    }
                }
            }
        }

        parts.join(" ")
    }

    /// Parses a CSS at-rule (`@media`, `@import`, etc.).
    fn parse_at_rule(&mut self) -> Option<CssRule> {
        let token = self.consume_token()?;
        let name = token.raw;
        let lower_name = name.to_ascii_lowercase();

        match lower_name.as_str() {
            "@media" => self.parse_media_rule(),
            "@import" => self.parse_import_rule(),
            "@charset" => self.parse_charset_rule(),
            "@namespace" => self.parse_namespace_rule(),
            "@font-face" => self.parse_font_face_rule(),
            "@keyframes" => self.parse_keyframes_rule(),
            "@supports" => self.parse_supports_rule(),
            "@page" => self.parse_page_rule(),
            _ => {
                let prelude = self.collect_prelude();
                let block = if let Some(t) = self.peek_token() {
                    if t.kind == CssTokenKind::OpenBrace {
                        self.consume_token();
                        let content = self.collect_block_content();
                        self.expect_token(CssTokenKind::CloseBrace);
                        Some(content)
                    } else {
                        None
                    }
                } else {
                    None
                };

                Some(CssRule::Other(AtRule {
                    name: SmolStr::from(&name[1..]),
                    prelude,
                    block,
                }))
            }
        }
    }

    /// Collects raw token text as an at-rule prelude.
    fn collect_prelude(&mut self) -> String {
        let mut parts: Vec<String> = Vec::new();
        while let Some(t) = self.peek_token() {
            let kind = t.kind;
            match kind {
                CssTokenKind::OpenBrace | CssTokenKind::Semicolon | CssTokenKind::Eof => break,
                _ => {
                    if let Some(t) = self.consume_token() {
                        parts.push(t.raw);
                    }
                }
            }
        }
        parts.join(" ")
    }

    /// Collects raw block content, tracking nested brace depth.
    fn collect_block_content(&mut self) -> String {
        let mut parts: Vec<String> = Vec::new();
        let mut depth = 1u32;
        while let Some(t) = self.peek_token() {
            let kind = t.kind;
            match kind {
                CssTokenKind::OpenBrace => {
                    depth += 1;
                    if let Some(t) = self.consume_token() {
                        parts.push(t.raw);
                    }
                }
                CssTokenKind::CloseBrace => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    if let Some(t) = self.consume_token() {
                        parts.push(t.raw);
                    }
                }
                CssTokenKind::Eof => break,
                _ => {
                    if let Some(t) = self.consume_token() {
                        parts.push(t.raw);
                    }
                }
            }
        }
        parts.join(" ")
    }

    /// Parses an `@media` rule.
    fn parse_media_rule(&mut self) -> Option<CssRule> {
        let mut conditions: Vec<MediaCondition> = Vec::new();
        let mut parts: Vec<String> = Vec::new();

        while let Some(t) = self.peek_token() {
            let kind = t.kind;
            match kind {
                CssTokenKind::OpenBrace => break,
                CssTokenKind::Eof => break,
                _ => {
                    if let Some(t) = self.consume_token() {
                        parts.push(t.raw);
                    }
                }
            }
        }

        for part in &parts {
            let lower = part.to_ascii_lowercase();
            match lower.as_str() {
                "all" => conditions.push(MediaCondition::All),
                "print" => conditions.push(MediaCondition::Print),
                "screen" => conditions.push(MediaCondition::Screen),
                "speech" => conditions.push(MediaCondition::Speech),
                _ => {
                    if lower.starts_with("min-width") {
                        let val = part.split(':').nth(1).unwrap_or("").trim().to_string();
                        conditions.push(MediaCondition::MinWidth(val));
                    } else if lower.starts_with("max-width") {
                        let val = part.split(':').nth(1).unwrap_or("").trim().to_string();
                        conditions.push(MediaCondition::MaxWidth(val));
                    } else if part.contains(':') {
                        conditions.push(MediaCondition::Feature {
                            name: SmolStr::from(lower),
                            value: None,
                        });
                    }
                }
            }
        }

        // Parse nested rules
        self.consume_token(); // consume '{'
        let mut rules: Vec<CssRule> = Vec::new();
        while let Some(t) = self.peek_token() {
            let kind = t.kind;
            match kind {
                CssTokenKind::CloseBrace => {
                    self.consume_token();
                    break;
                }
                CssTokenKind::Eof => break,
                _ => {
                    if let Some(rule) = self.parse_style_rule() {
                        rules.push(rule);
                    } else {
                        self.consume_token();
                    }
                }
            }
        }

        let query = MediaQuery {
            conditions: if conditions.is_empty() {
                vec![MediaCondition::All]
            } else {
                conditions
            },
        };

        Some(CssRule::Media(MediaRule { query, rules }))
    }

    /// Parses an `@import` rule.
    fn parse_import_rule(&mut self) -> Option<CssRule> {
        let url = self.collect_prelude().trim().to_string();
        if let Some(t) = self.peek_token() {
            if t.kind == CssTokenKind::Semicolon {
                self.consume_token();
            }
        }
        Some(CssRule::Import(ImportRule {
            url: SmolStr::from(url),
            media: None,
        }))
    }

    /// Parses an `@charset` rule.
    fn parse_charset_rule(&mut self) -> Option<CssRule> {
        let encoding = self.collect_prelude().trim().to_string();
        if let Some(t) = self.peek_token() {
            if t.kind == CssTokenKind::Semicolon {
                self.consume_token();
            }
        }
        Some(CssRule::Charset(CharsetRule {
            encoding: SmolStr::from(encoding),
        }))
    }

    /// Parses an `@namespace` rule.
    fn parse_namespace_rule(&mut self) -> Option<CssRule> {
        let prelude = self.collect_prelude();
        let ns_parts: Vec<&str> = prelude.split_whitespace().collect();
        let (prefix, url) = if ns_parts.len() >= 2 {
            (Some(SmolStr::from(ns_parts[0])), SmolStr::from(ns_parts[1]))
        } else if ns_parts.len() == 1 {
            (None, SmolStr::from(ns_parts[0]))
        } else {
            (None, SmolStr::from(""))
        };
        if let Some(t) = self.peek_token() {
            if t.kind == CssTokenKind::Semicolon {
                self.consume_token();
            }
        }
        Some(CssRule::Namespace(NamespaceRule { prefix, url }))
    }

    /// Parses an `@font-face` rule.
    fn parse_font_face_rule(&mut self) -> Option<CssRule> {
        if !self.expect_token(CssTokenKind::OpenBrace) {
            return None;
        }
        let declarations = self.parse_declarations();
        self.expect_token(CssTokenKind::CloseBrace);
        Some(CssRule::FontFace(FontFaceRule { declarations }))
    }

    /// Parses an `@keyframes` rule.
    fn parse_keyframes_rule(&mut self) -> Option<CssRule> {
        let name = match self.peek_token() {
            Some(t) if t.kind == CssTokenKind::Ident => self.consume_token()?.raw,
            _ => {
                self.diagnostics
                    .push_error(codes::CSS_PARSE_ERROR, "Expected keyframes name");
                return None;
            }
        };

        if !self.expect_token(CssTokenKind::OpenBrace) {
            return None;
        }

        let mut keyframes: Vec<Keyframe> = Vec::new();
        while let Some(t) = self.peek_token() {
            let kind = t.kind;
            match kind {
                CssTokenKind::CloseBrace => {
                    self.consume_token();
                    break;
                }
                CssTokenKind::Eof => break,
                CssTokenKind::Ident | CssTokenKind::Percentage => {
                    let raw = self.consume_token()?.raw;
                    let mut selectors: SmallVec<[SmolStr; 2]> = SmallVec::new();
                    selectors.push(SmolStr::from(raw));
                    while let Some(t) = self.peek_token() {
                        if t.kind == CssTokenKind::Comma {
                            self.consume_token();
                            if let Some(next) = self.peek_token() {
                                if next.kind == CssTokenKind::Ident
                                    || next.kind == CssTokenKind::Percentage
                                {
                                    selectors
                                        .push(SmolStr::from(self.consume_token()?.raw.clone()));
                                }
                            }
                        } else {
                            break;
                        }
                    }
                    if !self.expect_token(CssTokenKind::OpenBrace) {
                        continue;
                    }
                    let declarations = self.parse_declarations();
                    self.expect_token(CssTokenKind::CloseBrace);
                    keyframes.push(Keyframe {
                        selectors,
                        declarations,
                    });
                }
                _ => {
                    self.consume_token();
                }
            }
        }

        Some(CssRule::Keyframes(KeyframesRule {
            name: SmolStr::from(name),
            keyframes,
        }))
    }

    /// Parses an `@supports` rule.
    fn parse_supports_rule(&mut self) -> Option<CssRule> {
        let condition = self.collect_prelude();
        if !self.expect_token(CssTokenKind::OpenBrace) {
            return None;
        }
        let mut rules: Vec<CssRule> = Vec::new();
        while let Some(t) = self.peek_token() {
            let kind = t.kind;
            match kind {
                CssTokenKind::CloseBrace => {
                    self.consume_token();
                    break;
                }
                CssTokenKind::Eof => break,
                _ => {
                    if let Some(rule) = self.parse_style_rule() {
                        rules.push(rule);
                    } else {
                        self.consume_token();
                    }
                }
            }
        }
        Some(CssRule::Supports(SupportsRule { condition, rules }))
    }

    /// Parses an `@page` rule.
    fn parse_page_rule(&mut self) -> Option<CssRule> {
        let pseudo = if let Some(t) = self.peek_token() {
            if t.kind == CssTokenKind::Colon {
                self.consume_token();
                match self.peek_token() {
                    Some(p) if p.kind == CssTokenKind::Ident => {
                        Some(SmolStr::from(self.consume_token()?.raw))
                    }
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        if !self.expect_token(CssTokenKind::OpenBrace) {
            return None;
        }
        let declarations = self.parse_declarations();
        self.expect_token(CssTokenKind::CloseBrace);

        Some(CssRule::Page(PageRule {
            pseudo,
            declarations,
        }))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::panic)]
    use super::*;

    #[test]
    fn test_css_parse_empty() {
        let mut parser = CssParser::new("");
        let sheet = parser.parse().expect("Failed to parse");
        assert!(sheet.rules.is_empty());
    }

    #[test]
    fn test_css_parse_style_rule() {
        let mut parser = CssParser::new("div { color: red; }");
        let sheet = parser.parse().expect("Failed to parse");
        assert_eq!(sheet.rules.len(), 1);

        if let CssRule::Style(sr) = &sheet.rules[0] {
            assert_eq!(sr.selectors.len(), 1);
            assert_eq!(sr.declarations.len(), 1);
            assert_eq!(sr.declarations[0].property.as_str(), "color");
            assert_eq!(sr.declarations[0].value, "red");
            assert!(!sr.declarations[0].important);
        } else {
            panic!("Expected Style rule");
        }
    }

    #[test]
    fn test_css_parse_multiple_declarations() {
        let mut parser = CssParser::new("div { color: red; font-size: 16px; margin: 0; }");
        let sheet = parser.parse().expect("Failed to parse");
        assert_eq!(sheet.rules.len(), 1);

        if let CssRule::Style(sr) = &sheet.rules[0] {
            assert_eq!(sr.declarations.len(), 3);
            assert_eq!(sr.declarations[0].property.as_str(), "color");
            assert_eq!(sr.declarations[1].property.as_str(), "font-size");
            assert_eq!(sr.declarations[2].property.as_str(), "margin");
        } else {
            panic!("Expected Style rule");
        }
    }

    #[test]
    fn test_css_parse_class_selector() {
        let mut parser = CssParser::new(".container { padding: 10px; }");
        let sheet = parser.parse().expect("Failed to parse");
        assert_eq!(sheet.rules.len(), 1);
        if let CssRule::Style(sr) = &sheet.rules[0] {
            assert_eq!(sr.selectors.len(), 1);
            match &sr.selectors[0].simple_selectors[0] {
                SimpleSelector::Class(name) => {
                    assert_eq!(name.as_str(), "container");
                }
                _ => panic!("Expected Class selector"),
            }
        }
    }

    #[test]
    fn test_css_parse_id_selector() {
        let mut parser = CssParser::new("#header { background: blue; }");
        let sheet = parser.parse().expect("Failed to parse");
        if let CssRule::Style(sr) = &sheet.rules[0] {
            assert_eq!(sr.selectors.len(), 1);
            match &sr.selectors[0].simple_selectors[0] {
                SimpleSelector::Id(name) => {
                    assert_eq!(name.as_str(), "header");
                }
                _ => panic!("Expected Id selector"),
            }
        }
    }

    #[test]
    fn test_css_parse_at_media_rule() {
        let mut parser = CssParser::new("@media screen { div { color: black; } }");
        let sheet = parser.parse().expect("Failed to parse");
        assert!(!sheet.rules.is_empty());

        let has_media = sheet.rules.iter().any(|r| matches!(r, CssRule::Media(_)));
        assert!(has_media, "Expected at least one @media rule");

        if let CssRule::Media(mr) = &sheet.rules[0] {
            assert_eq!(mr.rules.len(), 1);
            if let CssRule::Style(sr) = &mr.rules[0] {
                assert_eq!(sr.declarations.len(), 1);
                assert_eq!(sr.declarations[0].property.as_str(), "color");
            }
        }
    }

    #[test]
    fn test_css_parse_universal_selector() {
        let mut parser = CssParser::new("* { margin: 0; }");
        let sheet = parser.parse().expect("Failed to parse");
        if let CssRule::Style(sr) = &sheet.rules[0] {
            assert_eq!(sr.selectors.len(), 1);
            match &sr.selectors[0].simple_selectors[0] {
                SimpleSelector::Universal => {}
                _ => panic!("Expected Universal selector"),
            }
        }
    }

    #[test]
    fn test_css_parse_important() {
        let mut parser = CssParser::new("div { color: red !important; }");
        let sheet = parser.parse().expect("Failed to parse");
        if let CssRule::Style(sr) = &sheet.rules[0] {
            assert_eq!(sr.declarations.len(), 1);
            assert!(sr.declarations[0].important);
        }
    }

    #[test]
    fn test_css_parse_grouped_selectors() {
        let mut parser = CssParser::new("div, span, p { color: red; }");
        let sheet = parser.parse().expect("Failed to parse");
        if let CssRule::Style(sr) = &sheet.rules[0] {
            assert_eq!(sr.selectors.len(), 3);
        }
    }

    #[test]
    fn test_css_parse_empty_block() {
        let mut parser = CssParser::new("div {}");
        let sheet = parser.parse().expect("Failed to parse");
        if let CssRule::Style(sr) = &sheet.rules[0] {
            assert!(sr.declarations.is_empty());
        }
    }

    #[test]
    fn test_css_parse_keyframes() {
        let mut parser =
            CssParser::new("@keyframes fade-in { from { opacity: 0; } to { opacity: 1; } }");
        let sheet = parser.parse().expect("Failed to parse");
        assert!(!sheet.rules.is_empty());

        let has_keyframes = sheet
            .rules
            .iter()
            .any(|r| matches!(r, CssRule::Keyframes(_)));
        assert!(has_keyframes);

        if let CssRule::Keyframes(kf) = &sheet.rules[0] {
            assert_eq!(kf.name.as_str(), "fade-in");
            assert_eq!(kf.keyframes.len(), 2);
        }
    }

    #[test]
    fn test_css_parse_font_face() {
        let mut parser =
            CssParser::new("@font-face { font-family: 'MyFont'; src: url('font.woff2'); }");
        let sheet = parser.parse().expect("Failed to parse");
        let has_font = sheet
            .rules
            .iter()
            .any(|r| matches!(r, CssRule::FontFace(_)));
        assert!(has_font);
    }

    #[test]
    fn test_css_parse_supports() {
        let mut parser = CssParser::new("@supports (display: flex) { div { display: flex; } }");
        let sheet = parser.parse().expect("Failed to parse");
        let has_supports = sheet
            .rules
            .iter()
            .any(|r| matches!(r, CssRule::Supports(_)));
        assert!(has_supports);
    }
}
