//! Expression parser: Pratt (precedence-climbing) for binary/unary, recursive descent for primary.

use crate::ast::expr::*;
use crate::ast::lit::*;
use crate::ast::pat::*;
use crate::ast::stmt::*;
use crate::diagnostics::JsDiagnosticCode;
use crate::parser::JsParser;
use crate::token::{JsTokenKind, TokenValue};

impl JsParser {
    pub(crate) fn parse_expression(&mut self) -> Expression {
        let start = self.cur().span;
        let first = self.parse_assignment_expr();
        if self.eat(JsTokenKind::Comma) {
            let mut exprs = vec![first];
            loop {
                exprs.push(self.parse_assignment_expr());
                if !self.eat(JsTokenKind::Comma) {
                    break;
                }
            }
            let span = self.span_from(start);
            return Expression::Sequence(Box::new(SequenceExpr { exprs, span }));
        }
        first
    }

    pub(crate) fn parse_assignment_expr(&mut self) -> Expression {
        if let Some(arrow) = self.try_parse_arrow() {
            return arrow;
        }

        let start = self.cur().span;
        if self.eat(JsTokenKind::Async)
            && !self.at_any(&[
                JsTokenKind::Arrow,
                JsTokenKind::LParen,
                JsTokenKind::Identifier,
            ])
        {
            self.pos -= 1;
        }

        let left = self.parse_conditional();

        if self.at_any(&[
            JsTokenKind::Assign,
            JsTokenKind::PlusAssign,
            JsTokenKind::MinusAssign,
            JsTokenKind::StarAssign,
            JsTokenKind::SlashAssign,
            JsTokenKind::PercentAssign,
            JsTokenKind::StarStarAssign,
            JsTokenKind::AmpAssign,
            JsTokenKind::PipeAssign,
            JsTokenKind::CaretAssign,
            JsTokenKind::LtLtAssign,
            JsTokenKind::GtGtAssign,
            JsTokenKind::GtGtGtAssign,
            JsTokenKind::AmpAmpAssign,
            JsTokenKind::PipePipeAssign,
            JsTokenKind::NullishAssign,
        ]) {
            let op = self.parse_assign_op();
            let value = self.parse_assignment_expr();
            let span = self.span_from(start);
            return Expression::Assignment(Box::new(AssignExpr {
                op,
                target: Box::new(left),
                value: Box::new(value),
                span,
            }));
        }
        left
    }

    fn parse_assign_op(&mut self) -> AssignOp {
        let tok = self.advance();
        match tok.kind {
            JsTokenKind::Assign => AssignOp::Assign,
            JsTokenKind::PlusAssign => AssignOp::AddAssign,
            JsTokenKind::MinusAssign => AssignOp::SubAssign,
            JsTokenKind::StarAssign => AssignOp::MulAssign,
            JsTokenKind::SlashAssign => AssignOp::DivAssign,
            JsTokenKind::PercentAssign => AssignOp::ModAssign,
            JsTokenKind::StarStarAssign => AssignOp::ExpAssign,
            JsTokenKind::AmpAssign => AssignOp::BitwiseAndAssign,
            JsTokenKind::PipeAssign => AssignOp::BitwiseOrAssign,
            JsTokenKind::CaretAssign => AssignOp::BitwiseXorAssign,
            JsTokenKind::LtLtAssign => AssignOp::LShiftAssign,
            JsTokenKind::GtGtAssign => AssignOp::RShiftAssign,
            JsTokenKind::GtGtGtAssign => AssignOp::RtShiftAssign,
            JsTokenKind::AmpAmpAssign => AssignOp::LogicalAndAssign,
            JsTokenKind::PipePipeAssign => AssignOp::LogicalOrAssign,
            JsTokenKind::NullishAssign => AssignOp::NullishAssign,
            _ => {
                self.error_with_code(
                    JsDiagnosticCode::JS_UNEXPECTED_TOKEN,
                    format!("unexpected token in assignment operator: {:?}", tok.kind),
                );
                AssignOp::Assign
            }
        }
    }

    fn try_parse_arrow(&mut self) -> Option<Expression> {
        let start = self.cur().span;

        if self.at(JsTokenKind::Identifier) && self.peek_kind_at(1) == JsTokenKind::Arrow {
            let name = self.advance().raw;
            self.advance();
            let span = self.span_from(start);
            let params = vec![Param {
                pat: Pattern::Ident(name, span),
                default: None,
                span,
            }];
            let body = self.parse_arrow_body();
            let span = self.span_from(start);
            return Some(Expression::Arrow(Box::new(ArrowFunction {
                params,
                body,
                r#async: false,
                span,
            })));
        }

        let async_arrow =
            self.at(JsTokenKind::Async) && self.peek_kind_at(1) == JsTokenKind::LParen;
        let async_ident = self.at(JsTokenKind::Async)
            && self.peek_kind_at(1) == JsTokenKind::Identifier
            && self.peek_kind_at(2) == JsTokenKind::Arrow;

        if async_ident {
            self.advance();
            let name = self.advance().raw;
            self.advance();
            let span = self.span_from(start);
            let params = vec![Param {
                pat: Pattern::Ident(name, span),
                default: None,
                span,
            }];
            let body = self.parse_arrow_body();
            let span = self.span_from(start);
            return Some(Expression::Arrow(Box::new(ArrowFunction {
                params,
                body,
                r#async: true,
                span,
            })));
        }

        if async_arrow {
            self.advance();
        }

        if self.at(JsTokenKind::LParen) && self.arrow_params_follow() {
            let params = self.parse_params();
            self.expect(JsTokenKind::Arrow, "=>");
            let body = self.parse_arrow_body();
            let span = self.span_from(start);
            return Some(Expression::Arrow(Box::new(ArrowFunction {
                params,
                body,
                r#async: true,
                span,
            })));
        }

        None
    }

