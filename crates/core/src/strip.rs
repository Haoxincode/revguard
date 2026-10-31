//! RevGuard strip + verify helpers.
//!
//! `strip` removes injected banner/dead-code regions by locating sentinel
//! markers, reverting byte-for-byte to the clean original. It is idempotent:
//! stripping clean content is a no-op.
//!
//! Supports both the canonical text injector markers and the SWC WASM plugin
//! variants (block-comment banner with newline after `/*!`, and AST dead-code
//! with string-literal sentinels inside the Symbol.for guard).

use crate::inject::{BANNER_END, BANNER_START, DEADCODE_END, DEADCODE_START};

/// Remove the leading RevGuard banner, if present.
fn strip_banner(content: &str) -> String {
    // Canonical: file starts with `/*! REVGUARD-BANNER-START` … `REVGUARD-BANNER-END */\n`
    if content.starts_with(BANNER_START) {
        if let Some(rel_end) = content.find(BANNER_END) {
            let end = rel_end + BANNER_END.len();
            return content[end..].to_string();
        }
    }
    // WASM / pretty-printed block comment: `/*!` then later `REVGUARD-BANNER-START` … `REVGUARD-BANNER-END` … `*/`
    if let Some(marker) = content.find("REVGUARD-BANNER-START") {
        if let Some(block_start) = content[..marker].rfind("/*") {
            if block_start <= 2 {
                // only treat as file-leading banner
                if let Some(end_marker) = content[marker..].find("REVGUARD-BANNER-END") {
                    let after = marker + end_marker + "REVGUARD-BANNER-END".len();
                    if let Some(close) = content[after..].find("*/") {
                        let mut end = after + close + 2;
                        if content[end..].starts_with('\n') {
                            end += 1;
                        }
                        return format!("{}{}", &content[..block_start], &content[end..]);
                    }
                }
            }
        }
    }
    content.to_string()
}

/// Remove the trailing (or embedded) RevGuard dead-code block, if present.
fn strip_dead_code(content: &str) -> String {
    // Canonical comment sentinels
    if let Some(start) = content.find(DEADCODE_START) {
        if let Some(rel_end) = content[start..].find(DEADCODE_END) {
            let end = start + rel_end + DEADCODE_END.len();
            return format!("{}{}", &content[..start], &content[end..]);
        }
    }
    // WASM plugin: if (… __REVGUARD_NEVER__ === Symbol.for("revguard.never")) { … REVGUARD-DEADCODE-… }
    if content.contains("REVGUARD-DEADCODE-START") && content.contains("__REVGUARD_NEVER__") {
        if let Some(if_pos) = content.find("if (typeof globalThis !== \"undefined\" && globalThis.__REVGUARD_NEVER__ === Symbol.for(\"revguard.never\"))") {
            // find matching closing brace for the if-block
            if let Some(brace) = content[if_pos..].find('{') {
                let mut i = if_pos + brace + 1;
                let bytes = content.as_bytes();
                let mut depth = 1i32;
                while i < bytes.len() && depth > 0 {
                    match bytes[i] {
                        b'{' => depth += 1,
                        b'}' => depth -= 1,
                        _ => {}
                    }
                    i += 1;
                }
                // include trailing newline if present
                if i < content.len() && content[i..].starts_with('\n') {
                    i += 1;
                }
                // also drop a leading newline immediately before the if
                let mut start = if_pos;
                if start > 0 && content.as_bytes()[start - 1] == b'\n' {
                    start -= 1;
                }
                return format!("{}{}", &content[..start], &content[i..]);
            }
        }
    }
    content.to_string()
}

/// Strip all RevGuard injections. Idempotent.
pub fn strip_text(content: &str) -> String {
    let c = strip_dead_code(content);
    strip_banner(&c)
}

/// Does this text contain any RevGuard injection markers?
pub fn is_injected(content: &str) -> bool {
    content.contains(BANNER_START)
        || content.contains("REVGUARD-BANNER-START")
        || content.contains(DEADCODE_START)
        || content.contains("REVGUARD-DEADCODE-START")
}
