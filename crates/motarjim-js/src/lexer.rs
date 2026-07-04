//! Zero-copy tokenizer for JavaScript source text.

use motarjim_lexer::{Cursor, Token};

use crate::token::{keyword_from_str, JsTokenKind};

/// Tokenizes JavaScript source text into a flat token stream.
///
/// Built on top of [`motarjim_lexer::Cursor`] for position tracking, mirroring
/// the HTML and CSS tokenizers in `motarjim-lexer`.
///
/// # Example
///
/// ```rust
/// use motarjim_js::JsLexer;
///
/// let mut lexer = JsLexer::new("let x = 1;");
/// let tokens = lexer.tokenize();
/// assert!(!tokens.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct JsLexer<'a> {
    /// The full source text, kept alongside the cursor so raw slices can be
    /// recovered for tokens that require custom scanning (template literals).
    source: &'a str,
    /// Internal cursor over the source text.
    cursor: Cursor<'a>,
}

impl<'a> JsLexer<'a> {
    /// Creates a new lexer over the given source text.
    #[must_use]
    pub const fn new(source: &'a str) -> Self {
        Self {
            source,
            cursor: Cursor::new(source),
        }
    }

    /// Tokenizes the entire input and returns all tokens, ending with `Eof`.
    pub fn tokenize(&mut self) -> Vec<Token<JsTokenKind>> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            let is_eof = token.kind == JsTokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }

    /// Returns the source slice from `start` to the cursor's current position.
    fn raw_since(&self, start: usize) -> &'a str {
        &self.source[start..self.cursor.pos()]
    }

    /// Scans and returns the next token.
    fn next_token(&mut self) -> Token<JsTokenKind> {
        self.skip_trivia();

        if self.cursor.is_eof() {
            let pos = self.cursor.pos();
            return Token::new(JsTokenKind::Eof, self.cursor.span_since(pos), "");
        }

        let start = self.cursor.pos();
        let c = self.cursor.peek().expect("not eof, so a character exists");

        match c {
            '`' => self.read_template(start),
            '"' | '\'' => self.read_string(start, c),
            _ if c.is_ascii_digit() => self.read_number(start),
            _ if is_ident_start(c) => self.read_ident_or_keyword(start),
            _ => self.read_punct(start),
        }
    }

    /// Skips whitespace, `//` line comments, and `/* */` block comments.
    fn skip_trivia(&mut self) {
        loop {
            self.cursor.skip_whitespace();
            if self.cursor.peek() == Some('/') && self.cursor.peek_at(1) == Some('/') {
                self.cursor.take_while(|c| c != '\n');
            } else if self.cursor.peek() == Some('/') && self.cursor.peek_at(1) == Some('*') {
                self.cursor.advance();
                self.cursor.advance();
                while !(self.cursor.peek() == Some('*') && self.cursor.peek_at(1) == Some('/')) {
                    if self.cursor.is_eof() {
                        return;
                    }
                    self.cursor.advance();
                }
                self.cursor.advance();
                self.cursor.advance();
            } else {
                break;
            }
        }
    }

    /// Reads a single- or double-quoted string literal.
    ///
    /// The returned raw text is the content between the quotes, with escape
    /// sequences left unprocessed (callers that need literal string values
    /// must unescape them separately).
    fn read_string(&mut self, start: usize, quote: char) -> Token<JsTokenKind> {
        self.cursor.advance(); // opening quote
        loop {
            match self.cursor.peek() {
                None => break,
                Some('\\') => {
                    self.cursor.advance();
                    self.cursor.advance();
                }
                Some(c) if c == quote => {
                    self.cursor.advance();
                    break;
                }
                Some(_) => {
                    self.cursor.advance();
                }
            }
        }
        let raw = self.raw_since(start);
        let inner = raw
            .strip_prefix(quote)
            .and_then(|s| s.strip_suffix(quote))
            .unwrap_or(raw);
        Token::new(JsTokenKind::String, self.cursor.span_since(start), inner)
    }

    /// Reads a backtick-delimited template literal, including any nested
    /// `${ ... }` interpolations. The raw text includes the surrounding
    /// backticks; splitting it into static and dynamic parts is the
    /// responsibility of the parser (see `crate::parser::split_template`).
    fn read_template(&mut self, start: usize) -> Token<JsTokenKind> {
        self.cursor.advance(); // opening backtick
        skip_template_body(&mut self.cursor);
        let raw = self.raw_since(start);
        Token::new(
            JsTokenKind::TemplateString,
            self.cursor.span_since(start),
            raw,
        )
    }

    /// Reads a numeric literal: decimal, fractional, exponent, or `0x` hex.
    fn read_number(&mut self, start: usize) -> Token<JsTokenKind> {
        if self.cursor.peek() == Some('0') && matches!(self.cursor.peek_at(1), Some('x' | 'X')) {
            self.cursor.advance();
            self.cursor.advance();
            self.cursor
                .take_while(|c| c.is_ascii_hexdigit() || c == '_');
        } else {
            self.cursor.take_while(|c| c.is_ascii_digit() || c == '_');
            if self.cursor.peek() == Some('.') {
                self.cursor.advance();
                self.cursor.take_while(|c| c.is_ascii_digit() || c == '_');
            }
            if matches!(self.cursor.peek(), Some('e' | 'E')) {
                self.cursor.advance();
                if matches!(self.cursor.peek(), Some('+' | '-')) {
                    self.cursor.advance();
                }
                self.cursor.take_while(|c| c.is_ascii_digit());
            }
        }
        let raw = self.raw_since(start);
        Token::new(JsTokenKind::Number, self.cursor.span_since(start), raw)
    }

    /// Reads an identifier and resolves it to a keyword token if reserved.
    fn read_ident_or_keyword(&mut self, start: usize) -> Token<JsTokenKind> {
        self.cursor.take_while(is_ident_part);
        let raw = self.raw_since(start);
        let kind = keyword_from_str(raw).unwrap_or(JsTokenKind::Identifier);
        Token::new(kind, self.cursor.span_since(start), raw)
    }

    /// Reads a punctuation or operator token, greedily matching the longest
    /// operator that starts at the current position.
    fn read_punct(&mut self, start: usize) -> Token<JsTokenKind> {
        let c = self
            .cursor
            .advance()
            .expect("caller checked cursor is not eof");
        let kind = match c {
            '(' => JsTokenKind::LParen,
            ')' => JsTokenKind::RParen,
            '{' => JsTokenKind::LBrace,
            '}' => JsTokenKind::RBrace,
            '[' => JsTokenKind::LBracket,
            ']' => JsTokenKind::RBracket,
            ';' => JsTokenKind::Semicolon,
            ',' => JsTokenKind::Comma,
            ':' => JsTokenKind::Colon,
            '.' => JsTokenKind::Dot,
            '?' => {
                if self.cursor.peek() == Some('?') {
                    self.cursor.advance();
                    JsTokenKind::Nullish
                } else {
                    JsTokenKind::Question
                }
            }
            '+' => {
                if self.cursor.peek() == Some('+') {
                    self.cursor.advance();
                    JsTokenKind::Increment
                } else if self.cursor.peek() == Some('=') {
                    self.cursor.advance();
                    JsTokenKind::PlusAssign
                } else {
                    JsTokenKind::Plus
                }
            }
            '-' => {
                if self.cursor.peek() == Some('-') {
                    self.cursor.advance();
                    JsTokenKind::Decrement
                } else if self.cursor.peek() == Some('=') {
                    self.cursor.advance();
                    JsTokenKind::MinusAssign
                } else {
                    JsTokenKind::Minus
                }
            }
            '*' => {
                if self.cursor.peek() == Some('=') {
                    self.cursor.advance();
                    JsTokenKind::StarAssign
                } else {
                    JsTokenKind::Star
                }
            }
            '/' => {
                if self.cursor.peek() == Some('=') {
                    self.cursor.advance();
                    JsTokenKind::SlashAssign
                } else {
                    JsTokenKind::Slash
                }
            }
            '%' => {
                if self.cursor.peek() == Some('=') {
                    self.cursor.advance();
                    JsTokenKind::PercentAssign
                } else {
                    JsTokenKind::Percent
                }
            }
            '=' => {
                if self.cursor.peek() == Some('=') && self.cursor.peek_at(1) == Some('=') {
                    self.cursor.advance();
                    self.cursor.advance();
                    JsTokenKind::EqEqEq
                } else if self.cursor.peek() == Some('=') {
                    self.cursor.advance();
                    JsTokenKind::EqEq
                } else if self.cursor.peek() == Some('>') {
                    self.cursor.advance();
                    JsTokenKind::Arrow
                } else {
                    JsTokenKind::Assign
                }
            }
            '!' => {
                if self.cursor.peek() == Some('=') && self.cursor.peek_at(1) == Some('=') {
                    self.cursor.advance();
                    self.cursor.advance();
                    JsTokenKind::NotEqEq
                } else if self.cursor.peek() == Some('=') {
                    self.cursor.advance();
                    JsTokenKind::NotEq
                } else {
                    JsTokenKind::Bang
                }
            }
            '<' => {
                if self.cursor.peek() == Some('=') {
                    self.cursor.advance();
                    JsTokenKind::LtEq
                } else {
                    JsTokenKind::Lt
                }
            }
            '>' => {
                if self.cursor.peek() == Some('=') {
                    self.cursor.advance();
                    JsTokenKind::GtEq
                } else {
                    JsTokenKind::Gt
                }
            }
            '&' if self.cursor.peek() == Some('&') => {
                self.cursor.advance();
                JsTokenKind::AmpAmp
            }
            '|' if self.cursor.peek() == Some('|') => {
                self.cursor.advance();
                JsTokenKind::PipePipe
            }
            _ => JsTokenKind::Identifier,
        };
        let raw = self.raw_since(start);
        Token::new(kind, self.cursor.span_since(start), raw)
    }
}

