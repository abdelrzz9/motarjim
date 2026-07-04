use super::*;
use crate::properties::parse_font_weight;

use motarjim_ast::css::{CssRule, CssStylesheet, Declaration};
use motarjim_ast::selector::{Selector, SimpleSelector};
use motarjim_ast::Element;
use smol_str::SmolStr;

fn div_el() -> Element {
    Element::new("div")
}

fn span_el() -> Element {
    Element::new("span")
}

fn make_style_rule(selector_str: &str, decls: Vec<Declaration>) -> StyleRule {
    let simple = SimpleSelector::Type(SmolStr::new(selector_str));
    StyleRule {
        selectors: vec![Selector {
            simple_selectors: vec![simple],
            combinators: vec![],
        }],
        declarations: decls.into(),
    }
}

fn make_css_rule(selector_str: &str, decls: Vec<Declaration>) -> CssRule {
    CssRule::Style(make_style_rule(selector_str, decls))
}

fn decl(prop: &str, value: &str) -> Declaration {
    Declaration {
        property: SmolStr::new(prop),
        value: value.to_string(),
        important: false,
    }
}

fn important_decl(prop: &str, value: &str) -> Declaration {
    Declaration {
        property: SmolStr::new(prop),
        value: value.to_string(),
        important: true,
    }
}

fn sheet(rules: Vec<CssRule>) -> CssStylesheet {
    CssStylesheet {
        rules,
        source_path: None,
    }
}

// -----------------------------------------------------------------------
// Cascade order tests
// -----------------------------------------------------------------------

