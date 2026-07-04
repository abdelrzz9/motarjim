#![allow(dead_code)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::pedantic, clippy::nursery)]

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;

use serde::Deserialize;

// ─── Cargo.toml parsing ──────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CargoToml {
    package: Option<Package>,
    dependencies: Option<BTreeMap<String, Dependency>>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Package {
    name: String,
    description: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Dependency {
    Simple(String),
    Detailed {
        version: Option<String>,
        path: Option<String>,
        git: Option<String>,
        optional: Option<bool>,
    },
}

/// A dependency edge from one crate to another.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Edge {
    from: String,
    to: String,
}

// ─── Module hierarchy extraction ─────────────────────────────────────────────

#[derive(Debug, Clone)]
struct ModuleTree {
    name: String,
    children: Vec<Self>,
}

fn parse_module_declarations(content: &str) -> Vec<String> {
    let mut modules = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("pub ") {
            if let Some(mod_name) = rest.strip_prefix("mod ") {
                if let Some(name) = mod_name.strip_suffix(';') {
                    modules.push(name.trim().to_string());
                }
            }
        } else if let Some(rest) = trimmed.strip_prefix("pub(crate) ") {
            if let Some(mod_name) = rest.strip_prefix("mod ") {
                if let Some(name) = mod_name.strip_suffix(';') {
                    modules.push(name.trim().to_string());
                }
            }
        }
    }
    modules
}

fn extract_module_tree(crate_path: &Path) -> Option<ModuleTree> {
    let lib_rs = crate_path.join("src/lib.rs");
    if !lib_rs.exists() {
        return None;
    }
    let content = fs::read_to_string(&lib_rs).ok()?;
    let package_name = crate_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    let children = parse_module_declarations(&content)
        .into_iter()
        .map(|name| ModuleTree {
            name,
            children: Vec::new(),
        })
        .collect();
    Some(ModuleTree {
        name: package_name,
        children,
    })
}

// ─── Public API surface extraction ───────────────────────────────────────────

#[derive(Debug, Clone)]
struct PublicItem {
    kind: &'static str,
    name: String,
    module_path: String,
}

fn extract_public_items(content: &str) -> Vec<PublicItem> {
    let mut items = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        let prefixes = [
            "pub fn ",
            "pub struct ",
            "pub enum ",
            "pub trait ",
            "pub type ",
            "pub const ",
            "pub mod ",
        ];
        for prefix in &prefixes {
            if let Some(rest) = trimmed.strip_prefix(prefix) {
                if let Some(name) = rest.split_whitespace().next() {
                    if let Some(clean) = name.split('(').next() {
                        if let Some(clean) = clean.split('<').next() {
                            let kind = prefix.trim_end_matches(' ').trim_start_matches("pub ");
                            items.push(PublicItem {
                                kind,
                                name: clean.to_string(),
                                module_path: String::new(),
                            });
                        }
                    }
                }
            }
        }
    }
    items
}

// ─── Pass dependency extraction from optimizer ───────────────────────────────

