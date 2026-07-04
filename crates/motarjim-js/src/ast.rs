//! Abstract syntax tree for the supported JavaScript subset.
//!
//! Every node carries a [`SourceSpan`] so diagnostics, source maps, and
//! tooling (hover, go-to-definition) can point back at the original text.

use motarjim_diag::{SourceLocation, SourceSpan};

/// A parsed JavaScript program: an ordered list of top-level statements.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// Top-level statements, in source order.
    pub body: Vec<Statement>,
    /// The span covering the entire program.
    pub span: SourceSpan,
}

/// A function parameter, optionally with a default value.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    /// The parameter's binding name.
    pub name: String,
    /// The default value expression, if any (`function f(x = 1)`).
    pub default: Option<Expression>,
    /// The span of the parameter.
    pub span: SourceSpan,
}

/// The declaration keyword used in a variable declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarKind {
    /// `var` — function-scoped.
    Var,
    /// `let` — block-scoped, mutable.
    Let,
    /// `const` — block-scoped, immutable.
    Const,
}

/// A single `name = init` binding within a variable declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct VarDeclarator {
    /// The bound variable name.
    pub name: String,
    /// The initializer expression, if any.
    pub init: Option<Expression>,
    /// The span of this declarator.
    pub span: SourceSpan,
}

/// A `var`/`let`/`const` declaration statement, e.g. `let x = 1, y;`.
#[derive(Debug, Clone, PartialEq)]
pub struct VarDecl {
    /// Which declaration keyword was used.
    pub kind: VarKind,
    /// One or more comma-separated declarators.
    pub declarators: Vec<VarDeclarator>,
    /// The span of the whole declaration.
    pub span: SourceSpan,
}

/// A named function declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    /// The function's name.
    pub name: String,
    /// The formal parameter list.
    pub params: Vec<Param>,
    /// The function body.
    pub body: BlockStmt,
    /// The span of the whole declaration.
    pub span: SourceSpan,
}

/// A brace-delimited block of statements.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockStmt {
    /// The statements contained in the block.
    pub body: Vec<Statement>,
    /// The span of the block, including its braces.
    pub span: SourceSpan,
}

/// A `return` statement, with an optional value.
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStmt {
    /// The returned expression, or `None` for a bare `return;`.
    pub argument: Option<Expression>,
    /// The span of the statement.
    pub span: SourceSpan,
}

/// An `if`/`else` statement.
#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    /// The branch condition.
    pub test: Expression,
    /// The statement executed when `test` is truthy.
    pub consequent: Box<Statement>,
    /// The statement executed otherwise, if an `else` branch is present.
    pub alternate: Option<Box<Statement>>,
    /// The span of the whole `if` statement.
    pub span: SourceSpan,
}

/// The initializer clause of a C-style `for` loop.
#[derive(Debug, Clone, PartialEq)]
pub enum ForInit {
    /// A `var`/`let`/`const` declaration, e.g. `for (let i = 0; ...)`.
    VarDecl(VarDecl),
    /// A bare expression, e.g. `for (i = 0; ...)`.
    Expr(Expression),
}

/// A C-style `for (init; test; update) body` loop.
#[derive(Debug, Clone, PartialEq)]
pub struct ForStmt {
    /// The initializer clause, if present.
    pub init: Option<ForInit>,
    /// The loop condition, if present.
    pub test: Option<Expression>,
    /// The per-iteration update expression, if present.
    pub update: Option<Expression>,
    /// The loop body.
    pub body: Box<Statement>,
    /// The span of the whole loop.
    pub span: SourceSpan,
}

/// A `for (left of right) body` loop over an iterable.
#[derive(Debug, Clone, PartialEq)]
pub struct ForOfStmt {
    /// Whether `left` introduces a new binding (`let`/`const`/`var`), and if so, which kind.
    pub decl_kind: Option<VarKind>,
    /// The loop variable name.
    pub left: String,
    /// The iterated expression.
    pub right: Expression,
    /// The loop body.
    pub body: Box<Statement>,
    /// The span of the whole loop.
    pub span: SourceSpan,
}

