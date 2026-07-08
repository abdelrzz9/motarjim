use motarjim_ast::css::{CssRule, CssStylesheet, Declaration, StyleRule};
use motarjim_ast::Element;
use motarjim_ast_css::{Selector, SimpleSelector};
use motarjim_css::{parse_color, parse_length, StyleResolver};
use smol_str::SmolStr;

fn make_stylesheet() -> CssStylesheet {
    CssStylesheet {
        rules: vec![
            CssRule::Style(StyleRule {
                selectors: vec![Selector {
                    simple_selectors: vec![SimpleSelector::Type(SmolStr::new_inline("div"))],
                    combinators: vec![],
                    span: None,
                }],
                declarations: smallvec::smallvec![
                    Declaration {
                        property: SmolStr::new_inline("color"),
                        value: "red".into(),
                        important: false,
                        parsed: None,
                        span: None,
                    },
                    Declaration {
                        property: SmolStr::new_inline("font-size"),
                        value: "16px".into(),
                        important: false,
                        parsed: None,
                        span: None,
                    },
                ],
                span: None,
            }),
            CssRule::Style(StyleRule {
                selectors: vec![Selector {
                    simple_selectors: vec![SimpleSelector::Class(SmolStr::new_inline("highlight"))],
                    combinators: vec![],
                    span: None,
                }],
                declarations: smallvec::smallvec![Declaration {
                    property: SmolStr::new_inline("background"),
                    value: "yellow".into(),
                    important: false,
                    parsed: None,
                    span: None,
                }],
                span: None,
            }),
        ],
        source_path: None,
    }
}

fn main() {
    let mut resolver = StyleResolver::new();
    resolver.add_stylesheet(make_stylesheet());

    let el = Element::new("div");
    let values = resolver.resolve(&el);
    println!("Resolved style for <div>:");
    println!("  color:       {:?}", values.style.color);
    println!("  font-size:   {:?}", values.style.font_size);
    println!("  background:  {:?}", values.style.background);

    let color = parse_color("#3366cc").unwrap();
    let length = parse_length("1.5rem").unwrap();
    println!("\nParsed color:   {:?}", color);
    println!("Parsed length:  {:?}", length);
}
