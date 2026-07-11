use motarjim_ast::css::Combinator;
use motarjim_ast::HtmlNode;

use crate::*;

/// Checks whether any selector on `rule` matches `node` in the DOM tree.
pub(crate) fn rule_matches_element(
    rule: &StyleRule,
    node: &HtmlNode,
    nodes: &[HtmlNode],
) -> bool {
    rule.selectors
        .iter()
        .any(|sel| selector_matches_element(sel, node, nodes))
}

/// Check whether a single `Selector` matches `node` in the DOM tree.
fn selector_matches_element(selector: &Selector, node: &HtmlNode, nodes: &[HtmlNode]) -> bool {
    if selector.combinators.is_empty() {
        // Simple compound selector (no combinators): all simple selectors must match.
        let element = match node.element.as_ref() {
            Some(el) => el,
            None => return false,
        };
        return selector
            .simple_selectors
            .iter()
            .all(|s| simple_selector_matches(s, element));
    }

    // The last group of simple_selectors must match the current element.
    let element = match node.element.as_ref() {
        Some(el) => el,
        None => return false,
    };

    let last_group_start = selector.combinators.len(); // index into simple_selectors for the last group
    let matches_last = selector.simple_selectors[last_group_start..]
        .iter()
        .all(|s| simple_selector_matches(s, element));

    if !matches_last {
        return false;
    }

    // Walk backward through combinators, matching each preceding simple selector group.
    let mut current_node_id = node.id;

    for i in (0..selector.combinators.len()).rev() {
        let comb = selector.combinators[i];
        // The simple selector group preceding this combinator is at index `i`.
        // We need a node that matches all selectors at index `i`.
        let target_group = &selector.simple_selectors[i];

        let matched = match comb {
            Combinator::Descendant => {
                // Walk ancestor links; find any ancestor where the simple selector matches.
                let mut found = false;
                let mut ancestor_id = {
                    let current = &nodes[current_node_id.0 as usize];
                    current.parent
                };
                while let Some(aid) = ancestor_id {
                    let ancestor = &nodes[aid.0 as usize];
                    if node_matches_simple_group(ancestor, target_group) {
                        found = true;
                        break;
                    }
                    ancestor_id = ancestor.parent;
                }
                found
            }
            Combinator::Child => {
                // Check only the immediate parent.
                let current = &nodes[current_node_id.0 as usize];
                match current.parent {
                    Some(pid) => node_matches_simple_group(&nodes[pid.0 as usize], target_group),
                    None => false,
                }
            }
            Combinator::AdjacentSibling => {
                // Check the immediately preceding sibling.
                let current = &nodes[current_node_id.0 as usize];
                match current.parent {
                    Some(pid) => {
                        let parent = &nodes[pid.0 as usize];
                        // Find the position of `current` in parent's children.
                        match parent.children.iter().position(|&cid| cid == current.id) {
                            Some(pos) if pos > 0 => {
                                let prev_id = parent.children[pos - 1];
                                node_matches_simple_group(&nodes[prev_id.0 as usize], target_group)
                            }
                            _ => false,
                        }
                    }
                    None => false,
                }
            }
            Combinator::GeneralSibling => {
                // Check any preceding sibling.
                let current = &nodes[current_node_id.0 as usize];
                match current.parent {
                    Some(pid) => {
                        let parent = &nodes[pid.0 as usize];
                        match parent.children.iter().position(|&cid| cid == current.id) {
                            Some(pos) if pos > 0 => parent.children[..pos]
                                .iter()
                                .any(|&sid| node_matches_simple_group(&nodes[sid.0 as usize], target_group)),
                            _ => false,
                        }
                    }
                    None => false,
                }
            }
            Combinator::Column => false,
        };

        if !matched {
            return false;
        }

        // Move current_node_id to the node that matched this group
        // so the next (earlier) combinator can reference it.
        current_node_id = match comb {
            Combinator::Descendant => {
                // The matched ancestor.
                let mut ancestor_id = nodes[current_node_id.0 as usize].parent;
                let mut matched_id = current_node_id;
                while let Some(aid) = ancestor_id {
                    if node_matches_simple_group(&nodes[aid.0 as usize], target_group) {
                        matched_id = aid;
                        break;
                    }
                    ancestor_id = nodes[aid.0 as usize].parent;
                }
                matched_id
            }
            Combinator::Child => {
                nodes[current_node_id.0 as usize].parent.unwrap_or(current_node_id)
            }
            Combinator::AdjacentSibling => {
                let current = &nodes[current_node_id.0 as usize];
                let parent = &nodes[current.parent.unwrap().0 as usize];
                let pos = parent.children.iter().position(|&cid| cid == current.id).unwrap();
                parent.children[pos - 1]
            }
            Combinator::GeneralSibling => {
                let current = &nodes[current_node_id.0 as usize];
                let parent = &nodes[current.parent.unwrap().0 as usize];
                let pos = parent.children.iter().position(|&cid| cid == current.id).unwrap();
                // Find the first preceding sibling that matches
                parent.children[..pos]
                    .iter()
                    .rev()
                    .find(|&&sid| node_matches_simple_group(&nodes[sid.0 as usize], target_group))
                    .copied()
                    .unwrap_or(current_node_id)
            }
            Combinator::Column => current_node_id,
        };
    }

    true
}

