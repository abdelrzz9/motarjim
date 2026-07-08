//! Visitor, VisitorMut, and Fold traits for AST traversal and transformation.

use crate::ast::expr::*;
use crate::ast::pat::*;
use crate::ast::program::Program;
use crate::ast::stmt::*;

pub trait Visitor {
    fn visit_program(&mut self, program: &Program) {
        walk_program(self, program);
    }
    fn visit_statement(&mut self, stmt: &Statement) {
        walk_statement(self, stmt);
    }
    fn visit_expression(&mut self, expr: &Expression) {
        walk_expression(self, expr);
    }
    fn visit_pattern(&mut self, pat: &Pattern) {
        walk_pattern(self, pat);
    }
}

pub trait VisitorMut {
    fn visit_program_mut(&mut self, program: &mut Program) {
        walk_program_mut(self, program);
    }
    fn visit_statement_mut(&mut self, stmt: &mut Statement) {
        walk_statement_mut(self, stmt);
    }
    fn visit_expression_mut(&mut self, expr: &mut Expression) {
        walk_expression_mut(self, expr);
    }
    fn visit_pattern_mut(&mut self, pat: &mut Pattern) {
        walk_pattern_mut(self, pat);
    }
}

pub trait Fold: Sized {
    fn fold_program(&mut self, program: Program) -> Program {
        walk_fold_program(self, program)
    }
    fn fold_statement(&mut self, stmt: Statement) -> Statement {
        walk_fold_statement(self, stmt)
    }
    fn fold_expression(&mut self, expr: Expression) -> Expression {
        walk_fold_expression(self, expr)
    }
    fn fold_pattern(&mut self, pat: Pattern) -> Pattern {
        walk_fold_pattern(self, pat)
    }
}

pub fn walk_program<V: Visitor + ?Sized>(visitor: &mut V, program: &Program) {
    for stmt in &program.body {
        visitor.visit_statement(stmt);
    }
}

