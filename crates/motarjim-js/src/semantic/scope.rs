//! Scope stack and binding tracking.

use std::collections::HashMap;

use motarjim_span::SourceSpan;

use crate::ast::stmt::VarKind;

#[derive(Debug, Clone, Copy)]
pub struct Binding {
    pub kind: VarKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeType {
    Block,
    Function,
}

#[derive(Debug)]
pub struct Scope {
    pub bindings: HashMap<String, Binding>,
    pub scope_type: ScopeType,
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            bindings: HashMap::new(),
            scope_type: ScopeType::Block,
        }
    }
}

#[derive(Debug)]
pub struct ScopeStack {
    scopes: Vec<Scope>,
}

impl ScopeStack {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope {
                scope_type: ScopeType::Function,
                ..Default::default()
            }],
        }
    }

    pub fn push(&mut self) {
        self.scopes.push(Scope::default());
    }

    pub fn push_function(&mut self) {
        self.scopes.push(Scope {
            scope_type: ScopeType::Function,
            ..Default::default()
        });
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

    /// Declare a variable in the nearest enclosing function scope (walking up past block scopes).
    /// Used for `var` declarations which hoist to function scope.
    pub fn declare_in_function_scope(
        &mut self,
        name: &str,
        kind: VarKind,
        span: SourceSpan,
    ) -> Option<Binding> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.scope_type == ScopeType::Function {
                if let Some(existing) = scope.bindings.get(name) {
                    if kind != VarKind::Var {
                        return Some(*existing);
                    }
                }
                scope
                    .bindings
                    .insert(name.to_string(), Binding { kind, span });
                return None;
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.scopes.len()
    }

    pub fn lookup(&self, name: &str) -> Option<Binding> {
        self.scopes
            .iter()
            .rev()
            .find_map(|s| s.bindings.get(name).copied())
    }

    /// Like `lookup` but also returns the index of the scope that contains the binding.
    pub fn lookup_with_depth(&self, name: &str) -> Option<(Binding, usize)> {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if let Some(binding) = scope.bindings.get(name) {
                return Some((*binding, i));
            }
        }
        None
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
