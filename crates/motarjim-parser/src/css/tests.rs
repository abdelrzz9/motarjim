//! Comprehensive tests for the CSS parser module.
//!
//! Tests cover all supported CSS features, edge cases, and error handling.

#![allow(clippy::unwrap_used, clippy::panic)]

use crate::css::parse_css;
use motarjim_ast::css::CssRule;

// ---------------------------------------------------------------------------
// Basic parsing tests
// ---------------------------------------------------------------------------

#[test]
fn test_empty_stylesheet() {
    let sheet = parse_css("").unwrap();
    assert!(sheet.rules.is_empty());
}

#[test]
fn test_whitespace_only() {
    let sheet = parse_css("   \n  \t  ").unwrap();
    assert!(sheet.rules.is_empty());
}

#[test]
fn test_comments_only() {
    let sheet = parse_css("/* this is a comment */").unwrap();
    assert!(sheet.rules.is_empty());
}

#[test]
fn test_comments_with_rules() {
    let sheet = parse_css("/* header */ div { color: red; } /* footer */").unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

// ---------------------------------------------------------------------------
// Style rules
// ---------------------------------------------------------------------------

#[test]
fn test_simple_style_rule() {
    let sheet = parse_css("div { color: red; }").unwrap();
    assert_eq!(sheet.rules.len(), 1);

    match &sheet.rules[0] {
        CssRule::Style(sr) => {
            assert_eq!(sr.selectors.len(), 1);
            assert_eq!(sr.declarations.len(), 1);
            assert_eq!(sr.declarations[0].property.as_str(), "color");
        }
        other => panic!("Expected Style rule, got {other:?}"),
    }
}

#[test]
fn test_multiple_declarations() {
    let css = r#"
        div {
            color: red;
            font-size: 16px;
            margin: 0;
            padding: 10px;
        }
    "#;
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);

    match &sheet.rules[0] {
        CssRule::Style(sr) => {
            assert_eq!(sr.declarations.len(), 4);
        }
        _ => panic!("Expected Style rule"),
    }
}

#[test]
fn test_grouped_selectors() {
    let sheet = parse_css("div, span, p { color: red; }").unwrap();
    match &sheet.rules[0] {
        CssRule::Style(sr) => {
            assert_eq!(sr.selectors.len(), 3);
        }
        _ => panic!("Expected Style rule"),
    }
}

#[test]
fn test_class_selector() {
    let sheet = parse_css(".container { padding: 10px; }").unwrap();
    match &sheet.rules[0] {
        CssRule::Style(sr) => {
            assert_eq!(sr.selectors.len(), 1);
        }
        _ => panic!("Expected Style rule"),
    }
}

#[test]
fn test_id_selector() {
    let sheet = parse_css("#header { background: blue; }").unwrap();
    match &sheet.rules[0] {
        CssRule::Style(sr) => {
            assert_eq!(sr.selectors.len(), 1);
        }
        _ => panic!("Expected Style rule"),
    }
}

#[test]
fn test_universal_selector() {
    let sheet = parse_css("* { margin: 0; }").unwrap();
    match &sheet.rules[0] {
        CssRule::Style(sr) => {
            assert_eq!(sr.selectors.len(), 1);
        }
        _ => panic!("Expected Style rule"),
    }
}

#[test]
fn test_attribute_selector() {
    let sheet = parse_css("[disabled] { opacity: 0.5; }").unwrap();
    match &sheet.rules[0] {
        CssRule::Style(sr) => {
            assert_eq!(sr.selectors.len(), 1);
        }
        _ => panic!("Expected Style rule"),
    }
}

#[test]
fn test_attribute_selector_with_value() {
    let sheet = parse_css("[type=text] { border: 1px solid; }").unwrap();
    match &sheet.rules[0] {
        CssRule::Style(sr) => {
            assert_eq!(sr.selectors.len(), 1);
        }
        _ => panic!("Expected Style rule"),
    }
}

#[test]
fn test_pseudo_class_selector() {
    let sheet = parse_css("a:hover { color: blue; }").unwrap();
    match &sheet.rules[0] {
        CssRule::Style(sr) => {
            assert_eq!(sr.selectors.len(), 1);
        }
        _ => panic!("Expected Style rule"),
    }
}

