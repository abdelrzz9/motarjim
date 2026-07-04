use motarjim_lexer::css::CssTokenizer;
use motarjim_lexer::html::{HtmlTokenKind, HtmlTokenizer};

fn main() {
    let html = r#"<div class="greeting">Hello</div>"#;
    let mut tokenizer = HtmlTokenizer::new(html);
    let tokens = tokenizer.tokenize();
    println!("HTML tokens:");
    for t in &tokens {
        match t.kind {
            HtmlTokenKind::Eof => break,
            _ => println!("  {:?}: {:?}", t.kind, t.raw),
        }
    }

    let css = r#"div { color: red; font-size: 16px; }"#;
    let mut css_tokenizer = CssTokenizer::new(css);
    let css_tokens = css_tokenizer.tokenize();
    println!("\nCSS tokens:");
    for t in &css_tokens {
        println!("  {:?}: {:?}", t.kind, t.raw);
    }
}
