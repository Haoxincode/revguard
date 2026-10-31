//! RevGuard napi-rs bindings (in-process).

#![deny(clippy::all)]

use napi_derive::napi;
use revguard_core::{
    inject_dir, inject_text, strip_dir, strip_text, verify_dir, Config, Wordlist,
};
use std::path::PathBuf;

fn cfg(config_json: Option<String>) -> napi::Result<Config> {
    match config_json {
        Some(s) => Config::from_json(&s).map_err(|e| napi::Error::from_reason(e.to_string())),
        None => Ok(Config::default()),
    }
}

#[napi]
pub fn inject(dir: String, config_json: Option<String>) -> napi::Result<String> {
    let m = inject_dir(PathBuf::from(dir).as_path(), &cfg(config_json)?)
        .map_err(|e| napi::Error::from_reason(e))?;
    Ok(m.to_json_pretty())
}

#[napi]
pub fn strip(dir: String) -> napi::Result<String> {
    let r = strip_dir(PathBuf::from(dir).as_path()).map_err(|e| napi::Error::from_reason(e))?;
    Ok(format!(
        "{{\"stripped\":{},\"hash_ok\":{},\"mismatches\":{}}}",
        r.stripped,
        r.hash_ok,
        r.hash_mismatch.len()
    ))
}

#[napi]
pub fn verify(dir: String) -> napi::Result<String> {
    let r = verify_dir(PathBuf::from(dir).as_path()).map_err(|e| napi::Error::from_reason(e))?;
    Ok(format!(
        "{{\"checked\":{},\"ok\":{},\"missing\":{},\"mismatches\":{},\"no_manifest\":{}}}",
        r.checked,
        r.ok,
        r.missing_injection.len(),
        r.hash_mismatch.len(),
        r.no_manifest
    ))
}

/// Inject into a single source string (for SWC / bundler transform hooks).
#[napi]
pub fn inject_string(content: String, config_json: Option<String>) -> napi::Result<String> {
    let c = cfg(config_json)?.apply_env();
    if !c.is_active() {
        return Ok(content);
    }
    let wl = if let Some(ref p) = c.wordlist_path {
        let s = std::fs::read_to_string(p).map_err(|e| napi::Error::from_reason(e.to_string()))?;
        Wordlist::from_json(&s).map_err(|e| napi::Error::from_reason(e.to_string()))?
    } else {
        Wordlist::default_builtin()
    };
    Ok(inject_text(&content, &c, &wl).content)
}

#[napi]
pub fn strip_string(content: String) -> napi::Result<String> {
    Ok(strip_text(&content))
}
