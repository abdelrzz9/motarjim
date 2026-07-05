use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use motarjim_js::{
    transform::{run_transforms, TemplateLiteralToConcat},
    JsLexer, JsParser, SemanticAnalyzer,
};

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

const JS_LARGE: &str = r#"
// Large synthetic benchmark
const data = [];
for (let i = 0; i < 100; i++) {
    data.push({
        id: i,
        name: `User ${i}`,
        email: `user${i}@example.com`,
        role: i % 2 === 0 ? 'admin' : 'user',
    });
}

async function fetchData(url) {
    try {
        const response = await fetch(url);
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        return await response.json();
    } catch (err) {
        console.error('Fetch failed:', err);
        return null;
    }
}

class DataManager {
    constructor(initial) {
        this._cache = initial ?? [];
        this._listeners = new Set();
    }

    addItem(item) {
        this._cache.push(item);
        this._notify('add', item);
    }

    removeItem(id) {
        const idx = this._cache.findIndex(x => x.id === id);
        if (idx !== -1) {
            const [item] = this._cache.splice(idx, 1);
            this._notify('remove', item);
        }
    }

    _notify(type, item) {
        for (const listener of this._listeners) {
            listener({ type, item });
        }
    }
}

function processItems(items, transform) {
    return items
        .filter(x => x.active)
        .map(x => ({ ...x, processed: transform(x.value) }))
        .reduce((acc, x) => { acc[x.key] = x; return acc; }, {});
}

const mgr = new DataManager();
mgr.addItem({ id: 1, active: true, value: 42 });
mgr.addItem({ id: 2, active: false, value: 7 });
const result = processItems(mgr._cache, v => v * 2);
console.log(result);
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

fn bench_lex_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("js_lex");
    group.throughput(Throughput::Bytes(JS_LARGE.len() as u64));
    group.bench_function("large", |b| {
        b.iter(|| {
            let mut lexer = JsLexer::new(black_box(JS_LARGE));
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

fn bench_parse_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("js_parse");
    group.throughput(Throughput::Bytes(JS_LARGE.len() as u64));
    group.bench_function("large", |b| {
        b.iter(|| {
            let mut parser = JsParser::new(black_box(JS_LARGE));
            parser.parse()
        });
    });
    group.finish();
}

fn bench_semantic_medium(c: &mut Criterion) {
    let mut parser = JsParser::new(JS_MEDIUM);
    let program = parser.parse().unwrap();
    let mut group = c.benchmark_group("js_semantic");
    group.bench_function("medium", |b| {
        b.iter(|| {
            let analyzer = SemanticAnalyzer::new();
            analyzer.analyze(black_box(&program))
        });
    });
    group.finish();
}

fn bench_transform_medium(c: &mut Criterion) {
    let mut parser = JsParser::new(JS_MEDIUM);
    let program = parser.parse().unwrap();
    let mut group = c.benchmark_group("js_transform");
    group.bench_function("template_to_concat", |b| {
        b.iter(|| {
            run_transforms(
                black_box(program.clone()),
                &mut [&mut TemplateLiteralToConcat],
            )
        });
    });
    group.finish();
}

fn bench_pipeline_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("js_pipeline");
    group.bench_function("full_pipeline", |b| {
        b.iter(|| {
            let mut parser = JsParser::new(black_box(JS_MEDIUM));
            let program = parser.parse().unwrap();
            let diags = SemanticAnalyzer::new().analyze(&program);
            let _transformed = run_transforms(program, &mut [&mut TemplateLiteralToConcat]);
            (program, diags)
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_lex_small,
    bench_lex_medium,
    bench_lex_large,
    bench_parse_medium,
    bench_parse_large,
    bench_semantic_medium,
    bench_transform_medium,
    bench_pipeline_medium
);
criterion_main!(benches);