fn extract_pass_dependencies(content: &str) -> Vec<(String, Vec<String>, Vec<String>)> {
    let mut passes = Vec::new();

    // Simple approach: find all `impl Pass for` blocks using line-based search,
    // then scan forward for `fn prerequisites` and `fn invalidated_by`,
    // and extract bracket content from the next non-comment lines.
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("impl Pass for ") {
            continue;
        }
        let pass_name = trimmed
            .strip_prefix("impl Pass for ")
            .and_then(|s| s.split_whitespace().next())
            .map(|s| s.trim_end_matches('{').trim().to_string());

        let Some(ref name) = pass_name else {
            continue;
        };

        // Extract the entire impl block body
        let marker = format!("impl Pass for {name}");
        let block_start = content.find(&marker);
        let Some(start_pos) = block_start else {
            passes.push((name.clone(), Vec::new(), Vec::new()));
            continue;
        };

        let rest = &content[start_pos..];
        // Find the opening brace of the impl block
        let Some(brace_pos) = rest.find('{') else {
            passes.push((name.clone(), Vec::new(), Vec::new()));
            continue;
        };

        // Now find the matching closing brace by tracking depth
        let mut depth = 1;
        let mut block_end = brace_pos + 1;
        for (idx, ch) in rest[brace_pos + 1..].char_indices() {
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    block_end = brace_pos + 1 + idx;
                    break;
                }
            }
        }

        let block_body = &rest[brace_pos + 1..block_end];

        // Within the block body, find prerequisites and invalidated_by
        let mut prerequisites = Vec::new();
        let mut invalidated_by = Vec::new();

        // Find fn prerequisites <...> { ...body... }
        let prereq_fn = "fn prerequisites";
        if let Some(pf_start) = block_body.find(prereq_fn) {
            let after_fn = &block_body[pf_start + prereq_fn.len()..];
            if let Some(fn_brace) = after_fn.find('{') {
                let body = &after_fn[fn_brace + 1..];
                // Find matching }
                let mut pd = 1;
                let mut body_end = 0;
                for (idx, ch) in body.char_indices() {
                    if ch == '{' {
                        pd += 1;
                    } else if ch == '}' {
                        pd -= 1;
                        if pd == 0 {
                            body_end = idx;
                            break;
                        }
                    }
                }
                let fn_body = &body[..body_end];
                if let Some(sb) = fn_body.find('[') {
                    if let Some(eb) = fn_body[sb..].find(']') {
                        prerequisites = fn_body[sb + 1..sb + eb]
                            .split(',')
                            .map(|s| s.trim().trim_matches('"').to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                }
            }
        }

        // Find fn invalidated_by
        let inv_fn = "fn invalidated_by";
        if let Some(if_start) = block_body.find(inv_fn) {
            let after_fn = &block_body[if_start + inv_fn.len()..];
            if let Some(fn_brace) = after_fn.find('{') {
                let body = &after_fn[fn_brace + 1..];
                let mut pd = 1;
                let mut body_end = 0;
                for (idx, ch) in body.char_indices() {
                    if ch == '{' {
                        pd += 1;
                    } else if ch == '}' {
                        pd -= 1;
                        if pd == 0 {
                            body_end = idx;
                            break;
                        }
                    }
                }
                let fn_body = &body[..body_end];
                if let Some(sb) = fn_body.find('[') {
                    if let Some(eb) = fn_body[sb..].find(']') {
                        invalidated_by = fn_body[sb + 1..sb + eb]
                            .split(',')
                            .map(|s| s.trim().trim_matches('"').to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                }
            }
        }

        passes.push((name.clone(), prerequisites, invalidated_by));
    }

    passes
}

// ─── Dependency graph extraction from Cargo.toml files ──────────────────────

fn extract_dependency_graph(crates_dir: &Path) -> Vec<Edge> {
    let mut edges = Vec::new();

    let entries = match fs::read_dir(crates_dir) {
        Ok(e) => e,
        Err(_) => return edges,
    };

    for entry in entries.flatten() {
        let crate_path = entry.path();
        if !crate_path.is_dir() {
            continue;
        }
        let cargo_toml = crate_path.join("Cargo.toml");
        if !cargo_toml.exists() {
            continue;
        }
        let content = match fs::read_to_string(&cargo_toml) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let parsed: CargoToml = match toml::from_str(&content) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let package_name = match parsed.package {
            Some(ref pkg) => pkg.name.clone(),
            None => continue,
        };

        if let Some(deps) = parsed.dependencies {
            for (dep_name, dep) in &deps {
                if !dep_name.starts_with("motarjim-") {
                    continue;
                }
                let is_path_dep = match dep {
                    Dependency::Detailed { path, .. } => path.is_some(),
                    _ => false,
                };
                if is_path_dep || dep_name.starts_with("motarjim-") {
                    edges.push(Edge {
                        from: package_name.clone(),
                        to: dep_name.clone(),
                    });
                }
            }
        }
    }

    edges.sort();
    edges.dedup();
    edges
}

// ─── Renderers ───────────────────────────────────────────────────────────────

fn render_mermaid(edges: &[Edge]) -> String {
    let mut out = String::new();
    out.push_str("# Dependency Graph\n\n");
    out.push_str("```mermaid\n");
    out.push_str("graph TD\n");

    let mut nodes = BTreeMap::new();
    for edge in edges {
        let from = edge.from.replace('-', "_");
        let to = edge.to.replace('-', "_");
        nodes.insert(from.clone(), true);
        nodes.insert(to.clone(), true);
    }

    for node in nodes.keys() {
        let label = node.replace('_', "-");
        out.push_str(&format!("    {node}[[{label}]]\n"));
    }

    out.push('\n');
    for edge in edges {
        let from = edge.from.replace('-', "_");
        let to = edge.to.replace('-', "_");
        out.push_str(&format!("    {from} --> {to}\n"));
    }

    out.push_str("```\n");
    out
}

fn render_dot(edges: &[Edge]) -> String {
    let mut out = String::new();
    out.push_str("digraph dependencies {\n");
    out.push_str("    rankdir=LR;\n");
    out.push_str("    bgcolor=\"#1a1a2e\";\n");
    out.push_str(
        "    node [shape=box, style=filled, fillcolor=\"#16213e\", fontcolor=white, color=\"#0f3460\"];\n",
    );
    out.push_str("    edge [color=\"#e94560\", penwidth=1.5];\n\n");

    let mut nodes = BTreeMap::new();
    for edge in edges {
        let from = edge.from.replace('-', "_");
        let to = edge.to.replace('-', "_");
        nodes.insert(from, true);
        nodes.insert(to, true);
    }

    for node in nodes.keys() {
        let label = node.replace('_', "-");
        out.push_str(&format!("    {node} [label=\"{label}\"];\n"));
    }

    out.push('\n');
    for edge in edges {
        let from = edge.from.replace('-', "_");
        let to = edge.to.replace('-', "_");
        out.push_str(&format!("    {from} -> {to};\n"));
    }

    out.push_str("}\n");
    out
}

fn render_module_text_trees(trees: &[ModuleTree]) -> String {
    let mut out = String::new();

    out.push_str("# Module Hierarchy\n\n");
    out.push_str("```\n");

    for tree in trees {
        out.push_str(&format!("{}\n", tree.name));
        for child in &tree.children {
            out.push_str(&format!("  └── {}\n", child.name));
        }
        out.push('\n');
    }

    out.push_str("```\n");
    out
}

fn render_public_api(items: &[PublicItem]) -> String {
    let mut out = String::new();
    out.push_str("# Public API Surface\n\n");

    let mut by_kind = BTreeMap::new();
    for item in items {
        by_kind
            .entry(item.kind)
            .or_insert_with(Vec::new)
            .push(item.name.clone());
    }

    for (kind, names) in &by_kind {
        out.push_str(&format!("## {kind}(s)\n\n"));
        for name in names {
            out.push_str(&format!("- `{name}`\n"));
        }
        out.push('\n');
    }

    out
}

fn render_pass_dag(passes: &[(String, Vec<String>, Vec<String>)]) -> String {
    let mut out = String::new();

    out.push_str("# Optimization Pass Dependency Graph\n\n");
    out.push_str("```mermaid\n");
    out.push_str("graph TD\n");

    for (pass_name, _prereqs, _invalidates) in passes {
        let id = pass_name.replace([' ', '-'], "_");
        out.push_str(&format!("    {id}[[{pass_name}]]\n"));
    }

    out.push('\n');
    for (pass_name, prereqs, _invalidates) in passes {
        let id = pass_name.replace([' ', '-'], "_");
        for prereq in prereqs {
            let prereq_id = prereq.replace([' ', '-'], "_");
            out.push_str(&format!("    {prereq_id} --> {id}\n"));
        }
    }

    out.push_str("```\n\n");
    out.push_str("## Pass Details\n\n");
    out.push_str("| Pass | Prerequisites | Invalidates |\n");
    out.push_str("|------|--------------|-------------|\n");
    for (pass_name, prereqs, invalidates) in passes {
        let prereq_str = if prereqs.is_empty() {
            "none".to_string()
        } else {
            prereqs.join(", ")
        };
        let invalidates_str = if invalidates.is_empty() {
            "none".to_string()
        } else {
            invalidates.join(", ")
        };
        out.push_str(&format!(
            "| {pass_name} | {prereq_str} | {invalidates_str} |\n"
        ));
    }
    out.push('\n');

    out
}

// ─── Output writing ──────────────────────────────────────────────────────────

const OUTPUT_DIR: &str = "docs/architecture";

fn ensure_output_dir() {
    if let Err(e) = fs::create_dir_all(OUTPUT_DIR) {
        eprintln!("Failed to create output directory: {e}");
        std::process::exit(1);
    }
}

fn write_output(filename: &str, content: &str) {
    if let Err(e) = fs::write(format!("{OUTPUT_DIR}/{filename}"), content) {
        eprintln!("Failed to write {filename}: {e}");
        std::process::exit(1);
    }
}

fn write_output_abs(path: &str, content: &str) {
    if let Err(e) = fs::write(path, content) {
        eprintln!("Failed to write {path}: {e}");
        std::process::exit(1);
    }
}

// ─── Subcommands ─────────────────────────────────────────────────────────────

fn cmd_diagrams() {
    ensure_output_dir();

    let crates_dir = Path::new("crates");
    let edges = extract_dependency_graph(crates_dir);

    let mermaid = render_mermaid(&edges);
    write_output("dependency-graph.md", &mermaid);

    let dot = render_dot(&edges);
    write_output("dependency-graph.dot", &dot);

    let mut module_trees = Vec::new();
    if let Ok(entries) = fs::read_dir(crates_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(tree) = extract_module_tree(&entry.path()) {
                    module_trees.push(tree);
                }
            }
        }
    }
    let module_diagram = render_module_text_trees(&module_trees);
    write_output("module-hierarchy.md", &module_diagram);

    let mut all_items = Vec::new();
    if let Ok(entries) = fs::read_dir(crates_dir) {
        for entry in entries.flatten() {
            let crate_path = entry.path();
            if !crate_path.is_dir() {
                continue;
            }
            let crate_name = crate_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            let lib_rs = crate_path.join("src/lib.rs");
            if lib_rs.exists() {
                if let Ok(content) = fs::read_to_string(&lib_rs) {
                    let mut items = extract_public_items(&content);
                    for item in &mut items {
                        item.module_path = crate_name.clone();
                    }
                    all_items.extend(items);
                }
            }
        }
    }
    if let Err(e) = fs::create_dir_all("docs/api") {
        eprintln!("Failed to create docs/api: {e}");
        std::process::exit(1);
    }
    let api_report = render_public_api(&all_items);
    write_output_abs("docs/api/public-surface.md", &api_report);

    let optimizer_lib = Path::new("crates/motarjim-optimizer/src/lib.rs");
    if optimizer_lib.exists() {
        if let Ok(content) = fs::read_to_string(optimizer_lib) {
            let passes = extract_pass_dependencies(&content);
            let pass_dag = render_pass_dag(&passes);
            write_output("pass-graph.md", &pass_dag);
        }
    }

    println!("Diagrams generated in {OUTPUT_DIR}/");
    println!("  - dependency-graph.md (Mermaid.js)");
    println!("  - dependency-graph.dot (Graphviz)");
    println!("  - module-hierarchy.md");
    println!("  - pass-graph.md");
    println!("API report generated in docs/api/public-surface.md");
}