/// A `for (left in right) body` loop over an object's enumerable keys.
#[derive(Debug, Clone, PartialEq)]
pub struct ForInStmt {
    /// Whether `left` introduces a new binding (`let`/`const`/`var`), and if so, which kind.
    pub decl_kind: Option<VarKind>,
    /// The loop variable name.
    pub left: String,
    /// The enumerated expression.
    pub right: Expression,
    /// The loop body.
    pub body: Box<Statement>,
    /// The span of the whole loop.
    pub span: SourceSpan,
}

/// A `while (test) body` loop.
#[derive(Debug, Clone, PartialEq)]
pub struct WhileStmt {
    /// The loop condition.
    pub test: Expression,
    /// The loop body.
    pub body: Box<Statement>,
    /// The span of the whole loop.
    pub span: SourceSpan,
}

/// A `do body while (test);` loop.
#[derive(Debug, Clone, PartialEq)]
pub struct DoWhileStmt {
    /// The loop body.
    pub body: Box<Statement>,
    /// The loop condition, tested after each iteration.
    pub test: Expression,
    /// The span of the whole loop.
    pub span: SourceSpan,
}

/// An expression evaluated for its side effects, e.g. `foo();`.
#[derive(Debug, Clone, PartialEq)]
pub struct ExprStmt {
    /// The wrapped expression.
    pub expr: Expression,
    /// The span of the statement.
    pub span: SourceSpan,
}

/// A single imported binding, e.g. `x` or `x as y` inside `import { x as y } from '...'`.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportSpecifier {
    /// The name exported by the module.
    pub imported: String,
    /// The local binding name (equal to `imported` unless renamed with `as`).
    pub local: String,
    /// The span of this specifier.
    pub span: SourceSpan,
}

/// An `import` declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportDecl {
    /// The default import binding, e.g. `x` in `import x from '...'`.
    pub default: Option<String>,
    /// The namespace import binding, e.g. `ns` in `import * as ns from '...'`.
    pub namespace: Option<String>,
    /// Named imports, e.g. `{ a, b as c }`.
    pub named: Vec<ImportSpecifier>,
    /// The imported module specifier string.
    pub source: String,
    /// The span of the whole declaration.
    pub span: SourceSpan,
}

/// A named export declaration, e.g. `export const x = 1;` or `export { a, b };`.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportNamedDecl {
    /// The wrapped declaration, when exporting a fresh declaration.
    pub declaration: Option<Box<Statement>>,
    /// Re-exported local names, when using `export { a, b };` form.
    pub specifiers: Vec<String>,
    /// The span of the whole declaration.
    pub span: SourceSpan,
}

/// An `export default` declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportDefaultDecl {
    /// The exported expression.
    pub expr: Expression,
    /// The span of the whole declaration.
    pub span: SourceSpan,
}

/// A single JavaScript statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// A `var`/`let`/`const` declaration.
    VarDecl(VarDecl),
    /// A named function declaration.
    FunctionDecl(FunctionDecl),
    /// A `return` statement.
    Return(ReturnStmt),
    /// An `if`/`else` statement.
    If(IfStmt),
    /// A C-style `for` loop.
    For(Box<ForStmt>),
    /// A `for...of` loop.
    ForOf(ForOfStmt),
    /// A `for...in` loop.
    ForIn(ForInStmt),
    /// A `while` loop.
    While(WhileStmt),
    /// A `do...while` loop.
    DoWhile(DoWhileStmt),
    /// A brace-delimited block.
    Block(BlockStmt),
    /// A `break;` statement.
    Break(SourceSpan),
    /// A `continue;` statement.
    Continue(SourceSpan),
    /// An expression statement.
    Expr(ExprStmt),
    /// An `import` declaration.
    Import(ImportDecl),
    /// A named `export` declaration.
    ExportNamed(ExportNamedDecl),
    /// An `export default` declaration.
    ExportDefault(ExportDefaultDecl),
    /// An empty statement (a lone `;`).
    Empty(SourceSpan),
}

