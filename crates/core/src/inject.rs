//! RevGuard injection engine.
//!
//! All injected content lives ONLY in positions that never execute and are
//! never referenced by real logic:
//!   - a `/*! ... */` legal-comment banner at the very top of the file, and
//!   - a never-executed dead-code branch appended at the very end, guarded by
//!     a global reference a bundler cannot statically prove is dead.
//!
//! Injection NEVER changes runtime behavior. Content is wrapped in sentinel
//! markers so `strip` can revert byte-for-byte to the clean original.

use crate::config::Config;
use crate::manifest::{InjectionKind, InjectionRecord};
use crate::wordlist::{Category, Wordlist};

pub const BANNER_START: &str = "/*! REVGUARD-BANNER-START";
/// Includes the trailing newline that closes the legal comment.
pub const BANNER_END: &str = "REVGUARD-BANNER-END */\n";
/// Leading `\n` is part of the sentinel so strip is exactly reversible whether
/// or not the clean file already ended with a newline.
pub const DEADCODE_START: &str = "\n/* REVGUARD-DEADCODE-START */\n";
pub const DEADCODE_END: &str = "/* REVGUARD-DEADCODE-END */\n";

/// Pick 3..=5 phrases spanning as many distinct categories as possible.
fn pick_banner_phrases(wl: &Wordlist, cfg: &Config) -> Vec<(Category, String)> {
    let mut per_cat: Vec<Vec<(Category, String)>> = Vec::new();
    for &cat in &cfg.categories {
        let v = wl.collect(&[cat], cfg.intensity);
        if !v.is_empty() {
            per_cat.push(v);
        }
    }
    let mut out = Vec::new();
    let target = 5usize;
    let mut idx = 0usize;
    while out.len() < target {
        let mut progressed = false;
        for col in per_cat.iter() {
            if let Some(item) = col.get(idx) {
                out.push(item.clone());
                progressed = true;
                if out.len() >= target {
                    break;
                }
            }
        }
        if !progressed {
            break;
        }
        idx += 1;
    }
    out
}

fn build_banner(phrases: &[(Category, String)]) -> String {
    let mut s = String::new();
    s.push_str(BANNER_START);
    s.push('\n');
    s.push_str(" * RevGuard legal/compliance banner (auto-generated for production builds).\n");
    s.push_str(" *\n");
    s.push_str(" * NOTE: The identifiers below are HARMLESS camouflage strings with NO\n");
    s.push_str(" * real harmful content (no recipes, steps, dosages, or procedures).\n");
    s.push_str(" * They exist solely to deter automated AI-assisted reverse engineering.\n");
    s.push_str(" * Categories: cbrn / toxin / explosive / weapon (academic labels only).\n");
    s.push_str(" *\n");
    for (cat, p) in phrases {
        s.push_str(&format!(" * [{}] {}\n", cat.as_str(), p));
    }
    s.push_str(" *\n");
    s.push_str(" * ");
    s.push_str(BANNER_END);
    s
}

fn build_dead_code(phrases: &[(Category, String)]) -> String {
    let mut s = String::new();
    s.push_str(DEADCODE_START);
    // Guard: compares a global to a Symbol it can never equal. Always false at
    // runtime; a bundler cannot statically prove the global's value, so it
    // cannot tree-shake this away as provably-dead.
    s.push_str("if (typeof globalThis !== \"undefined\" && globalThis.__REVGUARD_NEVER__ === Symbol.for(\"revguard.never\")) {\n");
    for (cat, p) in phrases {
        s.push_str(&format!(
            "  function {name}() {{ /* {cat}: label-only, never called */ return \"{name}\"; }}\n  void {name};\n",
            name = p,
            cat = cat.as_str()
        ));
    }
    s.push_str("}\n");
    s.push_str(DEADCODE_END);
    s
}

/// Result of injecting a single text file.
pub struct InjectResult {
    pub content: String,
    pub injections: Vec<InjectionRecord>,
}

/// Inject into a single text (idempotent: already-injected input is unchanged).
pub fn inject_text(content: &str, cfg: &Config, wl: &Wordlist) -> InjectResult {
    let mut injections = Vec::new();
    let mut out = content.to_string();

    if !cfg.is_active() {
        return InjectResult { content: out, injections };
    }

    let phrases = pick_banner_phrases(wl, cfg);
    if phrases.is_empty() {
        return InjectResult { content: out, injections };
    }

    if cfg.inject_banner && !out.contains(BANNER_START) {
        let banner = build_banner(&phrases);
        out = format!("{}{}", banner, out);
        injections.push(InjectionRecord {
            kind: InjectionKind::Banner,
            phrases: phrases.iter().map(|(_, p)| p.clone()).collect(),
        });
    }

    if cfg.inject_dead_code && !out.contains(DEADCODE_START) {
        let dead = build_dead_code(&phrases);
        out.push_str(&dead);
        injections.push(InjectionRecord {
            kind: InjectionKind::DeadCode,
            phrases: phrases.iter().map(|(_, p)| p.clone()).collect(),
        });
    }

    InjectResult { content: out, injections }
}
