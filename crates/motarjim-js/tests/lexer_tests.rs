use motarjim_js::{JsLexer, JsTokenKind};

fn kinds(src: &str) -> Vec<JsTokenKind> {
    JsLexer::new(src)
        .tokenize()
        .iter()
        .map(|t| t.kind)
        .collect()
}

#[test]
fn test_var_decl() {
    let k = kinds("let x = 1;");
    assert_eq!(
        k,
        vec![
            JsTokenKind::Let,
            JsTokenKind::Identifier,
            JsTokenKind::Assign,
            JsTokenKind::Number,
            JsTokenKind::Semicolon,
            JsTokenKind::Eof,
        ]
    );
}

#[test]
fn test_arrow_function() {
    let k = kinds("(x) => x + 1");
    assert!(k.contains(&JsTokenKind::Arrow));
}

#[test]
fn test_string_literal() {
    let mut lexer = JsLexer::new(r#"'hello'"#);
    let tokens = lexer.tokenize();
    assert_eq!(tokens[0].kind, JsTokenKind::String);
}

#[test]
fn test_template_literal() {
    let mut lexer = JsLexer::new("`hi ${name}!`");
    let tokens = lexer.tokenize();
    assert_eq!(tokens[0].kind, JsTokenKind::TemplateString);
}

#[test]
fn test_template_nested() {
    let mut lexer = JsLexer::new("`a${`b${c}`}d`");
    let tokens = lexer.tokenize();
    assert_eq!(tokens[0].kind, JsTokenKind::TemplateString);
}

#[test]
fn test_operators() {
    let k = kinds("a === b !== c <= d >= e && f || g ?? h");
    assert!(k.contains(&JsTokenKind::EqEqEq));
    assert!(k.contains(&JsTokenKind::NotEqEq));
    assert!(k.contains(&JsTokenKind::LtEq));
    assert!(k.contains(&JsTokenKind::GtEq));
    assert!(k.contains(&JsTokenKind::AmpAmp));
    assert!(k.contains(&JsTokenKind::PipePipe));
    assert!(k.contains(&JsTokenKind::Nullish));
}

#[test]
fn test_increment_decrement() {
    let k = kinds("i++; --j;");
    assert!(k.contains(&JsTokenKind::Increment));
    assert!(k.contains(&JsTokenKind::Decrement));
}

#[test]
fn test_line_comment_skipped() {
    let k = kinds("let x = 1; // comment\nlet y = 2;");
    assert_eq!(k.iter().filter(|t| **t == JsTokenKind::Let).count(), 2);
}

#[test]
fn test_block_comment_skipped() {
    let k = kinds("let /* comment */ x = 1;");
    assert_eq!(k[0], JsTokenKind::Let);
    assert_eq!(k[1], JsTokenKind::Identifier);
}

#[test]
fn test_hex_number() {
    let mut lexer = JsLexer::new("0xFF");
    let tokens = lexer.tokenize();
    assert_eq!(tokens[0].kind, JsTokenKind::Number);
}

#[test]
fn test_octal_number() {
    let mut lexer = JsLexer::new("0o77");
    let tokens = lexer.tokenize();
    assert_eq!(tokens[0].kind, JsTokenKind::Number);
}

#[test]
fn test_binary_number() {
    let mut lexer = JsLexer::new("0b1010");
    let tokens = lexer.tokenize();
    assert_eq!(tokens[0].kind, JsTokenKind::Number);
}

#[test]
fn test_bigint() {
    let mut lexer = JsLexer::new("123n");
    let tokens = lexer.tokenize();
    assert_eq!(tokens[0].kind, JsTokenKind::BigInt);
}

#[test]
fn test_float_with_exponent() {
    let mut lexer = JsLexer::new("1.5e10");
    let tokens = lexer.tokenize();
    assert_eq!(tokens[0].kind, JsTokenKind::Number);
}

#[test]
fn test_numeric_separator() {
    let mut lexer = JsLexer::new("1_000_000");
    let tokens = lexer.tokenize();
    assert_eq!(tokens[0].kind, JsTokenKind::Number);
}

#[test]
fn test_import_export_keywords() {
    let k = kinds("import x from 'mod'; export default x;");
    assert_eq!(
        k,
        vec![
            JsTokenKind::Import, JsTokenKind::Identifier,
            JsTokenKind::From, JsTokenKind::String,
            JsTokenKind::Semicolon, JsTokenKind::Export,
            JsTokenKind::Default, JsTokenKind::Identifier,
            JsTokenKind::Semicolon, JsTokenKind::Eof,
        ]
    );
}

#[test]
fn test_dollar_identifier() {
    let k = kinds("$el.addEventListener");
    assert_eq!(k[0], JsTokenKind::Identifier);
}

#[test]
fn test_private_identifier() {
    let k = kinds("#foo");
    assert_eq!(k[0], JsTokenKind::PrivateIdentifier);
}

#[test]
fn test_optional_chaining() {
    let k = kinds("a?.b");
    assert!(k.contains(&JsTokenKind::QuestionDot));
}

#[test]
fn test_nullish_assignment() {
    let k = kinds("x ??= y");
    assert!(k.contains(&JsTokenKind::NullishAssign));
}

#[test]
fn test_logical_and_assignment() {
    let k = kinds("x &&= y");
    assert!(k.contains(&JsTokenKind::AmpAmpAssign));
}

#[test]
fn test_logical_or_assignment() {
    let k = kinds("x ||= y");
    assert!(k.contains(&JsTokenKind::PipePipeAssign));
}

#[test]
fn test_exponentiation() {
    let k = kinds("x ** y");
    assert!(k.contains(&JsTokenKind::StarStar));
}

#[test]
fn test_exponentiation_assignment() {
    let k = kinds("x **= y");
    assert!(k.contains(&JsTokenKind::StarStarAssign));
}

#[test]
fn test_spread() {
    let k = kinds("...x");
    assert!(k.contains(&JsTokenKind::Ellipsis));
}

#[test]
fn test_async_await_keywords() {
    let k = kinds("async function foo() { await bar(); }");
    assert!(k.contains(&JsTokenKind::Async));
    assert!(k.contains(&JsTokenKind::Await));
}

#[test]
fn test_class_keywords() {
    let k = kinds("class Foo extends Bar {}");
    assert!(k.contains(&JsTokenKind::Class));
    assert!(k.contains(&JsTokenKind::Extends));
}

#[test]
fn test_switch_case_default() {
    let k = kinds("switch(x) { case 1: break; default: break; }");
    assert!(k.contains(&JsTokenKind::Switch));
    assert!(k.contains(&JsTokenKind::Case));
    assert!(k.contains(&JsTokenKind::Default));
}

#[test]
fn test_try_catch_finally() {
    let k = kinds("try {} catch(e) {} finally {}");
    assert!(k.contains(&JsTokenKind::Try));
    assert!(k.contains(&JsTokenKind::Catch));
    assert!(k.contains(&JsTokenKind::Finally));
}

#[test]
fn test_throw() {
    let k = kinds("throw new Error('msg');");
    assert!(k.contains(&JsTokenKind::Throw));
}

#[test]
fn test_empty_input() {
    let mut lexer = JsLexer::new("");
    let tokens = lexer.tokenize();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, JsTokenKind::Eof);
}

#[test]
fn test_only_whitespace() {
    let mut lexer = JsLexer::new("   \t\n  ");
    let tokens = lexer.tokenize();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, JsTokenKind::Eof);
}

#[test]
fn test_unicode_identifier() {
    let k = kinds("café = 1;");
    assert_eq!(k[0], JsTokenKind::Identifier);
}

#[test]
fn test_template_with_unicode_escape() {
    let mut lexer = JsLexer::new(r#"`\u0041`"#);
    let tokens = lexer.tokenize();
    assert_eq!(tokens[0].kind, JsTokenKind::TemplateString);
}