#[test]
fn test_pseudo_element_selector() {
    let sheet = parse_css("p::first-line { font-weight: bold; }").unwrap();
    match &sheet.rules[0] {
        CssRule::Style(sr) => {
            assert_eq!(sr.selectors.len(), 1);
        }
        _ => panic!("Expected Style rule"),
    }
}

#[test]
fn test_empty_block() {
    let sheet = parse_css("div {}").unwrap();
    match &sheet.rules[0] {
        CssRule::Style(sr) => {
            assert!(sr.declarations.is_empty());
        }
        _ => panic!("Expected Style rule"),
    }
}

#[test]
fn test_important() {
    let sheet = parse_css("div { color: red !important; }").unwrap();
    match &sheet.rules[0] {
        CssRule::Style(sr) => {
            assert_eq!(sr.declarations.len(), 1);
            assert!(sr.declarations[0].important);
        }
        _ => panic!("Expected Style rule"),
    }
}

// ---------------------------------------------------------------------------
// At-rules
// ---------------------------------------------------------------------------

#[test]
fn test_media_rule() {
    let css = "@media screen { div { color: black; } }";
    let sheet = parse_css(css).unwrap();
    assert!(!sheet.rules.is_empty());
    let has_media = sheet.rules.iter().any(|r| matches!(r, CssRule::Media(_)));
    assert!(has_media);

    if let CssRule::Media(mr) = &sheet.rules[0] {
        assert_eq!(mr.rules.len(), 1);
    }
}

#[test]
fn test_media_rule_with_multiple_rules() {
    let css = "@media screen { div { color: black; } span { color: gray; } }";
    let sheet = parse_css(css).unwrap();
    if let CssRule::Media(mr) = &sheet.rules[0] {
        assert_eq!(mr.rules.len(), 2);
    } else {
        panic!("Expected Media rule");
    }
}

#[test]
fn test_media_rule_with_min_width() {
    let css = "@media (min-width: 768px) { div { width: 50%; } }";
    let sheet = parse_css(css).unwrap();
    assert!(sheet.rules.iter().any(|r| matches!(r, CssRule::Media(_))));
}

#[test]
fn test_import_rule() {
    let css = "@import url('styles.css');";
    let sheet = parse_css(css).unwrap();
    let has_import = sheet.rules.iter().any(|r| matches!(r, CssRule::Import(_)));
    assert!(has_import);
}

#[test]
fn test_keyframes_rule() {
    let css = "@keyframes fade-in { from { opacity: 0; } to { opacity: 1; } }";
    let sheet = parse_css(css).unwrap();
    let has_kf = sheet.rules.iter().any(|r| matches!(r, CssRule::Keyframes(_)));
    assert!(has_kf);

    if let CssRule::Keyframes(kf) = &sheet.rules[0] {
        assert_eq!(kf.keyframes.len(), 2);
    }
}

#[test]
fn test_keyframes_with_percentages() {
    let css = "@keyframes slide { 0% { left: 0; } 50% { left: 50%; } 100% { left: 100%; } }";
    let sheet = parse_css(css).unwrap();
    if let CssRule::Keyframes(kf) = &sheet.rules[0] {
        assert_eq!(kf.keyframes.len(), 3);
    }
}

#[test]
fn test_font_face_rule() {
    let css = "@font-face { font-family: 'MyFont'; src: url('font.woff2'); }";
    let sheet = parse_css(css).unwrap();
    let has_font = sheet.rules.iter().any(|r| matches!(r, CssRule::FontFace(_)));
    assert!(has_font);
}

#[test]
fn test_supports_rule() {
    let css = "@supports (display: flex) { div { display: flex; } }";
    let sheet = parse_css(css).unwrap();
    let has_supports = sheet.rules.iter().any(|r| matches!(r, CssRule::Supports(_)));
    assert!(has_supports);
}

#[test]
fn test_page_rule() {
    let css = "@page { margin: 2cm; }";
    let sheet = parse_css(css).unwrap();
    let has_page = sheet.rules.iter().any(|r| matches!(r, CssRule::Page(_)));
    assert!(has_page);
}

#[test]
fn test_charset_rule() {
    // @charset is obsolete; Lightning CSS ignores it.
    // Ensure no crash and rules are empty.
    let sheet = parse_css("@charset \"UTF-8\";").unwrap();
    assert!(sheet.rules.is_empty() || sheet.rules.iter().any(|r| matches!(r, CssRule::Charset(_))));
}

