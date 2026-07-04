use motarjim_diag::codes;
use motarjim_diag::{Diagnostic, DiagnosticBag, DiagnosticCode, Severity};

fn main() {
    let mut bag = DiagnosticBag::new();

    bag.push(Diagnostic::new(
        Severity::Error,
        DiagnosticCode::new(1, "TEST"),
        "a manual error",
    ));

    bag.push_error(codes::PARSER_UNEXPECTED_TOKEN, "unexpected '<'");
    bag.push_warning(codes::A11Y_MISSING_ALT, "image missing alt text");
    bag.push_info(codes::CONFIG_FILE_NOT_FOUND, "using default config");

    println!("Collected {} diagnostic(s)", bag.len());
    println!("Has errors? {}", bag.has_errors());
    println!("Has warnings? {}", bag.has_warnings());

    for diag in bag.iter() {
        println!(
            "  [{}] {:?}: {}",
            diag.severity().as_str(),
            diag.code(),
            diag.message()
        );
    }
}