/// Skips over a template literal body (the opening backtick must already be
/// consumed) up to and including its closing backtick. Any `${ ... }`
/// interpolation is skipped as an opaque, brace-balanced region so it may
/// contain arbitrary expressions, including nested template literals.
fn skip_template_body(cursor: &mut Cursor<'_>) {
    loop {
        match cursor.peek() {
            None => break,
            Some('`') => {
                cursor.advance();
                break;
            }
            Some('\\') => {
                cursor.advance();
                cursor.advance();
            }
            Some('$') if cursor.peek_at(1) == Some('{') => {
                cursor.advance();
                cursor.advance();
                skip_balanced_braces(cursor);
            }
            Some(_) => {
                cursor.advance();
            }
        }
    }
}

/// Skips a `${ ... }` interpolation body (the opening `{` must already be
/// consumed) up to and including its matching closing `}`. String and
/// template literals encountered inside are skipped atomically so their
/// contents never miscount the brace depth.
fn skip_balanced_braces(cursor: &mut Cursor<'_>) {
    let mut depth: u32 = 1;
    while depth > 0 {
        match cursor.peek() {
            None => break,
            Some('{') => {
                depth += 1;
                cursor.advance();
            }
            Some('}') => {
                depth -= 1;
                cursor.advance();
            }
            Some(quote @ ('\'' | '"')) => {
                cursor.advance();
                skip_string_body(cursor, quote);
            }
            Some('`') => {
                cursor.advance();
                skip_template_body(cursor);
            }
            Some(_) => {
                cursor.advance();
            }
        }
    }
}

