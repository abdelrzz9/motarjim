use super::*;
use crate::properties::{parse_font_weight, parse_grid_template, parse_grid_placement, parse_grid_template_areas, resolve_var};

use motarjim_ast::css::{CssRule, CssStylesheet, Declaration};
use motarjim_ast::Element;
use motarjim_ast_css::{Selector, SimpleSelector};
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
            span: None,
        }],
        declarations: decls.into(),
        span: None,
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
        parsed: None,
        span: None,
    }
}

fn important_decl(prop: &str, value: &str) -> Declaration {
    Declaration {
        property: SmolStr::new(prop),
        value: value.to_string(),
        important: true,
        parsed: None,
        span: None,
    }
}

fn sheet(rules: Vec<CssRule>) -> CssStylesheet {
    CssStylesheet {
        rules,
        source_path: None,
    }
}

// -----------------------------------------------------------------------
// Media query resolver integration tests (TASK 3)
// -----------------------------------------------------------------------

#[test]
#[allow(clippy::unwrap_used)]
fn test_media_query_applies_when_viewport_matches() {
    use motarjim_ast_css::{MediaCondition, MediaRule, MediaQuery};

    let media_rule = MediaRule {
        query: MediaQuery {
            conditions: vec![MediaCondition::MaxWidth("768px".to_string())],
        },
        rules: vec![make_css_rule("div", vec![decl("color", "red")])],
        span: None,
    };

    let mut resolver = StyleResolver::new();
    resolver.set_viewport(375, 667); // Mobile viewport
    resolver.add_stylesheet(sheet(vec![CssRule::Media(media_rule)]));

    let cv = resolver.resolve(&div_el());
    assert_eq!(cv.style.color.as_deref(), Some("red"));
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_media_query_skips_when_viewport_does_not_match() {
    use motarjim_ast_css::{MediaCondition, MediaRule, MediaQuery};

    let media_rule = MediaRule {
        query: MediaQuery {
            conditions: vec![MediaCondition::MaxWidth("768px".to_string())],
        },
        rules: vec![make_css_rule("div", vec![decl("color", "red")])],
        span: None,
    };

    let mut resolver = StyleResolver::new();
    resolver.set_viewport(1920, 1080); // Desktop viewport
    resolver.add_stylesheet(sheet(vec![CssRule::Media(media_rule)]));

    let cv = resolver.resolve(&div_el());
    assert!(cv.style.color.is_none());
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_media_query_min_width() {
    use motarjim_ast_css::{MediaCondition, MediaRule, MediaQuery};

    let media_rule = MediaRule {
        query: MediaQuery {
            conditions: vec![MediaCondition::MinWidth("1024px".to_string())],
        },
        rules: vec![make_css_rule("div", vec![decl("color", "blue")])],
        span: None,
    };

    let mut resolver = StyleResolver::new();
    resolver.set_viewport(1920, 1080);
    resolver.add_stylesheet(sheet(vec![CssRule::Media(media_rule)]));

    let cv = resolver.resolve(&div_el());
    assert_eq!(cv.style.color.as_deref(), Some("blue"));
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_media_query_prefers_color_scheme() {
    use motarjim_ast_css::{MediaCondition, MediaRule, MediaQuery};

    let media_rule = MediaRule {
        query: MediaQuery {
            conditions: vec![MediaCondition::PrefersColorScheme("dark".to_string())],
        },
        rules: vec![make_css_rule("div", vec![decl("background", "#1a1a2e")])],
        span: None,
    };

    let mut resolver = StyleResolver::new();
    resolver.set_color_scheme("dark".to_string());
    resolver.add_stylesheet(sheet(vec![CssRule::Media(media_rule)]));

    let cv = resolver.resolve(&div_el());
    let bg = cv.style.background.as_ref().unwrap();
    assert_eq!(bg.color.as_deref(), Some("#1a1a2e"));
}

// -----------------------------------------------------------------------
// Grid parsing tests (TASK 4)
// -----------------------------------------------------------------------

#[test]
fn test_parse_grid_template_fr() {
    let t = parse_grid_template("1fr 1fr 1fr").unwrap();
    assert_eq!(t.tracks.len(), 3);
    assert_eq!(t.tracks[0], motarjim_ast_html::grid::GridTrack::Fr(1.0));
    assert_eq!(t.tracks[1], motarjim_ast_html::grid::GridTrack::Fr(1.0));
    assert_eq!(t.tracks[2], motarjim_ast_html::grid::GridTrack::Fr(1.0));
}

#[test]
fn test_parse_grid_template_mixed() {
    use motarjim_ast_html::grid::GridTrack;
    let t = parse_grid_template("200px auto 1fr").unwrap();
    assert_eq!(t.tracks.len(), 3);
    assert_eq!(t.tracks[0], GridTrack::Fixed(200.0));
    assert_eq!(t.tracks[1], GridTrack::Auto);
    assert_eq!(t.tracks[2], GridTrack::Fr(1.0));
}

#[test]
fn test_parse_grid_template_minmax() {
    use motarjim_ast_html::grid::GridTrack;
    let t = parse_grid_template("minmax(100px, 1fr)").unwrap();
    assert_eq!(t.tracks.len(), 1);
    assert!(matches!(t.tracks[0], GridTrack::MinMax(_, _)));
}

#[test]
fn test_parse_grid_template_repeat() {
    use motarjim_ast_html::grid::GridTrack;
    let t = parse_grid_template("repeat(3, 1fr)").unwrap();
    assert_eq!(t.tracks.len(), 1);
    assert!(matches!(t.tracks[0], GridTrack::Repeat(3, _)));
}

#[test]
fn test_parse_grid_placement_number() {
    use motarjim_ast_html::grid::GridLine;
    let (start, end) = parse_grid_placement("1 / 3").unwrap();
    assert_eq!(start.line, GridLine::Number(1));
    assert!(end.is_some());
    assert_eq!(end.unwrap().line, GridLine::Number(3));
}

#[test]
fn test_parse_grid_placement_span() {
    use motarjim_ast_html::grid::GridLine;
    let (start, end) = parse_grid_placement("span 2").unwrap();
    assert_eq!(start.line, GridLine::Auto);
    assert_eq!(start.span, Some(2));
    assert!(end.is_none());
}

#[test]
fn test_parse_grid_placement_auto() {
    use motarjim_ast_html::grid::GridLine;
    let (start, end) = parse_grid_placement("auto").unwrap();
    assert_eq!(start.line, GridLine::Auto);
    assert!(end.is_none());
}

#[test]
fn test_parse_grid_template_areas() {
    let areas = parse_grid_template_areas("\"header header\" \"sidebar main\" \"footer footer\"");
    assert_eq!(areas, vec!["header", "header", "sidebar", "main", "footer", "footer"]);
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_grid_template_columns_stored() {
    let mut resolver = StyleResolver::new();
    resolver.add_stylesheet(sheet(vec![make_css_rule(
        "div",
        vec![decl("grid-template-columns", "1fr 1fr 1fr")],
    )]));
    let cv = resolver.resolve(&div_el());
    assert_eq!(cv.style.grid_template_columns.as_deref(), Some("1fr 1fr 1fr"));
    assert!(cv.style.grid_template_columns_structured.is_some());
    let t = cv.style.grid_template_columns_structured.as_ref().unwrap();
    assert_eq!(t.tracks.len(), 3);
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_grid_column_stored() {
    let mut resolver = StyleResolver::new();
    resolver.add_stylesheet(sheet(vec![make_css_rule(
        "div",
        vec![decl("grid-column", "1 / 3")],
    )]));
    let cv = resolver.resolve(&div_el());
    assert_eq!(cv.style.grid_column.as_deref(), Some("1 / 3"));
    assert!(cv.style.grid_column_start.is_some());
    assert!(cv.style.grid_column_end.is_some());
}

// -----------------------------------------------------------------------
// Animation tests (TASK 6)
// -----------------------------------------------------------------------

#[test]
#[allow(clippy::unwrap_used)]
fn test_animation_name_stored() {
    let mut resolver = StyleResolver::new();
    resolver.add_stylesheet(sheet(vec![make_css_rule(
        "div",
        vec![decl("animation-name", "slide")],
    )]));
    let cv = resolver.resolve(&div_el());
    assert_eq!(cv.style.animation_name.as_deref(), Some("slide"));
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_animation_duration_stored() {
    let mut resolver = StyleResolver::new();
    resolver.add_stylesheet(sheet(vec![make_css_rule(
        "div",
        vec![decl("animation-duration", "1s")],
    )]));
    let cv = resolver.resolve(&div_el());
    assert_eq!(cv.style.animation_duration.as_deref(), Some("1s"));
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_animation_shorthand() {
    let mut resolver = StyleResolver::new();
    resolver.add_stylesheet(sheet(vec![make_css_rule(
        "div",
        vec![decl("animation", "slide 1s ease-in-out")],
    )]));
    let cv = resolver.resolve(&div_el());
    assert_eq!(cv.style.animation_name.as_deref(), Some("slide"));
    assert_eq!(cv.style.animation_duration.as_deref(), Some("1s"));
    assert_eq!(
        cv.style.animation_timing_function.as_deref(),
        Some("ease-in-out")
    );
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_keyframes_collected() {
    use motarjim_ast::css::{CssRule, Keyframe, KeyframesRule};
    use smol_str::SmolStr;

    let keyframes_rule = KeyframesRule {
        name: SmolStr::new("slide"),
        keyframes: vec![Keyframe {
            selectors: smallvec::smallvec![SmolStr::new("from")],
            declarations: smallvec::smallvec![],
            span: None,
        }],
        span: None,
    };

    let mut resolver = StyleResolver::new();
    resolver.add_stylesheet(sheet(vec![CssRule::Keyframes(keyframes_rule)]));

    let keyframes = resolver.collect_keyframes();
    assert!(keyframes.contains_key("slide"));
    assert_eq!(keyframes["slide"].keyframes.len(), 1);
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_animation_fill_mode() {
    let mut resolver = StyleResolver::new();
    resolver.add_stylesheet(sheet(vec![make_css_rule(
        "div",
        vec![decl("animation-fill-mode", "forwards")],
    )]));
    let cv = resolver.resolve(&div_el());
    assert_eq!(cv.style.animation_fill_mode.as_deref(), Some("forwards"));
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_animation_iteration_count() {
    let mut resolver = StyleResolver::new();
    resolver.add_stylesheet(sheet(vec![make_css_rule(
        "div",
        vec![decl("animation-iteration-count", "infinite")],
    )]));
    let cv = resolver.resolve(&div_el());
    assert_eq!(
        cv.style.animation_iteration_count.as_deref(),
        Some("infinite")
    );
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
            span: None,
        }],
        declarations: smallvec::smallvec![decl("color", "green")],
        span: None,
    };
    // Class selector: .highlight
    let class_rule = StyleRule {
        selectors: vec![Selector {
            simple_selectors: vec![SimpleSelector::Class(SmolStr::new("highlight"))],
            combinators: vec![],
            span: None,
        }],
        declarations: smallvec::smallvec![decl("color", "yellow")],
        span: None,
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
            span: None,
        }],
        declarations: smallvec::smallvec![decl("color", "purple")],
        span: None,
    };
    // #id has specificity (1,0,0)
    let id_rule = StyleRule {
        selectors: vec![Selector {
            simple_selectors: vec![SimpleSelector::Id(SmolStr::new("id"))],
            combinators: vec![],
            span: None,
        }],
        declarations: smallvec::smallvec![decl("color", "orange")],
        span: None,
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

// -----------------------------------------------------------------------
// CSS Variable resolution tests (TASK 1)
// -----------------------------------------------------------------------

#[test]
fn test_var_resolution_basic() {
    let mut style = ComputedStyle::default();
    style
        .custom_properties
        .insert("primary".to_string(), "blue".to_string());
    let resolved = resolve_var("var(--primary)", &style);
    assert_eq!(resolved, "blue");
}

#[test]
fn test_var_resolution_with_fallback() {
    let style = ComputedStyle::default(); // no custom properties
    let resolved = resolve_var("var(--undefined, red)", &style);
    assert_eq!(resolved, "red");
}

#[test]
fn test_var_resolution_no_fallback() {
    let style = ComputedStyle::default();
    let resolved = resolve_var("var(--undefined)", &style);
    assert_eq!(resolved, "var(--undefined)");
}

#[test]
fn test_var_resolution_nested() {
    let mut style = ComputedStyle::default();
    style
        .custom_properties
        .insert("b".to_string(), "10px".to_string());
    style
        .custom_properties
        .insert("a".to_string(), "var(--b)".to_string());
    let resolved = resolve_var("var(--a)", &style);
    assert_eq!(resolved, "10px");
}

#[test]
fn test_var_resolution_circular_prevented() {
    let mut style = ComputedStyle::default();
    style
        .custom_properties
        .insert("a".to_string(), "var(--b)".to_string());
    style
        .custom_properties
        .insert("b".to_string(), "var(--a)".to_string());
    let resolved = resolve_var("var(--a)", &style);
    // Should not infinite loop; returns a string (possibly var reference)
    assert!(!resolved.is_empty());
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_apply_declarations_stores_custom_properties() {
    let mut style = ComputedStyle::default();
    let mut map = HashMap::new();
    map.insert(SmolStr::new("--primary"), "blue".to_string());
    map.insert(SmolStr::new("color"), "var(--primary)".to_string());
    apply_declarations(&mut style, &map);
    assert_eq!(
        style.custom_properties.get("primary").unwrap(),
        "blue"
    );
    assert_eq!(style.color.as_deref(), Some("blue"));
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_apply_declarations_var_in_color() {
    let mut style = ComputedStyle::default();
    let mut map = HashMap::new();
    map.insert(SmolStr::new("--bg"), "#fff".to_string());
    map.insert(SmolStr::new("background-color"), "var(--bg)".to_string());
    apply_declarations(&mut style, &map);
    let bg = style.background.as_ref().unwrap();
    assert_eq!(bg.color.as_deref(), Some("#fff"));
}

// -----------------------------------------------------------------------
// Positioning offsets tests (TASK 2)
// -----------------------------------------------------------------------

#[test]
#[allow(clippy::unwrap_used)]
fn test_positioning_top_left() {
    let mut resolver = StyleResolver::new();
    resolver.add_stylesheet(sheet(vec![make_css_rule(
        "div",
        vec![
            decl("position", "absolute"),
            decl("top", "10px"),
            decl("left", "20px"),
        ],
    )]));
    let cv = resolver.resolve(&div_el());
    assert_eq!(cv.style.position, PositionType::Absolute);
    assert_eq!(cv.style.top.as_deref(), Some("10px"));
    assert_eq!(cv.style.left.as_deref(), Some("20px"));
    assert!(cv.style.bottom.is_none());
    assert!(cv.style.right.is_none());
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_positioning_inset_shorthand() {
    let mut resolver = StyleResolver::new();
    resolver.add_stylesheet(sheet(vec![make_css_rule(
        "div",
        vec![decl("position", "absolute"), decl("inset", "10px")],
    )]));
    let cv = resolver.resolve(&div_el());
    assert_eq!(cv.style.top.as_deref(), Some("10px"));
    assert_eq!(cv.style.right.as_deref(), Some("10px"));
    assert_eq!(cv.style.bottom.as_deref(), Some("10px"));
    assert_eq!(cv.style.left.as_deref(), Some("10px"));
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_positioning_inset_two_values() {
    let mut resolver = StyleResolver::new();
    resolver.add_stylesheet(sheet(vec![make_css_rule(
        "div",
        vec![decl("position", "relative"), decl("inset", "5px 10px")],
    )]));
    let cv = resolver.resolve(&div_el());
    assert_eq!(cv.style.top.as_deref(), Some("5px"));
    assert_eq!(cv.style.right.as_deref(), Some("10px"));
    assert_eq!(cv.style.bottom.as_deref(), Some("5px"));
    assert_eq!(cv.style.left.as_deref(), Some("10px"));
}

#[test]
#[allow(clippy::unwrap_used)]
fn test_positioning_sticky_top() {
    let mut resolver = StyleResolver::new();
    resolver.add_stylesheet(sheet(vec![make_css_rule(
        "div",
        vec![decl("position", "sticky"), decl("top", "0")],
    )]));
    let cv = resolver.resolve(&div_el());
    assert_eq!(cv.style.position, PositionType::Sticky);
    assert_eq!(cv.style.top.as_deref(), Some("0"));
}
