//! Character-level tokenizer for ECMAScript (JavaScript).

use motarjim_span::SourceSpan;

use crate::token::{keyword_from_str, JsToken, JsTokenKind, TokenValue};

pub struct JsLexer<'a> {
    source: &'a str,
    pos: usize,
    start: usize,
    line: u32,
    column: u32,
    regex_allowed: bool,
}

impl<'a> JsLexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            pos: 0,
            start: 0,
            line: 1,
            column: 1,
            regex_allowed: true,
        }
    }

    pub fn tokenize(&mut self) -> Vec<JsToken> {
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

    pub fn set_regex_allowed(&mut self, allowed: bool) {
        self.regex_allowed = allowed;
    }

    fn next_token(&mut self) -> JsToken {
        self.skip_trivia();
        self.start = self.pos;

        if self.is_eof() {
            return self.make_token(JsTokenKind::Eof);
        }

        let c = self.peek().unwrap();
        match c {
            '\n' | '\r' => {
                self.advance();
                self.make_token(JsTokenKind::Semicolon)
            }
            '{' => self.simple_token(JsTokenKind::LBrace),
            '}' => self.simple_token(JsTokenKind::RBrace),
            '(' => self.simple_token(JsTokenKind::LParen),
            ')' => self.simple_token(JsTokenKind::RParen),
            '[' => self.simple_token(JsTokenKind::LBracket),
            ']' => self.simple_token(JsTokenKind::RBracket),
            ';' => self.simple_token(JsTokenKind::Semicolon),
            ',' => self.simple_token(JsTokenKind::Comma),
            ':' => self.simple_token(JsTokenKind::Colon),
            '?' => self.read_question(),
            '.' => self.read_dot(),
            '+' => self.read_plus(),
            '-' => self.read_minus(),
            '*' => self.read_star(),
            '%' => self.read_percent(),
            '=' => self.read_equals(),
            '!' => self.read_bang(),
            '<' => self.read_lt(),
            '>' => self.read_gt(),
            '&' => self.read_amp(),
            '|' => self.read_pipe(),
            '^' => self.simple_token(JsTokenKind::Caret),
            '~' => self.simple_token(JsTokenKind::Tilde),
            '#' => self.read_hash(),
            '`' => self.read_template(),
            '\'' | '"' => self.read_string(c),
            '/' if !self.regex_allowed => self.read_slash(),
            '/' => self.read_regex_or_slash(),
            _ if c.is_ascii_digit() => self.read_number(),
            _ if is_ident_start(c) => self.read_ident_or_keyword(),
            _ => {
                self.advance();
                self.make_token(JsTokenKind::Error)
            }
        }
    }

    fn skip_trivia(&mut self) {
        loop {
            self.skip_whitespace();
            if self.peek() == Some('/') && self.peek_at(1) == Some('/') {
                self.skip_line_comment();
            } else if self.peek() == Some('/') && self.peek_at(1) == Some('*') {
                self.skip_block_comment();
            } else {
                break;
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            match c {
                ' ' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.advance();
                    self.line += 1;
                    self.column = 1;
                }
                '\r' => {
                    self.advance();
                    if self.peek() == Some('\n') {
                        self.advance();
                    }
                    self.line += 1;
                    self.column = 1;
                }
                _ => break,
            }
        }
    }

    fn skip_line_comment(&mut self) {
        self.advance();
        self.advance();
        while let Some(c) = self.peek() {
            if c == '\n' || c == '\r' {
                break;
            }
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) {
        self.advance();
        self.advance();
        while !self.is_eof() {
            if self.peek() == Some('*') && self.peek_at(1) == Some('/') {
                self.advance();
                self.advance();
                return;
            }
            self.advance();
        }
    }

    // ---- token readers ----------------------------------------------------

    fn simple_token(&mut self, kind: JsTokenKind) -> JsToken {
        self.advance();
        self.make_token(kind)
    }

    fn read_question(&mut self) -> JsToken {
        self.advance();
        match self.peek() {
            Some('?') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    self.make_token(JsTokenKind::NullishAssign)
                } else {
                    self.make_token(JsTokenKind::Nullish)
                }
            }
            Some('.') if self.peek_at(1).map_or(true, |c| !c.is_ascii_digit()) => {
                self.advance();
                self.make_token(JsTokenKind::QuestionDot)
            }
            _ => self.make_token(JsTokenKind::Question),
        }
    }

    fn read_dot(&mut self) -> JsToken {
        self.advance();
        if self.peek() == Some('.') && self.peek_at(1) == Some('.') {
            self.advance();
            self.advance();
            return self.make_token(JsTokenKind::Ellipsis);
        }
        if self.peek().map_or(false, |c| c.is_ascii_digit()) {
            self.read_number_after_dot();
            return self.make_token(JsTokenKind::Number);
        }
        self.make_token(JsTokenKind::Dot)
    }

    fn read_plus(&mut self) -> JsToken {
        self.advance();
        match self.peek() {
            Some('+') => {
                self.advance();
                self.make_token(JsTokenKind::Increment)
            }
            Some('=') => {
                self.advance();
                self.make_token(JsTokenKind::PlusAssign)
            }
            _ => self.make_token(JsTokenKind::Plus),
        }
    }

    fn read_minus(&mut self) -> JsToken {
        self.advance();
        match self.peek() {
            Some('-') => {
                self.advance();
                self.make_token(JsTokenKind::Decrement)
            }
            Some('=') => {
                self.advance();
                self.make_token(JsTokenKind::MinusAssign)
            }
            _ => self.make_token(JsTokenKind::Minus),
        }
    }

    fn read_star(&mut self) -> JsToken {
        self.advance();
        match self.peek() {
            Some('*') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    self.make_token(JsTokenKind::StarStarAssign)
                } else {
                    self.make_token(JsTokenKind::StarStar)
                }
            }
            Some('=') => {
                self.advance();
                self.make_token(JsTokenKind::StarAssign)
            }
            _ => self.make_token(JsTokenKind::Star),
        }
    }

    fn read_percent(&mut self) -> JsToken {
        self.advance();
        if self.peek() == Some('=') {
            self.advance();
            self.make_token(JsTokenKind::PercentAssign)
        } else {
            self.make_token(JsTokenKind::Percent)
        }
    }

    fn read_equals(&mut self) -> JsToken {
        self.advance();
        match self.peek() {
            Some('=') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    self.make_token(JsTokenKind::EqEqEq)
                } else {
                    self.make_token(JsTokenKind::EqEq)
                }
            }
            Some('>') => {
                self.advance();
                self.make_token(JsTokenKind::Arrow)
            }
            _ => self.make_token(JsTokenKind::Assign),
        }
    }

    fn read_bang(&mut self) -> JsToken {
        self.advance();
        match self.peek() {
            Some('=') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    self.make_token(JsTokenKind::NotEqEq)
                } else {
                    self.make_token(JsTokenKind::NotEq)
                }
            }
            _ => self.make_token(JsTokenKind::Bang),
        }
    }

    fn read_lt(&mut self) -> JsToken {
        self.advance();
        match self.peek() {
            Some('<') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    self.make_token(JsTokenKind::LtLtAssign)
                } else {
                    self.make_token(JsTokenKind::LtLt)
                }
            }
            Some('=') => {
                self.advance();
                self.make_token(JsTokenKind::LtEq)
            }
            _ => self.make_token(JsTokenKind::Lt),
        }
    }

    fn read_gt(&mut self) -> JsToken {
        self.advance();
        match self.peek() {
            Some('>') => {
                self.advance();
                if self.peek() == Some('>') {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        self.make_token(JsTokenKind::GtGtGtAssign)
                    } else {
                        self.make_token(JsTokenKind::GtGtGt)
                    }
                } else if self.peek() == Some('=') {
                    self.advance();
                    self.make_token(JsTokenKind::GtGtAssign)
                } else {
                    self.make_token(JsTokenKind::GtGt)
                }
            }
            Some('=') => {
                self.advance();
                self.make_token(JsTokenKind::GtEq)
            }
            _ => self.make_token(JsTokenKind::Gt),
        }
    }

    fn read_amp(&mut self) -> JsToken {
        self.advance();
        match self.peek() {
            Some('&') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    self.make_token(JsTokenKind::AmpAmpAssign)
                } else {
                    self.make_token(JsTokenKind::AmpAmp)
                }
            }
            Some('=') => {
                self.advance();
                self.make_token(JsTokenKind::AmpAssign)
            }
            _ => self.make_token(JsTokenKind::Amp),
        }
    }

    fn read_pipe(&mut self) -> JsToken {
        self.advance();
        match self.peek() {
            Some('|') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    self.make_token(JsTokenKind::PipePipeAssign)
                } else {
                    self.make_token(JsTokenKind::PipePipe)
                }
            }
            Some('=') => {
                self.advance();
                self.make_token(JsTokenKind::PipeAssign)
            }
            _ => self.make_token(JsTokenKind::Pipe),
        }
    }

    fn read_hash(&mut self) -> JsToken {
        self.advance();
        let start = self.pos;
        while self.peek().map_or(false, is_ident_part) {
            self.advance();
        }
        if self.pos > start {
            self.make_token(JsTokenKind::PrivateIdentifier)
        } else {
            self.make_token(JsTokenKind::Error)
        }
    }

    fn read_string(&mut self, quote: char) -> JsToken {
        self.advance();
        let mut value = String::new();
        loop {
            match self.peek() {
                None => break,
                Some('\\') => {
                    self.advance();
                    value.push(self.read_escape());
                }
                Some(c) if c == quote => {
                    self.advance();
                    break;
                }
                Some('\n') | Some('\r') => break,
                Some(c) => {
                    self.advance();
                    value.push(c);
                }
            }
        }
        self.make_token_value(JsTokenKind::String, TokenValue::String(value))
    }

    fn read_template(&mut self) -> JsToken {
        self.advance();
        loop {
            match self.peek() {
                None => return self.make_token(JsTokenKind::UnterminatedTemplate),
                Some('`') => {
                    self.advance();
                    return self.make_token(JsTokenKind::TemplateString);
                }
                Some('\\') => {
                    self.advance();
                    self.advance();
                }
                Some('$') if self.peek_at(1) == Some('{') => {
                    self.advance();
                    self.advance();
                    self.skip_balanced_braces();
                }
                Some(_) => {
                    self.advance();
                }
            }
        }
    }

    fn skip_balanced_braces(&mut self) {
        let mut depth: u32 = 1;
        while depth > 0 {
            match self.peek() {
                None => break,
                Some('{') => {
                    depth += 1;
                    self.advance();
                }
                Some('}') => {
                    depth -= 1;
                    self.advance();
                }
                Some('\'' | '"') => {
                    let q = self.peek().unwrap();
                    self.advance();
                    while self.peek().map_or(false, |c| c != q) {
                        if self.peek() == Some('\\') {
                            self.advance();
                        }
                        self.advance();
                    }
                    self.advance();
                }
                Some('`') => {
                    self.advance();
                    loop {
                        match self.peek() {
                            None | Some('`') => {
                                self.advance();
                                break;
                            }
                            Some('\\') => {
                                self.advance();
                                self.advance();
                            }
                            Some('$') if self.peek_at(1) == Some('{') => {
                                self.advance();
                                self.advance();
                                self.skip_balanced_braces();
                            }
                            Some(_) => {
                                self.advance();
                            }
                        }
                    }
                }
                Some(_) => {
                    self.advance();
                }
            }
        }
    }

    fn read_escape(&mut self) -> char {
        match self.peek() {
            None => '\\',
            Some('n') => {
                self.advance();
                '\n'
            }
            Some('t') => {
                self.advance();
                '\t'
            }
            Some('r') => {
                self.advance();
                '\r'
            }
            Some('0') => {
                self.advance();
                '\0'
            }
            Some('\'') => {
                self.advance();
                '\''
            }
            Some('\"') => {
                self.advance();
                '"'
            }
            Some('\\') => {
                self.advance();
                '\\'
            }
            Some('`') => {
                self.advance();
                '`'
            }
            Some('x') => {
                self.advance();
                let hi = self.read_hex_digit();
                let lo = self.read_hex_digit();
                ((hi << 4) | lo) as u8 as char
            }
            Some('u') => {
                self.advance();
                if self.peek() == Some('{') {
                    self.advance();
                    let mut code = 0u32;
                    while let Some(c) = self.peek() {
                        if c == '}' {
                            self.advance();
                            break;
                        }
                        code = code * 16 + self.read_hex_digit() as u32;
                    }
                    char::from_u32(code).unwrap_or('\u{FFFD}')
                } else {
                    let mut code = 0u32;
                    for _ in 0..4 {
                        code = code * 16 + self.read_hex_digit() as u32;
                    }
                    char::from_u32(code).unwrap_or('\u{FFFD}')
                }
            }
            Some(c) => {
                self.advance();
                c
            }
        }
    }

    fn read_hex_digit(&mut self) -> u8 {
        match self.peek() {
            Some(c) if c.is_ascii_hexdigit() => {
                let v = c.to_digit(16).unwrap_or(0) as u8;
                self.advance();
                v
            }
            _ => 0,
        }
    }

    fn read_number(&mut self) -> JsToken {
        if self.peek() == Some('0') {
            match self.peek_at(1) {
                Some('x' | 'X') => {
                    self.advance();
                    self.advance();
                    self.take_while(|c| c.is_ascii_hexdigit() || c == '_');
                    let raw = self.slice();
                    let cleaned: String = raw.chars().filter(|c| *c != '_').collect();
                    let value = u64::from_str_radix(&cleaned, 16).unwrap_or(0) as f64;
                    return self.make_token_value(JsTokenKind::Number, TokenValue::Number(value));
                }
                Some('o' | 'O') => {
                    self.advance();
                    self.advance();
                    self.take_while(|c| matches!(c, '0'..='7') || c == '_');
                    let raw = self.slice();
                    let cleaned: String = raw.chars().filter(|c| *c != '_').collect();
                    let value = u64::from_str_radix(&cleaned, 8).unwrap_or(0) as f64;
                    return self.make_token_value(JsTokenKind::Number, TokenValue::Number(value));
                }
                Some('b' | 'B') => {
                    self.advance();
                    self.advance();
                    self.take_while(|c| c == '0' || c == '1' || c == '_');
                    let raw = self.slice();
                    let cleaned: String = raw.chars().filter(|c| *c != '_').collect();
                    let value = u64::from_str_radix(&cleaned, 2).unwrap_or(0) as f64;
                    return self.make_token_value(JsTokenKind::Number, TokenValue::Number(value));
                }
                _ => {}
            }
        }

        self.take_while(|c| c.is_ascii_digit() || c == '_');
        if self.peek() == Some('.') && self.peek_at(1).map_or(false, |c| c.is_ascii_digit()) {
            self.advance();
            self.take_while(|c| c.is_ascii_digit() || c == '_');
        }
        if matches!(self.peek(), Some('e' | 'E')) {
            self.advance();
            if matches!(self.peek(), Some('+' | '-')) {
                self.advance();
            }
            self.take_while(|c| c.is_ascii_digit());
        }

        let is_bigint = self.peek() == Some('n')
            && !self.slice().contains('.')
            && !self.slice().contains('e')
            && !self.slice().contains('E');
        if is_bigint {
            self.advance();
            let raw = self.slice();
            let cleaned: String = raw[..raw.len() - 1].chars().filter(|c| *c != '_').collect();
            let value = cleaned.parse::<u128>().unwrap_or(0);
            return self.make_token_value(JsTokenKind::BigInt, TokenValue::BigInt(value));
        }

        let raw = self.slice();
        let cleaned: String = raw.chars().filter(|c| *c != '_').collect();
        let value = cleaned.parse::<f64>().unwrap_or(0.0);
        self.make_token_value(JsTokenKind::Number, TokenValue::Number(value))
    }

    fn read_number_after_dot(&mut self) {
        self.take_while(|c| c.is_ascii_digit() || c == '_');
        if matches!(self.peek(), Some('e' | 'E')) {
            self.advance();
            if matches!(self.peek(), Some('+' | '-')) {
                self.advance();
            }
            self.take_while(|c| c.is_ascii_digit());
        }
    }

    fn read_ident_or_keyword(&mut self) -> JsToken {
        self.take_while(is_ident_part);
        let raw = self.slice();
        match keyword_from_str(&raw) {
            Some(kind) => self.make_token(kind),
            None => {
                self.make_token_value(JsTokenKind::Identifier, TokenValue::Ident(raw.to_string()))
            }
        }
    }

    fn read_slash(&mut self) -> JsToken {
        self.advance();
        if self.peek() == Some('=') {
            self.advance();
            self.make_token(JsTokenKind::SlashAssign)
        } else {
            self.make_token(JsTokenKind::Slash)
        }
    }

    fn read_regex_or_slash(&mut self) -> JsToken {
        self.advance();
        loop {
            match self.peek() {
                None | Some('\n') | Some('\r') => break,
                Some('/') => {
                    self.advance();
                    while self.peek().map_or(false, is_ident_part) {
                        self.advance();
                    }
                    return self.make_token(JsTokenKind::Regex);
                }
                Some('\\') => {
                    self.advance();
                    self.advance();
                }
                Some('[') => {
                    self.advance();
                    loop {
                        match self.peek() {
                            None | Some('\n') | Some('\r') => break,
                            Some(']') => {
                                self.advance();
                                break;
                            }
                            Some('\\') => {
                                self.advance();
                                self.advance();
                            }
                            Some(_) => {
                                self.advance();
                            }
                        }
                    }
                }
                Some(_) => {
                    self.advance();
                }
            }
        }
        self.make_token(JsTokenKind::Slash)
    }

    // ---- helpers ----------------------------------------------------------

    fn peek(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.source[self.pos..].chars().nth(offset)
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.source[self.pos..].chars().next()?;
        let byte_len = c.len_utf8();
        self.pos += byte_len;
        self.column += 1;
        Some(c)
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.source.len()
    }

    fn slice(&self) -> &'a str {
        &self.source[self.start..self.pos]
    }

    fn span(&self) -> SourceSpan {
        SourceSpan {
            start: motarjim_span::SourceLocation {
                offset: self.start,
                line: self.line,
                column: self.column.saturating_sub(1),
            },
            end: motarjim_span::SourceLocation {
                offset: self.pos,
                line: self.line,
                column: self.column,
            },
        }
    }

    fn make_token(&self, kind: JsTokenKind) -> JsToken {
        JsToken::new(kind, self.span(), self.slice())
    }

    fn make_token_value(&self, kind: JsTokenKind, value: TokenValue) -> JsToken {
        JsToken::new(kind, self.span(), self.slice()).with_value(value)
    }

    fn take_while(&mut self, mut f: impl FnMut(char) -> bool) {
        while let Some(c) = self.peek() {
            if f(c) {
                self.advance();
            } else {
                break;
            }
        }
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == '$'
}

fn is_ident_part(c: char) -> bool {
    is_ident_start(c) || c.is_ascii_digit() || c == '\u{200C}' || c == '\u{200D}'
}
