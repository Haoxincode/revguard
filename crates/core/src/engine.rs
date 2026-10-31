//! Directory-level inject / strip / verify orchestration.

use crate::config::Config;
use crate::hashutil::{sha256_hex, sha256_str};
use crate::inject::inject_text;
use crate::manifest::{FileRecord, Manifest, MANIFEST_NAME};
use crate::sourcemap::{inject_sourcemap, is_sourcemap_injected, strip_sourcemap};
use crate::strip::{is_injected, strip_text};
use crate::wasm_section::{has_revguard_section, inject_wasm, strip_wasm};
use crate::wordlist::Wordlist;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const JS_EXTS: &[&str] = &["js", "mjs", "cjs", "jsx", "ts", "tsx"];

fn is_js_like(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| JS_EXTS.iter().any(|x| x.eq_ignore_ascii_case(e)))
        .unwrap_or(false)
}

fn is_map(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("map")
        || path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.ends_with(".js.map") || n.ends_with(".mjs.map") || n.ends_with(".cjs.map"))
            .unwrap_or(false)
}

fn is_wasm(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("wasm")
}

fn walk_files(dir: &Path, pred: &dyn Fn(&Path) -> bool) -> io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }
    fn rec(dir: &Path, out: &mut Vec<PathBuf>, pred: &dyn Fn(&Path) -> bool) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let p = entry.path();
            if p.is_dir() {
                if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                    if name == "node_modules" || name == ".git" || name == "target" {
                        continue;
                    }
                }
                rec(&p, out, pred)?;
            } else if pred(&p) {
                out.push(p);
            }
        }
        Ok(())
    }
    rec(dir, &mut out, pred)?;
    out.sort();
    Ok(out)
}

fn rel_path(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}

pub fn load_wordlist(cfg: &Config) -> Result<Wordlist, String> {
    if let Some(ref path) = cfg.wordlist_path {
        let s = fs::read_to_string(path).map_err(|e| format!("wordlist read {}: {}", path, e))?;
        Wordlist::from_json(&s).map_err(|e| format!("wordlist parse: {}", e))
    } else {
        Ok(Wordlist::default_builtin())
    }
}

/// Inject into JS-like files (and optionally `.map` / `.wasm`) under `dir`.
pub fn inject_dir(dir: &Path, cfg: &Config) -> Result<Manifest, String> {
    let cfg = cfg.clone().apply_env();
    let mut manifest = Manifest::new();
    manifest.wordlist_id = Some(cfg.wordlist_id());
    if !cfg.is_active() {
        let man_path = dir.join(MANIFEST_NAME);
        let _ = fs::write(&man_path, manifest.to_json_pretty());
        return Ok(manifest);
    }
    let wl = load_wordlist(&cfg)?;

    for file in walk_files(dir, &|p| is_js_like(p)).map_err(|e| e.to_string())? {
        let before = fs::read_to_string(&file).map_err(|e| format!("read {}: {}", file.display(), e))?;
        if is_injected(&before) {
            continue;
        }
        let sha_before = sha256_str(&before);
        let result = inject_text(&before, &cfg, &wl);
        if result.injections.is_empty() {
            continue;
        }
        let sha_after = sha256_str(&result.content);
        fs::write(&file, &result.content).map_err(|e| format!("write {}: {}", file.display(), e))?;
        manifest.files.push(FileRecord {
            path: rel_path(dir, &file),
            sha256_before: sha_before,
            sha256_after: sha_after,
            injections: result.injections,
        });
    }

    if cfg.inject_source_map {
        for file in walk_files(dir, &|p| is_map(p)).map_err(|e| e.to_string())? {
            let before = fs::read_to_string(&file).map_err(|e| format!("read {}: {}", file.display(), e))?;
            if is_sourcemap_injected(&before) {
                continue;
            }
            let sha_before = sha256_str(&before);
            let result = inject_sourcemap(&before, &cfg, &wl)?;
            if result.injections.is_empty() {
                continue;
            }
            let sha_after = sha256_str(&result.content);
            fs::write(&file, &result.content).map_err(|e| format!("write {}: {}", file.display(), e))?;
            manifest.files.push(FileRecord {
                path: rel_path(dir, &file),
                sha256_before: sha_before,
                sha256_after: sha_after,
                injections: result.injections,
            });
        }
    }

    if cfg.inject_wasm_section {
        for file in walk_files(dir, &|p| is_wasm(p)).map_err(|e| e.to_string())? {
            let before = fs::read(&file).map_err(|e| format!("read {}: {}", file.display(), e))?;
            if has_revguard_section(&before) {
                continue;
            }
            let sha_before = sha256_hex(&before);
            let result = inject_wasm(&before, &cfg, &wl)?;
            if result.injections.is_empty() {
                continue;
            }
            let sha_after = sha256_hex(&result.bytes);
            fs::write(&file, &result.bytes).map_err(|e| format!("write {}: {}", file.display(), e))?;
            manifest.files.push(FileRecord {
                path: rel_path(dir, &file),
                sha256_before: sha_before,
                sha256_after: sha_after,
                injections: result.injections,
            });
        }
    }

    let man_path = dir.join(MANIFEST_NAME);
    fs::write(&man_path, manifest.to_json_pretty())
        .map_err(|e| format!("write manifest: {}", e))?;
    Ok(manifest)
}