    fn arrow_params_follow(&self) -> bool {
        let mut depth: i32 = 0;
        let mut i = self.pos;
        loop {
            match self.tokens.get(i).map(|t| t.kind) {
                Some(JsTokenKind::LParen) => depth += 1,
                Some(JsTokenKind::RParen) => {
                    depth -= 1;
                    if depth == 0 {
                        return self.peek_kind_at(i - self.pos + 1) == JsTokenKind::Arrow;
                    }
                }
                Some(JsTokenKind::Eof) | None => return false,
                _ => {}
            }
            i += 1;
        }
    }

    fn parse_arrow_body(&mut self) -> ArrowBody {
        if self.at(JsTokenKind::LBrace) {
            ArrowBody::Block(self.parse_block())
        } else {
            ArrowBody::Expr(Box::new(self.parse_assignment_expr()))
        }
    }

    fn parse_conditional(&mut self) -> Expression {
        let start = self.cur().span;
        let test = self.parse_logical();
        if self.eat(JsTokenKind::Question) {
            let consequent = self.parse_assignment_expr();
            self.expect(JsTokenKind::Colon, ":");
            let alternate = self.parse_assignment_expr();
            let span = self.span_from(start);
            return Expression::Conditional(Box::new(CondExpr {
                test: Box::new(test),
                consequent: Box::new(consequent),
                alternate: Box::new(alternate),
                span,
            }));
        }
        test
    }

    fn parse_logical(&mut self) -> Expression {
        let start = self.cur().span;
        let mut left = self.parse_bitwise_or();
        loop {
            let op = match self.kind() {
                JsTokenKind::Nullish => LogicalOp::NullishCoalesce,
                JsTokenKind::PipePipe => LogicalOp::Or,
                JsTokenKind::AmpAmp => LogicalOp::And,
                _ => break,
            };
            // JS forbids mixing ?? with && or || without parentheses
            if let Expression::Logical(existing) = &left {
                if existing.op != op
                    && (existing.op == LogicalOp::NullishCoalesce
                        || op == LogicalOp::NullishCoalesce)
                {
                    self.error_with_help(
                        JsDiagnosticCode::JS_MIXING_NULLISH_AND_LOGICAL,
                        "cannot mix '??' with '&&' or '||' without parentheses",
                        "wrap the expression in parentheses to clarify precedence",
                    );
                }
            }
            self.advance();
            let right = self.parse_bitwise_or();
            if let Expression::Logical(r) = &right {
                if r.op != op
                    && (r.op == LogicalOp::NullishCoalesce || op == LogicalOp::NullishCoalesce)
                {
                    self.error_with_help(
                        JsDiagnosticCode::JS_MIXING_NULLISH_AND_LOGICAL,
                        "cannot mix '??' with '&&' or '||' without parentheses",
                        "wrap the expression in parentheses to clarify precedence",
                    );
                }
            }
            let span = self.span_from(start);
            left = Expression::Logical(Box::new(LogicalExpr {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            }));
        }
        left
    }

    fn parse_bitwise_or(&mut self) -> Expression {
        let start = self.cur().span;
        let mut left = self.parse_bitwise_xor();
        while self.eat(JsTokenKind::Pipe) {
            let right = self.parse_bitwise_xor();
            let span = self.span_from(start);
            left = Expression::Binary(Box::new(BinaryExpr {
                op: BinaryOp::BitwiseOr,
                left: Box::new(left),
                right: Box::new(right),
                span,
            }));
        }
        left
    }

    fn parse_bitwise_xor(&mut self) -> Expression {
        let start = self.cur().span;
        let mut left = self.parse_bitwise_and();
        while self.eat(JsTokenKind::Caret) {
            let right = self.parse_bitwise_and();
            let span = self.span_from(start);
            left = Expression::Binary(Box::new(BinaryExpr {
                op: BinaryOp::BitwiseXor,
                left: Box::new(left),
                right: Box::new(right),
                span,
            }));
        }
        left
    }

    fn parse_bitwise_and(&mut self) -> Expression {
        let start = self.cur().span;
        let mut left = self.parse_equality();
        while self.eat(JsTokenKind::Amp) {
            let right = self.parse_equality();
            let span = self.span_from(start);
            left = Expression::Binary(Box::new(BinaryExpr {
                op: BinaryOp::BitwiseAnd,
                left: Box::new(left),
                right: Box::new(right),
                span,
            }));
        }
        left
    }

