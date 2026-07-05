//! Scope stack and binding tracking.

use std::collections::HashMap;

use motarjim_span::SourceSpan;

use crate::ast::stmt::VarKind;

#[derive(Debug, Clone, Copy)]
pub struct Binding {
    pub kind: VarKind,
    pub span: SourceSpan,
}

#[derive(Debug, Default)]
pub struct Scope {
    pub bindings: HashMap<String, Binding>,
}

#[derive(Debug)]
pub struct ScopeStack {
    scopes: Vec<Scope>,
}

impl ScopeStack {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::default()],
        }
    }

    pub fn push(&mut self) {
        self.scopes.push(Scope::default());
    }

    pub fn pop(&mut self) {
        self.scopes.pop();
    }

    pub fn declare(&mut self, name: &str, kind: VarKind, span: SourceSpan) -> Option<Binding> {
        let scope = match self.scopes.last_mut() {
            Some(s) => s,
            None => return None,
        };
        if let Some(existing) = scope.bindings.get(name) {
            if kind != VarKind::Var {
                return Some(*existing);
            }
        }
        scope
            .bindings
            .insert(name.to_string(), Binding { kind, span });
        None
    }

    pub fn lookup(&self, name: &str) -> Option<Binding> {
        self.scopes
            .iter()
            .rev()
            .find_map(|s| s.bindings.get(name).copied())
    }
}

impl Default for ScopeStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::stmt::VarKind;

    #[test]
    fn test_declare_on_empty_scope_does_not_panic() {
        let mut stack = ScopeStack::new();
        stack.pop(); // remove initial scope — now empty
        let result = stack.declare("x", VarKind::Let, (0..1).into());
        assert!(
            result.is_none(),
            "declare on empty stack should return None instead of panicking"
        );
    }
}
