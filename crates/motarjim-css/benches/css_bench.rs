use criterion::{black_box, criterion_group, criterion_main, Criterion};
use motarjim_ast::css::Declaration;
use motarjim_css::{parse_color, parse_length, Cascade};
use smol_str::SmolStr;

fn bench_cascade_small(c: &mut Criterion) {
    let mut cascade = Cascade::new();
    let declarations = vec![
        Declaration {
            property: SmolStr::new("color"),
            value: "red".into(),
            important: false,
            parsed: None,
            span: None,
        },
        Declaration {
            property: SmolStr::new("font-size"),
            value: "16px".into(),
            important: false,
            parsed: None,
            span: None,
        },
    ];
    cascade.add_declarations(&declarations, (0, 0, 1));

    c.bench_function("cascade_small", |b| {
        b.iter(|| {
            let c = black_box(&cascade);
            c.resolve()
        });
    });
}

fn bench_cascade_medium(c: &mut Criterion) {
    let mut cascade = Cascade::new();
    cascade.add_declarations(
        &[
            Declaration {
                property: SmolStr::new("color"),
                value: "red".into(),
                important: false,
                parsed: None,
                span: None,
            },
            Declaration {
                property: SmolStr::new("font-size"),
                value: "14px".into(),
                important: false,
                parsed: None,
                span: None,
            },
        ],
        (0, 0, 1),
    );
    cascade.add_declarations(
        &[
            Declaration {
                property: SmolStr::new("color"),
                value: "blue".into(),
                important: false,
                parsed: None,
                span: None,
            },
            Declaration {
                property: SmolStr::new("font-weight"),
                value: "bold".into(),
                important: false,
                parsed: None,
                span: None,
            },
        ],
        (0, 0, 1),
    );
    cascade.add_declarations(
        &[
            Declaration {
                property: SmolStr::new("margin"),
                value: "10px".into(),
                important: false,
                parsed: None,
                span: None,
            },
            Declaration {
                property: SmolStr::new("padding"),
                value: "5px".into(),
                important: false,
                parsed: None,
                span: None,
            },
            Declaration {
                property: SmolStr::new("color"),
                value: "green".into(),
                important: true,
                parsed: None,
                span: None,
            },
        ],
        (1, 0, 0),
    );

    c.bench_function("cascade_medium", |b| {
        b.iter(|| {
            let c = black_box(&cascade);
            c.resolve()
        });
    });
}

fn bench_css_value_parsing(c: &mut Criterion) {
    let colors = &[
        "#ff0000",
        "#00ff00",
        "#0000ff",
        "#fff",
        "#123456",
        "red",
        "blue",
        "green",
        "rgb(255, 0, 0)",
        "rgba(0, 255, 0, 0.5)",
        "transparent",
        "currentcolor",
        "#aabbccdd",
        "navy",
        "coral",
        "tomato",
        "mediumseagreen",
        "rebeccapurple",
    ];
    let lengths = &[
        "10px", "1.5em", "2rem", "50%", "100vw", "100vh", "0", "3.14", "auto", "inherit", "12px",
        "0.5px",
    ];

    c.bench_function("parse_css_colors", |b| {
        b.iter(|| {
            for color in colors {
                black_box(parse_color(black_box(color)));
            }
        });
    });

    c.bench_function("parse_css_lengths", |b| {
        b.iter(|| {
            for length in lengths {
                black_box(parse_length(black_box(length)));
            }
        });
    });

    c.bench_function("parse_css_values_mixed", |b| {
        b.iter(|| {
            for c in colors {
                black_box(parse_color(black_box(c)));
            }
            for l in lengths {
                black_box(parse_length(black_box(l)));
            }
        });
    });
}

criterion_group!(
    benches,
    bench_cascade_small,
    bench_cascade_medium,
    bench_css_value_parsing
);
criterion_main!(benches);
