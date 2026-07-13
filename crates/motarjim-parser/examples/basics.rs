use motarjim_parser::CssParser;

fn main() {
    let css = r".intro { color: blue; font-weight: bold; }";
    let css_parser = CssParser::new(css);
    match css_parser.parse() {
        Ok(sheet) => {
            println!("CSS parsed successfully — {} rule(s)", sheet.rules.len());
        }
        Err(err) => {
            for d in err.diagnostics() {
                eprintln!("CSS error: {:?} — {}", d.code(), d.message());
            }
        }
    }
}