    fn parse_equality(&mut self) -> Expression {
        let start = self.cur().span;
        let mut left = self.parse_relational();
        loop {
            let op = match self.kind() {
                JsTokenKind::EqEq => BinaryOp::Eq,
                JsTokenKind::EqEqEq => BinaryOp::StrictEq,
                JsTokenKind::NotEq => BinaryOp::NotEq,
                JsTokenKind::NotEqEq => BinaryOp::StrictNotEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_relational();
            let span = self.span_from(start);
            left = Expression::Binary(Box::new(BinaryExpr {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            }));
        }
        left
    }

    fn parse_relational(&mut self) -> Expression {
        let start = self.cur().span;
        let mut left = self.parse_shift();
        loop {
            let op = match self.kind() {
                JsTokenKind::Lt => BinaryOp::Lt,
                JsTokenKind::Gt => BinaryOp::Gt,
                JsTokenKind::LtEq => BinaryOp::LtEq,
                JsTokenKind::GtEq => BinaryOp::GtEq,
                JsTokenKind::In => BinaryOp::In,
                JsTokenKind::Instanceof => BinaryOp::Instanceof,
                _ => break,
            };
            self.advance();
            let right = self.parse_shift();
            let span = self.span_from(start);
            left = Expression::Binary(Box::new(BinaryExpr {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            }));
        }
        left
    }

    fn parse_shift(&mut self) -> Expression {
        let start = self.cur().span;
        let mut left = self.parse_additive();
        loop {
            let op = match self.kind() {
                JsTokenKind::LtLt => BinaryOp::LShift,
                JsTokenKind::GtGt => BinaryOp::RShift,
                JsTokenKind::GtGtGt => BinaryOp::RtShift,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive();
            let span = self.span_from(start);
            left = Expression::Binary(Box::new(BinaryExpr {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            }));
        }
        left
    }

    fn parse_additive(&mut self) -> Expression {
        let start = self.cur().span;
        let mut left = self.parse_multiplicative();
        loop {
            let op = match self.kind() {
                JsTokenKind::Plus => BinaryOp::Add,
                JsTokenKind::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative();
            let span = self.span_from(start);
            left = Expression::Binary(Box::new(BinaryExpr {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            }));
        }
        left
    }

    fn parse_multiplicative(&mut self) -> Expression {
        let start = self.cur().span;
        let mut left = self.parse_exponentiation();
        loop {
            let op = match self.kind() {
                JsTokenKind::Star => BinaryOp::Mul,
                JsTokenKind::Slash => BinaryOp::Div,
                JsTokenKind::Percent => BinaryOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_exponentiation();
            let span = self.span_from(start);
            left = Expression::Binary(Box::new(BinaryExpr {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            }));
        }
        left
    }

    fn parse_exponentiation(&mut self) -> Expression {
        let start = self.cur().span;
        if self.at(JsTokenKind::StarStar) {
            self.error_with_code(
                JsDiagnosticCode::JS_UNEXPECTED_TOKEN,
                "exponentiation operator requires a left operand",
            );
        }
        let left = self.parse_unary();
        if self.eat(JsTokenKind::StarStar) {
            let right = self.parse_exponentiation();
            let span = self.span_from(start);
            return Expression::Binary(Box::new(BinaryExpr {
                op: BinaryOp::Exp,
                left: Box::new(left),
                right: Box::new(right),
                span,
            }));
        }
        left
    }

    fn parse_unary(&mut self) -> Expression {
        let start = self.cur().span;
        let op = match self.kind() {
            JsTokenKind::Minus => Some(UnaryOp::Minus),
            JsTokenKind::Plus => Some(UnaryOp::Plus),
            JsTokenKind::Bang => Some(UnaryOp::Not),
            JsTokenKind::Tilde => Some(UnaryOp::BitwiseNot),
            JsTokenKind::Typeof => Some(UnaryOp::Typeof),
            JsTokenKind::Void => Some(UnaryOp::Void),
            JsTokenKind::Delete => Some(UnaryOp::Delete),
            JsTokenKind::Increment => Some(UnaryOp::Increment),
            JsTokenKind::Decrement => Some(UnaryOp::Decrement),
            JsTokenKind::Await => {
                self.advance();
                let argument = self.parse_unary();
                let span = self.span_from(start);
                return Expression::Await(Box::new(AwaitExpr {
                    argument: Box::new(argument),
                    span,
                }));
            }
            _ => None,
        };
        if let Some(op) = op {
            self.advance();
            let argument = self.parse_unary();
            let span = self.span_from(start);
            return Expression::Unary(Box::new(UnaryExpr {
                op,
                argument: Box::new(argument),
                prefix: true,
                span,
            }));
        }
        self.parse_update()
    }

    fn parse_update(&mut self) -> Expression {
        self.parse_left_hand_side_expr()
    }

    fn parse_left_hand_side_expr(&mut self) -> Expression {
        let start = self.cur().span;
        let mut expr = if self.eat(JsTokenKind::New) {
            let callee = self.parse_new_callee();
            let args = if self.at(JsTokenKind::LParen) {
                self.parse_arguments()
            } else {
                Vec::new()
            };
            let span = self.span_from(start);
            Expression::New(Box::new(NewExpr {
                callee: Box::new(callee),
                args,
                span,
            }))
        } else if self.at(JsTokenKind::Super) {
            let span = self.advance().span;
            Expression::Super(span)
        } else if self.at(JsTokenKind::Import) && self.peek_kind_at(1) == JsTokenKind::LParen {
            let start = self.cur().span;
            self.advance();
            let args = self.parse_arguments();
            let span = self.span_from(start);
            Expression::Call(Box::new(CallExpr {
                callee: Box::new(Expression::Identifier("import".to_string(), start)),
                args,
                optional: false,
                span,
            }))
        } else {
            self.parse_primary()
        };

        loop {
            match self.kind() {
                JsTokenKind::Dot => {
                    self.advance();
                    if self.at(JsTokenKind::PrivateIdentifier) {
                        let name = self.advance().raw;
                        let span = self.span_from(start);
                        expr = Expression::Member(Box::new(MemberExpr {
                            object: Box::new(expr),
                            property: MemberProp::PrivateIdent(name),
                            optional: false,
                            span,
                        }));
                    } else {
                        let name = self.parse_property_name();
                        let span = self.span_from(start);
                        expr = Expression::Member(Box::new(MemberExpr {
                            object: Box::new(expr),
                            property: MemberProp::Ident(name),
                            optional: false,
                            span,
                        }));
                    }
                }
                JsTokenKind::QuestionDot => {
                    self.advance();
                    if self.at(JsTokenKind::LParen) {
                        let args = self.parse_arguments();
                        let span = self.span_from(start);
                        expr = Expression::Call(Box::new(CallExpr {
                            callee: Box::new(expr),
                            args,
                            optional: true,
                            span,
                        }));
                    } else if self.at(JsTokenKind::LBracket) {
                        self.advance();
                        let prop = self.parse_expression();
                        self.expect(JsTokenKind::RBracket, "]");
                        let span = self.span_from(start);
                        expr = Expression::Member(Box::new(MemberExpr {
                            object: Box::new(expr),
                            property: MemberProp::Computed(Box::new(prop)),
                            optional: true,
                            span,
                        }));
                    } else {
                        let name = self.advance().raw;
                        let span = self.span_from(start);
                        expr = Expression::Member(Box::new(MemberExpr {
                            object: Box::new(expr),
                            property: MemberProp::Ident(name),
                            optional: true,
                            span,
                        }));
                    }
                }
                JsTokenKind::LBracket => {
                    self.advance();
                    let prop = self.parse_expression();
                    self.expect(JsTokenKind::RBracket, "]");
                    let span = self.span_from(start);
                    expr = Expression::Member(Box::new(MemberExpr {
                        object: Box::new(expr),
                        property: MemberProp::Computed(Box::new(prop)),
                        optional: false,
                        span,
                    }));
                }
                JsTokenKind::LParen => {
                    let args = self.parse_arguments();
                    let span = self.span_from(start);
                    expr = Expression::Call(Box::new(CallExpr {
                        callee: Box::new(expr),
                        args,
                        optional: false,
                        span,
                    }));
                }
                _ => break,
            }
        }

        if self.at_any(&[JsTokenKind::Increment, JsTokenKind::Decrement]) {
            self.advance();
            let span = self.span_from(start);
            let op = match self.tokens[self.pos - 1].kind {
                JsTokenKind::Increment => UnaryOp::Increment,
                _ => UnaryOp::Decrement,
            };
            expr = Expression::Update(Box::new(UnaryExpr {
                op,
                argument: Box::new(expr),
                prefix: false,
                span,
            }));
        }

        expr
    }

    fn parse_new_callee(&mut self) -> Expression {
        let start = self.cur().span;
        let mut expr = self.parse_primary();
        loop {
            match self.kind() {
                JsTokenKind::Dot => {
                    self.advance();
                    let name = self.parse_property_name();
                    let span = self.span_from(start);
                    expr = Expression::Member(Box::new(MemberExpr {
                        object: Box::new(expr),
                        property: MemberProp::Ident(name),
                        optional: false,
                        span,
                    }));
                }
                JsTokenKind::LBracket => {
                    self.advance();
                    let prop = self.parse_expression();
                    self.expect(JsTokenKind::RBracket, "]");
                    let span = self.span_from(start);
                    expr = Expression::Member(Box::new(MemberExpr {
                        object: Box::new(expr),
                        property: MemberProp::Computed(Box::new(prop)),
                        optional: false,
                        span,
                    }));
                }
                _ => break,
            }
        }
        expr
    }

    fn parse_arguments(&mut self) -> Vec<Expression> {
        self.expect(JsTokenKind::LParen, "(");
        let mut args = Vec::new();
        if !self.at(JsTokenKind::RParen) {
            loop {
                if self.eat(JsTokenKind::Ellipsis) {
                    let arg = self.parse_assignment_expr();
                    args.push(Expression::Spread(Box::new(arg)));
                } else {
                    args.push(self.parse_assignment_expr());
                }
                if !self.eat(JsTokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(JsTokenKind::RParen, ")");
        args
    }

    fn parse_property_name(&mut self) -> String {
        self.advance().raw
    }

    fn parse_primary(&mut self) -> Expression {
        let start = self.cur().span;
        match self.kind() {
            JsTokenKind::Number => {
                let tok = self.advance();
                let value = match &tok.value {
                    TokenValue::Number(n) => *n,
                    _ => parse_number_literal(&tok.raw),
                };
                Expression::Number(Box::new(NumberLit {
                    value,
                    raw: tok.raw,
                    span: tok.span,
                }))
            }
            JsTokenKind::BigInt => {
                let tok = self.advance();
                let value = match &tok.value {
                    TokenValue::BigInt(n) => n.to_string(),
                    _ => tok.raw.trim_end_matches('n').to_string(),
                };
                Expression::BigInt(Box::new(BigIntLit {
                    value,
                    raw: tok.raw,
                    span: tok.span,
                }))
            }
            JsTokenKind::String => {
                let tok = self.advance();
                let value = match &tok.value {
                    TokenValue::String(s) => s.clone(),
                    _ => tok.raw.clone(),
                };
                Expression::String(Box::new(StringLit {
                    value,
                    span: tok.span,
                }))
            }
            JsTokenKind::TemplateString => self.parse_template_literal(),
            JsTokenKind::Regex => {
                let tok = self.advance();
                let last_slash = tok.raw.rfind('/').unwrap_or(0);
                let pattern = tok.raw[1..last_slash].to_string();
                let flags = tok.raw[last_slash + 1..].to_string();
                Expression::Regex(Box::new(RegexLit {
                    pattern,
                    flags,
                    span: tok.span,
                }))
            }
            JsTokenKind::True => Expression::Bool(Box::new(BoolLit {
                value: true,
                span: self.advance().span,
            })),
            JsTokenKind::False => Expression::Bool(Box::new(BoolLit {
                value: false,
                span: self.advance().span,
            })),
            JsTokenKind::Null => Expression::Null(self.advance().span),
            JsTokenKind::Undefined => Expression::Undefined(self.advance().span),
            JsTokenKind::This => Expression::This(self.advance().span),
            JsTokenKind::Identifier => {
                let tok = self.advance();
                Expression::Identifier(tok.raw, tok.span)
            }
            JsTokenKind::PrivateIdentifier => {
                let tok = self.advance();
                Expression::PrivateIdentifier(tok.raw, tok.span)
            }
            JsTokenKind::LParen => {
                self.advance();
                let expr = self.parse_expression();
                self.expect(JsTokenKind::RParen, ")");
                Expression::Parenthesized(Box::new(expr))
            }
            JsTokenKind::LBracket => self.parse_array_literal(),
            JsTokenKind::LBrace => self.parse_object_literal(),
            JsTokenKind::Function => self.parse_function_expr(),
            JsTokenKind::Async if self.peek_kind_at(1) == JsTokenKind::Function => {
                self.parse_async_function_expr()
            }
            JsTokenKind::Class => self.parse_class_expr(),
            JsTokenKind::Import => {
                self.advance();
                if self.eat(JsTokenKind::Dot) {
                    // import.meta is not supported yet
                    self.advance(); // consume the meta identifier
                    self.error_with_code(
                        JsDiagnosticCode::JS_UNSUPPORTED_SYNTAX,
                        "'import.meta' is not supported",
                    );
                    Expression::Undefined(self.cur().span)
                } else {
                    self.error_with_code(
                        JsDiagnosticCode::JS_UNEXPECTED_TOKEN,
                        format!("unexpected token: {:?}", self.kind()),
                    );
                    Expression::Undefined(self.cur().span)
                }
            }
            JsTokenKind::Star => {
                self.advance();
                let argument = self.parse_unary();
                let span = self.span_from(start);
                Expression::Yield(Box::new(YieldExpr {
                    argument: Some(Box::new(argument)),
                    delegate: true,
                    span,
                }))
            }
            JsTokenKind::Yield => self.parse_yield_expr(),
            JsTokenKind::Slash | JsTokenKind::SlashAssign => self.parse_regex_literal(),
            _ => {
                self.error_with_code(
                    JsDiagnosticCode::JS_UNEXPECTED_TOKEN,
                    format!("unexpected token in expression: {:?}", self.kind()),
                );
                Expression::Undefined(self.advance().span)
            }
        }
    }

    fn parse_yield_expr(&mut self) -> Expression {
        let start = self.advance().span;
        let delegate = self.eat(JsTokenKind::Star);
        let argument = if !self.at_any(&[
            JsTokenKind::Semicolon,
            JsTokenKind::RBrace,
            JsTokenKind::RParen,
            JsTokenKind::RBracket,
            JsTokenKind::Comma,
            JsTokenKind::Colon,
            JsTokenKind::Eof,
        ]) {
            Some(Box::new(self.parse_assignment_expr()))
        } else {
            None
        };
        let span = self.span_from(start);
        Expression::Yield(Box::new(YieldExpr {
            argument,
            delegate,
            span,
        }))
    }

    fn parse_array_literal(&mut self) -> Expression {
        let start = self.advance().span;
        let mut elements = Vec::new();
        while !self.at(JsTokenKind::RBracket) && !self.at(JsTokenKind::Eof) {
            if self.eat(JsTokenKind::Comma) {
                elements.push(ArrayElement::None(self.cur().span));
                continue;
            }
            if self.eat(JsTokenKind::Ellipsis) {
                let expr = self.parse_assignment_expr();
                elements.push(ArrayElement::Spread(Box::new(expr)));
            } else {
                elements.push(ArrayElement::Some(self.parse_assignment_expr()));
            }
            self.eat(JsTokenKind::Comma);
        }
        self.expect(JsTokenKind::RBracket, "]");
        let span = self.span_from(start);
        Expression::Array(Box::new(ArrayLit { elements, span }))
    }

    fn parse_object_literal(&mut self) -> Expression {
        let start = self.advance().span;
        let mut props = Vec::new();
        while !self.at(JsTokenKind::RBrace) && !self.at(JsTokenKind::Eof) {
            let prop_start = self.cur().span;
            let key = self.parse_prop_key();
            let shorthand;
            let computed = matches!(key, PropKey::Computed(_));
            if self.eat(JsTokenKind::Colon) {
                shorthand = false;
                let value = self.parse_assignment_expr();
                let span = self.span_from(prop_start);
                props.push(ObjectProp {
                    key,
                    value,
                    shorthand,
                    computed,
                    span,
                });
            } else if let PropKey::Ident(ref name) = key {
                shorthand = true;
                let value = Expression::Identifier(name.clone(), prop_start);
                let span = self.span_from(prop_start);
                props.push(ObjectProp {
                    key,
                    value,
                    shorthand,
                    computed,
                    span,
                });
            } else {
                self.error_with_code(
                    JsDiagnosticCode::JS_EXPECTED_TOKEN,
                    "expected ':' in object literal",
                );
            }
            self.eat(JsTokenKind::Comma);
        }
        self.expect(JsTokenKind::RBrace, "}");
        let span = self.span_from(start);
        Expression::Object(Box::new(ObjectLit { props, span }))
    }

    fn parse_function_expr(&mut self) -> Expression {
        self.parse_function_expr_inner(false)
    }

    fn parse_async_function_expr(&mut self) -> Expression {
        self.parse_function_expr_inner(true)
    }

    fn parse_function_expr_inner(&mut self, r#async: bool) -> Expression {
        let start = self.cur().span;
        if r#async {
            self.advance();
        }
        self.advance();
        let id = if self.at(JsTokenKind::Identifier) {
            Some(Box::new(Pattern::Ident(
                self.advance().raw,
                self.cur().span,
            )))
        } else {
            None
        };
        let params = self.parse_params();
        let body = self.parse_block();
        let span = self.span_from(start);
        Expression::Function(Box::new(FunctionExpr {
            id,
            params,
            body,
            generator: false,
            r#async,
            span,
        }))
    }

    fn parse_class_expr(&mut self) -> Expression {
        let start = self.advance().span;
        let name = if self.at(JsTokenKind::Identifier) {
            Some(Pattern::Ident(self.advance().raw, self.cur().span))
        } else {
            None
        };
        let super_class = if self.eat(JsTokenKind::Extends) {
            Some(self.parse_expression())
        } else {
            None
        };
        let body = self.parse_class_body();
        let span = self.span_from(start);
        Expression::ClassExpr(Box::new(ClassExpr {
            name,
            super_class,
            body,
            span,
        }))
    }

    fn parse_regex_literal(&mut self) -> Expression {
        let tok = self.advance();
        let last_slash = tok.raw.rfind('/').unwrap_or(0);
        let pattern = tok.raw[1..last_slash].to_string();
        let flags = tok.raw[last_slash + 1..].to_string();
        Expression::Regex(Box::new(RegexLit {
            pattern,
            flags,
            span: tok.span,
        }))
    }

    fn parse_template_literal(&mut self) -> Expression {
        let tok = self.advance();
        let base_offset = tok.span.start.offset;
        let inner = tok
            .raw
            .get(1..tok.raw.len().saturating_sub(1))
            .unwrap_or("");

        let mut quasis = Vec::new();
        let mut exprs = Vec::new();
        for part in split_template_parts(inner) {
            match part {
                TemplatePart::Quasi(text) => quasis.push(text),
                TemplatePart::Expr {
                    source,
                    offset_in_inner,
                } => {
                    let mut sub_parser = Self::new(source);
                    let mut expr = sub_parser.parse_expression();
                    self.errors.extend(sub_parser.errors);
                    let delta = base_offset + 1 + offset_in_inner;
                    expr.shift_spans(delta);
                    exprs.push(expr);
                }
            }
        }
        quasis.resize(exprs.len() + 1, String::new());

        Expression::TemplateLiteral(Box::new(TemplateLiteral {
            quasis,
            exprs,
            span: tok.span,
        }))
    }

    pub(crate) fn parse_pattern(&mut self) -> Pattern {
        self.parse_pattern_inner(false)
    }

    fn parse_pattern_inner(&mut self, consume_assign: bool) -> Pattern {
        let start = self.cur().span;
        if self.eat(JsTokenKind::LBrace) {
            let mut props = Vec::new();
            while !self.at(JsTokenKind::RBrace) && !self.at(JsTokenKind::Eof) {
                if self.eat(JsTokenKind::Ellipsis) {
                    let rest = self.parse_pattern_inner(true);
                    let span = self.span_from(start);
                    props.push(ObjectPatProp::Rest(Box::new(rest), span));
                    break;
                }
                let prop_start = self.cur().span;
                let key = self.parse_prop_key();
                if self.eat(JsTokenKind::Colon) {
                    let value = self.parse_pattern_inner(true);
                    let span = self.span_from(prop_start);
                    props.push(ObjectPatProp::KeyValue {
                        key,
                        value: Box::new(value),
                        span,
                    });
                } else if let PropKey::Ident(name) = &key {
                    let span = self.span_from(prop_start);
                    props.push(ObjectPatProp::Shorthand {
                        name: name.clone(),
                        span,
                    });
                } else {
                    self.error_with_code(
                        JsDiagnosticCode::JS_INVALID_DESTRUCTURING_PATTERN,
                        "invalid destructuring pattern",
                    );
                }
                self.eat(JsTokenKind::Comma);
            }
            self.expect(JsTokenKind::RBrace, "}");
            let span = self.span_from(start);
            return Pattern::Object(ObjectPat { props, span });
        }

        if self.eat(JsTokenKind::LBracket) {
            let mut elements = Vec::new();
            while !self.at(JsTokenKind::RBracket) && !self.at(JsTokenKind::Eof) {
                if self.eat(JsTokenKind::Comma) {
                    elements.push(None);
                    continue;
                }
                let pat = if self.eat(JsTokenKind::Ellipsis) {
                    Some(Pattern::Rest(Box::new(self.parse_pattern_inner(true))))
                } else {
                    Some(self.parse_pattern_inner(true))
                };
                elements.push(pat);
                self.eat(JsTokenKind::Comma);
            }
            self.expect(JsTokenKind::RBracket, "]");
            let span = self.span_from(start);
            return Pattern::Array(ArrayPat { elements, span });
        }

        if self.eat(JsTokenKind::Ellipsis) {
            let arg = self.parse_pattern_inner(true);
            return Pattern::Rest(Box::new(arg));
        }

        let name = if self.at(JsTokenKind::Identifier) {
            self.advance().raw
        } else {
            self.error_with_code(
                JsDiagnosticCode::JS_EXPECTED_TOKEN,
                "expected a binding identifier",
            );
            String::new()
        };

        if consume_assign && self.eat(JsTokenKind::Assign) {
            let right = self.parse_assignment_expr();
            let span = self.span_from(start);
            return Pattern::Default(Box::new(DefaultPat {
                left: Pattern::Ident(name.clone(), start),
                right,
                span,
            }));
        }

        Pattern::Ident(name, start)
    }
}

fn parse_number_literal(raw: &str) -> f64 {
    let cleaned: String = raw.chars().filter(|c| *c != '_').collect();
    if let Some(hex) = cleaned
        .strip_prefix("0x")
        .or_else(|| cleaned.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16).map_or(0.0, |v| v as f64)
    } else {
        cleaned.parse::<f64>().unwrap_or(0.0)
    }
}

enum TemplatePart<'a> {
    Quasi(String),
    Expr {
        source: &'a str,
        offset_in_inner: usize,
    },
}

fn split_template_parts(inner: &str) -> Vec<TemplatePart<'_>> {
    let bytes = inner.as_bytes();
    let mut parts = Vec::new();
    let mut quasi_start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i = (i + 2).min(bytes.len());
            continue;
        }
        if bytes[i] == b'$' && bytes.get(i + 1) == Some(&b'{') {
            parts.push(TemplatePart::Quasi(
                inner.get(quasi_start..i).unwrap_or("").to_string(),
            ));
            let expr_start = i + 2;
            let expr_end = find_matching_brace(inner, expr_start);
            let source = inner.get(expr_start..expr_end).unwrap_or("");
            parts.push(TemplatePart::Expr {
                source,
                offset_in_inner: expr_start,
            });
            i = (expr_end + 1).min(bytes.len());
            quasi_start = i;
            continue;
        }
        i += 1;
    }
    parts.push(TemplatePart::Quasi(inner[quasi_start..].to_string()));
    parts
}

fn find_matching_brace(inner: &str, start: usize) -> usize {
    let bytes = inner.as_bytes();
    let mut i = start;
    let mut depth: u32 = 1;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => {
                depth += 1;
                i += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
                i += 1;
            }
            b'\'' | b'"' => {
                let quote = bytes[i];
                i += 1;
                while i < bytes.len() && bytes[i] != quote {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                i = (i + 1).min(bytes.len());
            }
            b'`' => {
                i = skip_nested_template(inner, i + 1);
            }
            _ => i += 1,
        }
    }
    bytes.len()
}

