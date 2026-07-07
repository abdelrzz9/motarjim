use motarjim_parser::{CssParser, HtmlParser};

fn main() {
    let html = r#"<!DOCTYPE html>
<html>
  <body>
    <h1 id="title">Hello</h1>
    <p class="intro">World</p>
  </body>
</html>"#;

    let mut parser = HtmlParser::new(html);
    match parser.parse() {
        Ok(doc) => {
            println!("HTML parsed successfully — {} node(s)", doc.nodes.len());
            for node in &doc.nodes {
                let tag = node.element.as_ref().map_or("?", |e| e.tag_name.as_str());
                println!("  [{:?}] <{}>", node.id, tag);
            }
        }
        Err(diags) => {
            for d in &diags {
                eprintln!("Parse error: {:?} — {}", d.code(), d.message());
            }
        }
    }

    let css = r".intro { color: blue; font-weight: bold; }";
    let css_parser = CssParser::new(css);
    match css_parser.parse() {
        Ok(sheet) => {
            println!("\nCSS parsed successfully — {} rule(s)", sheet.rules.len());
        }
        Err(err) => {
            for d in err.diagnostics() {
                eprintln!("CSS error: {:?} — {}", d.code(), d.message());
            }
        }
    }
}
