//! Extraction of DOM event bindings from JavaScript source.
//!
//! Recognizes the two idiomatic ways JavaScript wires an event handler to an
//! element:
//!
//! - `target.addEventListener('event', handler)`
//! - `target.onevent = handler`
//!
//! This is the seam future work will use to feed JavaScript behavior into
//! the HTML/CSS → native UI pipeline (e.g. wiring a Flutter `onPressed` to
//! the handler bound to a `click` listener). Extraction only; nothing here
//! is wired into `motarjim-ir` yet.

use motarjim_diag::SourceSpan;

use crate::ast::{AssignExpr, CallExpr, Expression, MemberProp, Program};
use crate::visitor::{walk_expression, Visitor};

/// A single DOM event handler binding found in the source.
#[derive(Debug, Clone, PartialEq)]
pub struct DomEventBinding {
    /// A best-effort textual description of the target expression, e.g.
    /// `"button"` or `"document.body"`.
    pub target: String,
    /// The event name, e.g. `"click"` (without a leading `on`).
    pub event_name: String,
    /// The span of the handler expression itself.
    pub handler_span: SourceSpan,
    /// The span of the whole binding (the call or assignment expression).
    pub span: SourceSpan,
}

/// Finds every DOM event binding in `program`.
///
/// # Example
///
/// ```rust
/// use motarjim_js::{find_dom_event_bindings, JsParser};
///
/// let mut parser = JsParser::new("button.addEventListener('click', onClick);");
/// let program = parser.parse().expect("valid syntax");
/// let bindings = find_dom_event_bindings(&program);
/// assert_eq!(bindings[0].event_name, "click");
/// ```
#[must_use]
pub fn find_dom_event_bindings(program: &Program) -> Vec<DomEventBinding> {
    let mut collector = EventCollector {
        bindings: Vec::new(),
    };
    collector.visit_program(program);
    collector.bindings
}

/// A [`Visitor`] that collects [`DomEventBinding`]s as it walks the tree.
struct EventCollector {
    /// Bindings found so far.
    bindings: Vec<DomEventBinding>,
}

impl EventCollector {
    /// Detects `target.addEventListener('event', handler)` calls.
    fn check_add_event_listener(&mut self, call: &CallExpr) {
        let Expression::Member(member) = call.callee.as_ref() else {
            return;
        };
        let MemberProp::Ident(name) = &member.property else {
            return;
        };
        if name != "addEventListener" || call.args.len() < 2 {
            return;
        }
        let Expression::String(event_name) = &call.args[0] else {
            return;
        };
        self.bindings.push(DomEventBinding {
            target: describe_expr(&member.object),
            event_name: event_name.value.clone(),
            handler_span: call.args[1].span(),
            span: call.span,
        });
    }

    /// Detects `target.onevent = handler` assignments.
    fn check_handler_assignment(&mut self, assign: &AssignExpr) {
        let Expression::Member(member) = assign.target.as_ref() else {
            return;
        };
        let MemberProp::Ident(name) = &member.property else {
            return;
        };
        let Some(event_name) = name.strip_prefix("on").filter(|s| !s.is_empty()) else {
            return;
        };
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

/// Renders a best-effort textual description of a target expression, used
/// for diagnostics and reporting rather than exact reconstruction.
fn describe_expr(expr: &Expression) -> String {
    match expr {
        Expression::Identifier(id) => id.name.clone(),
        Expression::This(_) => "this".to_string(),
        Expression::Member(member) => {
            let object = describe_expr(&member.object);
            match &member.property {
                MemberProp::Ident(name) => format!("{object}.{name}"),
                MemberProp::Computed(_) => format!("{object}[...]"),
            }
        }
        Expression::Call(call) => format!("{}(...)", describe_expr(&call.callee)),
        _ => "<expr>".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::JsParser;

    fn bindings(src: &str) -> Vec<DomEventBinding> {
        let mut parser = JsParser::new(src);
        let program = parser.parse().expect("should parse");
        find_dom_event_bindings(&program)
    }

    #[test]
    fn test_add_event_listener() {
        let b = bindings("button.addEventListener('click', handleClick);");
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].target, "button");
        assert_eq!(b[0].event_name, "click");
    }

    #[test]
    fn test_on_event_assignment() {
        let b = bindings("form.onsubmit = handleSubmit;");
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].target, "form");
        assert_eq!(b[0].event_name, "submit");
    }

    #[test]
    fn test_nested_member_target() {
        let b = bindings("document.getElementById('go').addEventListener('click', go);");
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].target, "document.getElementById(...)");
    }

    #[test]
    fn test_inline_arrow_handler() {
        let b = bindings("btn.addEventListener('click', () => console.log('hi'));");
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].event_name, "click");
    }

    #[test]
    fn test_no_false_positive_on_unrelated_assignment() {
        let b = bindings("const options = { onLoad: true };");
        assert!(b.is_empty());
    }

    #[test]
    fn test_no_false_positive_on_unrelated_call() {
        let b = bindings("list.append(item);");
        assert!(b.is_empty());
    }

    #[test]
    fn test_multiple_bindings_in_one_program() {
        let b = bindings(
            "a.addEventListener('click', f); b.onmouseover = g; c.addEventListener('keydown', h);",
        );
        assert_eq!(b.len(), 3);
    }
}
