use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use motarjim_lexer::css::CssTokenizer;
use motarjim_lexer::html::HtmlTokenizer;

const HTML_SMALL: &str = "<div>hello</div>";
const HTML_MEDIUM: &str = r#"<html><head><title>Test Page</title></head><body><div class="container"><h1>Welcome</h1><p>This is a medium HTML snippet with <strong>nested</strong> elements and attributes.</p></div></body></html>"#;
const HTML_LARGE: &str = r##"<!DOCTYPE html>
<html lang="en">
<head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>Large Test Page</title></head>
<body>
<header><nav><a href="/">Home</a><a href="/about">About</a><a href="/contact">Contact</a><a href="/blog">Blog</a><a href="/careers">Careers</a></nav></header>
<main>
<section class="hero"><h1>Welcome to Our Site</h1><p>This is a large HTML document for benchmarking the tokenizer performance under load.</p></section>
<section class="features"><div class="feature"><h2>Feature One</h2><p>Description of feature one with <em>emphasized</em> text.</p></div><div class="feature"><h2>Feature Two</h2><p>Description of feature two with <strong>strong</strong> text.</p></div><div class="feature"><h2>Feature Three</h2><p>Description of feature three with <a href="#">a link</a>.</p></div><div class="feature"><h2>Feature Four</h2><p>Description of feature four.</p></div><div class="feature"><h2>Feature Five</h2><p>Description of feature five.</p></div></section>
<section class="content"><article><h2>Article Title</h2><p>Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.</p><p>Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.</p></article></section>
<section class="gallery"><div class="card"><img src="img1.jpg" alt="Image 1"/><p>Caption 1</p></div><div class="card"><img src="img2.jpg" alt="Image 2"/><p>Caption 2</p></div><div class="card"><img src="img3.jpg" alt="Image 3"/><p>Caption 3</p></div><div class="card"><img src="img4.jpg" alt="Image 4"/><p>Caption 4</p></div><div class="card"><img src="img5.jpg" alt="Image 5"/><p>Caption 5</p></div></section>
</main>
<footer><p>&copy; 2024 Test Company. All rights reserved.</p><nav><a href="/privacy">Privacy</a><a href="/terms">Terms</a></nav></footer>
</body>
</html>"##;

const CSS_SMALL: &str = "div { color: red; }";
const CSS_MEDIUM: &str = r#"
.container { display: flex; flex-direction: row; justify-content: center; align-items: center; padding: 20px; margin: 10px; background-color: #ffffff; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
.header { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 40px 20px; text-align: center; }
.nav { display: flex; gap: 20px; list-style: none; padding: 0; margin: 0; }
.nav a { text-decoration: none; color: #333; font-weight: 500; transition: color 0.3s ease; }
.nav a:hover { color: #667eea; }
.footer { background: #f5f5f5; padding: 20px; text-align: center; font-size: 14px; color: #666; }
"#;

fn bench_html_tokenize_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("html_tokenize");
    group.throughput(Throughput::Bytes(HTML_SMALL.len() as u64));
    group.bench_function("small", |b| {
        b.iter(|| {
            let mut tokenizer = HtmlTokenizer::new(black_box(HTML_SMALL));
            tokenizer.tokenize()
        });
    });
    group.finish();
}

fn bench_html_tokenize_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("html_tokenize");
    group.throughput(Throughput::Bytes(HTML_MEDIUM.len() as u64));
    group.bench_function("medium", |b| {
        b.iter(|| {
            let mut tokenizer = HtmlTokenizer::new(black_box(HTML_MEDIUM));
            tokenizer.tokenize()
        });
    });
    group.finish();
}

fn bench_html_tokenize_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("html_tokenize");
    group.throughput(Throughput::Bytes(HTML_LARGE.len() as u64));
    group.bench_function("large", |b| {
        b.iter(|| {
            let mut tokenizer = HtmlTokenizer::new(black_box(HTML_LARGE));
            tokenizer.tokenize()
        });
    });
    group.finish();
}

fn bench_css_tokenize_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("css_tokenize");
    group.throughput(Throughput::Bytes(CSS_SMALL.len() as u64));
    group.bench_function("small", |b| {
        b.iter(|| {
            let mut tokenizer = CssTokenizer::new(black_box(CSS_SMALL));
            tokenizer.tokenize()
        });
    });
    group.finish();
}

fn bench_css_tokenize_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("css_tokenize");
    group.throughput(Throughput::Bytes(CSS_MEDIUM.len() as u64));
    group.bench_function("medium", |b| {
        b.iter(|| {
            let mut tokenizer = CssTokenizer::new(black_box(CSS_MEDIUM));
            tokenizer.tokenize()
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_html_tokenize_small,
    bench_html_tokenize_medium,
    bench_html_tokenize_large,
    bench_css_tokenize_small,
    bench_css_tokenize_medium
);
criterion_main!(benches);
