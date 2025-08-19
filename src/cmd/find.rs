use clap::Args;
use std::path::PathBuf;

use super::fsutil::filter::{Filter, Matcher, merge};
use super::fsutil::iter::FindIter;
use super::fsutil::util::{parse_human_age, parse_human_size};

#[derive(Args, Debug, Clone)]
pub struct FindArgs {
    #[arg(short, long, default_value = ".")]
    pub root: String,

    /// pattern, glob, suffix
    pub pattern: Option<String>,

    /// DSL (e.g. `contains:report ext:txt age>2d size<10MB hidden:true type:file`)
    #[arg(long)]
    pub select: Option<String>,

    // preset / shortcut
    #[arg(long)] pub images: bool,
    #[arg(long)] pub videos: bool,
    #[arg(long)] pub docs: bool,

    #[arg(long)] pub older: Option<String>,   // "30d"  -> min_age_secs
    #[arg(long)] pub newer: Option<String>,   // "2h"   -> max_age_secs
    #[arg(long)] pub larger: Option<String>,  // "100MB"-> min_size
    #[arg(long)] pub smaller: Option<String>, // "64KB" -> max_size
}

/// Return a typed iterator over matches for the given `Filter`.
pub fn find_with_filter(root: &str, filter: Filter) -> FindIter<Filter> {
    FindIter::new(root, filter)
}

/// Collect all matching paths into a Vec for convenience.
pub fn find_collect(root: &str, filter: Filter) -> Vec<PathBuf> {
    find_with_filter(root, filter).collect()
}

/// A generic helper that accepts any Matcher (not just Filter).
pub fn find_with_matcher<M: Matcher>(root: &str, matcher: M) -> FindIter<M> {
    FindIter::new(root, matcher)
}

/// Execute `find` with CLI arguments.
pub fn run(args: FindArgs) {
    // Build base filter from presets and pattern.
    let mut f = Filter::default();
    apply_presets(&mut f, &args);

    // Merge DSL if provided.
    if let Some(dsl) = &args.select {
        let dsl_filter = parse_select_dsl(dsl);
        f = merge(f, dsl_filter);
    }

    // Stream results (print one per line).
    for p in find_with_filter(&args.root, f) {
        println!("{}", p.display());
    }
}

/// Apply preset flags, human sizes/ages and simple pattern heuristic.
fn apply_presets(f: &mut Filter, args: &FindArgs) {
    // Extensions by preset flags
    if args.images { f.exts.extend(["jpg","jpeg","png","gif","webp"].map(String::from)); }
    if args.videos { f.exts.extend(["mp4","mov","mkv","avi"].map(String::from)); }
    if args.docs   { f.exts.extend(["pdf","docx","txt","md"].map(String::from)); }

    // Age presets
    if let Some(s) = &args.older   { if let Some(v) = parse_human_age(s)  { f.min_age_secs = Some(v); } }
    if let Some(s) = &args.newer   { if let Some(v) = parse_human_age(s)  { f.max_age_secs = Some(v); } }

    // Size presets
    if let Some(s) = &args.larger  { if let Some(v) = parse_human_size(s) { f.min_size = Some(v); } }
    if let Some(s) = &args.smaller { if let Some(v) = parse_human_size(s) { f.max_size = Some(v); } }

    // Simple pattern heuristic:
    // - If it looks like a glob, try to derive suffix by final extension.
    // - If it starts with '.', treat as suffix (e.g., ".log")
    // - Otherwise, treat as substring `contains`.
    if let Some(p) = &args.pattern {
        if p.contains('*') || p.contains('?') || p.contains('[') {
            if let Some((_, suf)) = p.rsplit_once('.') {
                f.suffix.push(format!(".{}", suf));
            }
        } else if p.starts_with('.') {
            f.suffix.push(p.clone());
        } else {
            f.contains.push(p.clone());
        }
    }

    // Normalize extensions to lowercase without leading dot.
    if !f.exts.is_empty() {
        for e in &mut f.exts {
            *e = e.trim_start_matches('.').to_ascii_lowercase();
        }
    }
}

/// Parse a tiny DSL into a Filter.
/// Supported tokens:
/// - `contains:<s>` / `name:<s>`
/// - `prefix:<s>` / `suffix:<s>` / `ext:<e>`
/// - `type:file|dir`
/// - `hidden:true|false|1|0`
/// - `age>1d` / `age<2h`
/// - `size>10MB` / `size<64KB`
fn parse_select_dsl(dsl: &str) -> Filter {
    let mut f = Filter::default();
    for tok in dsl.split_whitespace() {
        if let Some((k, v)) = tok.split_once(':').or(tok.split_once('=')) {
            match k {
                "contains" | "name" => f.contains.push(v.to_string()),
                "prefix" => f.prefix.push(v.to_string()),
                "suffix" => f.suffix.push(v.to_string()),
                "ext" => f.exts.push(v.trim_start_matches('.').to_ascii_lowercase()),
                "type" => {
                    f.ty_file = v == "file";
                    f.ty_dir  = v == "dir";
                }
                "hidden" => {
                    f.include_hidden = matches!(v, "1" | "true" | "yes" | "on");
                }
                _ => {}
            }
        } else if tok.starts_with("age>") {
            f.min_age_secs = parse_human_age(&tok[4..]);
        } else if tok.starts_with("age<") {
            f.max_age_secs = parse_human_age(&tok[4..]);
        } else if tok.starts_with("size>") {
            f.min_size = parse_human_size(&tok[5..]);
        } else if tok.starts_with("size<") {
            f.max_size = parse_human_size(&tok[5..]);
        }
    }
    f
}
