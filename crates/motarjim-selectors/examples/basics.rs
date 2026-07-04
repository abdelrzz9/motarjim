use motarjim_selectors::{parse_selector, Specificity};

fn main() {
    let inputs = vec![
        "div",
        ".container",
        "#main",
        "div.container",
        "div > p.highlight",
        "ul li.active",
        "a:hover",
    ];

    for input in inputs {
        match parse_selector(input) {
            Ok(selector) => {
                let spec = Specificity::of(&selector);
                println!(
                    "{:30} => specificity ({}, {}, {}) — {:?}",
                    input, spec.ids, spec.classes, spec.types, selector
                );
            }
            Err(e) => {
                eprintln!("Failed to parse {input:?}: {e}");
            }
        }
    }
}
