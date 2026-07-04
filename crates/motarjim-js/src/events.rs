//! DOM event binding extraction from ASTs.

use motarjim_span::SourceSpan;

use crate::ast::expr::*;
use crate::ast::program::Program;
use crate::visitor::{walk_expression, Visitor};

#[derive(Debug, Clone, PartialEq)]
pub struct DomEventBinding {
    pub target: String,
    pub event_name: String,
    pub handler_span: SourceSpan,
    pub span: SourceSpan,
}

pub fn find_dom_event_bindings(program: &Program) -> Vec<DomEventBinding> {
    let mut collector = EventCollector { bindings: Vec::new() };
    collector.visit_program(program);
    collector.bindings
}

struct EventCollector {
    bindings: Vec<DomEventBinding>,
}

impl EventCollector {
    fn check_add_event_listener(&mut self, call: &CallExpr) {
        let Expression::Member(member) = call.callee.as_ref() else { return };
        let MemberProp::Ident(name) = &member.property else { return };
        if name != "addEventListener" || call.args.len() < 2 { return }
        let Expression::String(event_name) = &call.args[0] else { return };
        self.bindings.push(DomEventBinding {
            target: describe_expr(&member.object),
            event_name: event_name.value.clone(),
            handler_span: call.args[1].span(),
            span: call.span,
        });
    }

    fn check_handler_assignment(&mut self, assign: &AssignExpr) {
        let Expression::Member(member) = assign.target.as_ref() else { return };
        let MemberProp::Ident(name) = &member.property else { return };
        let Some(event_name) = name.strip_prefix("on").filter(|s| !s.is_empty()) else { return };
        self.bindings.push(DomEventBinding {
            target: describe_expr(&member.object),
            event_name: event_name.to_string(),
            handler_span: assign.value.span(),
            span: assign.span,
        });
    }
}

impl Visitor for EventCollector {
    fn visit_expression(&mut self, expr: &Expression) {
        if let Expression::Call(call) = expr {
            self.check_add_event_listener(call);
        }
        if let Expression::Assignment(assign) = expr {
            self.check_handler_assignment(assign);
        }
        walk_expression(self, expr);
    }
}

fn describe_expr(expr: &Expression) -> String {
    match expr {
        Expression::Identifier(name, _) => name.clone(),
        Expression::This(_) => "this".to_string(),
        Expression::Member(member) => {
            let object = describe_expr(&member.object);
            match &member.property {
                MemberProp::Ident(name) => format!("{object}.{name}"),
                MemberProp::Computed(_) => format!("{object}[...]"),
                MemberProp::PrivateIdent(name) => format!("{object}.{name}"),
            }
        }
        Expression::Call(call) => format!("{}(...)", describe_expr(&call.callee)),
        _ => "<expr>".to_string(),
    }
}