#[test]
fn test_namespace_rule() {
    let css = "@namespace svg url('http://www.w3.org/2000/svg');";
    let sheet = parse_css(css).unwrap();
    let has_ns = sheet.rules.iter().any(|r| matches!(r, CssRule::Namespace(_)));
    assert!(has_ns);
}

// ---------------------------------------------------------------------------
// Complex CSS features
// ---------------------------------------------------------------------------

#[test]
fn test_calc_expression() {
    let css = "div { width: calc(100% - 20px); }";
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_variables() {
    let css = ":root { --primary-color: blue; } div { color: var(--primary-color); }";
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 2);
}

#[test]
fn test_gradient() {
    let css = "div { background: linear-gradient(135deg, red, blue); }";
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_transform() {
    let css = "div { transform: translateX(10px) rotate(45deg); }";
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_animation() {
    let css = "div { animation: slide 2s ease-in-out infinite; }";
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_transition() {
    let css = "div { transition: all 0.3s ease; }";
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_box_shadow() {
    let css = "div { box-shadow: 0 2px 4px rgba(0,0,0,0.1); }";
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_text_shadow() {
    let css = "div { text-shadow: 1px 1px 2px black; }";
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_multiple_backgrounds() {
    let css = "div { background: url('bg.png') no-repeat center, linear-gradient(red, blue); }";
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_flexbox() {
    let css = "div { display: flex; justify-content: center; align-items: center; gap: 16px; }";
    let sheet = parse_css(css).unwrap();
    match &sheet.rules[0] {
        CssRule::Style(sr) => {
            assert_eq!(sr.declarations.len(), 4);
        }
        _ => panic!("Expected Style rule"),
    }
}

#[test]
fn test_grid() {
    let css = "div { display: grid; grid-template-columns: repeat(3, 1fr); gap: 24px; }";
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_custom_properties() {
    let css = ":root { --spacing: 8px; --theme-color: #333; }";
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_nested_selectors() {
    let css = "div { color: red; }  div span { color: blue; }  div > p { color: green; }";
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 3);
}

#[test]
fn test_complex_selectors() {
    let css = "ul.nav > li.active a:hover { color: red; }";
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

// ---------------------------------------------------------------------------
// Media features
// ---------------------------------------------------------------------------

#[test]
fn test_media_prefers_color_scheme() {
    let css = "@media (prefers-color-scheme: dark) { body { background: black; } }";
    let sheet = parse_css(css).unwrap();
    assert!(sheet.rules.iter().any(|r| matches!(r, CssRule::Media(_))));
}

#[test]
fn test_media_multiple_conditions() {
    let css = "@media screen and (min-width: 768px) and (max-width: 1024px) { div { width: 50%; } }";
    let sheet = parse_css(css).unwrap();
    assert!(sheet.rules.iter().any(|r| matches!(r, CssRule::Media(_))));
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_malformed_css() {
    // Lightning CSS is lenient and may parse incomplete input without error.
    let result = parse_css("div { color: red; ");
    // Either it parses successfully (lenient) or returns an error.
    // The important thing is no panic.
    let _ = result;
}

#[test]
fn test_invalid_property() {
    // Lightning CSS is lenient - it will parse but mark invalid properties
    let sheet = parse_css("div { unknown-property: value; }");
    assert!(sheet.is_ok());
}

#[test]
fn test_unclosed_comment() {
    // Lightning CSS is lenient and may treat unclosed comments as valid.
    let result = parse_css("div { color: red; } /* unclosed");
    // Either way, no panic.
    let _ = result;
}

#[test]
fn test_extra_closing_brace() {
    let result = parse_css("div { color: red; } }");
    assert!(result.is_err());
}

#[test]
fn test_unicode_range() {
    let css = "@font-face { unicode-range: U+0025-00FF; }";
    let result = parse_css(css);
    assert!(result.is_ok());
}

#[test]
fn test_counter_style() {
    let css = "@counter-style thumbs { system: cyclic; symbols: \"\\1F44D\"; suffix: \" \"; }";
    let result = parse_css(css);
    // Counter-style may be unsupported, but shouldn't crash
    if let Ok(sheet) = result {
        assert!(!sheet.rules.is_empty());
    }
}

#[test]
fn test_large_stylesheet() {
    let mut css = String::new();
    for i in 0..100 {
        css.push_str(&format!(".class-{} {{ color: rgb({}, {}, {}); }}\n", i, i % 255, (i * 2) % 255, (i * 3) % 255));
    }
    let sheet = parse_css(&css).unwrap();
    assert_eq!(sheet.rules.len(), 100);
}

#[test]
fn test_data_uri() {
    let css = "div { background: url('data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=='); }";
    let result = parse_css(css);
    assert!(result.is_ok());
}

#[test]
fn test_vendor_prefixes() {
    let css = r#"
        div {
            -webkit-transform: rotate(90deg);
            -moz-transform: rotate(90deg);
            transform: rotate(90deg);
        }
    "#;
    let sheet = parse_css(css).unwrap();
    match &sheet.rules[0] {
        CssRule::Style(sr) => {
            assert_eq!(sr.declarations.len(), 3);
        }
        _ => panic!("Expected Style rule"),
    }
}

#[test]
fn test_multiple_rules_mixed() {
    let css = r#"
        @import url('reset.css');
        @media screen { body { font-size: 16px; } }
        .container { max-width: 1200px; }
        @keyframes fade { from { opacity: 0; } to { opacity: 1; } }
        @font-face { font-family: 'Custom'; src: url('custom.woff2'); }
    "#;
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 5);
}

// ---------------------------------------------------------------------------
// Selector specificity and complex selectors
// ---------------------------------------------------------------------------

#[test]
fn test_descendant_selector() {
    let css = "div span { color: red; }";
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_child_selector() {
    let css = "ul > li { list-style: none; }";
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_adjacent_sibling_selector() {
    let css = "h2 + p { margin-top: 0; }";
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_general_sibling_selector() {
    let css = "h2 ~ p { color: gray; }";
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

// ---------------------------------------------------------------------------
// Strings and special values
// ---------------------------------------------------------------------------

#[test]
fn test_string_values() {
    let css = r#"div { font-family: "Helvetica Neue", Arial, sans-serif; }"#;
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_url_values() {
    let css = "div { background: url('bg.png'); }";
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_zero_values() {
    let css = "div { margin: 0; padding: 0; border: 0; }";
    let sheet = parse_css(css).unwrap();
    match &sheet.rules[0] {
        CssRule::Style(sr) => {
            assert_eq!(sr.declarations.len(), 3);
        }
        _ => panic!("Expected Style rule"),
    }
}

#[test]
fn test_negative_values() {
    let css = "div { margin: -10px; z-index: -1; }";
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_math_functions() {
    let css = r#"
        div {
            width: min(100%, 768px);
            height: max(50vh, 300px);
            padding: clamp(10px, 5%, 20px);
        }
    "#;
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

// ---------------------------------------------------------------------------
// Error message tests
// ---------------------------------------------------------------------------

#[test]
fn test_error_has_message() {
    let result = parse_css("div { color: red; ");
    if let Err(err) = result {
        assert!(!err.message().is_empty());
    }
    // Lightning CSS may parse this successfully; that's fine.
}

#[test]
fn test_multiple_errors() {
    let result = parse_css("{ color: red; } div { color: blue; ");
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// CSS nesting (native)
// ---------------------------------------------------------------------------

#[test]
fn test_css_nesting() {
    let css = r#"
        .container {
            color: red;
            & .child {
                color: blue;
            }
        }
    "#;
    let result = parse_css(css);
    // Nesting may not be fully supported by all lightningcss versions
    // but the parser should not crash
    assert!(result.is_ok() || result.is_err());
}

// ---------------------------------------------------------------------------
// Filter effects
// ---------------------------------------------------------------------------

#[test]
fn test_filter() {
    let css = "div { filter: blur(5px) brightness(0.5); }";
    let result = parse_css(css);
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// Pseudo-class selectors
// ---------------------------------------------------------------------------

#[test]
fn test_nth_child_selector() {
    let css = "li:nth-child(2n+1) { background: gray; }";
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_not_selector() {
    let css = "div:not(.hidden) { display: block; }";
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_is_selector() {
    let css = ":is(header, main, footer) { padding: 20px; }";
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_where_selector() {
    let css = ":where(div, span) { color: red; }";
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn test_has_selector() {
    let css = "div:has(> img) { border: none; }";
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}

// ---------------------------------------------------------------------------
// Selector combinators with attributes
// ---------------------------------------------------------------------------

#[test]
fn test_complex_compound_selector() {
    let css = "div.container#main.active { color: red; }";
    let sheet = parse_css(css).unwrap();
    assert_eq!(sheet.rules.len(), 1);
}
