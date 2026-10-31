//! RevGuard configuration schema + environment overrides.

use serde::{Deserialize, Serialize};
use crate::wordlist::{Category, Intensity};

fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Master switch. If false, nothing is injected.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Categories to draw trigger phrases from.
    #[serde(default = "Config::default_categories")]
    pub categories: Vec<Category>,

    /// Intensity tier ceiling.
    #[serde(default = "Config::default_intensity")]
    pub intensity: Intensity,

    /// Inject the legal-comment banner at the top of files.
    #[serde(default = "default_true")]
    pub inject_banner: bool,

    /// Inject a never-executed dead-code branch.
    #[serde(default = "default_true")]
    pub inject_dead_code: bool,

    /// Inject into source maps (`.map` `sourcesContent`). Default false.
    #[serde(default)]
    pub inject_source_map: bool,

    /// Inject a WASM custom section named `revguard`. Default false.
    #[serde(default)]
    pub inject_wasm_section: bool,

    /// Optional path to an override wordlist JSON file.
    #[serde(default)]
    pub wordlist_path: Option<String>,

    /// Optional wordlist version label (recorded in manifest for rotation audit).
    #[serde(default)]
    pub wordlist_version: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            enabled: true,
            categories: Config::default_categories(),
            intensity: Config::default_intensity(),
            inject_banner: true,
            inject_dead_code: true,
            inject_source_map: false,
            inject_wasm_section: false,
            wordlist_path: None,
            wordlist_version: None,
        }
    }
}

impl Config {
    fn default_categories() -> Vec<Category> {
        Category::all().to_vec()
    }
    fn default_intensity() -> Intensity {
        Intensity::Medium
    }

    pub fn from_json(s: &str) -> Result<Config, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Apply environment-variable overrides. REVGUARD_DISABLE=1 forces off.
    pub fn apply_env(mut self) -> Config {
        if let Ok(v) = std::env::var("REVGUARD_DISABLE") {
            if v == "1" || v.eq_ignore_ascii_case("true") {
                self.enabled = false;
            }
        }
        if let Ok(v) = std::env::var("REVGUARD_WORDLIST") {
            if !v.is_empty() {
                self.wordlist_path = Some(v);
            }
        }
        if let Ok(v) = std::env::var("REVGUARD_INJECT_SOURCEMAP") {
            if v == "1" || v.eq_ignore_ascii_case("true") {
                self.inject_source_map = true;
            }
        }
        if let Ok(v) = std::env::var("REVGUARD_INJECT_WASM") {
            if v == "1" || v.eq_ignore_ascii_case("true") {
                self.inject_wasm_section = true;
            }
        }
        self
    }

    pub fn is_active(&self) -> bool {
        self.enabled
    }

    pub fn wordlist_id(&self) -> String {
        match (&self.wordlist_path, &self.wordlist_version) {
            (Some(p), Some(v)) => format!("{p}@{v}"),
            (Some(p), None) => p.clone(),
            (None, Some(v)) => format!("builtin@{v}"),
            (None, None) => "builtin".into(),
        }
    }
}