#[test]
#[allow(clippy::unwrap_used)]
fn test_cascade_later_overrides_earlier() {
    let mut resolver = StyleResolver::new();
    resolver.add_stylesheet(sheet(vec![
        make_css_rule("div", vec![decl("color", "red")]),
        make_css_rule("div", vec![decl("color", "blue")]),
    ]));
    let cv = resolver.resolve(&div_el());
    assert_eq!(cv.style.color.as_deref(), Some("blue"));
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_cascade_important_overrides_normal() {
    let mut resolver = StyleResolver::new();
    resolver.add_stylesheet(sheet(vec![
        make_css_rule("div", vec![important_decl("color", "red")]),
        make_css_rule("div", vec![decl("color", "blue")]),
    ]));
    let cv = resolver.resolve(&div_el());
    assert_eq!(cv.style.color.as_deref(), Some("red"));
}

// -----------------------------------------------------------------------
// Specificity tests
// -----------------------------------------------------------------------

#[test]
#[allow(clippy::unwrap_used)]
fn test_specificity_id_wins_over_class() {
    let mut resolver = StyleResolver::new();
    // ID selector: #main
    let id_rule = StyleRule {
        selectors: vec![Selector {
            simple_selectors: vec![SimpleSelector::Id(SmolStr::new("main"))],
            combinators: vec![],
        }],
        declarations: smallvec::smallvec![decl("color", "green")],
    };
    // Class selector: .highlight
    let class_rule = StyleRule {
        selectors: vec![Selector {
            simple_selectors: vec![SimpleSelector::Class(SmolStr::new("highlight"))],
            combinators: vec![],
        }],
        declarations: smallvec::smallvec![decl("color", "yellow")],
    };
    // Note: style rule with class comes second (later source order),
    // but ID should still win due to higher specificity.
    resolver.add_stylesheet(sheet(vec![
        CssRule::Style(id_rule),
        CssRule::Style(class_rule),
    ]));

    let mut el = div_el();
    el.id = Some(SmolStr::new("main"));
    el.classes.push(SmolStr::new("highlight"));
    let cv = resolver.resolve(&el);
    assert_eq!(cv.style.color.as_deref(), Some("green"));
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_specificity_multiple_classes_vs_id() {
    let mut resolver = StyleResolver::new();
    // .a.b has specificity (0,2,0)
    let class_rule = StyleRule {
        selectors: vec![Selector {
            simple_selectors: vec![
                SimpleSelector::Class(SmolStr::new("a")),
                SimpleSelector::Class(SmolStr::new("b")),
            ],
            combinators: vec![],
        }],
        declarations: smallvec::smallvec![decl("color", "purple")],
    };
    // #id has specificity (1,0,0)
    let id_rule = StyleRule {
        selectors: vec![Selector {
            simple_selectors: vec![SimpleSelector::Id(SmolStr::new("id"))],
            combinators: vec![],
        }],
        declarations: smallvec::smallvec![decl("color", "orange")],
    };
    resolver.add_stylesheet(sheet(vec![
        CssRule::Style(class_rule),
        CssRule::Style(id_rule),
    ]));

    let mut el = div_el();
    el.id = Some(SmolStr::new("id"));
    el.classes.push(SmolStr::new("a"));
    el.classes.push(SmolStr::new("b"));
    let cv = resolver.resolve(&el);
    assert_eq!(cv.style.color.as_deref(), Some("orange"));
}

// -----------------------------------------------------------------------
// Computed values tests
// -----------------------------------------------------------------------

#[test]
#[allow(clippy::unwrap_used)]
fn test_computed_display() {
    let mut resolver = StyleResolver::new();
    resolver.add_stylesheet(sheet(vec![make_css_rule(
        "div",
        vec![decl("display", "flex")],
    )]));
    let cv = resolver.resolve(&div_el());
    assert_eq!(cv.style.display, DisplayType::Flex);
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_computed_margin_padding() {
    let mut resolver = StyleResolver::new();
    resolver.add_stylesheet(sheet(vec![make_css_rule(
        "div",
        vec![decl("margin", "10px"), decl("padding", "5px 10px")],
    )]));
    let cv = resolver.resolve(&div_el());
    assert_eq!(cv.style.margin.top, 10.0);
    assert_eq!(cv.style.margin.right, 10.0);
    assert_eq!(cv.style.margin.bottom, 10.0);
    assert_eq!(cv.style.margin.left, 10.0);
    assert_eq!(cv.style.padding.top, 5.0);
    assert_eq!(cv.style.padding.right, 10.0);
    assert_eq!(cv.style.padding.bottom, 5.0);
    assert_eq!(cv.style.padding.left, 10.0);
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_computed_flex_properties() {
    let mut resolver = StyleResolver::new();
    resolver.add_stylesheet(sheet(vec![make_css_rule(
        "div",
        vec![
            decl("display", "flex"),
            decl("flex-direction", "column"),
            decl("justify-content", "center"),
            decl("align-items", "center"),
            decl("flex-grow", "1"),
            decl("flex-wrap", "wrap"),
        ],
    )]));
    let cv = resolver.resolve(&div_el());
    assert_eq!(cv.style.display, DisplayType::Flex);
    assert_eq!(cv.style.flex_direction, Some(FlexDirection::Column));
    assert_eq!(cv.style.justify_content, Some(JustifyContent::Center));
    assert_eq!(cv.style.align_items, Some(AlignItems::Center));
    assert!((cv.style.flex_grow - 1.0).abs() < f64::EPSILON);
    assert_eq!(cv.style.flex_wrap, Some(FlexWrap::Wrap));
}

// -----------------------------------------------------------------------
// Inheritance tests
// -----------------------------------------------------------------------

#[test]
#[allow(clippy::unwrap_used)]
fn test_inheritance_color() {
    // Color is inherited. If parent has a color, child should inherit it.
    let parent_style = {
        let mut cv = ComputedValues::new();
        cv.style.color = Some("red".to_string());
        cv
    };

    let mut resolver = StyleResolver::new();
    // Child element has no matching rule for color, so it should inherit from parent.
    resolver.add_stylesheet(sheet(vec![make_css_rule(
        "span",
        vec![decl("font-size", "14px")],
    )]));

    let cv = resolver.resolve_with_parent(&span_el(), Some(&parent_style));
    assert_eq!(cv.style.color.as_deref(), Some("red"));
    assert_eq!(cv.style.font_size.as_deref(), Some("14px"));
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_child_override_inherited() {
    let parent_style = {
        let mut cv = ComputedValues::new();
        cv.style.color = Some("red".to_string());
        cv.style.font_size = Some("16px".to_string());
        cv
    };

    let mut resolver = StyleResolver::new();
    // Child overrides color but not font-size.
    resolver.add_stylesheet(sheet(vec![make_css_rule(
        "span",
        vec![decl("color", "blue")],
    )]));

    let cv = resolver.resolve_with_parent(&span_el(), Some(&parent_style));
    assert_eq!(cv.style.color.as_deref(), Some("blue"));
    assert_eq!(cv.style.font_size.as_deref(), Some("16px"));
}

// -----------------------------------------------------------------------
// Parallel matching test
// -----------------------------------------------------------------------

#[test]
#[allow(clippy::unwrap_used)]
fn test_parallel_resolve() {
    let mut resolver = StyleResolver::new();
    resolver.add_stylesheet(sheet(vec![
        make_css_rule("div", vec![decl("color", "red"), decl("font-size", "12px")]),
        make_css_rule("span", vec![decl("color", "blue")]),
    ]));

    let elements = vec![div_el(), span_el(), div_el(), span_el()];
    let results = resolver.resolve_parallel(&elements);
    assert_eq!(results.len(), 4);
    assert_eq!(results[0].style.color.as_deref(), Some("red"));
    assert_eq!(results[1].style.color.as_deref(), Some("blue"));
    assert_eq!(results[2].style.color.as_deref(), Some("red"));
    assert_eq!(results[3].style.color.as_deref(), Some("blue"));
    assert_eq!(results[0].style.font_size.as_deref(), Some("12px"));
    assert!(results[1].style.font_size.is_none());
}

// -----------------------------------------------------------------------
// Color parsing tests
// -----------------------------------------------------------------------

#[test]
#[allow(clippy::unwrap_used)]
fn test_parse_named_color() {
    let c = parse_color("red").expect("should parse red");
    assert_eq!(c, CssColor::Rgba(255, 0, 0, 1.0));
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_parse_hex_color() {
    let c = parse_color("#ff0000").expect("should parse hex");
    assert_eq!(c, CssColor::Hex(255, 0, 0, 255));
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_parse_hex_short_color() {
    let c = parse_color("#f00").expect("should parse short hex");
    assert_eq!(c, CssColor::Hex(255, 0, 0, 255));
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_parse_hex_alpha_color() {
    let c = parse_color("#ff000080").expect("should parse hex with alpha");
    assert_eq!(c, CssColor::Hex(255, 0, 0, 128));
}

#[test]
fn test_parse_rgb_color() {
    let c = parse_color("rgb(255, 0, 0)").expect("should parse rgb");
    assert_eq!(c, CssColor::Rgba(255, 0, 0, 1.0));
}

#[test]
fn test_parse_rgba_color() {
    let c = parse_color("rgba(255, 0, 0, 0.5)").expect("should parse rgba");
    assert_eq!(c, CssColor::Rgba(255, 0, 0, 0.5));
}

#[test]
fn test_parse_transparent() {
    let c = parse_color("transparent").expect("should parse transparent");
    assert_eq!(c, CssColor::Transparent);
}

#[test]
fn test_parse_invalid_color() {
    assert!(parse_color("not-a-color").is_none());
}

// -----------------------------------------------------------------------
// Length / unit parsing tests
// -----------------------------------------------------------------------

#[test]
#[allow(clippy::unwrap_used)]
fn test_parse_length_px() {
    let l = parse_length("42px").expect("should parse px");
    assert_eq!(l, CssLength::Px(42.0));
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_parse_length_em() {
    let l = parse_length("1.5em").expect("should parse em");
    assert_eq!(l, CssLength::Em(1.5));
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_parse_length_rem() {
    let l = parse_length("2rem").expect("should parse rem");
    assert_eq!(l, CssLength::Rem(2.0));
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_parse_length_percent() {
    let l = parse_length("50%").expect("should parse percent");
    assert_eq!(l, CssLength::Percent(50.0));
}

#[test]
fn test_parse_length_auto_returns_none() {
    assert!(parse_length("auto").is_none());
}

#[test]
fn test_parse_length_invalid() {
    assert!(parse_length("abc").is_none());
}

// -----------------------------------------------------------------------
// ComputedValues builder test
// -----------------------------------------------------------------------

#[test]
#[allow(clippy::unwrap_used)]
fn test_from_map_basic() {
    let mut map = HashMap::new();
    map.insert(SmolStr::new("color"), "green".to_string());
    map.insert(SmolStr::new("display"), "flex".to_string());
    map.insert(SmolStr::new("opacity"), "0.75".to_string());

    let cv = ComputedValues::from_map(&map, None);
    assert_eq!(cv.style.color.as_deref(), Some("green"));
    assert_eq!(cv.style.display, DisplayType::Flex);
    assert!((cv.style.opacity - 0.75).abs() < f64::EPSILON);
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_resolver_clear() {
    let mut resolver = StyleResolver::new();
    resolver.add_stylesheet(sheet(vec![make_css_rule(
        "div",
        vec![decl("color", "red")],
    )]));
    resolver.clear();
    assert!(resolver.stylesheets().is_empty());
    let cv = resolver.resolve(&div_el());
    assert!(cv.style.color.is_none());
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_font_weight_parsing() {
    let fw = parse_font_weight("bold");
    assert_eq!(fw, Some(FontWeight::Bold));

    let fw = parse_font_weight("700");
    assert_eq!(fw, Some(FontWeight::Bold));

    let fw = parse_font_weight("400");
    assert_eq!(fw, Some(FontWeight::Normal));

    let fw = parse_font_weight("850");
    assert_eq!(fw, Some(FontWeight::Custom(850)));
}