impl Statement {
    /// Returns the span covering this statement.
    #[must_use]
    pub fn span(&self) -> SourceSpan {
        match self {
            Self::VarDecl(s) => s.span,
            Self::FunctionDecl(s) => s.span,
            Self::Return(s) => s.span,
            Self::If(s) => s.span,
            Self::For(s) => s.span,
            Self::ForOf(s) => s.span,
            Self::ForIn(s) => s.span,
            Self::While(s) => s.span,
            Self::DoWhile(s) => s.span,
            Self::Block(s) => s.span,
            Self::Break(span) | Self::Continue(span) | Self::Empty(span) => *span,
            Self::Expr(s) => s.span,
            Self::Import(s) => s.span,
            Self::ExportNamed(s) => s.span,
            Self::ExportDefault(s) => s.span,
        }
    }
}

/// A numeric literal.
#[derive(Debug, Clone, PartialEq)]
pub struct NumberLit {
    /// The parsed numeric value.
    pub value: f64,
    /// The original source text.
    pub raw: String,
    /// The span of the literal.
    pub span: SourceSpan,
}

/// A string literal.
#[derive(Debug, Clone, PartialEq)]
pub struct StringLit {
    /// The string content (escape sequences are preserved verbatim, unprocessed).
    pub value: String,
    /// The span of the literal.
    pub span: SourceSpan,
}

/// A boolean literal.
#[derive(Debug, Clone, PartialEq)]
pub struct BoolLit {
    /// The literal's value.
    pub value: bool,
    /// The span of the literal.
    pub span: SourceSpan,
}

/// An identifier reference, e.g. `foo`.
#[derive(Debug, Clone, PartialEq)]
pub struct Ident {
    /// The identifier's name.
    pub name: String,
    /// The span of the identifier.
    pub span: SourceSpan,
}

/// A template literal, e.g. `` `Hello, ${name}!` ``.
///
/// `quasis` holds `exprs.len() + 1` static string chunks; `quasis[i]`
/// precedes `exprs[i]` for each `i`, and the final quasi is the trailing tail.
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateLiteral {
    /// The static string chunks between interpolations.
    pub quasis: Vec<String>,
    /// The interpolated expressions.
    pub exprs: Vec<Expression>,
    /// The span of the whole template literal.
    pub span: SourceSpan,
}

/// An array literal, e.g. `[1, 2, 3]`.
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayLit {
    /// The array's elements, in order.
    pub elements: Vec<Expression>,
    /// The span of the literal.
    pub span: SourceSpan,
}

/// The key of an object literal property.
#[derive(Debug, Clone, PartialEq)]
pub enum PropKey {
    /// A plain or shorthand identifier key, e.g. `x` in `{ x: 1 }` or `{ x }`.
    Ident(String),
    /// A string literal key, e.g. `"x"` in `{ "x": 1 }`.
    String(String),
    /// A computed key, e.g. `[expr]` in `{ [expr]: 1 }`.
    Computed(Box<Expression>),
}

/// A single `key: value` entry in an object literal.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectProp {
    /// The property key.
    pub key: PropKey,
    /// The property value.
    pub value: Expression,
    /// The span of this property.
    pub span: SourceSpan,
}

/// An object literal, e.g. `{ a: 1, b }`.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectLit {
    /// The object's properties, in order.
    pub props: Vec<ObjectProp>,
    /// The span of the literal.
    pub span: SourceSpan,
}

/// A function expression, e.g. `function (x) { return x; }`.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionExpr {
    /// The function's name, if given (function expressions may be anonymous).
    pub name: Option<String>,
    /// The formal parameter list.
    pub params: Vec<Param>,
    /// The function body.
    pub body: BlockStmt,
    /// The span of the whole expression.
    pub span: SourceSpan,
}

/// The body of an arrow function: either a block or a single expression.
#[derive(Debug, Clone, PartialEq)]
pub enum ArrowBody {
    /// A `{ ... }` block body.
    Block(BlockStmt),
    /// A concise expression body, e.g. `x => x + 1`.
    Expr(Box<Expression>),
}

/// An arrow function expression, e.g. `(x, y) => x + y`.
#[derive(Debug, Clone, PartialEq)]
pub struct ArrowFunction {
    /// The formal parameter list.
    pub params: Vec<Param>,
    /// The function body.
    pub body: ArrowBody,
    /// The span of the whole expression.
    pub span: SourceSpan,
}