fn skip_nested_template(inner: &str, mut i: usize) -> usize {
    let bytes = inner.as_bytes();
    while i < bytes.len() {
        match bytes[i] {
            b'`' => return i + 1,
            b'\\' => i = (i + 2).min(bytes.len()),
            b'$' if bytes.get(i + 1) == Some(&b'{') => {
                i = find_matching_brace(inner, i + 2) + 1;
            }
            _ => i += 1,
        }
    }
    i
}

#[allow(dead_code)]
pub(crate) trait ShiftSpans {
    fn shift_spans(&mut self, delta: usize);
}

impl Expression {
    pub fn shift_spans(&mut self, delta: usize) {
        match self {
            Self::Identifier(_, span)
            | Self::PrivateIdentifier(_, span)
            | Self::Null(span)
            | Self::Undefined(span)
            | Self::This(span)
            | Self::Super(span) => {
                *span = SourceSpan {
                    start: SourceLocation {
                        offset: span.start.offset + delta,
                        ..span.start
                    },
                    end: SourceLocation {
                        offset: span.end.offset + delta,
                        ..span.end
                    },
                };
            }
            Self::Number(lit) => lit.span.shift(delta),
            Self::BigInt(lit) => lit.span.shift(delta),
            Self::String(lit) => lit.span.shift(delta),
            Self::Bool(lit) => lit.span.shift(delta),
            Self::Regex(lit) => lit.span.shift(delta),
            Self::TemplateLiteral(e) => {
                e.span.shift(delta);
                for expr in &mut e.exprs {
                    expr.shift_spans(delta);
                }
            }
            Self::Array(e) => {
                e.span.shift(delta);
                for el in &mut e.elements {
                    match el {
                        ArrayElement::Some(expr) => expr.shift_spans(delta),
                        ArrayElement::Spread(expr) => expr.shift_spans(delta),
                        ArrayElement::None(_) => {}
                    }
                }
            }
            Self::Object(e) => {
                e.span.shift(delta);
                for prop in &mut e.props {
                    prop.span.shift(delta);
                    if let PropKey::Computed(key) = &mut prop.key {
                        key.shift_spans(delta);
                    }
                    prop.value.shift_spans(delta);
                }
            }
            Self::Function(e) => {
                e.span.shift(delta);
                for p in &mut e.params {
                    p.pat.shift_spans(delta);
                    if let Some(default) = &mut p.default {
                        default.shift_spans(delta);
                    }
                }
                e.body.shift_spans(delta);
            }
            Self::Arrow(e) => {
                e.span.shift(delta);
                for p in &mut e.params {
                    p.pat.shift_spans(delta);
                    if let Some(default) = &mut p.default {
                        default.shift_spans(delta);
                    }
                }
                match &mut e.body {
                    ArrowBody::Block(b) => b.shift_spans(delta),
                    ArrowBody::Expr(expr) => expr.shift_spans(delta),
                }
            }
            Self::Unary(e) | Self::Update(e) => {
                e.span.shift(delta);
                e.argument.shift_spans(delta);
            }
            Self::Binary(e) => {
                e.span.shift(delta);
                e.left.shift_spans(delta);
                e.right.shift_spans(delta);
            }
            Self::Logical(e) => {
                e.span.shift(delta);
                e.left.shift_spans(delta);
                e.right.shift_spans(delta);
            }
            Self::Assignment(e) => {
                e.span.shift(delta);
                e.target.shift_spans(delta);
                e.value.shift_spans(delta);
            }
            Self::Conditional(e) => {
                e.span.shift(delta);
                e.test.shift_spans(delta);
                e.consequent.shift_spans(delta);
                e.alternate.shift_spans(delta);
            }
            Self::Call(e) => {
                e.span.shift(delta);
                e.callee.shift_spans(delta);
                for a in &mut e.args {
                    a.shift_spans(delta);
                }
            }
            Self::New(e) => {
                e.span.shift(delta);
                e.callee.shift_spans(delta);
                for a in &mut e.args {
                    a.shift_spans(delta);
                }
            }
            Self::Member(e) => {
                e.span.shift(delta);
                e.object.shift_spans(delta);
                if let MemberProp::Computed(p) = &mut e.property {
                    p.shift_spans(delta);
                }
            }
            Self::Sequence(e) => {
                e.span.shift(delta);
                for expr in &mut e.exprs {
                    expr.shift_spans(delta);
                }
            }
            Self::Yield(e) => {
                e.span.shift(delta);
                if let Some(arg) = &mut e.argument {
                    arg.shift_spans(delta);
                }
            }
            Self::Await(e) => {
                e.span.shift(delta);
                e.argument.shift_spans(delta);
            }
            Self::MetaProperty(e) => {
                e.span.shift(delta);
            }
            Self::Spread(e) => {
                e.shift_spans(delta);
            }
            Self::Parenthesized(e) => {
                e.shift_spans(delta);
            }
            Self::ClassExpr(e) => {
                e.span.shift(delta);
            }
        }
    }
}

