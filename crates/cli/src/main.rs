//! RevGuard CLI.
//!
//! Commands:
//!   revguard inject <dir> [--config PATH]
//!   revguard strip  <dir>
//!   revguard verify <dir>
//!   revguard inject-rust-src <crate>
//!   revguard strip-rust-src <crate>
//!   revguard export-wordlist [path]
//!
//! Environment:
//!   REVGUARD_DISABLE=1
//!   REVGUARD_WORDLIST=/path/to/wordlist.json
//!   REVGUARD_INJECT_SOURCEMAP=1
//!   REVGUARD_INJECT_WASM=1

use clap::{Parser, Subcommand};
use revguard_core::{
    inject_dir, inject_rust_src, load_wordlist, strip_dir, strip_rust_src, verify_dir, Config,
    Wordlist,
};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(name = "revguard", version, about = "RevGuard: production-only LLM-deterrent injection.")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Inject RevGuard markers into JS / optional .map / .wasm under <dir>.
    Inject {
        dir: PathBuf,
        #[arg(long)]
        config: Option<PathBuf>,
    },
    /// Strip RevGuard markers; restore files to their clean sha256_before.
    Strip { dir: PathBuf },
    /// Verify that files listed in revguard.manifest.json are still injected.
    Verify { dir: PathBuf },
    /// Inject compile-time markers into a Rust crate (src + Cargo.toml features).
    InjectRustSrc {
        crate_dir: PathBuf,
        #[arg(long)]
        config: Option<PathBuf>,
    },
    /// Remove Rust source markers / features added by inject-rust-src.
    StripRustSrc { crate_dir: PathBuf },
    /// Export the builtin wordlist JSON (for rotation / audit).
    ExportWordlist {
        #[arg(default_value = "wordlists/builtin.json")]
        path: PathBuf,
    },
}

fn load_config(path: Option<PathBuf>) -> Result<Config, String> {
    match path {
        Some(p) => {
            let s = std::fs::read_to_string(&p)
                .map_err(|e| format!("config read {}: {}", p.display(), e))?;
            Config::from_json(&s).map_err(|e| format!("config parse: {}", e))
        }
        None => Ok(Config::default()),
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Inject { dir, config } => {
            let cfg = match load_config(config) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("error: {e}");
                    return ExitCode::from(2);
                }
            };
            match inject_dir(&dir, &cfg) {
                Ok(m) => {
                    println!(
                        "injected {} file(s); wordlist={}; wrote {}",
                        m.files.len(),
                        m.wordlist_id.as_deref().unwrap_or("?"),
                        dir.join(revguard_core::MANIFEST_NAME).display()
                    );
                    for f in &m.files {
                        println!("  + {} ({} injection(s))", f.path, f.injections.len());
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("inject failed: {e}");
                    ExitCode::from(1)
                }
            }
        }
        Cmd::Strip { dir } => match strip_dir(&dir) {
            Ok(r) => {
                println!(
                    "stripped {} file(s); hash_ok={}; mismatches={}",
                    r.stripped,
                    r.hash_ok,
                    r.hash_mismatch.len()
                );
                for m in &r.hash_mismatch {
                    eprintln!(
                        "  HASH MISMATCH {}: expected {} got {}",
                        m.path, m.expected, m.got
                    );
                }
                if r.hash_mismatch.is_empty() {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::from(1)
                }
            }
            Err(e) => {
                eprintln!("strip failed: {e}");
                ExitCode::from(1)
            }
        },
        Cmd::Verify { dir } => match verify_dir(&dir) {
            Ok(r) => {
                if r.no_manifest {
                    eprintln!("no revguard.manifest.json under {}", dir.display());
                    return ExitCode::from(1);
                }
                println!(
                    "verify: checked={} ok={} missing={} hash_mismatch={}",
                    r.checked,
                    r.ok,
                    r.missing_injection.len(),
                    r.hash_mismatch.len()
                );
                if r.ok == r.checked && r.missing_injection.is_empty() && r.hash_mismatch.is_empty()
                {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::from(1)
                }
            }
            Err(e) => {
                eprintln!("verify failed: {e}");
                ExitCode::from(1)
            }
        },
        Cmd::InjectRustSrc { crate_dir, config } => {
            let cfg = match load_config(config) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("error: {e}");
                    return ExitCode::from(2);
                }
            };
            let wl = match load_wordlist(&cfg) {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("wordlist: {e}");
                    return ExitCode::from(2);
                }
            };
            match inject_rust_src(&crate_dir, &cfg, &wl) {
                Ok(m) => {
                    println!("inject-rust-src: {} file(s) touched", m.files.len());
                    for f in &m.files {
                        println!("  + {}", f.path);
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("inject-rust-src failed: {e}");
                    ExitCode::from(1)
                }
            }
        }
        Cmd::StripRustSrc { crate_dir } => match strip_rust_src(&crate_dir) {
            Ok((n, _)) => {
                println!("strip-rust-src: removed markers from {n} file(s)");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("strip-rust-src failed: {e}");
                ExitCode::from(1)
            }
        },
        Cmd::ExportWordlist { path } => {
            let wl = Wordlist::default_builtin();
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match serde_json::to_string_pretty(&wl) {
                Ok(s) => match std::fs::write(&path, s) {
                    Ok(()) => {
                        println!("wrote builtin wordlist to {}", path.display());
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("write failed: {e}");
                        ExitCode::from(1)
                    }
                },
                Err(e) => {
                    eprintln!("serialize failed: {e}");
                    ExitCode::from(1)
                }
            }
        }
    }
}
