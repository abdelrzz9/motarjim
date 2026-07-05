//! ECMAScript token types, keyword lookup, and operator precedence.

use motarjim_span::SourceSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsTokenKind {
    // ---- keywords --------------------------------------------------------
    Var,
    Let,
    Const,
    Function,
    Return,
    If,
    Else,
    For,
    While,
    Do,
    Break,
    Continue,
    Switch,
    Case,
    Default,
    Try,
    Catch,
    Finally,
    Throw,
    Async,
    Await,
    Yield,
    Class,
    Extends,
    Super,
    New,
    This,
    Typeof,
    Void,
    Delete,
    True,
    False,
    Null,
    Undefined,
    Import,
    Export,
    From,
    Of,
    In,
    Instanceof,
    As,
    Debugger,
    Static,
    Get,
    Set,
    With,

    // ---- identifiers & literals -------------------------------------------
    Identifier,
    PrivateIdentifier,
    Number,
    BigInt,
    String,
    TemplateString,
    Regex,

    // ---- punctuation ------------------------------------------------------
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Semicolon,
    Comma,
    Colon,
    Dot,
    QuestionDot,
    Arrow,
    Question,
    Nullish,
    Ellipsis,

    // ---- assignment operators ---------------------------------------------
    Assign,
    PlusAssign,
    MinusAssign,
    StarAssign,
    SlashAssign,
    PercentAssign,
    StarStarAssign,
    AmpAssign,
    PipeAssign,
    CaretAssign,
    LtLtAssign,
    GtGtAssign,
    GtGtGtAssign,
    AmpAmpAssign,
    PipePipeAssign,
    NullishAssign,

    // ---- arithmetic operators ---------------------------------------------
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    StarStar,

    // ---- comparison operators ---------------------------------------------
    EqEq,
    EqEqEq,
    NotEq,
    NotEqEq,
    Lt,
    Gt,
    LtEq,
    GtEq,

    // ---- logical operators ------------------------------------------------
    AmpAmp,
    PipePipe,
    Bang,

    // ---- bitwise operators ------------------------------------------------
    Amp,
    Pipe,
    Caret,
    Tilde,
    LtLt,
    GtGt,
    GtGtGt,

    // ---- update operators -------------------------------------------------
    Increment,
    Decrement,

    // ---- special ----------------------------------------------------------
    At,
    Hash,
    TemplateTail,
    TemplateMiddle,
    UnterminatedTemplate,
    Error,
    Eof,
}

impl JsTokenKind {
    pub const fn is_keyword(self) -> bool {
        matches!(
            self,
            Self::Var
                | Self::Let
                | Self::Const
                | Self::Function
                | Self::Return
                | Self::If
                | Self::Else
                | Self::For
                | Self::While
                | Self::Do
                | Self::Break
                | Self::Continue
                | Self::Switch
                | Self::Case
                | Self::Default
                | Self::Try
                | Self::Catch
                | Self::Finally
                | Self::Throw
                | Self::Async
                | Self::Await
                | Self::Yield
                | Self::Class
                | Self::Extends
                | Self::Super
                | Self::New
                | Self::This
                | Self::Typeof
                | Self::Void
                | Self::Delete
                | Self::True
                | Self::False
                | Self::Null
                | Self::Undefined
                | Self::Import
                | Self::Export
                | Self::From
                | Self::Of
                | Self::In
                | Self::Instanceof
                | Self::As
                | Self::Debugger
                | Self::Static
                | Self::Get
                | Self::Set
                | Self::With
        )
    }

    pub const fn is_terminator(self) -> bool {
        matches!(
            self,
            Self::Semicolon | Self::RBrace | Self::RParen | Self::RBracket | Self::Eof
        )
    }

    pub const fn is_binary_operator(self) -> bool {
        matches!(
            self,
            Self::Plus
                | Self::Minus
                | Self::Star
                | Self::Slash
                | Self::Percent
                | Self::StarStar
                | Self::EqEq
                | Self::EqEqEq
                | Self::NotEq
                | Self::NotEqEq
                | Self::Lt
                | Self::Gt
                | Self::LtEq
                | Self::GtEq
                | Self::In
                | Self::Instanceof
                | Self::Amp
                | Self::Pipe
                | Self::Caret
                | Self::LtLt
                | Self::GtGt
                | Self::GtGtGt
                | Self::AmpAmp
                | Self::PipePipe
        )
    }

