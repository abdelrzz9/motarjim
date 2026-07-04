use crate::cursor::Cursor;
use crate::token::Token;

/// CSS token kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssTokenKind {
    /// `selector {`
    OpenBrace,
    /// `}`
    CloseBrace,
    /// `(`
    OpenParen,
    /// `)`
    CloseParen,
    /// `;`
    Semicolon,
    /// `:`
    Colon,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `#`
    Hash,
    /// `*`
    Star,
    /// `~`
    Tilde,
    /// `>`
    GreaterThan,
    /// `+`
    Plus,
    /// `[`
    OpenBracket,
    /// `]`
    CloseBracket,
    /// `=`
    Equals,
    /// `|`
    Pipe,
    /// `^`
    Caret,
    /// `$`
    Dollar,
    /// An identifier (selector, property name, etc.)
    Ident,
    /// A string value
    String,
    /// A numeric value
    Number,
    /// A dimension (number + unit)
    Dimension,
    /// A percentage value
    Percentage,
    /// A URL
    Url,
    /// `@import`, `@media`, etc.
    AtKeyword,
    /// `!important`
    Important,
    /// Whitespace
    Whitespace,
    /// Comment `/* ... */`
    Comment,
    /// End of input
    Eof,
}

/// CSS tokenizer.
#[derive(Debug, Clone)]
pub struct CssTokenizer<'a> {
    /// Internal cursor over the source text.
    cursor: Cursor<'a>,
}

impl<'a> CssTokenizer<'a> {
    /// Creates a new CSS tokenizer.
    #[must_use]
    pub const fn new(source: &'a str) -> Self {
        Self {
            cursor: Cursor::new(source),
        }
    }

