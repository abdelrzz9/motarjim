use crate::cursor::Cursor;
use crate::token::Token;

/// HTML token kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlTokenKind {
    /// `<!DOCTYPE html>`
    Doctype,
    /// `<tag`
    OpenTagStart,
    /// `</tag`
    CloseTagStart,
    /// `>`
    TagEnd,
    /// `/>`
    SelfClosingTagEnd,
    /// Attribute name (e.g. `class`)
    AttrName,
    /// `=`
    AttrEquals,
    /// `"value"` or `'value'`
    AttrValue,
    /// Text content between tags
    Text,
    /// `<!-- ... -->`
    Comment,
    /// `<!` (other markup declaration)
    MarkupDeclaration,
    /// End of input
    Eof,
}

/// HTML tokenizer.
#[derive(Debug, Clone)]
pub struct HtmlTokenizer<'a> {
    /// Internal cursor over the source text.
    cursor: Cursor<'a>,
}

impl<'a> HtmlTokenizer<'a> {
    /// Creates a new HTML tokenizer for the given source.
    #[must_use]
    pub const fn new(source: &'a str) -> Self {
        Self { cursor: Cursor::new(source) }
    }

    /// Tokenizes the entire input and returns all tokens.
    pub fn tokenize(&mut self) -> Vec<Token<HtmlTokenKind>> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            let is_eof = token.kind == HtmlTokenKind::Eof;
            tokens.push(token);
            if is_eof { break; }
        }
        tokens
    }

    /// Returns the next token.
    fn next_token(&mut self) -> Token<HtmlTokenKind> {
        if self.cursor.is_eof() {
            let pos = self.cursor.pos();
            return Token::new(HtmlTokenKind::Eof, self.cursor.span_since(pos), "");
        }

        let start = self.cursor.pos();
        match self.cursor.peek() {
            Some('<') => self.read_tag_start(start),
            Some('>') => {
                self.cursor.advance();
                Token::new(HtmlTokenKind::TagEnd, self.cursor.span_since(start), ">")
            }
            Some('/') if self.cursor.peek_at(1) == Some('>') => {
                self.cursor.advance();
                self.cursor.advance();
                Token::new(HtmlTokenKind::TagEnd, self.cursor.span_since(start), "/>")
            }
            _ => {
                let text = self.cursor.take_while(|c| c != '<');
                Token::new(
                    HtmlTokenKind::Text,
                    self.cursor.span_since(start),
                    text.to_string(),
                )
            }
        }
    }

    /// Reads an HTML tag starting at `<`.
    fn read_tag_start(&mut self, start: usize) -> Token<HtmlTokenKind> {
        self.cursor.advance(); // consume '<'

        match self.cursor.peek() {
            Some('/') => {
                self.cursor.advance(); // consume '/'
                let name = self.cursor.take_while(|c| c != '>');
                Token::new(
                    HtmlTokenKind::CloseTagStart,
                    self.cursor.span_since(start),
                    format!("</{name}"),
                )
            }
            Some('!') => {
                self.cursor.advance(); // consume '!'
                if self.cursor.peek() == Some('-') && self.cursor.peek_at(1) == Some('-') {
                    self.cursor.advance();
                    self.cursor.advance(); // consume '--'
                    while !(self.cursor.peek() == Some('-')
                        && self.cursor.peek_at(1) == Some('-')
                        && self.cursor.peek_at(2) == Some('>'))
                    {
                        if self.cursor.is_eof() {
                            break;
                        }
                        self.cursor.advance();
                    }
                    self.cursor.advance();
                    self.cursor.advance();
                    self.cursor.advance(); // consume '-->'
                    Token::new(HtmlTokenKind::Comment, self.cursor.span_since(start), "")
                } else {
                    while self.cursor.peek() != Some('>') && !self.cursor.is_eof() {
                        self.cursor.advance();
                    }
                    if self.cursor.peek() == Some('>') {
                        self.cursor.advance();
                    }
                    Token::new(
                        HtmlTokenKind::MarkupDeclaration,
                        self.cursor.span_since(start),
                        "",
                    )
                }
            }
            Some('?') => {
                self.cursor.advance();
                while self.cursor.peek() != Some('>') && !self.cursor.is_eof() {
                    self.cursor.advance();
                }
                if self.cursor.peek() == Some('>') {
                    self.cursor.advance();
                }
                Token::new(
                    HtmlTokenKind::MarkupDeclaration,
                    self.cursor.span_since(start),
                    "",
                )
            }
            _ => {
                let name =
                    self.cursor.take_while(|c| !c.is_ascii_whitespace() && c != '>' && c != '/');
                Token::new(
                    HtmlTokenKind::OpenTagStart,
                    self.cursor.span_since(start),
                    format!("<{name}"),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tag() {
        let mut t = HtmlTokenizer::new("<div>");
        let tokens = t.tokenize();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, HtmlTokenKind::OpenTagStart);
        assert_eq!(tokens[1].kind, HtmlTokenKind::TagEnd);
        assert_eq!(tokens[2].kind, HtmlTokenKind::Eof);
    }

    #[test]
    fn test_open_close_tag() {
        let mut t = HtmlTokenizer::new("<div></div>");
        let tokens = t.tokenize();
        assert_eq!(tokens[0].kind, HtmlTokenKind::OpenTagStart);
        assert_eq!(tokens[1].kind, HtmlTokenKind::TagEnd);
        assert_eq!(tokens[2].kind, HtmlTokenKind::CloseTagStart);
        assert_eq!(tokens[3].kind, HtmlTokenKind::TagEnd);
        assert_eq!(tokens[4].kind, HtmlTokenKind::Eof);
    }

    #[test]
    fn test_text_content() {
        let mut t = HtmlTokenizer::new("<p>hello</p>");
        let tokens = t.tokenize();
        assert_eq!(tokens[0].kind, HtmlTokenKind::OpenTagStart);
        assert_eq!(tokens[1].kind, HtmlTokenKind::TagEnd);
        assert_eq!(tokens[2].kind, HtmlTokenKind::Text);
        assert_eq!(tokens[2].raw, "hello");
        assert_eq!(tokens[3].kind, HtmlTokenKind::CloseTagStart);
        assert_eq!(tokens[4].kind, HtmlTokenKind::TagEnd);
        assert_eq!(tokens[5].kind, HtmlTokenKind::Eof);
    }

    #[test]
    fn test_self_closing_tag() {
        let mut t = HtmlTokenizer::new("<br/>");
        let tokens = t.tokenize();
        assert_eq!(tokens[0].kind, HtmlTokenKind::OpenTagStart);
        assert_eq!(tokens[1].kind, HtmlTokenKind::TagEnd);
        assert_eq!(tokens[2].kind, HtmlTokenKind::Eof);
    }

    #[test]
    fn test_comment() {
        let mut t = HtmlTokenizer::new("<!-- comment -->");
        let tokens = t.tokenize();
        assert_eq!(tokens[0].kind, HtmlTokenKind::Comment);
        assert_eq!(tokens[1].kind, HtmlTokenKind::Eof);
    }

    #[test]
    fn test_empty_input() {
        let mut t = HtmlTokenizer::new("");
        let tokens = t.tokenize();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, HtmlTokenKind::Eof);
    }

    #[test]
    fn test_nested_tags() {
        let mut t = HtmlTokenizer::new("<ul><li>item</li></ul>");
        let tokens = t.tokenize();
        let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        assert_eq!(kinds[0], HtmlTokenKind::OpenTagStart);
        assert_eq!(kinds[1], HtmlTokenKind::TagEnd);
        assert_eq!(kinds[2], HtmlTokenKind::OpenTagStart);
        assert_eq!(kinds[3], HtmlTokenKind::TagEnd);
        assert_eq!(kinds[4], HtmlTokenKind::Text);
        assert_eq!(kinds[5], HtmlTokenKind::CloseTagStart);
        assert_eq!(kinds[6], HtmlTokenKind::TagEnd);
        assert_eq!(kinds[7], HtmlTokenKind::CloseTagStart);
        assert_eq!(kinds[8], HtmlTokenKind::TagEnd);
        assert_eq!(kinds[9], HtmlTokenKind::Eof);
    }
}
