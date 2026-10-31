//! RevGuard manifest: records every injection so strip can precisely revert.

use serde::{Deserialize, Serialize};

pub const MANIFEST_NAME: &str = "revguard.manifest.json";
pub const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InjectionKind {
    Banner,
    DeadCode,
    SourceMap,
    WasmSection,
    RustSrc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionRecord {
    pub kind: InjectionKind,
    /// Phrases embedded by this injection (for audit).
    pub phrases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecord {
    /// Relative path from the injected root.
    pub path: String,
    /// sha256 of the clean (pre-injection) bytes.
    pub sha256_before: String,
    /// sha256 of the injected bytes.
    pub sha256_after: String,
    pub injections: Vec<InjectionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub format_version: u32,
    pub tool_version: String,
    /// Optional wordlist identity used for this inject (path or "builtin").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wordlist_id: Option<String>,
    pub files: Vec<FileRecord>,
}

impl Manifest {
    pub fn new() -> Manifest {
        Manifest {
            format_version: FORMAT_VERSION,
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            wordlist_id: None,
            files: Vec::new(),
        }
    }

    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }

    pub fn from_json(s: &str) -> Result<Manifest, serde_json::Error> {
        serde_json::from_str(s)
    }

    pub fn find(&self, path: &str) -> Option<&FileRecord> {
        self.files.iter().find(|f| f.path == path)
    }
}