/// A prefix or postfix unary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// Arithmetic negation, `-x`.
    Neg,
    /// Unary plus, `+x`.
    Plus,
    /// Logical negation, `!x`.
    Not,
    /// `typeof x`.
    Typeof,
    /// `void x`.
    Void,
    /// `delete x`.
    Delete,
    /// Prefix or postfix increment, `++x` / `x++`.
    Increment,
    /// Prefix or postfix decrement, `--x` / `x--`.
    Decrement,
}

/// A unary expression, e.g. `-x`, `typeof x`, or `x++`.
#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
    /// The operator applied.
    pub op: UnaryOp,
    /// The operand.
    pub argument: Box<Expression>,
    /// `true` for prefix position (`++x`), `false` for postfix (`x++`).
    pub prefix: bool,
    /// The span of the whole expression.
    pub span: SourceSpan,
}

/// A binary arithmetic, comparison, or relational operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/`
    Div,
    /// `%`
    Mod,
    /// `==`
    Eq,
    /// `===`
    StrictEq,
    /// `!=`
    NotEq,
    /// `!==`
    StrictNotEq,
    /// `<`
    Lt,
    /// `>`
    Gt,
    /// `<=`
    LtEq,
    /// `>=`
    GtEq,
    /// `in`
    In,
    /// `instanceof`
    Instanceof,
}

/// A binary expression, e.g. `a + b`.
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    /// The operator applied.
    pub op: BinaryOp,
    /// The left-hand operand.
    pub left: Box<Expression>,
    /// The right-hand operand.
    pub right: Box<Expression>,
    /// The span of the whole expression.
    pub span: SourceSpan,
}

/// A short-circuiting logical operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalOp {
    /// `&&`
    And,
    /// `||`
    Or,
    /// `??`
    NullishCoalesce,
}

/// A logical expression, e.g. `a && b`.
#[derive(Debug, Clone, PartialEq)]
pub struct LogicalExpr {
    /// The operator applied.
    pub op: LogicalOp,
    /// The left-hand operand.
    pub left: Box<Expression>,
    /// The right-hand operand.
    pub right: Box<Expression>,
    /// The span of the whole expression.
    pub span: SourceSpan,
}

/// An assignment operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    /// `=`
    Assign,
    /// `+=`
    AddAssign,
    /// `-=`
    SubAssign,
    /// `*=`
    MulAssign,
    /// `/=`
    DivAssign,
    /// `%=`
    ModAssign,
}

/// An assignment expression, e.g. `x = 1` or `x += 1`.
#[derive(Debug, Clone, PartialEq)]
pub struct AssignExpr {
    /// The operator applied.
    pub op: AssignOp,
    /// The assignment target (an identifier or member expression).
    pub target: Box<Expression>,
    /// The assigned value.
    pub value: Box<Expression>,
    /// The span of the whole expression.
    pub span: SourceSpan,
}

/// A ternary conditional expression, `test ? consequent : alternate`.
#[derive(Debug, Clone, PartialEq)]
pub struct CondExpr {
    /// The branch condition.
    pub test: Box<Expression>,
    /// The value when `test` is truthy.
    pub consequent: Box<Expression>,
    /// The value when `test` is falsy.
    pub alternate: Box<Expression>,
    /// The span of the whole expression.
    pub span: SourceSpan,
}

/// A function call expression, e.g. `foo(1, 2)`.
#[derive(Debug, Clone, PartialEq)]
pub struct CallExpr {
    /// The called expression.
    pub callee: Box<Expression>,
    /// The call arguments.
    pub args: Vec<Expression>,
    /// The span of the whole expression.
    pub span: SourceSpan,
}

/// A `new` expression, e.g. `new Foo(1, 2)`.
#[derive(Debug, Clone, PartialEq)]
pub struct NewExpr {
    /// The constructor expression.
    pub callee: Box<Expression>,
    /// The constructor arguments.
    pub args: Vec<Expression>,
    /// The span of the whole expression.
    pub span: SourceSpan,
}

