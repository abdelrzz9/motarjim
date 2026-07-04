use motarjim_js::{JsParser, events::find_dom_event_bindings};

#[test]
fn test_add_event_listener() {
    let mut parser = JsParser::new(
        "button.addEventListener('click', handler);"
    );
    let program = parser.parse().expect("should parse");
    let bindings = find_dom_event_bindings(&program);
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0].target, "button");
    assert_eq!(bindings[0].event_name, "click");
}

#[test]
fn test_onevent_assignment() {
    let mut parser = JsParser::new(
        "button.onclick = handler;"
    );
    let program = parser.parse().expect("should parse");
    let bindings = find_dom_event_bindings(&program);
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0].target, "button");
    assert_eq!(bindings[0].event_name, "click");
}

#[test]
fn test_multiple_bindings() {
    let mut parser = JsParser::new(
        "btn.addEventListener('click', onClick);
         btn.addEventListener('mouseover', onHover);
         btn.onsubmit = handleSubmit;"
    );
    let program = parser.parse().expect("should parse");
    let bindings = find_dom_event_bindings(&program);
    assert_eq!(bindings.len(), 3);
}

#[test]
fn test_no_bindings_in_clean_code() {
    let mut parser = JsParser::new("let x = 1; console.log(x);");
    let program = parser.parse().expect("should parse");
    let bindings = find_dom_event_bindings(&program);
    assert!(bindings.is_empty());
}

#[test]
fn test_add_event_listener_with_options() {
    let mut parser = JsParser::new(
        "el.addEventListener('scroll', handler, { passive: true });"
    );
    let program = parser.parse().expect("should parse");
    let bindings = find_dom_event_bindings(&program);
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0].event_name, "scroll");
}

#[test]
fn test_remove_event_listener_not_counted() {
    let mut parser = JsParser::new(
        "el.removeEventListener('click', handler);"
    );
    let program = parser.parse().expect("should parse");
    let bindings = find_dom_event_bindings(&program);
    assert!(bindings.is_empty());
}

#[test]
fn test_chained_member_target() {
    let mut parser = JsParser::new(
        "document.getElementById('btn').addEventListener('click', onClick);"
    );
    let program = parser.parse().expect("should parse");
    let bindings = find_dom_event_bindings(&program);
    assert_eq!(bindings.len(), 1);
    assert!(bindings[0].target.contains("getElementById"));
}
