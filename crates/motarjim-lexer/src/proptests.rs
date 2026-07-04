#![cfg(test)]

use crate::css::{CssTokenKind, CssTokenizer};
use crate::html::{HtmlTokenKind, HtmlTokenizer};
use proptest::prelude::*;

proptest! {
    #[test]
    fn html_tokenizer_never_panics(s in "\\PC*") {
        let mut tokenizer = HtmlTokenizer::new(&s);
        let tokens = tokenizer.tokenize();
        prop_assert!(!tokens.is_empty());
        prop_assert_eq!(tokens.last().map(|t| t.kind), Some(HtmlTokenKind::Eof));
    }

    #[test]
    fn css_tokenizer_never_panics(s in "\\PC*") {
        let mut tokenizer = CssTokenizer::new(&s);
        let tokens = tokenizer.tokenize();
        prop_assert!(!tokens.is_empty());
        prop_assert_eq!(tokens.last().map(|t| t.kind), Some(CssTokenKind::Eof));
    }
}
