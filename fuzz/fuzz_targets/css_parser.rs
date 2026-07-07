#![no_main]
use libfuzzer_sys::fuzz_target;
use motarjim_parser::CssParser;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let parser = CssParser::new(s);
        let _ = parser.parse();
    }
});