/// The property accessed by a member expression.
#[derive(Debug, Clone, PartialEq)]
pub enum MemberProp {
    /// A static `.name` access.
    Ident(String),
    /// A computed `[expr]` access.
    Computed(Box<Expression>),
}

/// A member access expression, e.g. `a.b` or `a[b]`.
#[derive(Debug, Clone, PartialEq)]
pub struct MemberExpr {
    /// The accessed object.
    pub object: Box<Expression>,
    /// The accessed property.
    pub property: MemberProp,
    /// The span of the whole expression.
    pub span: SourceSpan,
}

/// A comma expression, e.g. `(a, b, c)`, evaluating to its last operand.
#[derive(Debug, Clone, PartialEq)]
pub struct SequenceExpr {
    /// The comma-separated expressions, in evaluation order.
    pub exprs: Vec<Expression>,
    /// The span of the whole expression.
    pub span: SourceSpan,
}

/// A single JavaScript expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// An identifier reference.
    Identifier(Ident),
    /// A numeric literal.
    Number(NumberLit),
    /// A string literal.
    String(StringLit),
    /// A boolean literal.
    Bool(BoolLit),
    /// The `null` literal.
    Null(SourceSpan),
    /// The `undefined` literal.
    Undefined(SourceSpan),
    /// The `this` keyword.
    This(SourceSpan),
    /// A template literal.
    TemplateLiteral(TemplateLiteral),
    /// An array literal.
    Array(ArrayLit),
    /// An object literal.
    Object(ObjectLit),
    /// A function expression.
    Function(FunctionExpr),
    /// An arrow function expression.
    Arrow(ArrowFunction),
    /// A unary expression.
    Unary(UnaryExpr),
    /// A binary expression.
    Binary(BinaryExpr),
    /// A logical (`&&`, `||`, `??`) expression.
    Logical(LogicalExpr),
    /// An assignment expression.
    Assignment(AssignExpr),
    /// A ternary conditional expression.
    Conditional(CondExpr),
    /// A function call expression.
    Call(CallExpr),
    /// A `new` expression.
    New(NewExpr),
    /// A member access expression.
    Member(MemberExpr),
    /// A comma-separated sequence expression.
    Sequence(SequenceExpr),
}

impl Expression {
    /// Returns the span covering this expression.
    #[must_use]
    pub fn span(&self) -> SourceSpan {
        match self {
            Self::Identifier(e) => e.span,
            Self::Number(e) => e.span,
            Self::String(e) => e.span,
            Self::Bool(e) => e.span,
            Self::Null(span) | Self::Undefined(span) | Self::This(span) => *span,
            Self::TemplateLiteral(e) => e.span,
            Self::Array(e) => e.span,
            Self::Object(e) => e.span,
            Self::Function(e) => e.span,
            Self::Arrow(e) => e.span,
            Self::Unary(e) => e.span,
            Self::Binary(e) => e.span,
            Self::Logical(e) => e.span,
            Self::Assignment(e) => e.span,
            Self::Conditional(e) => e.span,
            Self::Call(e) => e.span,
            Self::New(e) => e.span,
            Self::Member(e) => e.span,
            Self::Sequence(e) => e.span,
        }
    }