    pub const fn is_assignment(self) -> bool {
        matches!(
            self,
            Self::Assign
                | Self::PlusAssign
                | Self::MinusAssign
                | Self::StarAssign
                | Self::SlashAssign
                | Self::PercentAssign
                | Self::StarStarAssign
                | Self::AmpAssign
                | Self::PipeAssign
                | Self::CaretAssign
                | Self::LtLtAssign
                | Self::GtGtAssign
                | Self::GtGtGtAssign
                | Self::AmpAmpAssign
                | Self::PipePipeAssign
                | Self::NullishAssign
        )
    }

    pub fn precedence(self) -> Option<u8> {
        Some(match self {
            Self::PipePipe => 1,
            Self::AmpAmp => 2,
            Self::Pipe => 3,
            Self::Caret => 4,
            Self::Amp => 5,
            Self::EqEq | Self::EqEqEq | Self::NotEq | Self::NotEqEq => 6,
            Self::Lt | Self::Gt | Self::LtEq | Self::GtEq | Self::In | Self::Instanceof => 7,
            Self::LtLt | Self::GtGt | Self::GtGtGt => 8,
            Self::Plus | Self::Minus => 9,
            Self::Star | Self::Slash | Self::Percent => 10,
            Self::StarStar => 11,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
    None,
    Number(f64),
    BigInt(u128),
    String(String),
    Ident(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsToken {
    pub kind: JsTokenKind,
    pub span: SourceSpan,
    pub value: TokenValue,
    pub raw: String,
}

impl JsToken {
    pub fn new(kind: JsTokenKind, span: SourceSpan, raw: impl Into<String>) -> Self {
        Self {
            kind,
            span,
            value: TokenValue::None,
            raw: raw.into(),
        }
    }

    pub fn with_value(mut self, value: TokenValue) -> Self {
        self.value = value;
        self
    }
}

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
        "switch" => JsTokenKind::Switch,
        "case" => JsTokenKind::Case,
        "default" => JsTokenKind::Default,
        "try" => JsTokenKind::Try,
        "catch" => JsTokenKind::Catch,
        "finally" => JsTokenKind::Finally,
        "throw" => JsTokenKind::Throw,
        "async" => JsTokenKind::Async,
        "await" => JsTokenKind::Await,
        "yield" => JsTokenKind::Yield,
        "class" => JsTokenKind::Class,
        "extends" => JsTokenKind::Extends,
        "super" => JsTokenKind::Super,
        "new" => JsTokenKind::New,
        "this" => JsTokenKind::This,
        "typeof" => JsTokenKind::Typeof,
        "void" => JsTokenKind::Void,
        "delete" => JsTokenKind::Delete,
        "true" => JsTokenKind::True,
        "false" => JsTokenKind::False,
        "null" => JsTokenKind::Null,
        "undefined" => JsTokenKind::Undefined,
        "import" => JsTokenKind::Import,
        "export" => JsTokenKind::Export,
        "from" => JsTokenKind::From,
        "of" => JsTokenKind::Of,
        "in" => JsTokenKind::In,
        "instanceof" => JsTokenKind::Instanceof,
        "as" => JsTokenKind::As,
        "debugger" => JsTokenKind::Debugger,
        "static" => JsTokenKind::Static,
        "get" => JsTokenKind::Get,
        "set" => JsTokenKind::Set,
        "with" => JsTokenKind::With,
        _ => return None,
    })
}

pub fn is_reserved_word(s: &str) -> bool {
    matches!(
        s,
        "var"
            | "let"
            | "const"
            | "function"
            | "return"
            | "if"
            | "else"
            | "for"
            | "while"
            | "do"
            | "break"
            | "continue"
            | "switch"
            | "case"
            | "default"
            | "try"
            | "catch"
            | "finally"
            | "throw"
            | "async"
            | "await"
            | "yield"
            | "class"
            | "extends"
            | "super"
            | "new"
            | "this"
            | "typeof"
            | "void"
            | "delete"
            | "true"
            | "false"
            | "null"
            | "import"
            | "export"
            | "instanceof"
            | "in"
            | "of"
            | "as"
            | "debugger"
            | "static"
    )
}
