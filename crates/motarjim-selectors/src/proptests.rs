#![allow(clippy::unwrap_used)]
use crate::{parse_selector, Specificity};
use proptest::prelude::*;

proptest! {
    #[test]
    fn selector_parse_never_panics(s in "\\PC*") {
        let _ = parse_selector(&s);
    }

    #[test]
    fn specificity_is_non_negative(s in "\\PC*") {
        if let Ok(sel) = parse_selector(&s) {
            let spec = Specificity::of(&sel);
            prop_assert!(spec.ids <= 100_000, "ids specificity overflow");
            prop_assert!(spec.classes <= 100_000, "classes specificity overflow");
            prop_assert!(spec.types <= 100_000, "types specificity overflow");
        }
    }
}