    /// Shifts every span in this expression (and its descendants) forward by
    /// `delta` bytes.
    ///
    /// Used after re-parsing a substring in isolation — most notably a
    /// template literal interpolation — to translate the substring-relative
    /// spans produced by that parse back into the outer source's coordinate
    /// space.
    pub fn shift_spans(&mut self, delta: u32) {
        match self {
            Self::Identifier(e) => e.span = shift_span(e.span, delta),
            Self::Number(e) => e.span = shift_span(e.span, delta),
            Self::String(e) => e.span = shift_span(e.span, delta),
            Self::Bool(e) => e.span = shift_span(e.span, delta),
            Self::Null(span) | Self::Undefined(span) | Self::This(span) => {
                *span = shift_span(*span, delta);
            }
            Self::TemplateLiteral(e) => {
                e.span = shift_span(e.span, delta);
                for expr in &mut e.exprs {
                    expr.shift_spans(delta);
                }
            }
            Self::Array(e) => {
                e.span = shift_span(e.span, delta);
                for el in &mut e.elements {
                    el.shift_spans(delta);
                }
            }
            Self::Object(e) => {
                e.span = shift_span(e.span, delta);
                for prop in &mut e.props {
                    prop.span = shift_span(prop.span, delta);
                    if let PropKey::Computed(key) = &mut prop.key {
                        key.shift_spans(delta);
                    }
                    prop.value.shift_spans(delta);
                }
            }
            Self::Function(e) => {
                e.span = shift_span(e.span, delta);
                for p in &mut e.params {
                    p.shift_spans(delta);
                }
                e.body.shift_spans(delta);
            }
            Self::Arrow(e) => {
                e.span = shift_span(e.span, delta);
                for p in &mut e.params {
                    p.shift_spans(delta);
                }
                match &mut e.body {
                    ArrowBody::Block(b) => b.shift_spans(delta),
                    ArrowBody::Expr(expr) => expr.shift_spans(delta),
                }
            }
            Self::Unary(e) => {
                e.span = shift_span(e.span, delta);
                e.argument.shift_spans(delta);
            }
            Self::Binary(e) => {
                e.span = shift_span(e.span, delta);
                e.left.shift_spans(delta);
                e.right.shift_spans(delta);
            }
            Self::Logical(e) => {
                e.span = shift_span(e.span, delta);
                e.left.shift_spans(delta);
                e.right.shift_spans(delta);
            }
            Self::Assignment(e) => {
                e.span = shift_span(e.span, delta);
                e.target.shift_spans(delta);
                e.value.shift_spans(delta);
            }
            Self::Conditional(e) => {
                e.span = shift_span(e.span, delta);
                e.test.shift_spans(delta);
                e.consequent.shift_spans(delta);
                e.alternate.shift_spans(delta);
            }
            Self::Call(e) => {
                e.span = shift_span(e.span, delta);
                e.callee.shift_spans(delta);
                for a in &mut e.args {
                    a.shift_spans(delta);
                }
            }
            Self::New(e) => {
                e.span = shift_span(e.span, delta);
                e.callee.shift_spans(delta);
                for a in &mut e.args {
                    a.shift_spans(delta);
                }
            }
            Self::Member(e) => {
                e.span = shift_span(e.span, delta);
                e.object.shift_spans(delta);
                if let MemberProp::Computed(p) = &mut e.property {
                    p.shift_spans(delta);
                }
            }
            Self::Sequence(e) => {
                e.span = shift_span(e.span, delta);
                for expr in &mut e.exprs {
                    expr.shift_spans(delta);
                }
            }
        }
    }
}

impl Param {
    /// Shifts this parameter's span (and its default value's spans, if any)
    /// forward by `delta` bytes.
    pub fn shift_spans(&mut self, delta: u32) {
        self.span = shift_span(self.span, delta);
        if let Some(default) = &mut self.default {
            default.shift_spans(delta);
        }
    }
}

impl BlockStmt {
    /// Shifts this block's span and every statement it contains forward by
    /// `delta` bytes.
    pub fn shift_spans(&mut self, delta: u32) {
        self.span = shift_span(self.span, delta);
        for stmt in &mut self.body {
            stmt.shift_spans(delta);
        }
    }
}