pub fn walk_statement<V: Visitor + ?Sized>(visitor: &mut V, stmt: &Statement) {
    match stmt {
        Statement::VarDecl(decl) => {
            for declarator in &decl.declarators {
                if let Some(init) = &declarator.init {
                    visitor.visit_expression(init);
                }
            }
        }
        Statement::FunctionDecl(func) => {
            for param in &func.params {
                if let Some(default) = &param.default {
                    visitor.visit_expression(default);
                }
            }
            for stmt in &func.body.body {
                visitor.visit_statement(stmt);
            }
        }
        Statement::Return(ret) => {
            if let Some(arg) = &ret.argument {
                visitor.visit_expression(arg);
            }
        }
        Statement::If(if_stmt) => {
            visitor.visit_expression(&if_stmt.test);
            visitor.visit_statement(&if_stmt.consequent);
            if let Some(alt) = &if_stmt.alternate {
                visitor.visit_statement(alt);
            }
        }
        Statement::For(for_stmt) => {
            if let Some(init) = &for_stmt.init {
                match init {
                    ForInit::VarDecl(decl) => {
                        for declarator in &decl.declarators {
                            if let Some(init) = &declarator.init {
                                visitor.visit_expression(init);
                            }
                        }
                    }
                    ForInit::Expr(expr) => visitor.visit_expression(expr),
                }
            }
            if let Some(test) = &for_stmt.test {
                visitor.visit_expression(test);
            }
            if let Some(update) = &for_stmt.update {
                visitor.visit_expression(update);
            }
            visitor.visit_statement(&for_stmt.body);
        }
        Statement::ForOf(for_of) => {
            visitor.visit_expression(&for_of.right);
            visitor.visit_statement(&for_of.body);
        }
        Statement::ForIn(for_in) => {
            visitor.visit_expression(&for_in.right);
            visitor.visit_statement(&for_in.body);
        }
        Statement::While(while_stmt) => {
            visitor.visit_expression(&while_stmt.test);
            visitor.visit_statement(&while_stmt.body);
        }
        Statement::DoWhile(do_while) => {
            visitor.visit_statement(&do_while.body);
            visitor.visit_expression(&do_while.test);
        }
        Statement::Block(block) => {
            for stmt in &block.body {
                visitor.visit_statement(stmt);
            }
        }
        Statement::Break(_)
        | Statement::Continue(_)
        | Statement::Empty(_)
        | Statement::Debugger(_) => {}
        Statement::Throw(s) => visitor.visit_expression(&s.argument),
        Statement::Try(s) => {
            for stmt in &s.block.body {
                visitor.visit_statement(stmt);
            }
            if let Some(handler) = &s.handler {
                for stmt in &handler.body.body {
                    visitor.visit_statement(stmt);
                }
            }
            if let Some(finalizer) = &s.finalizer {
                for stmt in &finalizer.body {
                    visitor.visit_statement(stmt);
                }
            }
        }
        Statement::Switch(s) => {
            visitor.visit_expression(&s.discriminant);
            for case in &s.cases {
                if let Some(test) = &case.test {
                    visitor.visit_expression(test);
                }
                for stmt in &case.consequent {
                    visitor.visit_statement(stmt);
                }
            }
        }
        Statement::Expr(expr_stmt) => visitor.visit_expression(&expr_stmt.expr),
        Statement::Import(_) => {}
        Statement::ExportNamed(export) => {
            if let Some(decl) = &export.declaration {
                visitor.visit_statement(decl);
            }
        }
        Statement::ExportDefault(export) => match &export.declaration {
            ExportDefaultKind::Expression(expr) => visitor.visit_expression(expr),
            ExportDefaultKind::FunctionDecl(f) => {
                for stmt in &f.body.body {
                    visitor.visit_statement(stmt);
                }
            }
            ExportDefaultKind::ClassDecl(c) => {
                for member in &c.body.body {
                    match member {
                        ClassMember::Method(m) => {
                            for param in &m.function.params {
                                if let Some(default) = &param.default {
                                    visitor.visit_expression(default);
                                }
                            }
                            for stmt in &m.function.body.body {
                                visitor.visit_statement(stmt);
                            }
                        }
                        ClassMember::Property(p) => {
                            if let Some(value) = &p.value {
                                visitor.visit_expression(value);
                            }
                        }
                    }
                }
            }
        },
        Statement::ClassDecl(c) => {
            if let Some(super_class) = &c.super_class {
                visitor.visit_expression(super_class);
            }
            for member in &c.body.body {
                match member {
                    ClassMember::Method(m) => {
                        for param in &m.function.params {
                            if let Some(default) = &param.default {
                                visitor.visit_expression(default);
                            }
                        }
                        for stmt in &m.function.body.body {
                            visitor.visit_statement(stmt);
                        }
                    }
                    ClassMember::Property(p) => {
                        if let Some(value) = &p.value {
                            visitor.visit_expression(value);
                        }
                    }
                }
            }
        }
        Statement::Labelled { body, .. } => visitor.visit_statement(body),
    }
}