fn cmd_codegen() {
    println!("Code generation is not yet implemented");
    println!("This will generate AST types from a spec file in a future release");
}

fn cmd_check() {
    let output_dir = Path::new(OUTPUT_DIR);
    if !output_dir.exists() {
        eprintln!("ERROR: {OUTPUT_DIR} does not exist. Run `cargo xtask diagrams` first.");
        std::process::exit(1);
    }

    let crates_dir = Path::new("crates");
    let edges = extract_dependency_graph(crates_dir);

    let mermaid = render_mermaid(&edges);
    let mermaid_path = output_dir.join("dependency-graph.md");
    if mermaid_path.exists() {
        let existing = if let Ok(c) = fs::read_to_string(&mermaid_path) {
            c
        } else {
            eprintln!("ERROR: could not read dependency-graph.md");
            std::process::exit(1);
        };
        if existing != mermaid {
            eprintln!(
                "ERROR: dependency-graph.md is out of date. Run `cargo xtask diagrams` to regenerate."
            );
            std::process::exit(1);
        }
    } else {
        eprintln!("ERROR: dependency-graph.md not found. Run `cargo xtask diagrams` first.");
        std::process::exit(1);
    }

    let dot = render_dot(&edges);
    let dot_path = output_dir.join("dependency-graph.dot");
    if dot_path.exists() {
        let existing = if let Ok(c) = fs::read_to_string(&dot_path) {
            c
        } else {
            eprintln!("ERROR: could not read dependency-graph.dot");
            std::process::exit(1);
        };
        if existing != dot {
            eprintln!(
                "ERROR: dependency-graph.dot is out of date. Run `cargo xtask diagrams` to regenerate."
            );
            std::process::exit(1);
        }
    } else {
        eprintln!("ERROR: dependency-graph.dot not found. Run `cargo xtask diagrams` first.");
        std::process::exit(1);
    }

    println!("All diagrams are up to date.");
}

fn cmd_help() {
    println!("Motarjim xtask — Build scripts");
    println!();
    println!("USAGE:");
    println!("  cargo xtask <subcommand>");
    println!();
    println!("SUBCOMMANDS:");
    println!("  diagrams    Generate architecture diagrams (Mermaid.js, Graphviz, module tree, pass DAG)");
    println!("  codegen     Code generation from spec (placeholder)");
    println!("  check       Verify diagrams are up to date");
    println!("  help        Print this help message");
}

// ─── Entry point ─────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = env::args().collect();

    let subcommand = args
        .iter()
        .position(|a| a == "xtask")
        .and_then(|i| args.get(i + 1));
    let subcommand = subcommand.or_else(|| args.get(1));

    match subcommand.map(String::as_str) {
        Some("diagrams") => cmd_diagrams(),
        Some("codegen") => cmd_codegen(),
        Some("check") => cmd_check(),
        Some("help") | None => cmd_help(),
        Some(other) => {
            eprintln!("Unknown subcommand: {other}");
            eprintln!("Run `cargo xtask help` for usage.");
            std::process::exit(1);
        }
    }
}
