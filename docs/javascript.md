# JavaScript Support

## Purpose

`motarjim-js` is Motarjim's JavaScript front end: lexer, AST, parser,
visitor, semantic analysis, DOM event extraction, and AST-to-AST transforms.
It is the newest crate in the workspace and is intentionally standalone
(everything lives in one crate) rather than split across several crates like
HTML/CSS — that split is a natural follow-up once it outgrows the crate size
guidance in `ARCHITECTURE_REVIEW.md`.

**Crate:** `crates/motarjim-js`
**Dependencies:** `motarjim-diag` (spans, diagnostics), `motarjim-lexer`
(shared `Cursor`/`Token` types) — no other crate in the workspace depends on
`motarjim-js` yet.

## Pipeline

```
source text ──▶ JsLexer ──▶ Vec<Token<JsTokenKind>> ──▶ JsParser ──▶ Program (AST)
                                                                        │
                                    ┌───────────────────────────────────┼───────────────────────────────┐
                                    ▼                                   ▼                                ▼
                          SemanticAnalyzer::analyze          find_dom_event_bindings          Transform::apply (e.g. TemplateLiteralToConcat)
                          (Vec<Diagnostic>)                   (Vec<DomEventBinding>)           (rewrites the Program in place)
```

## Supported syntax

- `var`/`let`/`const` declarations, including multiple comma-separated declarators
- Named and anonymous function declarations/expressions
- Arrow functions — single bare-identifier params (`x => ...`), parenthesized
  param lists, concise expression bodies and block bodies
- Template literals, including nested interpolations (`` `${`${x}`}` ``) and
  interpolations containing string/brace characters
- `if`/`else`, `for` (C-style, and `for...of`/`for...in` in declaration form:
  `for (const x of xs)`), `while`, `do...while`, `break`/`continue`
- `import` (default, named with `as`, namespace) and `export` (default,
  named declaration, `export { a, b }`)
- The common expression grammar: arithmetic/comparison/logical/assignment
  operators, ternary, member access (`.` and `[]`), calls, `new`,
  array/object literals (with shorthand properties), comma sequences

## Known gaps

These are deliberate scope cuts for the first version, not oversights:

- No automatic semicolon insertion — semicolons are simply optional everywhere
- No destructuring, spread/rest parameters, classes, generators, or `async`/`await`
- No regular expression literals
- Bare (non-declaration) `for (x of xs)` / `for (x in xs)` loops are not
  supported; use `for (const x of xs)` instead
- Scoping in `SemanticAnalyzer` approximates every block as its own scope for
  `var` as well as `let`/`const`. Real JavaScript hoists `var` to the nearest
  enclosing function, so a `var` declared in one block and read after that
  block (but still in the same function) may produce a spurious "undeclared
  variable" warning. This check is a warning, not an error, so it never
  blocks anything — it's an editor hint, not a gate.

## Error recovery

Every parse function that can fail still makes forward progress: an
unexpected token is reported as an `E07xx` diagnostic and skipped, so one
syntax error does not prevent the rest of the file from being parsed and
analyzed. See `motarjim-diag`'s code table for the `700-799` JavaScript range.

## Diagnostics

| Code | Severity | Meaning |
|------|----------|---------|
| E0700 | Error | Unexpected token |
| E0701 | Error | Unexpected end of input |
| E0710 | Error | Duplicate `let`/`const` declaration in the same scope |
| E0711 | Warning | Reference to a name with no matching declaration (small allowlist of browser/JS globals excluded) |
| E0712 | Error | Assignment to a `const` binding |

## DOM event extraction

`find_dom_event_bindings(&program)` walks the AST and recognizes the two
idiomatic ways JavaScript wires a handler to an element:

```js
button.addEventListener("click", handler);
form.onsubmit = handleSubmit;
```

This is extraction only — nothing downstream consumes `DomEventBinding`s yet.
The intended future use is feeding a `click` listener's handler into the
HTML/CSS → native UI pipeline so a generated `<button>` gets a real
`onPressed`/`onClick`/`.onTapGesture` wired to the same logic, instead of the
empty callback the generators emit today.

## CLI integration

`motarjim check <file>.js` (also `.mjs`, `.jsx`) parses the file with
`JsParser` and runs `SemanticAnalyzer` over the result, printing diagnostics
the same way HTML/CSS `check` does. It does not go through the HTML/CSS
compiler pipeline (`motarjim-core::Compiler`) since there is no JS→IR bridge
yet.

## Example

```rust
use motarjim_js::{find_dom_event_bindings, JsParser, SemanticAnalyzer};

let source = r#"
    const button = document.getElementById("go");
    button.addEventListener("click", () => {
        console.log(`Clicked ${count} times`);
    });
"#;

let mut parser = JsParser::new(source);
let program = parser.parse().expect("valid syntax");

let diagnostics = SemanticAnalyzer::new().analyze(&program);
let bindings = find_dom_event_bindings(&program);
```

See `crates/motarjim-js/examples/basics.rs` for a runnable version
(`cargo run --example basics -p motarjim-js`).
