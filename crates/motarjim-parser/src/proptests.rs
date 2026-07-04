#![allow(clippy::unwrap_used)]
use crate::{CssParser, HtmlParser};
use proptest::prelude::*;

proptest! {
    #[test]
    fn html_parser_never_panics(s in "\\PC*") {
        let mut parser = HtmlParser::new(&s);
        let _ = parser.parse();
    }

    #[test]
    fn css_parser_never_panics(s in "\\PC*") {
        let mut parser = CssParser::new(&s);
        let _ = parser.parse();
    }

    #[test]
    fn parser_output_is_well_formed(s in "\\PC*") {
        let mut parser = HtmlParser::new(&s);
        if let Ok(doc) = parser.parse() {
            let n = doc.nodes.len();
            prop_assert!((doc.root_id.0 as usize) < n, "root_id out of bounds");
            for node in &doc.nodes {
                for &child_id in &node.children {
                    prop_assert!(
                        (child_id.0 as usize) < n,
                        "child id {} out of bounds (n={})",
                        child_id.0, n
                    );
                }
                if let Some(parent_id) = node.parent {
                    prop_assert!(
                        (parent_id.0 as usize) < n,
                        "parent id {} out of bounds (n={})",
                        parent_id.0, n
                    );
                }
            }
        }
    }
}