/// Check whether a `HtmlNode` matches a single `SimpleSelector`.
fn node_matches_simple_group(node: &HtmlNode, sel: &SimpleSelector) -> bool {
    match node.element.as_ref() {
        Some(element) => simple_selector_matches(sel, element),
        None => false,
    }
}

/// Check whether a single `SimpleSelector` matches an element.
fn simple_selector_matches(sel: &SimpleSelector, element: &Element) -> bool {
    match sel {
        SimpleSelector::Universal => true,
        SimpleSelector::Type(name) => element.tag_name.as_str() == name.as_str(),
        SimpleSelector::Class(name) => element.has_class(name.as_str()),
        SimpleSelector::Id(name) => element
            .id
            .as_ref()
            .is_some_and(|id| id.as_str() == name.as_str()),
        SimpleSelector::Attribute {
            name,
            operator,
            value,
            case_sensitive: _,
            span: _,
        } => {
            let attr_val = match element.get_attribute(name.as_str()) {
                Some(v) => v,
                None => return false,
            };
            match operator {
                None => true,
                Some(AttributeOperator::Equals) => {
                    value.as_ref().is_some_and(|v| attr_val == v.as_str())
                }
                Some(AttributeOperator::Includes) => value
                    .as_ref()
                    .is_some_and(|v| attr_val.split_whitespace().any(|part| part == v.as_str())),
                Some(AttributeOperator::DashMatch) => value.as_ref().is_some_and(|v| {
                    attr_val == v.as_str() || attr_val.starts_with(&format!("{}-", v.as_str()))
                }),
                Some(AttributeOperator::PrefixMatch) => value
                    .as_ref()
                    .is_some_and(|v| attr_val.starts_with(v.as_str())),
                Some(AttributeOperator::SuffixMatch) => value
                    .as_ref()
                    .is_some_and(|v| attr_val.ends_with(v.as_str())),
                Some(AttributeOperator::SubstringMatch) => value
                    .as_ref()
                    .is_some_and(|v| attr_val.contains(v.as_str())),
            }
        }
        SimpleSelector::PseudoClass(_) | SimpleSelector::PseudoElement(_) => {
            // Pseudo-classes and pseudo-elements are conservatively treated as matching.
            true
        }
    }
}