pub fn strip_dir(dir: &Path) -> Result<StripReport, String> {
    let man_path = dir.join(MANIFEST_NAME);
    let manifest = if man_path.exists() {
        let s = fs::read_to_string(&man_path).map_err(|e| e.to_string())?;
        Some(Manifest::from_json(&s).map_err(|e| e.to_string())?)
    } else {
        None
    };

    let mut report = StripReport {
        stripped: 0,
        hash_ok: 0,
        hash_mismatch: Vec::new(),
        missing_manifest: manifest.is_none(),
    };

    for file in walk_files(dir, &|p| is_js_like(p)).map_err(|e| e.to_string())? {
        let content = fs::read_to_string(&file).map_err(|e| format!("read {}: {}", file.display(), e))?;
        if !is_injected(&content) {
            continue;
        }
        let cleaned = strip_text(&content);
        fs::write(&file, &cleaned).map_err(|e| format!("write {}: {}", file.display(), e))?;
        report.stripped += 1;
        check_hash(&manifest, dir, &file, &cleaned, &mut report, false);
    }

    for file in walk_files(dir, &|p| is_map(p)).map_err(|e| e.to_string())? {
        let content = fs::read_to_string(&file).map_err(|e| format!("read {}: {}", file.display(), e))?;
        if !is_sourcemap_injected(&content) {
            continue;
        }
        let cleaned = strip_sourcemap(&content)?;
        fs::write(&file, &cleaned).map_err(|e| format!("write {}: {}", file.display(), e))?;
        report.stripped += 1;
        check_hash(&manifest, dir, &file, &cleaned, &mut report, false);
    }

    for file in walk_files(dir, &|p| is_wasm(p)).map_err(|e| e.to_string())? {
        let content = fs::read(&file).map_err(|e| format!("read {}: {}", file.display(), e))?;
        if !has_revguard_section(&content) {
            continue;
        }
        let cleaned = strip_wasm(&content)?;
        fs::write(&file, &cleaned).map_err(|e| format!("write {}: {}", file.display(), e))?;
        report.stripped += 1;
        let got = sha256_hex(&cleaned);
        if let Some(ref m) = manifest {
            let rel = rel_path(dir, &file);
            if let Some(rec) = m.find(&rel) {
                if got == rec.sha256_before {
                    report.hash_ok += 1;
                } else {
                    report.hash_mismatch.push(HashMismatch {
                        path: rel,
                        expected: rec.sha256_before.clone(),
                        got,
                    });
                }
            }
        }
    }

    if man_path.exists() && report.hash_mismatch.is_empty() {
        let _ = fs::remove_file(&man_path);
    }

    Ok(report)
}

fn check_hash(
    manifest: &Option<Manifest>,
    dir: &Path,
    file: &Path,
    cleaned: &str,
    report: &mut StripReport,
    _binary: bool,
) {
    if let Some(ref m) = manifest {
        let rel = rel_path(dir, file);
        if let Some(rec) = m.find(&rel) {
            let got = sha256_str(cleaned);
            if got == rec.sha256_before {
                report.hash_ok += 1;
            } else {
                report.hash_mismatch.push(HashMismatch {
                    path: rel,
                    expected: rec.sha256_before.clone(),
                    got,
                });
            }
        }
    }
}

#[derive(Debug)]
pub struct HashMismatch {
    pub path: String,
    pub expected: String,
    pub got: String,
}

#[derive(Debug)]
pub struct StripReport {
    pub stripped: usize,
    pub hash_ok: usize,
    pub hash_mismatch: Vec<HashMismatch>,
    pub missing_manifest: bool,
}

#[derive(Debug)]
pub struct VerifyReport {
    pub checked: usize,
    pub ok: usize,
    pub missing_injection: Vec<String>,
    pub hash_mismatch: Vec<HashMismatch>,
    pub no_manifest: bool,
}

pub fn verify_dir(dir: &Path) -> Result<VerifyReport, String> {
    let man_path = dir.join(MANIFEST_NAME);
    if !man_path.exists() {
        return Ok(VerifyReport {
            checked: 0,
            ok: 0,
            missing_injection: Vec::new(),
            hash_mismatch: Vec::new(),
            no_manifest: true,
        });
    }
    let s = fs::read_to_string(&man_path).map_err(|e| e.to_string())?;
    let manifest = Manifest::from_json(&s).map_err(|e| e.to_string())?;

    let mut report = VerifyReport {
        checked: 0,
        ok: 0,
        missing_injection: Vec::new(),
        hash_mismatch: Vec::new(),
        no_manifest: false,
    };

    for rec in &manifest.files {
        report.checked += 1;
        let path = dir.join(&rec.path);
        let is_bin = path.extension().and_then(|e| e.to_str()) == Some("wasm");
        if is_bin {
            let content = match fs::read(&path) {
                Ok(c) => c,
                Err(_) => {
                    report.missing_injection.push(rec.path.clone());
                    continue;
                }
            };
            if !has_revguard_section(&content) {
                report.missing_injection.push(rec.path.clone());
                continue;
            }
            let got = sha256_hex(&content);
            if got == rec.sha256_after {
                report.ok += 1;
            } else {
                report.hash_mismatch.push(HashMismatch {
                    path: rec.path.clone(),
                    expected: rec.sha256_after.clone(),
                    got,
                });
            }
        } else {
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => {
                    report.missing_injection.push(rec.path.clone());
                    continue;
                }
            };
            let injected = is_injected(&content) || is_sourcemap_injected(&content);
            if !injected {
                report.missing_injection.push(rec.path.clone());
                continue;
            }
            let got = sha256_str(&content);
            if got == rec.sha256_after {
                report.ok += 1;
            } else {
                report.hash_mismatch.push(HashMismatch {
                    path: rec.path.clone(),
                    expected: rec.sha256_after.clone(),
                    got,
                });
            }
        }
    }
    Ok(report)
}