pub fn walk_expression<V: Visitor + ?Sized>(visitor: &mut V, expr: &Expression) {
    match expr {
        Expression::Identifier(..)
        | Expression::PrivateIdentifier(..)
        | Expression::Number(_)
        | Expression::BigInt(_)
        | Expression::String(_)
        | Expression::Bool(_)
        | Expression::Regex(_)
        | Expression::Null(_)
        | Expression::Undefined(_)
        | Expression::This(_)
        | Expression::Super(_) => {}
        Expression::TemplateLiteral(tpl) => {
            for expr in &tpl.exprs {
                visitor.visit_expression(expr);
            }
        }
        Expression::Array(arr) => {
            for el in &arr.elements {
                match el {
                    ArrayElement::Some(expr) => visitor.visit_expression(expr),
                    ArrayElement::Spread(expr) => visitor.visit_expression(expr),
                    ArrayElement::None(_) => {}
                }
            }
        }
        Expression::Object(obj) => {
            for prop in &obj.props {
                if let PropKey::Computed(key) = &prop.key {
                    visitor.visit_expression(key);
                }
                visitor.visit_expression(&prop.value);
            }
        }
        Expression::Function(func) => {
            for param in &func.params {
                if let Some(default) = &param.default {
                    visitor.visit_expression(default);
                }
            }
            for stmt in &func.body.body {
                visitor.visit_statement(stmt);
            }
        }
        Expression::Arrow(arrow) => {
            for param in &arrow.params {
                if let Some(default) = &param.default {
                    visitor.visit_expression(default);
                }
            }
            match &arrow.body {
                ArrowBody::Block(block) => {
                    for stmt in &block.body {
                        visitor.visit_statement(stmt);
                    }
                }
                ArrowBody::Expr(expr) => visitor.visit_expression(expr),
            }
        }
        Expression::Unary(unary) | Expression::Update(unary) => {
            visitor.visit_expression(&unary.argument)
        }
        Expression::Binary(bin) => {
            visitor.visit_expression(&bin.left);
            visitor.visit_expression(&bin.right);
        }
        Expression::Logical(logical) => {
            visitor.visit_expression(&logical.left);
            visitor.visit_expression(&logical.right);
        }
        Expression::Assignment(assign) => {
            visitor.visit_expression(&assign.target);
            visitor.visit_expression(&assign.value);
        }
        Expression::Conditional(cond) => {
            visitor.visit_expression(&cond.test);
            visitor.visit_expression(&cond.consequent);
            visitor.visit_expression(&cond.alternate);
        }
        Expression::Call(call) => {
            visitor.visit_expression(&call.callee);
            for arg in &call.args {
                visitor.visit_expression(arg);
            }
        }
        Expression::New(new_expr) => {
            visitor.visit_expression(&new_expr.callee);
            for arg in &new_expr.args {
                visitor.visit_expression(arg);
            }
        }
        Expression::Member(member) => {
            visitor.visit_expression(&member.object);
            if let MemberProp::Computed(prop) = &member.property {
                visitor.visit_expression(prop);
            }
        }
        Expression::Sequence(seq) => {
            for expr in &seq.exprs {
                visitor.visit_expression(expr);
            }
        }
        Expression::Yield(yield_expr) => {
            if let Some(arg) = &yield_expr.argument {
                visitor.visit_expression(arg);
            }
        }
        Expression::Await(await_expr) => {
            visitor.visit_expression(&await_expr.argument);
        }
        Expression::MetaProperty(_) => {}
        Expression::Spread(expr) => visitor.visit_expression(expr),
        Expression::Parenthesized(expr) => visitor.visit_expression(expr),
        Expression::ClassExpr(c) => {
            if let Some(super_class) = &c.super_class {
                visitor.visit_expression(super_class);
            }
            for member in &c.body.body {
                match member {
                    ClassMember::Method(m) => {
                        for param in &m.function.params {
                            if let Some(default) = &param.default {
                                visitor.visit_expression(default);
                            }
                        }
                        for stmt in &m.function.body.body {
                            visitor.visit_statement(stmt);
                        }
                    }
                    ClassMember::Property(p) => {
                        if let Some(value) = &p.value {
                            visitor.visit_expression(value);
                        }
                    }
                }
            }
        }
    }
}

pub fn walk_pattern<V: Visitor + ?Sized>(visitor: &mut V, pat: &Pattern) {
    match pat {
        Pattern::Ident(_, _) => {}
        Pattern::Object(obj) => {
            for prop in &obj.props {
                match prop {
                    ObjectPatProp::KeyValue { value, .. } => visitor.visit_pattern(value),
                    ObjectPatProp::Shorthand { .. } => {}
                    ObjectPatProp::Rest(pat, _) => visitor.visit_pattern(pat),
                }
            }
        }
        Pattern::Array(arr) => {
            for pat in arr.elements.iter().flatten() {
                visitor.visit_pattern(pat);
            }
        }
        Pattern::Assign(assign) => {
            visitor.visit_pattern(&assign.left);
            visitor.visit_expression(&assign.right);
        }
        Pattern::Rest(pat) => visitor.visit_pattern(pat),
        Pattern::Member(member) => {
            visitor.visit_expression(&member.object);
        }
        Pattern::Default(default) => {
            visitor.visit_pattern(&default.left);
            visitor.visit_expression(&default.right);
        }
    }
}

pub fn walk_program_mut<V: VisitorMut + ?Sized>(visitor: &mut V, program: &mut Program) {
    for stmt in &mut program.body {
        visitor.visit_statement_mut(stmt);
    }
}

