#![allow(clippy::missing_docs_in_private_items)]
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use motarjim_parser::{CssParser, HtmlParser};

const HTML_SMALL: &str = "<div>hello world</div>";
const HTML_MEDIUM: &str = r##"<div class="container" id="main"><header class="hero"><h1>Welcome</h1><p class="subtitle">This is a medium HTML document with <strong>nested</strong> elements and attributes.</p></header><section class="content"><article><h2>Article</h2><p>Some paragraph text here with <a href="#">a link</a> and <em>emphasis</em>.</p></article></section></div>"##;
const HTML_LARGE: &str = r#"<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><title>Test</title></head><body><header><nav><a href="/">Home</a><a href="/about">About</a><a href="/contact">Contact</a></nav></header><main><section class="hero"><h1>Welcome</h1><p>Hero description text.</p></section><section class="features"><div class="feature"><h2>Feature 1</h2><p>Desc 1</p></div><div class="feature"><h2>Feature 2</h2><p>Desc 2</p></div><div class="feature"><h2>Feature 3</h2><p>Desc 3</p></div></section><section class="gallery"><div class="card"><img src="a.jpg" alt="A"/><p>A</p></div><div class="card"><img src="b.jpg" alt="B"/><p>B</p></div><div class="card"><img src="c.jpg" alt="C"/><p>C</p></div></section><footer><p>&copy; 2024</p></footer></main></body></html>"#;

const CSS_SMALL: &str = "div { color: red; font-size: 16px; }";
const CSS_LARGE: &str = r"
.container { display: flex; flex-wrap: wrap; justify-content: space-between; align-items: center; padding: 20px; margin: 0 auto; max-width: 1200px; background: #fff; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); }
@media screen and (max-width: 768px) { .container { flex-direction: column; padding: 10px; } .nav { display: none; } .mobile-menu { display: block; } }
@media screen and (min-width: 769px) and (max-width: 1024px) { .container { padding: 15px; } .sidebar { width: 250px; } }
.header { background: linear-gradient(135deg, #667eea, #764ba2); color: white; padding: 60px 20px; text-align: center; }
.header h1 { font-size: 48px; font-weight: bold; margin-bottom: 10px; }
.header p { font-size: 18px; opacity: 0.9; }
.nav { display: flex; gap: 24px; list-style: none; padding: 0; margin: 20px 0; justify-content: center; }
.nav a { color: #333; text-decoration: none; font-weight: 500; padding: 8px 16px; border-radius: 4px; transition: all 0.3s ease; }
.nav a:hover { background: #667eea; color: white; }
.footer { background: #2d3748; color: #a0aec0; padding: 40px 20px; text-align: center; font-size: 14px; }
.footer a { color: #63b3ed; text-decoration: none; }
.card { border: 1px solid #e2e8f0; border-radius: 12px; padding: 24px; margin: 16px 0; transition: transform 0.2s ease; }
.card:hover { transform: translateY(-4px); box-shadow: 0 4px 12px rgba(0,0,0,0.15); }
.btn { display: inline-block; padding: 12px 24px; border-radius: 6px; font-weight: 600; text-decoration: none; cursor: pointer; }
.btn-primary { background: #667eea; color: white; }
.btn-secondary { background: #edf2f7; color: #2d3748; }
@keyframes fadeIn { from { opacity: 0; transform: translateY(20px); } to { opacity: 1; transform: translateY(0); } }
@supports (display: grid) { .grid-layout { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 24px; } }
";

fn bench_parse_small_html(c: &mut Criterion) {
    c.bench_function("parse_html_small", |b| {
        b.iter(|| {
            let mut parser = HtmlParser::new(black_box(HTML_SMALL));
            parser.parse()
        });
    });
}

fn bench_parse_medium_html(c: &mut Criterion) {
    c.bench_function("parse_html_medium", |b| {
        b.iter(|| {
            let mut parser = HtmlParser::new(black_box(HTML_MEDIUM));
            parser.parse()
        });
    });
}

fn bench_parse_large_html(c: &mut Criterion) {
    c.bench_function("parse_html_large", |b| {
        b.iter(|| {
            let mut parser = HtmlParser::new(black_box(HTML_LARGE));
            parser.parse()
        });
    });
}

fn bench_parse_css_small(c: &mut Criterion) {
    c.bench_function("parse_css_small", |b| {
        b.iter(|| {
            let parser = CssParser::new(black_box(CSS_SMALL));
            parser.parse()
        });
    });
}

fn bench_parse_css_large(c: &mut Criterion) {
    c.bench_function("parse_css_large", |b| {
        b.iter(|| {
            let parser = CssParser::new(black_box(CSS_LARGE));
            parser.parse()
        });
    });
}

criterion_group!(
    benches,
    bench_parse_small_html,
    bench_parse_medium_html,
    bench_parse_large_html,
    bench_parse_css_small,
    bench_parse_css_large
);
criterion_main!(benches);