/// Compute the *maximum* specificity among all selectors in a rule.
pub(crate) fn rule_max_specificity(rule: &StyleRule) -> (u32, u32, u32) {
    rule.selectors
        .iter()
        .map(motarjim_ast::Selector::specificity)
        .max()
        .unwrap_or((0, 0, 0))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use motarjim_ast::css::{Declaration, StyleRule};
    use motarjim_ast::{NodeType, NodeId};
    use smallvec::{smallvec, SmallVec};
    use smol_str::SmolStr;

    fn make_element(tag: &str) -> Element {
        Element::new(tag)
    }

    fn make_classed_element(tag: &str, classes: &[&str]) -> Element {
        let mut el = Element::new(tag);
        for c in classes {
            el.classes.push(SmolStr::new(*c));
        }
        el
    }

    fn make_node(id: u32, element: Option<Element>, parent: Option<u32>, children: Vec<u32>) -> HtmlNode {
        HtmlNode {
            id: NodeId(id),
            node_type: NodeType::Element,
            element,
            value: None,
            children: children.into_iter().map(NodeId).collect(),
            parent: parent.map(NodeId),
            depth: 0,
            document_type: None,
        }
    }

    fn build_dom() -> Vec<HtmlNode> {
        // <div class="container">
        //   <p id="intro">Hello</p>
        //   <div class="wrapper">
        //     <h1>Title</h1>
        //     <p class="desc">World</p>
        //     <p>Plain</p>
        //   </div>
        // </div>
        vec![
            make_node(0, None, None, vec![1]),              // root (document)
            make_node(1, Some(make_classed_element("div", &["container"])), Some(0), vec![2, 3]), // div.container
            make_node(2, Some({
                let mut el = Element::new("p");
                el.id = Some(SmolStr::new("intro"));
                el
            }), Some(1), vec![]),                            // p#intro
            make_node(3, Some(make_classed_element("div", &["wrapper"])), Some(1), vec![4, 5, 6]), // div.wrapper
            make_node(4, Some(make_element("h1")), Some(3), vec![]),   // h1
            make_node(5, Some(make_classed_element("p", &["desc"])), Some(3), vec![]), // p.desc
            make_node(6, Some(make_element("p")), Some(3), vec![]),    // p
        ]
    }

    #[test]
    fn test_descendant_combinator() {
        // .container .desc should match the p.desc node
        let nodes = build_dom();
        let selector = Selector {
            simple_selectors: vec![
                SimpleSelector::Class(SmolStr::new("container")),
                SimpleSelector::Class(SmolStr::new("desc")),
            ],
            combinators: vec![Combinator::Descendant],
            span: None,
        };
        let rule = StyleRule {
            selectors: vec![selector],
            declarations: smallvec![Declaration {
                property: SmolStr::new("color"),
                value: "red".to_string(),
                important: false,
                parsed: None,
                span: None,
            }],
            span: None,
        };

        // p.desc (id=5) should match
        assert!(rule_matches_element(&rule, &nodes[5], &nodes));
        // p (id=6) should NOT match (no .desc class)
        assert!(!rule_matches_element(&rule, &nodes[6], &nodes));
        // h1 (id=4) should NOT match
        assert!(!rule_matches_element(&rule, &nodes[4], &nodes));
    }

    #[test]
    fn test_child_combinator() {
        // div > p should match only direct child p elements of div
        let nodes = build_dom();
        let selector = Selector {
            simple_selectors: vec![
                SimpleSelector::Type(SmolStr::new("div")),
                SimpleSelector::Type(SmolStr::new("p")),
            ],
            combinators: vec![Combinator::Child],
            span: None,
        };
        let rule = StyleRule {
            selectors: vec![selector],
            declarations: smallvec![Declaration {
                property: SmolStr::new("color"),
                value: "blue".to_string(),
                important: false,
                parsed: None,
                span: None,
            }],
            span: None,
        };

        // p#intro (id=2) is direct child of div.container -> match
        assert!(rule_matches_element(&rule, &nodes[2], &nodes));
        // p.desc (id=5) is child of div.wrapper, not div directly -> no match via this rule
        // Actually p.desc's parent is div.wrapper (id=3), so div > p doesn't match p.desc
        // because the selector requires the parent to be div, and p.desc's parent IS div (wrapper)
        // Wait - div.wrapper IS a div. So div > p should match p.desc (id=5).
        assert!(rule_matches_element(&rule, &nodes[5], &nodes));
        // p (id=6) is also direct child of div.wrapper -> match
        assert!(rule_matches_element(&rule, &nodes[6], &nodes));
    }

    #[test]
    fn test_child_combinator_no_grandchild() {
        // Create a simple tree: div > span > p
        let nodes = vec![
            make_node(0, Some(make_element("div")), None, vec![1]),
            make_node(1, Some(make_element("span")), Some(0), vec![2]),
            make_node(2, Some(make_element("p")), Some(1), vec![]),
        ];
        let selector = Selector {
            simple_selectors: vec![
                SimpleSelector::Type(SmolStr::new("div")),
                SimpleSelector::Type(SmolStr::new("p")),
            ],
            combinators: vec![Combinator::Child],
            span: None,
        };
        let rule = StyleRule {
            selectors: vec![selector],
            declarations: smallvec![Declaration {
                property: SmolStr::new("color"),
                value: "green".to_string(),
                important: false,
                parsed: None,
                span: None,
            }],
            span: None,
        };

        // p (id=2) is NOT a direct child of div (id=0), so should NOT match
        assert!(!rule_matches_element(&rule, &nodes[2], &nodes));
    }

    #[test]
    fn test_adjacent_sibling_combinator() {
        // h1 + p should match p immediately after h1
        let nodes = build_dom();
        let selector = Selector {
            simple_selectors: vec![
                SimpleSelector::Type(SmolStr::new("h1")),
                SimpleSelector::Type(SmolStr::new("p")),
            ],
            combinators: vec![Combinator::AdjacentSibling],
            span: None,
        };
        let rule = StyleRule {
            selectors: vec![selector],
            declarations: smallvec![Declaration {
                property: SmolStr::new("color"),
                value: "purple".to_string(),
                important: false,
                parsed: None,
                span: None,
            }],
            span: None,
        };

        // p.desc (id=5) is immediately after h1 (id=4) in div.wrapper -> match
        assert!(rule_matches_element(&rule, &nodes[5], &nodes));
        // p (id=6) is NOT immediately after h1 -> no match
        assert!(!rule_matches_element(&rule, &nodes[6], &nodes));
        // p#intro (id=2) is before h1 -> no match
        assert!(!rule_matches_element(&rule, &nodes[2], &nodes));
    }

    #[test]
    fn test_general_sibling_combinator() {
        // h1 ~ p should match any p after h1
        let nodes = build_dom();
        let selector = Selector {
            simple_selectors: vec![
                SimpleSelector::Type(SmolStr::new("h1")),
                SimpleSelector::Type(SmolStr::new("p")),
            ],
            combinators: vec![Combinator::GeneralSibling],
            span: None,
        };
        let rule = StyleRule {
            selectors: vec![selector],
            declarations: smallvec![Declaration {
                property: SmolStr::new("color"),
                value: "orange".to_string(),
                important: false,
                parsed: None,
                span: None,
            }],
            span: None,
        };

        // p.desc (id=5) is after h1 (id=4) -> match
        assert!(rule_matches_element(&rule, &nodes[5], &nodes));
        // p (id=6) is also after h1 -> match
        assert!(rule_matches_element(&rule, &nodes[6], &nodes));
        // p#intro (id=2) is before h1 -> no match
        assert!(!rule_matches_element(&rule, &nodes[2], &nodes));
    }

    #[test]
    fn test_simple_selector_no_combinator() {
        // div.container (compound selector, no combinators)
        let nodes = build_dom();
        let selector = Selector {
            simple_selectors: vec![
                SimpleSelector::Type(SmolStr::new("div")),
                SimpleSelector::Class(SmolStr::new("container")),
            ],
            combinators: vec![],
            span: None,
        };
        let rule = StyleRule {
            selectors: vec![selector],
            declarations: smallvec![Declaration {
                property: SmolStr::new("color"),
                value: "red".to_string(),
                important: false,
                parsed: None,
                span: None,
            }],
            span: None,
        };

        // div.container (id=1) -> match
        assert!(rule_matches_element(&rule, &nodes[1], &nodes));
        // div.wrapper (id=3) -> no match (has "wrapper" not "container")
        assert!(!rule_matches_element(&rule, &nodes[3], &nodes));
        // p#intro (id=2) -> no match (not a div)
        assert!(!rule_matches_element(&rule, &nodes[2], &nodes));
    }

    #[test]
    fn test_chained_cominators() {
        // div.container > div > h1: should match h1 that is child of a div that is child of div.container
        let nodes = build_dom();
        let selector = Selector {
            simple_selectors: vec![
                SimpleSelector::Class(SmolStr::new("container")),
                SimpleSelector::Type(SmolStr::new("div")),
                SimpleSelector::Type(SmolStr::new("h1")),
            ],
            combinators: vec![Combinator::Child, Combinator::Child],
            span: None,
        };
        let rule = StyleRule {
            selectors: vec![selector],
            declarations: smallvec![Declaration {
                property: SmolStr::new("color"),
                value: "teal".to_string(),
                important: false,
                parsed: None,
                span: None,
            }],
            span: None,
        };

        // h1 (id=4): parent is div.wrapper (id=3), grandparent is div.container (id=1)
        // div.wrapper IS a div, div.container HAS class "container" -> match
        assert!(rule_matches_element(&rule, &nodes[4], &nodes));
    }
}
