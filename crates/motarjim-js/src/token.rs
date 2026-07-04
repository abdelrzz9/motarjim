//! Token kinds produced by the JavaScript lexer.

/// The kind of a single JavaScript token.
///
/// Covers the subset of ECMAScript syntax supported by `motarjim-js`:
/// variable declarations, functions, arrow functions, template literals,
/// imports/exports, and the common expression/statement grammar needed to
/// analyze DOM event bindings. Classes, generators, `async`/`await`,
/// destructuring, and regular expression literals are not yet tokenized;
/// see the crate root documentation for the full list of gaps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsTokenKind {
    /// `var`
    Var,
    /// `let`
    Let,
    /// `const`
    Const,
    /// `function`
    Function,
    /// `return`
    Return,
    /// `if`
    If,
    /// `else`
    Else,
    /// `for`
    For,
    /// `while`
    While,
    /// `do`
    Do,
    /// `break`
    Break,
    /// `continue`
    Continue,
    /// `true`
    True,
    /// `false`
    False,
    /// `null`
    Null,
    /// `undefined`
    Undefined,
    /// `typeof`
    Typeof,
    /// `void`
    Void,
    /// `delete`
    Delete,
    /// `new`
    New,
    /// `this`
    This,
    /// `import`
    Import,
    /// `export`
    Export,
    /// `from`
    From,
    /// `default`
    Default,
    /// `as`
    As,
    /// `of`
    Of,
    /// `in`
    In,
    /// `instanceof`
    Instanceof,
    /// An identifier, e.g. `foo`.
    Identifier,
    /// A numeric literal, e.g. `42`, `3.14`.
    Number,
    /// A single- or double-quoted string literal.
    String,
    /// A backtick-delimited template literal, including any `${ }` interpolations.
    TemplateString,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `;`
    Semicolon,
    /// `,`
    Comma,
    /// `:`
    Colon,
    /// `.`
    Dot,
    /// `=>`
    Arrow,
    /// `?`
    Question,
    /// `??`
    Nullish,
    /// `=`
    Assign,
    /// `+=`
    PlusAssign,
    /// `-=`
    MinusAssign,
    /// `*=`
    StarAssign,
    /// `/=`
    SlashAssign,
    /// `%=`
    PercentAssign,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `%`
    Percent,
    /// `==`
    EqEq,
    /// `===`
    EqEqEq,
    /// `!=`
    NotEq,
    /// `!==`
    NotEqEq,
    /// `<`
    Lt,
    /// `>`
    Gt,
    /// `<=`
    LtEq,
    /// `>=`
    GtEq,
    /// `&&`
    AmpAmp,
    /// `||`
    PipePipe,
    /// `!`
    Bang,
    /// `++`
    Increment,
    /// `--`
    Decrement,
    /// End of input.
    Eof,
}

impl JsTokenKind {
    /// Returns `true` if this token kind can never start an expression or
    /// statement, i.e. it only terminates one (used for error recovery).
    #[must_use]
    pub const fn is_terminator(self) -> bool {
        matches!(
            self,
            Self::Semicolon | Self::RBrace | Self::RParen | Self::RBracket | Self::Eof
        )
    }
}

/// Resolves a raw identifier string to a reserved keyword token, if any.
///
/// Returns `None` for ordinary identifiers.
#[must_use]
pub fn keyword_from_str(s: &str) -> Option<JsTokenKind> {
    Some(match s {
        "var" => JsTokenKind::Var,
        "let" => JsTokenKind::Let,
        "const" => JsTokenKind::Const,
        "function" => JsTokenKind::Function,
        "return" => JsTokenKind::Return,
        "if" => JsTokenKind::If,
        "else" => JsTokenKind::Else,
        "for" => JsTokenKind::For,
        "while" => JsTokenKind::While,
        "do" => JsTokenKind::Do,
        "break" => JsTokenKind::Break,
        "continue" => JsTokenKind::Continue,
        "true" => JsTokenKind::True,
        "false" => JsTokenKind::False,
        "null" => JsTokenKind::Null,
        "undefined" => JsTokenKind::Undefined,
        "typeof" => JsTokenKind::Typeof,
        "void" => JsTokenKind::Void,
        "delete" => JsTokenKind::Delete,
        "new" => JsTokenKind::New,
        "this" => JsTokenKind::This,
        "import" => JsTokenKind::Import,
        "export" => JsTokenKind::Export,
        "from" => JsTokenKind::From,
        "default" => JsTokenKind::Default,
        "as" => JsTokenKind::As,
        "of" => JsTokenKind::Of,
        "in" => JsTokenKind::In,
        "instanceof" => JsTokenKind::Instanceof,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_lookup() {
        assert_eq!(keyword_from_str("let"), Some(JsTokenKind::Let));
        assert_eq!(keyword_from_str("const"), Some(JsTokenKind::Const));
        assert_eq!(keyword_from_str("banana"), None);
    }

    #[test]
    fn test_is_terminator() {
        assert!(JsTokenKind::Semicolon.is_terminator());
        assert!(JsTokenKind::Eof.is_terminator());
        assert!(!JsTokenKind::Identifier.is_terminator());
    }
}