impl Statement {
    /// Shifts every span in this statement (and its descendants) forward by
    /// `delta` bytes. See [`Expression::shift_spans`] for context.
    pub fn shift_spans(&mut self, delta: u32) {
        match self {
            Self::VarDecl(d) => {
                d.span = shift_span(d.span, delta);
                for decl in &mut d.declarators {
                    decl.span = shift_span(decl.span, delta);
                    if let Some(init) = &mut decl.init {
                        init.shift_spans(delta);
                    }
                }
            }
            Self::FunctionDecl(f) => {
                f.span = shift_span(f.span, delta);
                for p in &mut f.params {
                    p.shift_spans(delta);
                }
                f.body.shift_spans(delta);
            }
            Self::Return(r) => {
                r.span = shift_span(r.span, delta);
                if let Some(arg) = &mut r.argument {
                    arg.shift_spans(delta);
                }
            }
            Self::If(s) => {
                s.span = shift_span(s.span, delta);
                s.test.shift_spans(delta);
                s.consequent.shift_spans(delta);
                if let Some(alt) = &mut s.alternate {
                    alt.shift_spans(delta);
                }
            }
            Self::For(s) => {
                s.span = shift_span(s.span, delta);
                if let Some(init) = &mut s.init {
                    match init {
                        ForInit::VarDecl(d) => {
                            d.span = shift_span(d.span, delta);
                            for decl in &mut d.declarators {
                                decl.span = shift_span(decl.span, delta);
                                if let Some(e) = &mut decl.init {
                                    e.shift_spans(delta);
                                }
                            }
                        }
                        ForInit::Expr(e) => e.shift_spans(delta),
                    }
                }
                if let Some(t) = &mut s.test {
                    t.shift_spans(delta);
                }
                if let Some(u) = &mut s.update {
                    u.shift_spans(delta);
                }
                s.body.shift_spans(delta);
            }
            Self::ForOf(s) => {
                s.span = shift_span(s.span, delta);
                s.right.shift_spans(delta);
                s.body.shift_spans(delta);
            }
            Self::ForIn(s) => {
                s.span = shift_span(s.span, delta);
                s.right.shift_spans(delta);
                s.body.shift_spans(delta);
            }
            Self::While(s) => {
                s.span = shift_span(s.span, delta);
                s.test.shift_spans(delta);
                s.body.shift_spans(delta);
            }
            Self::DoWhile(s) => {
                s.span = shift_span(s.span, delta);
                s.test.shift_spans(delta);
                s.body.shift_spans(delta);
            }
            Self::Block(b) => b.shift_spans(delta),
            Self::Break(span) | Self::Continue(span) | Self::Empty(span) => {
                *span = shift_span(*span, delta);
            }
            Self::Expr(e) => {
                e.span = shift_span(e.span, delta);
                e.expr.shift_spans(delta);
            }
            Self::Import(i) => {
                i.span = shift_span(i.span, delta);
                for n in &mut i.named {
                    n.span = shift_span(n.span, delta);
                }
            }
            Self::ExportNamed(e) => {
                e.span = shift_span(e.span, delta);
                if let Some(d) = &mut e.declaration {
                    d.shift_spans(delta);
                }
            }
            Self::ExportDefault(e) => {
                e.span = shift_span(e.span, delta);
                e.expr.shift_spans(delta);
            }
        }
    }
}

/// Shifts a span's start and end byte offsets forward by `delta`, leaving
/// line/column information untouched.
fn shift_span(span: SourceSpan, delta: u32) -> SourceSpan {
    SourceSpan {
        start: SourceLocation {
            offset: span.start.offset + delta,
            ..span.start
        },
        end: SourceLocation {
            offset: span.end.offset + delta,
            ..span.end
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(offset: u32) -> SourceSpan {
        let loc = SourceLocation {
            line: 0,
            column: 0,
            offset,
        };
        SourceSpan {
            start: loc,
            end: loc,
        }
    }

    #[test]
    fn test_shift_span() {
        let s = shift_span(span(5), 10);
        assert_eq!(s.start.offset, 15);
        assert_eq!(s.end.offset, 15);
    }

    #[test]
    fn test_shift_spans_identifier() {
        let mut expr = Expression::Identifier(Ident {
            name: "x".to_string(),
            span: span(0),
        });
        expr.shift_spans(7);
        assert_eq!(expr.span().start.offset, 7);
    }

    #[test]
    fn test_shift_spans_nested_binary() {
        let mut expr = Expression::Binary(BinaryExpr {
            op: BinaryOp::Add,
            left: Box::new(Expression::Number(NumberLit {
                value: 1.0,
                raw: "1".to_string(),
                span: span(0),
            })),
            right: Box::new(Expression::Number(NumberLit {
                value: 2.0,
                raw: "2".to_string(),
                span: span(4),
            })),
            span: span(0),
        });
        expr.shift_spans(100);
        if let Expression::Binary(b) = &expr {
            assert_eq!(b.left.span().start.offset, 100);
            assert_eq!(b.right.span().start.offset, 104);
        } else {
            panic!("expected binary expression");
        }
    }
}