impl Pattern {
    fn shift_spans(&mut self, delta: usize) {
        match self {
            Pattern::Ident(_, span) => {
                *span = SourceSpan {
                    start: SourceLocation {
                        offset: span.start.offset + delta,
                        ..span.start
                    },
                    end: SourceLocation {
                        offset: span.end.offset + delta,
                        ..span.end
                    },
                };
            }
            Pattern::Object(p) => {
                p.span.shift(delta);
            }
            Pattern::Array(p) => {
                p.span.shift(delta);
                for pat in p.elements.iter_mut().flatten() {
                    pat.shift_spans(delta);
                }
            }
            Pattern::Assign(p) => {
                p.span.shift(delta);
                p.left.shift_spans(delta);
                p.right.shift_spans(delta);
            }
            Pattern::Rest(p) => {
                p.shift_spans(delta);
            }
            Pattern::Member(p) => {
                p.span.shift(delta);
            }
            Pattern::Default(p) => {
                p.span.shift(delta);
                p.left.shift_spans(delta);
                p.right.shift_spans(delta);
            }
        }
    }
}

impl BlockStmt {
    fn shift_spans(&mut self, delta: usize) {
        self.span.shift(delta);
        for stmt in &mut self.body {
            stmt.shift_spans(delta);
        }
    }
}