pub fn walk_statement_mut<V: VisitorMut + ?Sized>(visitor: &mut V, stmt: &mut Statement) {
    match stmt {
        Statement::VarDecl(decl) => {
            for declarator in &mut decl.declarators {
                if let Some(init) = &mut declarator.init {
                    visitor.visit_expression_mut(init);
                }
            }
        }
        Statement::FunctionDecl(func) => {
            for param in &mut func.params {
                if let Some(default) = &mut param.default {
                    visitor.visit_expression_mut(default);
                }
            }
            for stmt in &mut func.body.body {
                visitor.visit_statement_mut(stmt);
            }
        }
        Statement::Return(ret) => {
            if let Some(arg) = &mut ret.argument {
                visitor.visit_expression_mut(arg);
            }
        }
        Statement::If(if_stmt) => {
            visitor.visit_expression_mut(&mut if_stmt.test);
            visitor.visit_statement_mut(&mut if_stmt.consequent);
            if let Some(alt) = &mut if_stmt.alternate {
                visitor.visit_statement_mut(alt);
            }
        }
        Statement::For(for_stmt) => {
            if let Some(init) = &mut for_stmt.init {
                match init {
                    ForInit::VarDecl(decl) => {
                        for declarator in &mut decl.declarators {
                            if let Some(init) = &mut declarator.init {
                                visitor.visit_expression_mut(init);
                            }
                        }
                    }
                    ForInit::Expr(expr) => visitor.visit_expression_mut(expr),
                }
            }
            if let Some(test) = &mut for_stmt.test {
                visitor.visit_expression_mut(test);
            }
            if let Some(update) = &mut for_stmt.update {
                visitor.visit_expression_mut(update);
            }
            visitor.visit_statement_mut(&mut for_stmt.body);
        }
        Statement::ForOf(for_of) => {
            visitor.visit_expression_mut(&mut for_of.right);
            visitor.visit_statement_mut(&mut for_of.body);
        }
        Statement::ForIn(for_in) => {
            visitor.visit_expression_mut(&mut for_in.right);
            visitor.visit_statement_mut(&mut for_in.body);
        }
        Statement::While(while_stmt) => {
            visitor.visit_expression_mut(&mut while_stmt.test);
            visitor.visit_statement_mut(&mut while_stmt.body);
        }
        Statement::DoWhile(do_while) => {
            visitor.visit_statement_mut(&mut do_while.body);
            visitor.visit_expression_mut(&mut do_while.test);
        }
        Statement::Block(block) => {
            for stmt in &mut block.body {
                visitor.visit_statement_mut(stmt);
            }
        }
        Statement::Break(_)
        | Statement::Continue(_)
        | Statement::Empty(_)
        | Statement::Debugger(_) => {}
        Statement::Throw(s) => visitor.visit_expression_mut(&mut s.argument),
        Statement::Try(s) => {
            for stmt in &mut s.block.body {
                visitor.visit_statement_mut(stmt);
            }
            if let Some(handler) = &mut s.handler {
                for stmt in &mut handler.body.body {
                    visitor.visit_statement_mut(stmt);
                }
            }
            if let Some(finalizer) = &mut s.finalizer {
                for stmt in &mut finalizer.body {
                    visitor.visit_statement_mut(stmt);
                }
            }
        }
        Statement::Switch(s) => {
            visitor.visit_expression_mut(&mut s.discriminant);
            for case in &mut s.cases {
                if let Some(test) = &mut case.test {
                    visitor.visit_expression_mut(test);
                }
                for stmt in &mut case.consequent {
                    visitor.visit_statement_mut(stmt);
                }
            }
        }
        Statement::Expr(expr_stmt) => visitor.visit_expression_mut(&mut expr_stmt.expr),
        Statement::Import(_) => {}
        Statement::ExportNamed(export) => {
            if let Some(decl) = &mut export.declaration {
                visitor.visit_statement_mut(decl);
            }
        }
        Statement::ExportDefault(export) => match &mut export.declaration {
            ExportDefaultKind::Expression(expr) => visitor.visit_expression_mut(expr),
            ExportDefaultKind::FunctionDecl(f) => {
                for stmt in &mut f.body.body {
                    visitor.visit_statement_mut(stmt);
                }
            }
            ExportDefaultKind::ClassDecl(c) => {
                for member in &mut c.body.body {
                    match member {
                        ClassMember::Method(m) => {
                            for param in &mut m.function.params {
                                if let Some(default) = &mut param.default {
                                    visitor.visit_expression_mut(default);
                                }
                            }
                            for stmt in &mut m.function.body.body {
                                visitor.visit_statement_mut(stmt);
                            }
                        }
                        ClassMember::Property(p) => {
                            if let Some(value) = &mut p.value {
                                visitor.visit_expression_mut(value);
                            }
                        }
                    }
                }
            }
        },
        Statement::ClassDecl(c) => {
            if let Some(super_class) = &mut c.super_class {
                visitor.visit_expression_mut(super_class);
            }
            for member in &mut c.body.body {
                match member {
                    ClassMember::Method(m) => {
                        for param in &mut m.function.params {
                            if let Some(default) = &mut param.default {
                                visitor.visit_expression_mut(default);
                            }
                        }
                        for stmt in &mut m.function.body.body {
                            visitor.visit_statement_mut(stmt);
                        }
                    }
                    ClassMember::Property(p) => {
                        if let Some(value) = &mut p.value {
                            visitor.visit_expression_mut(value);
                        }
                    }
                }
            }
        }
        Statement::Labelled { body, .. } => visitor.visit_statement_mut(body),
    }
}