/// Skips a quoted string body (the opening quote must already be consumed)
/// up to and including its closing quote.
fn skip_string_body(cursor: &mut Cursor<'_>, quote: char) {
    loop {
        match cursor.peek() {
            None => break,
            Some('\\') => {
                cursor.advance();
                cursor.advance();
            }
            Some(c) if c == quote => {
                cursor.advance();
                break;
            }
            Some(_) => {
                cursor.advance();
            }
        }
    }
}

/// Returns `true` if `c` is a valid identifier start character.
fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == '$'
}

/// Returns `true` if `c` is a valid identifier continuation character.
fn is_ident_part(c: char) -> bool {
    is_ident_start(c) || c.is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<JsTokenKind> {
        JsLexer::new(src)
            .tokenize()
            .iter()
            .map(|t| t.kind)
            .collect()
    }

    #[test]
    fn test_var_decl() {
        let k = kinds("let x = 1;");
        assert_eq!(
            k,
            vec![
                JsTokenKind::Let,
                JsTokenKind::Identifier,
                JsTokenKind::Assign,
                JsTokenKind::Number,
                JsTokenKind::Semicolon,
                JsTokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_arrow_function() {
        let k = kinds("(x) => x + 1");
        assert!(k.contains(&JsTokenKind::Arrow));
    }

    #[test]
    fn test_string_literal() {
        let mut lexer = JsLexer::new(r#"'hello'"#);
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0].kind, JsTokenKind::String);
        assert_eq!(tokens[0].raw, "hello");
    }

    #[test]
    fn test_template_literal_raw_includes_backticks() {
        let mut lexer = JsLexer::new("`hi ${name}!`");
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0].kind, JsTokenKind::TemplateString);
        assert_eq!(tokens[0].raw, "`hi ${name}!`");
    }

    #[test]
    fn test_template_literal_with_nested_braces_in_string() {
        let mut lexer = JsLexer::new(r#"`${ "{" }`"#);
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0].kind, JsTokenKind::TemplateString);
        assert_eq!(tokens[0].kind, JsTokenKind::TemplateString);
    }

    #[test]
    fn test_template_literal_nested_template() {
        let mut lexer = JsLexer::new("`a${`b${c}`}d`");
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0].kind, JsTokenKind::TemplateString);
        assert_eq!(tokens[0].raw, "`a${`b${c}`}d`");
    }

    #[test]
    fn test_operators() {
        let k = kinds("a === b !== c <= d >= e && f || g ?? h");
        assert!(k.contains(&JsTokenKind::EqEqEq));
        assert!(k.contains(&JsTokenKind::NotEqEq));
        assert!(k.contains(&JsTokenKind::LtEq));
        assert!(k.contains(&JsTokenKind::GtEq));
        assert!(k.contains(&JsTokenKind::AmpAmp));
        assert!(k.contains(&JsTokenKind::PipePipe));
        assert!(k.contains(&JsTokenKind::Nullish));
    }

    #[test]
    fn test_increment_decrement() {
        let k = kinds("i++; --j;");
        assert!(k.contains(&JsTokenKind::Increment));
        assert!(k.contains(&JsTokenKind::Decrement));
    }

    #[test]
    fn test_line_comment_skipped() {
        let k = kinds("let x = 1; // comment\nlet y = 2;");
        assert_eq!(k.iter().filter(|t| **t == JsTokenKind::Let).count(), 2);
    }

    #[test]
    fn test_block_comment_skipped() {
        let k = kinds("let /* comment */ x = 1;");
        assert_eq!(k[0], JsTokenKind::Let);
        assert_eq!(k[1], JsTokenKind::Identifier);
    }

    #[test]
    fn test_hex_number() {
        let mut lexer = JsLexer::new("0xFF");
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0].kind, JsTokenKind::Number);
        assert_eq!(tokens[0].raw, "0xFF");
    }

    #[test]
    fn test_float_with_exponent() {
        let mut lexer = JsLexer::new("1.5e10");
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0].kind, JsTokenKind::Number);
        assert_eq!(tokens[0].raw, "1.5e10");
    }

    #[test]
    fn test_import_export_keywords() {
        let k = kinds("import x from 'mod'; export default x;");
        assert_eq!(
            k,
            vec![
                JsTokenKind::Import,
                JsTokenKind::Identifier,
                JsTokenKind::From,
                JsTokenKind::String,
                JsTokenKind::Semicolon,
                JsTokenKind::Export,
                JsTokenKind::Default,
                JsTokenKind::Identifier,
                JsTokenKind::Semicolon,
                JsTokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_dollar_sign_identifier() {
        let k = kinds("$el.addEventListener");
        assert_eq!(k[0], JsTokenKind::Identifier);
    }

    #[test]
    fn test_empty_input() {
        let mut lexer = JsLexer::new("");
        let tokens = lexer.tokenize();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, JsTokenKind::Eof);
    }
}