impl Statement {
    fn shift_spans(&mut self, delta: usize) {
        let shift_span = |span: &mut SourceSpan| {
            *span = SourceSpan {
                start: SourceLocation {
                    offset: span.start.offset + delta,
                    ..span.start
                },
                end: SourceLocation {
                    offset: span.end.offset + delta,
                    ..span.end
                },
            };
        };
        match self {
            Self::VarDecl(d) => {
                shift_span(&mut d.span);
                for decl in &mut d.declarators {
                    shift_span(&mut decl.span);
                    if let Some(init) = &mut decl.init {
                        init.shift_spans(delta);
                    }
                }
            }
            Self::FunctionDecl(f) => {
                shift_span(&mut f.span);
                f.name.shift_spans(delta);
                for p in &mut f.params {
                    p.pat.shift_spans(delta);
                    if let Some(default) = &mut p.default {
                        default.shift_spans(delta);
                    }
                }
                f.body.shift_spans(delta);
            }
            Self::Return(r) => {
                shift_span(&mut r.span);
                if let Some(arg) = &mut r.argument {
                    arg.shift_spans(delta);
                }
            }
            Self::If(s) => {
                shift_span(&mut s.span);
                s.test.shift_spans(delta);
                s.consequent.shift_spans(delta);
                if let Some(alt) = &mut s.alternate {
                    alt.shift_spans(delta);
                }
            }
            Self::For(s) => {
                shift_span(&mut s.span);
                if let Some(init) = &mut s.init {
                    match init {
                        ForInit::VarDecl(d) => {
                            shift_span(&mut d.span);
                            for decl in &mut d.declarators {
                                shift_span(&mut decl.span);
                                if let Some(e) = &mut decl.init {
                                    e.shift_spans(delta);
                                }
                                decl.name.shift_spans(delta);
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
                shift_span(&mut s.span);
                s.right.shift_spans(delta);
                s.body.shift_spans(delta);
            }
            Self::ForIn(s) => {
                shift_span(&mut s.span);
                s.right.shift_spans(delta);
                s.body.shift_spans(delta);
            }
            Self::While(s) => {
                shift_span(&mut s.span);
                s.test.shift_spans(delta);
                s.body.shift_spans(delta);
            }
            Self::DoWhile(s) => {
                shift_span(&mut s.span);
                s.test.shift_spans(delta);
                s.body.shift_spans(delta);
            }
            Self::Block(b) => b.shift_spans(delta),
            Self::Break(span)
            | Self::Continue(span)
            | Self::Empty(span)
            | Self::Debugger(DebuggerStmt { span }) => {
                shift_span(span);
            }
            Self::Throw(s) => {
                shift_span(&mut s.span);
                s.argument.shift_spans(delta);
            }
            Self::Try(s) => {
                shift_span(&mut s.span);
                s.block.shift_spans(delta);
                if let Some(handler) = &mut s.handler {
                    handler.body.shift_spans(delta);
                }
                if let Some(finalizer) = &mut s.finalizer {
                    finalizer.shift_spans(delta);
                }
            }
            Self::Switch(s) => {
                shift_span(&mut s.span);
                s.discriminant.shift_spans(delta);
                for case in &mut s.cases {
                    if let Some(test) = &mut case.test {
                        test.shift_spans(delta);
                    }
                }
            }
            Self::Expr(e) => {
                shift_span(&mut e.span);
                e.expr.shift_spans(delta);
            }
            Self::Import(i) => {
                shift_span(&mut i.span);
            }
            Self::ExportNamed(e) => {
                shift_span(&mut e.span);
                if let Some(d) = &mut e.declaration {
                    d.shift_spans(delta);
                }
            }
            Self::ExportDefault(e) => {
                shift_span(&mut e.span);
                match &mut e.declaration {
                    ExportDefaultKind::Expression(expr) => expr.shift_spans(delta),
                    ExportDefaultKind::FunctionDecl(f) => {
                        f.span.shift(delta);
                        for p in &mut f.params {
                            p.pat.shift_spans(delta);
                            if let Some(default) = &mut p.default {
                                default.shift_spans(delta);
                            }
                        }
                        f.body.shift_spans(delta);
                    }
                    ExportDefaultKind::ClassDecl(_) => {}
                }
            }
            Self::ClassDecl(c) => {
                shift_span(&mut c.span);
            }
            Self::Labelled { body, span, .. } => {
                shift_span(span);
                body.shift_spans(delta);
            }
        }
    }
}

use motarjim_span::{SourceLocation, SourceSpan};

trait SpanShift {
    fn shift(&mut self, delta: usize);
}

impl SpanShift for SourceSpan {
    fn shift(&mut self, delta: usize) {
        self.start = SourceLocation {
            offset: self.start.offset.saturating_add(delta),
            ..self.start
        };
        self.end = SourceLocation {
            offset: self.end.offset.saturating_add(delta),
            ..self.end
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use motarjim_span::{SourceLocation, SourceSpan};

    #[test]
    fn test_span_shift_no_overflow() {
        let mut span = SourceSpan {
            start: SourceLocation {
                offset: usize::MAX - 1,
                line: 1,
                column: 1,
            },
            end: SourceLocation {
                offset: usize::MAX,
                line: 1,
                column: 2,
            },
        };
        span.shift(5); // would overflow with plain +
        assert_eq!(span.start.offset, usize::MAX); // saturates
        assert_eq!(span.end.offset, usize::MAX);
    }
}