pub fn walk_expression_mut<V: VisitorMut + ?Sized>(visitor: &mut V, expr: &mut Expression) {
    match expr {
        Expression::Identifier(..)
        | Expression::PrivateIdentifier(..)
        | Expression::Number(_)
        | Expression::BigInt(_)
        | Expression::String(_)
        | Expression::Bool(_)
        | Expression::Regex(_)
        | Expression::Null(_)
        | Expression::Undefined(_)
        | Expression::This(_)
        | Expression::Super(_) => {}
        Expression::TemplateLiteral(tpl) => {
            for expr in &mut tpl.exprs {
                visitor.visit_expression_mut(expr);
            }
        }
        Expression::Array(arr) => {
            for el in &mut arr.elements {
                match el {
                    ArrayElement::Some(expr) => visitor.visit_expression_mut(expr),
                    ArrayElement::Spread(expr) => visitor.visit_expression_mut(expr),
                    ArrayElement::None(_) => {}
                }
            }
        }
        Expression::Object(obj) => {
            for prop in &mut obj.props {
                if let PropKey::Computed(key) = &mut prop.key {
                    visitor.visit_expression_mut(key);
                }
                visitor.visit_expression_mut(&mut prop.value);
            }
        }
        Expression::Function(func) => {
            for param in &mut func.params {
                if let Some(default) = &mut param.default {
                    visitor.visit_expression_mut(default);
                }
            }
            for stmt in &mut func.body.body {
                visitor.visit_statement_mut(stmt);
            }
        }
        Expression::Arrow(arrow) => {
            for param in &mut arrow.params {
                if let Some(default) = &mut param.default {
                    visitor.visit_expression_mut(default);
                }
            }
            match &mut arrow.body {
                ArrowBody::Block(block) => {
                    for stmt in &mut block.body {
                        visitor.visit_statement_mut(stmt);
                    }
                }
                ArrowBody::Expr(expr) => visitor.visit_expression_mut(expr),
            }
        }
        Expression::Unary(unary) | Expression::Update(unary) => {
            visitor.visit_expression_mut(&mut unary.argument)
        }
        Expression::Binary(bin) => {
            visitor.visit_expression_mut(&mut bin.left);
            visitor.visit_expression_mut(&mut bin.right);
        }
        Expression::Logical(logical) => {
            visitor.visit_expression_mut(&mut logical.left);
            visitor.visit_expression_mut(&mut logical.right);
        }
        Expression::Assignment(assign) => {
            visitor.visit_expression_mut(&mut assign.target);
            visitor.visit_expression_mut(&mut assign.value);
        }
        Expression::Conditional(cond) => {
            visitor.visit_expression_mut(&mut cond.test);
            visitor.visit_expression_mut(&mut cond.consequent);
            visitor.visit_expression_mut(&mut cond.alternate);
        }
        Expression::Call(call) => {
            visitor.visit_expression_mut(&mut call.callee);
            for arg in &mut call.args {
                visitor.visit_expression_mut(arg);
            }
        }
        Expression::New(new_expr) => {
            visitor.visit_expression_mut(&mut new_expr.callee);
            for arg in &mut new_expr.args {
                visitor.visit_expression_mut(arg);
            }
        }
        Expression::Member(member) => {
            visitor.visit_expression_mut(&mut member.object);
            if let MemberProp::Computed(prop) = &mut member.property {
                visitor.visit_expression_mut(prop);
            }
        }
        Expression::Sequence(seq) => {
            for expr in &mut seq.exprs {
                visitor.visit_expression_mut(expr);
            }
        }
        Expression::Yield(yield_expr) => {
            if let Some(arg) = &mut yield_expr.argument {
                visitor.visit_expression_mut(arg);
            }
        }
        Expression::Await(await_expr) => {
            visitor.visit_expression_mut(&mut await_expr.argument);
        }
        Expression::MetaProperty(_) => {}
        Expression::Spread(expr) => visitor.visit_expression_mut(expr),
        Expression::Parenthesized(expr) => visitor.visit_expression_mut(expr),
        Expression::ClassExpr(c) => {
            if let Some(super_class) = &mut c.super_class {
                visitor.visit_expression_mut(super_class);
            }
            for member in &mut c.body.body {
                match member {
                    ClassMember::Method(m) => {
                        for param in &mut m.function.params {
                            if let Some(default) = &mut param.default {
                                visitor.visit_expression_mut(default);
                            }
                        }
                        for stmt in &mut m.function.body.body {
                            visitor.visit_statement_mut(stmt);
                        }
                    }
                    ClassMember::Property(p) => {
                        if let Some(value) = &mut p.value {
                            visitor.visit_expression_mut(value);
                        }
                    }
                }
            }
        }
    }
}

