use motarjim_js::{find_dom_event_bindings, JsParser, SemanticAnalyzer};

fn main() {
    let source = r#"
        let count = 0;
        const button = document.getElementById("increment");
        button.addEventListener("click", () => {
            count = count + 1;
            label.textContent = `Clicked ${count} times`;
        });
    "#;

    let mut parser = JsParser::new(source);
    let program = match parser.parse() {
        Ok(program) => program,
        Err(diagnostics) => {
            for diag in &diagnostics {
                println!("error[JS{:04}]: {}", diag.code.number, diag.message);
            }
            return;
        }
    };
    println!("Parsed {} top-level statement(s)", program.body.len());

    let diagnostics = SemanticAnalyzer::new().analyze(&program);
    println!(
        "Semantic analysis found {} diagnostic(s)",
        diagnostics.len()
    );
    for diag in &diagnostics {
        println!("  [{}] {}", diag.severity.as_str(), diag.message);
    }

    let bindings = find_dom_event_bindings(&program);
    println!("Found {} DOM event binding(s)", bindings.len());
    for binding in &bindings {
        println!("  {} -> {}", binding.target, binding.event_name);
    }
}
