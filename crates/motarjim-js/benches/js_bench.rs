use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use motarjim_js::JsLexer;
use motarjim_js::JsParser;

const JS_SMALL: &str = "let x = 1;";

const JS_MEDIUM: &str = r#"
function renderCard(item) {
    const el = document.createElement('div');
    el.className = 'card';
    el.innerHTML = `<h2>${item.title}</h2><p>${item.body}</p>`;
    el.addEventListener('click', () => select(item));
    return el;
}

const items = [1, 2, 3].map(id => ({ id, title: `Item ${id}`, body: 'lorem ipsum' }));
for (const item of items) {
    container.appendChild(renderCard(item));
}
"#;

fn bench_lex_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("js_lex");
    group.throughput(Throughput::Bytes(JS_SMALL.len() as u64));
    group.bench_function("small", |b| {
        b.iter(|| {
            let mut lexer = JsLexer::new(black_box(JS_SMALL));
            lexer.tokenize()
        });
    });
    group.finish();
}

fn bench_lex_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("js_lex");
    group.throughput(Throughput::Bytes(JS_MEDIUM.len() as u64));
    group.bench_function("medium", |b| {
        b.iter(|| {
            let mut lexer = JsLexer::new(black_box(JS_MEDIUM));
            lexer.tokenize()
        });
    });
    group.finish();
}

fn bench_parse_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("js_parse");
    group.throughput(Throughput::Bytes(JS_MEDIUM.len() as u64));
    group.bench_function("medium", |b| {
        b.iter(|| {
            let mut parser = JsParser::new(black_box(JS_MEDIUM));
            parser.parse()
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_lex_small,
    bench_lex_medium,
    bench_parse_medium
);
criterion_main!(benches);