pub fn walk_pattern_mut<V: VisitorMut + ?Sized>(visitor: &mut V, pat: &mut Pattern) {
    match pat {
        Pattern::Ident(_, _) => {}
        Pattern::Object(obj) => {
            for prop in &mut obj.props {
                match prop {
                    ObjectPatProp::KeyValue { value, .. } => visitor.visit_pattern_mut(value),
                    ObjectPatProp::Shorthand { .. } => {}
                    ObjectPatProp::Rest(pat, _) => visitor.visit_pattern_mut(pat),
                }
            }
        }
        Pattern::Array(arr) => {
            for pat in arr.elements.iter_mut().flatten() {
                visitor.visit_pattern_mut(pat);
            }
        }
        Pattern::Assign(assign) => {
            visitor.visit_pattern_mut(&mut assign.left);
            visitor.visit_expression_mut(&mut assign.right);
        }
        Pattern::Rest(pat) => visitor.visit_pattern_mut(pat),
        Pattern::Member(member) => {
            visitor.visit_expression_mut(&mut member.object);
        }
        Pattern::Default(default) => {
            visitor.visit_pattern_mut(&mut default.left);
            visitor.visit_expression_mut(&mut default.right);
        }
    }
}

pub fn walk_fold_program<F: Fold>(folder: &mut F, program: Program) -> Program {
    Program {
        body: program
            .body
            .into_iter()
            .map(|s| folder.fold_statement(s))
            .collect(),
        ..program
    }
}

pub fn walk_fold_statement<F: Fold>(folder: &mut F, stmt: Statement) -> Statement {
    match stmt {
        Statement::VarDecl(mut decl) => {
            for declarator in &mut decl.declarators {
                if let Some(init) = &mut declarator.init {
                    let old = std::mem::replace(init, Expression::Null(SourceSpan::default()));
                    *init = folder.fold_expression(old);
                }
            }
            Statement::VarDecl(decl)
        }
        Statement::Return(mut ret) => {
            if let Some(arg) = &mut ret.argument {
                let old = std::mem::replace(arg, Expression::Null(SourceSpan::default()));
                *arg = folder.fold_expression(old);
            }
            Statement::Return(ret)
        }
        Statement::Block(block) => Statement::Block(BlockStmt {
            body: block
                .body
                .into_iter()
                .map(|s| folder.fold_statement(s))
                .collect(),
            ..block
        }),
        Statement::Expr(stmt) => Statement::Expr(ExprStmt {
            expr: folder.fold_expression(stmt.expr),
            ..stmt
        }),
        _ => stmt,
    }
}

pub fn walk_fold_expression<F: Fold>(_folder: &mut F, expr: Expression) -> Expression {
    expr
}

pub fn walk_fold_pattern<F: Fold>(_folder: &mut F, pat: Pattern) -> Pattern {
    pat
}

use motarjim_span::SourceSpan;
