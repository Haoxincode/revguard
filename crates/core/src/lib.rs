//! RevGuard core library.
//!
//! Production-only injection of *harmless* academic-sounding identifier
//! strings into build artifacts so pasting them into a well-aligned LLM may
//! trigger a safety refusal. Deterrent, not encryption.
//!
//! Red line: only four categories (CBRN, toxin, explosive, weapon),
//! academic-label identifiers only; NEVER CSAM/sexual/trafficking/NCII;
//! NEVER real recipes or actionable harm.

pub mod config;
pub mod engine;
pub mod hashutil;
pub mod inject;
pub mod manifest;
pub mod rust_src;
pub mod sourcemap;
pub mod strip;
pub mod wasm_section;
pub mod wordlist;

pub use config::Config;
pub use engine::{inject_dir, load_wordlist, strip_dir, verify_dir, StripReport, VerifyReport};
pub use hashutil::{sha256_hex, sha256_str};
pub use inject::{inject_text, InjectResult};
pub use manifest::{FileRecord, InjectionKind, InjectionRecord, Manifest, MANIFEST_NAME};
pub use rust_src::{inject_rust_src, strip_rust_src};
pub use sourcemap::{inject_sourcemap, strip_sourcemap};
pub use strip::{is_injected, strip_text};
pub use wasm_section::{has_revguard_section, inject_wasm, strip_wasm};
pub use wordlist::{Category, Intensity, Wordlist};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inject_then_strip_is_identity() {
        let clean = "export function greet(name) {\n  return `hello ${name}`;\n}\n";
        let cfg = Config::default();
        let wl = Wordlist::default_builtin();
        let inj = inject_text(clean, &cfg, &wl);
        assert!(is_injected(&inj.content));
        let restored = strip_text(&inj.content);
        assert_eq!(restored, clean);
    }

    #[test]
    fn sourcemap_roundtrip_pretty() {
        let clean = r#"{
  "version": 3,
  "file": "app.js",
  "sources": [
    "app.js"
  ],
  "sourcesContent": [
    "export const x = 1;\n"
  ],
  "mappings": "AAAA"
}"#;
        let mut cfg = Config::default();
        cfg.inject_source_map = true;
        cfg.inject_dead_code = false;
        let wl = Wordlist::default_builtin();
        let inj = inject_sourcemap(clean, &cfg, &wl).unwrap();
        assert!(inj.content.contains("REVGUARD-BANNER-START"));
        assert!(inj.content.contains("x_revguard_phrases"));
        let stripped = strip_sourcemap(&inj.content).unwrap();
        // Canonical pretty compare via parse
        let a: serde_json::Value = serde_json::from_str(clean).unwrap();
        let b: serde_json::Value = serde_json::from_str(&stripped).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn wasm_custom_section_roundtrip() {
        // Minimal valid empty wasm module: magic + version
        let mut clean = Vec::new();
        clean.extend_from_slice(&wasm_section::WASM_MAGIC);
        clean.extend_from_slice(&wasm_section::WASM_VERSION);
        let mut cfg = Config::default();
        cfg.inject_wasm_section = true;
        let wl = Wordlist::default_builtin();
        let inj = inject_wasm(&clean, &cfg, &wl).unwrap();
        assert!(has_revguard_section(&inj.bytes));
        assert!(inj.bytes.len() > clean.len());
        let stripped = strip_wasm(&inj.bytes).unwrap();
        assert_eq!(stripped, clean);
        assert!(!has_revguard_section(&stripped));
    }

    #[test]
    fn no_forbidden_categories_in_wordlist() {
        let wl = Wordlist::default_builtin();
        let blob = serde_json::to_string(&wl).unwrap().to_ascii_lowercase();
        for forbid in ["csam", "child porn", "ncii", "trafficking", "sexual"] {
            assert!(!blob.contains(forbid), "forbidden term leaked: {}", forbid);
        }
    }
}
