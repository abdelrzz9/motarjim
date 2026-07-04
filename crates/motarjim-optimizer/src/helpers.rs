use crate::*;

// ─── Helper functions ───────────────────────────────────────────────────────

/// Returns the text content of an IR node that has [`SemanticIr::Text`] role.
///
/// Looks for `"text"`, `"data"`, or `"content"` properties in Flutter, Compose,
/// and `SwiftUI` target variants. For `Generic` targets the node name is returned
/// when it is not `"Text"`.
#[must_use]
pub(crate) fn get_text_content(node: &IrNode) -> Option<String> {
    if !matches!(node.semantic, SemanticIr::Text) {
        return None;
    }
    match &node.target {
        TargetIr::Flutter { properties, .. }
        | TargetIr::Compose { properties, .. }
        | TargetIr::SwiftUI { properties, .. } => properties
            .iter()
            .find(|(k, _)| k.as_str() == "text" || k.as_str() == "data" || k.as_str() == "content")
            .map(|(_, v)| v.clone()),
        TargetIr::Generic { node: n, .. } => {
            if n.as_str() == "Text" {
                None
            } else {
                Some(n.to_string())
            }
        }
    }
}

/// Sets the text content of an IR node by updating its target properties.
pub(crate) fn set_text_content(node: &mut IrNode, text: String) {
    match &mut node.target {
        TargetIr::Flutter { properties, .. }
        | TargetIr::Compose { properties, .. }
        | TargetIr::SwiftUI { properties, .. } => {
            for (k, v) in properties.iter_mut() {
                if k.as_str() == "text" || k.as_str() == "data" || k.as_str() == "content" {
                    *v = text;
                    return;
                }
            }
            properties.push((smol_str::SmolStr::new_inline("text"), text));
        }
        TargetIr::Generic { node: n, .. } => {
            *n = smol_str::SmolStr::new(&text);
        }
    }
}

/// Returns `true` if the semantic role represents a container-like node.
#[must_use]
pub(crate) const fn is_container_role(semantic: &SemanticIr) -> bool {
    matches!(
        semantic,
        SemanticIr::Root
            | SemanticIr::Navigation
            | SemanticIr::NavigationBar
            | SemanticIr::HeroSection
            | SemanticIr::Card
            | SemanticIr::Section
            | SemanticIr::Article
            | SemanticIr::Aside
            | SemanticIr::Footer
            | SemanticIr::Header
            | SemanticIr::Main
            | SemanticIr::Container
            | SemanticIr::Grid
            | SemanticIr::Flex
            | SemanticIr::Column
            | SemanticIr::Row
            | SemanticIr::Stack
            | SemanticIr::Scroll
            | SemanticIr::List
            | SemanticIr::Form
            | SemanticIr::Dialog
            | SemanticIr::Table
            | SemanticIr::TableRow
    )
}

/// Returns `true` if the semantic role represents a flex container.
#[must_use]
pub(crate) const fn is_flex_container(semantic: &SemanticIr) -> bool {
    matches!(
        semantic,
        SemanticIr::Flex | SemanticIr::Row | SemanticIr::Column
    )
}

/// Returns `true` if the node identified by `id` is a text node.
#[must_use]
pub(crate) fn is_text_node(tree: &IrTree, id: NodeId) -> bool {
    tree.nodes
        .get(id.0 as usize)
        .is_some_and(|n| matches!(n.semantic, SemanticIr::Text))
}

/// Detaches a node from its parent, making it unreachable in the tree.
///
/// The node remains in the `nodes` vector but its `parent` is set to `None`
/// and its ID is removed from the parent's `children` list.
pub(crate) fn remove_node(tree: &mut IrTree, id: NodeId) {
    let parent_id = tree.nodes.get(id.0 as usize).and_then(|n| n.parent);
    if let Some(pid) = parent_id {
        if let Some(parent) = tree.nodes.get_mut(pid.0 as usize) {
            parent.children.retain(|c| *c != id);
        }
    }
    if let Some(node) = tree.nodes.get_mut(id.0 as usize) {
        node.parent = None;
    }
}

/// Collapses runs of whitespace characters into single spaces, trimming ends.
#[must_use]
pub(crate) fn collapse_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_whitespace = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !in_whitespace {
                result.push(' ');
                in_whitespace = true;
            }
        } else {
            result.push(ch);
            in_whitespace = false;
        }
    }
    let trimmed = result.trim().to_string();
    if trimmed.is_empty() && !s.is_empty() {
        String::new()
    } else {
        trimmed
    }
}