    /// Tokenizes the entire input and returns all tokens.
    pub fn tokenize(&mut self) -> Vec<Token<CssTokenKind>> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            let is_eof = token.kind == CssTokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }

    /// Returns the next token.
    fn next_token(&mut self) -> Token<CssTokenKind> {
        self.skip_whitespace_and_comments();

        if self.cursor.is_eof() {
            let pos = self.cursor.pos();
            return Token::new(CssTokenKind::Eof, self.cursor.span_since(pos), "");
        }

        let start = self.cursor.pos();
        let c = self.cursor.peek().expect("not EOF, so character exists");

        let (kind, raw) = match c {
            '{' => {
                self.cursor.advance();
                (CssTokenKind::OpenBrace, "{")
            }
            '}' => {
                self.cursor.advance();
                (CssTokenKind::CloseBrace, "}")
            }
            '(' => {
                self.cursor.advance();
                (CssTokenKind::OpenParen, "(")
            }
            ')' => {
                self.cursor.advance();
                (CssTokenKind::CloseParen, ")")
            }
            ';' => {
                self.cursor.advance();
                (CssTokenKind::Semicolon, ";")
            }
            ',' => {
                self.cursor.advance();
                (CssTokenKind::Comma, ",")
            }
            ':' => {
                self.cursor.advance();
                (CssTokenKind::Colon, ":")
            }
            '~' => {
                self.cursor.advance();
                (CssTokenKind::Tilde, "~")
            }
            '>' => {
                self.cursor.advance();
                (CssTokenKind::GreaterThan, ">")
            }
            '+' => {
                self.cursor.advance();
                (CssTokenKind::Plus, "+")
            }
            '[' => {
                self.cursor.advance();
                (CssTokenKind::OpenBracket, "[")
            }
            ']' => {
                self.cursor.advance();
                (CssTokenKind::CloseBracket, "]")
            }
            '=' => {
                self.cursor.advance();
                (CssTokenKind::Equals, "=")
            }
            '|' => {
                self.cursor.advance();
                (CssTokenKind::Pipe, "|")
            }
            '^' => {
                self.cursor.advance();
                (CssTokenKind::Caret, "^")
            }
            '$' => {
                self.cursor.advance();
                (CssTokenKind::Dollar, "$")
            }
            '*' => {
                self.cursor.advance();
                (CssTokenKind::Star, "*")
            }
            '.' => {
                self.cursor.advance();
                (CssTokenKind::Dot, ".")
            }
            '#' => {
                self.cursor.advance();
                (CssTokenKind::Hash, "#")
            }
            '"' | '\'' => return self.read_string(),
            '@' => {
                self.cursor.advance();
                let name = self.cursor.take_while(|c| c.is_alphanumeric() || c == '-');
                return Token::new(
                    CssTokenKind::AtKeyword,
                    self.cursor.span_since(start),
                    format!("@{name}"),
                );
            }
            _ if c.is_ascii_digit()
                || (c == '.' && self.cursor.peek_at(1).is_some_and(|d| d.is_ascii_digit())) =>
            {
                return self.read_numeric();
            }
            _ if is_ident_start(c) => {
                let ident = self.cursor.take_while(is_ident_part);
                let lower = ident.to_ascii_lowercase();
                if lower == "url" && self.cursor.peek() == Some('(') {
                    return self.read_url();
                }
                if lower == "important" {
                    return Token::new(
                        CssTokenKind::Important,
                        self.cursor.span_since(start),
                        ident,
                    );
                }
                return Token::new(CssTokenKind::Ident, self.cursor.span_since(start), ident);
            }
            _ => {
                self.cursor.advance();
                (CssTokenKind::Ident, " ")
            }
        };

        Token::new(kind, self.cursor.span_since(start), raw)
    }

    /// Reads a CSS quoted string token.
    fn read_string(&mut self) -> Token<CssTokenKind> {
        let start = self.cursor.pos();
        let quote = self
            .cursor
            .advance()
            .expect("quoted string must have opening quote");
        let content = self.cursor.take_while(|c| c != quote);
        self.cursor.advance(); // consume closing quote
        Token::new(
            CssTokenKind::String,
            self.cursor.span_since(start),
            content.to_string(),
        )
    }

    /// Reads a CSS numeric token (number, dimension, or percentage).
    fn read_numeric(&mut self) -> Token<CssTokenKind> {
        let start = self.cursor.pos();
        self.cursor.take_while(|c| c.is_ascii_digit() || c == '.');
        if self.cursor.peek().is_some_and(is_ident_start) {
            let unit = self.cursor.take_while(is_ident_part);
            Token::new(
                CssTokenKind::Dimension,
                self.cursor.span_since(start),
                unit.to_string(),
            )
        } else if self.cursor.peek() == Some('%') {
            self.cursor.advance();
            Token::new(CssTokenKind::Percentage, self.cursor.span_since(start), "%")
        } else {
            Token::new(
                CssTokenKind::Number,
                self.cursor.span_since(start),
                String::new(),
            )
        }
    }

    /// Reads a CSS `url()` token.
    fn read_url(&mut self) -> Token<CssTokenKind> {
        let start = self.cursor.pos();
        self.cursor.take_while(|c| c != ')');
        if self.cursor.peek() == Some(')') {
            self.cursor.advance();
        }
        Token::new(
            CssTokenKind::Url,
            self.cursor.span_since(start),
            String::new(),
        )
    }

    /// Skips whitespace and `/* ... */` comments in the CSS source.
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            self.cursor.skip_whitespace();
            if self.cursor.peek() == Some('/') && self.cursor.peek_at(1) == Some('*') {
                self.cursor.advance();
                self.cursor.advance();
                while !(self.cursor.peek() == Some('*') && self.cursor.peek_at(1) == Some('/')) {
                    if self.cursor.is_eof() {
                        break;
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
}

/// Returns `true` if `c` is a valid CSS identifier start character.
fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == '-' || c > '\x7f'
}

/// Returns `true` if `c` is a valid CSS identifier part character.
fn is_ident_part(c: char) -> bool {
    is_ident_start(c) || c.is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_rule() {
        let mut t = CssTokenizer::new("div { color: red; }");
        let tokens = t.tokenize();
        let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        assert_eq!(kinds[0], CssTokenKind::Ident);
        assert_eq!(kinds[1], CssTokenKind::OpenBrace);
        assert_eq!(kinds[2], CssTokenKind::Ident);
        assert_eq!(kinds[3], CssTokenKind::Colon);
        assert_eq!(kinds[4], CssTokenKind::Ident);
        assert_eq!(kinds[5], CssTokenKind::Semicolon);
        assert_eq!(kinds[6], CssTokenKind::CloseBrace);
        assert_eq!(kinds[7], CssTokenKind::Eof);
    }

    #[test]
    fn test_class_selector() {
        let mut t = CssTokenizer::new(".container { }");
        let tokens = t.tokenize();
        assert_eq!(tokens[0].kind, CssTokenKind::Dot);
        assert_eq!(tokens[1].kind, CssTokenKind::Ident);
        assert_eq!(tokens[2].kind, CssTokenKind::OpenBrace);
        assert_eq!(tokens[3].kind, CssTokenKind::CloseBrace);
    }

    #[test]
    fn test_id_selector() {
        let mut t = CssTokenizer::new("#main { }");
        let tokens = t.tokenize();
        assert_eq!(tokens[0].kind, CssTokenKind::Hash);
        assert_eq!(tokens[1].kind, CssTokenKind::Ident);
        assert_eq!(tokens[2].kind, CssTokenKind::OpenBrace);
        assert_eq!(tokens[3].kind, CssTokenKind::CloseBrace);
    }

    #[test]
    fn test_at_rule() {
        let mut t = CssTokenizer::new("@media screen { }");
        let tokens = t.tokenize();
        assert_eq!(tokens[0].kind, CssTokenKind::AtKeyword);
        assert_eq!(tokens[1].kind, CssTokenKind::Ident);
        assert_eq!(tokens[2].kind, CssTokenKind::OpenBrace);
    }

    #[test]
    fn test_string_value() {
        let mut t = CssTokenizer::new(r#"content: "hello";"#);
        let tokens = t.tokenize();
        assert_eq!(tokens[2].kind, CssTokenKind::String);
    }

    #[test]
    fn test_numeric_value() {
        let mut t = CssTokenizer::new("width: 100px;");
        let tokens = t.tokenize();
        let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        assert_eq!(kinds[0], CssTokenKind::Ident);
        assert_eq!(kinds[1], CssTokenKind::Colon);
        assert_eq!(kinds[2], CssTokenKind::Dimension);
        assert_eq!(kinds[3], CssTokenKind::Semicolon);
        assert_eq!(kinds[4], CssTokenKind::Eof);
    }

    #[test]
    fn test_empty_input() {
        let mut t = CssTokenizer::new("");
        let tokens = t.tokenize();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, CssTokenKind::Eof);
    }

    #[test]
    fn test_comment_skipped() {
        let mut t = CssTokenizer::new("div /* comment */ { }");
        let tokens = t.tokenize();
        assert_eq!(tokens[0].kind, CssTokenKind::Ident);
        assert_eq!(tokens[1].kind, CssTokenKind::OpenBrace);
    }

    #[test]
    fn test_multiple_selectors() {
        let mut t = CssTokenizer::new("div, span { }");
        let tokens = t.tokenize();
        assert_eq!(tokens[0].kind, CssTokenKind::Ident);
        assert_eq!(tokens[1].kind, CssTokenKind::Comma);
        assert_eq!(tokens[2].kind, CssTokenKind::Ident);
        assert_eq!(tokens[3].kind, CssTokenKind::OpenBrace);
    }
}
