//! Source map (`.map`) injection into `sourcesContent`.
//!
//! Default off (`inject_source_map: false`). When enabled, each string in
//! `sourcesContent` gets a RevGuard banner (no dead-code — it's source text,
//! not executable). Strip restores via JSON parse + banner strip.
//!
//! Demo `.map` files should be serde-canonical (compact or pretty) so that
//! inject→strip round-trips byte-identical when using the same serializer.

use crate::config::Config;
use crate::inject::{inject_text, BANNER_START};
use crate::manifest::{InjectionKind, InjectionRecord};
use crate::strip::strip_text;
use crate::wordlist::Wordlist;
use serde_json::{json, Value};

pub struct SourceMapInjectResult {
    pub content: String,
    pub injections: Vec<InjectionRecord>,
}

fn serialize_map(v: &Value) -> Result<String, String> {
    // Pretty for readability / stable demos.
    serde_json::to_string_pretty(v).map_err(|e| e.to_string())
}

/// Inject banners into every `sourcesContent` entry. Idempotent per entry.
pub fn inject_sourcemap(content: &str, cfg: &Config, wl: &Wordlist) -> Result<SourceMapInjectResult, String> {
    if !cfg.is_active() || !cfg.inject_source_map {
        return Ok(SourceMapInjectResult {
            content: content.to_string(),
            injections: Vec::new(),
        });
    }
    let mut v: Value = serde_json::from_str(content).map_err(|e| format!("sourcemap parse: {e}"))?;
    let mut phrases_all: Vec<String> = Vec::new();

    // Banner-only config for embedded source text.
    let mut sm_cfg = cfg.clone();
    sm_cfg.inject_dead_code = false;
    sm_cfg.inject_banner = true;

    if let Some(arr) = v.get_mut("sourcesContent").and_then(|x| x.as_array_mut()) {
        for item in arr.iter_mut() {
            if let Some(s) = item.as_str() {
                if s.contains(BANNER_START) {
                    continue;
                }
                let r = inject_text(s, &sm_cfg, wl);
                if !r.injections.is_empty() {
                    for inj in &r.injections {
                        phrases_all.extend(inj.phrases.iter().cloned());
                    }
                    *item = Value::String(r.content);
                }
            }
        }
    } else {
        // No sourcesContent — create one entry with banner-only stub so the
        // map still carries deterrent text without inventing fake sources.
        // Only if sources exists and is non-empty; attach parallel contents.
        if let Some(sources) = v.get("sources").and_then(|x| x.as_array()) {
            let mut contents: Vec<Value> = Vec::new();
            for (i, src) in sources.iter().enumerate() {
                let name = src.as_str().unwrap_or("unknown");
                let stub = format!("// source: {name}\nexport default null;\n");
                let r = inject_text(&stub, &sm_cfg, wl);
                if i == 0 {
                    for inj in &r.injections {
                        phrases_all.extend(inj.phrases.iter().cloned());
                    }
                }
                contents.push(Value::String(r.content));
            }
            v.as_object_mut()
                .ok_or_else(|| "sourcemap root not object".to_string())?
                .insert("sourcesContent".into(), Value::Array(contents));
        }
    }

    // Optionally annotate sources names with a harmless suffix marker in a
    // parallel x_revguard_sources note — we avoid renaming `sources` to not
    // break tooling. Instead stash phrases under a clearly-named extension
    // field that strip removes.
    if !phrases_all.is_empty() {
        if let Some(obj) = v.as_object_mut() {
            obj.insert(
                "x_revguard_phrases".into(),
                json!(phrases_all.iter().cloned().collect::<Vec<_>>()),
            );
        }
    }

    if phrases_all.is_empty() {
        return Ok(SourceMapInjectResult {
            content: content.to_string(),
            injections: Vec::new(),
        });
    }

    let out = serialize_map(&v)?;
    // Dedupe phrases for the record
    phrases_all.sort();
    phrases_all.dedup();
    Ok(SourceMapInjectResult {
        content: out,
        injections: vec![InjectionRecord {
            kind: InjectionKind::SourceMap,
            phrases: phrases_all,
        }],
    })
}

pub fn is_sourcemap_injected(content: &str) -> bool {
    content.contains(BANNER_START) || content.contains("x_revguard_phrases")
}

/// Strip RevGuard material from a source map. Returns pretty JSON.
pub fn strip_sourcemap(content: &str) -> Result<String, String> {
    let mut v: Value = serde_json::from_str(content).map_err(|e| format!("sourcemap parse: {e}"))?;
    if let Some(arr) = v.get_mut("sourcesContent").and_then(|x| x.as_array_mut()) {
        for item in arr.iter_mut() {
            if let Some(s) = item.as_str() {
                *item = Value::String(strip_text(s));
            }
        }
    }
    if let Some(obj) = v.as_object_mut() {
        obj.remove("x_revguard_phrases");
    }
    serialize_map(&v)
}
