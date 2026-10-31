//! WASM custom-section injector.
//!
//! Appends a custom section named `revguard` containing harmless camouflage
//! identifier strings (UTF-8). Does **not** modify code/data sections.
//!
//! Risks (documented in README):
//! - `wasm-opt` / bundlers may strip unknown custom sections
//! - LLMs may not see binary custom sections unless tools dump them
//! - Some CDNs / SRI pipelines hash the whole module — expect hash change
//!
//! Strip removes the `revguard` custom section(s) only.

use crate::config::Config;
use crate::hashutil::sha256_hex;
use crate::manifest::{InjectionKind, InjectionRecord};
use crate::wordlist::Wordlist;

pub const SECTION_NAME: &str = "revguard";
pub const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6d]; // \0asm
pub const WASM_VERSION: [u8; 4] = [0x01, 0x00, 0x00, 0x00];

pub struct WasmInjectResult {
    pub bytes: Vec<u8>,
    pub injections: Vec<InjectionRecord>,
}

fn read_leb128(bytes: &[u8], mut i: usize) -> Result<(u32, usize), String> {
    let mut result: u32 = 0;
    let mut shift = 0;
    loop {
        if i >= bytes.len() {
            return Err("leb128 truncated".into());
        }
        let b = bytes[i];
        i += 1;
        result |= u32::from(b & 0x7f) << shift;
        if b & 0x80 == 0 {
            return Ok((result, i));
        }
        shift += 7;
        if shift > 28 {
            return Err("leb128 too long".into());
        }
    }
}

fn write_leb128(mut v: u32) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let mut b = (v & 0x7f) as u8;
        v >>= 7;
        if v != 0 {
            b |= 0x80;
        }
        out.push(b);
        if v == 0 {
            break;
        }
    }
    out
}

fn encode_custom_section(name: &str, payload: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    let name_bytes = name.as_bytes();
    body.extend(write_leb128(name_bytes.len() as u32));
    body.extend(name_bytes);
    body.extend(payload);
    let mut section = Vec::new();
    section.push(0u8); // custom section id
    section.extend(write_leb128(body.len() as u32));
    section.extend(body);
    section
}

/// True if module already has a `revguard` custom section.
pub fn has_revguard_section(bytes: &[u8]) -> bool {
    if bytes.len() < 8 || bytes[0..4] != WASM_MAGIC {
        return false;
    }
    let mut i = 8usize;
    while i < bytes.len() {
        let id = bytes[i];
        i += 1;
        let (size, ni) = match read_leb128(bytes, i) {
            Ok(x) => x,
            Err(_) => return false,
        };
        i = ni;
        let end = i + size as usize;
        if end > bytes.len() {
            return false;
        }
        if id == 0 {
            // custom: name leb + name
            if let Ok((nlen, ni2)) = read_leb128(bytes, i) {
                let ns = ni2;
                let ne = ns + nlen as usize;
                if ne <= end {
                    if let Ok(name) = std::str::from_utf8(&bytes[ns..ne]) {
                        if name == SECTION_NAME {
                            return true;
                        }
                    }
                }
            }
        }
        i = end;
    }
    false
}

pub fn inject_wasm(bytes: &[u8], cfg: &Config, wl: &Wordlist) -> Result<WasmInjectResult, String> {
    if !cfg.is_active() || !cfg.inject_wasm_section {
        return Ok(WasmInjectResult {
            bytes: bytes.to_vec(),
            injections: Vec::new(),
        });
    }
    if bytes.len() < 8 || bytes[0..4] != WASM_MAGIC {
        return Err("not a wasm module".into());
    }
    if has_revguard_section(bytes) {
        return Ok(WasmInjectResult {
            bytes: bytes.to_vec(),
            injections: Vec::new(),
        });
    }
    let phrases = wl.collect(&cfg.categories, cfg.intensity);
    if phrases.is_empty() {
        return Ok(WasmInjectResult {
            bytes: bytes.to_vec(),
            injections: Vec::new(),
        });
    }
    let phrase_strs: Vec<String> = phrases.iter().map(|(_, p)| p.clone()).collect();
    // Payload: newline-separated identifiers + red-line note (no recipes).
    let mut payload = String::from(
        "REVGUARD custom section — harmless academic identifier strings only; no recipes/steps/dosages.\n",
    );
    for (cat, p) in &phrases {
        payload.push_str(&format!("[{}] {}\n", cat.as_str(), p));
    }
    let section = encode_custom_section(SECTION_NAME, payload.as_bytes());
    let mut out = bytes.to_vec();
    out.extend(section);
    Ok(WasmInjectResult {
        bytes: out,
        injections: vec![InjectionRecord {
            kind: InjectionKind::WasmSection,
            phrases: phrase_strs,
        }],
    })
}

/// Remove all custom sections named `revguard`.
pub fn strip_wasm(bytes: &[u8]) -> Result<Vec<u8>, String> {
    if bytes.len() < 8 || bytes[0..4] != WASM_MAGIC {
        return Err("not a wasm module".into());
    }
    let mut out = bytes[0..8].to_vec();
    let mut i = 8usize;
    while i < bytes.len() {
        let start = i;
        let id = bytes[i];
        i += 1;
        let (size, ni) = read_leb128(bytes, i)?;
        i = ni;
        let end = i + size as usize;
        if end > bytes.len() {
            return Err("section truncated".into());
        }
        let mut keep = true;
        if id == 0 {
            if let Ok((nlen, ni2)) = read_leb128(bytes, i) {
                let ns = ni2;
                let ne = ns + nlen as usize;
                if ne <= end {
                    if let Ok(name) = std::str::from_utf8(&bytes[ns..ne]) {
                        if name == SECTION_NAME {
                            keep = false;
                        }
                    }
                }
            }
        }
        if keep {
            out.extend_from_slice(&bytes[start..end]);
        }
        i = end;
    }
    Ok(out)
}

pub fn wasm_sha256(bytes: &[u8]) -> String {
    sha256_hex(bytes)
}
