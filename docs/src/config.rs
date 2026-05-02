use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SiteConfig {
    pub site_name: String,
    pub site_description: String,
    pub default_language: String,
    pub languages: Vec<LanguageConfig>,
    pub accent_color: String,
    pub base_url: String,
    pub source_edit_url: String,
    #[serde(default)]
    pub nav_sections: Vec<String>,
    #[serde(default)]
    pub nav_subsections: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct LanguageConfig {
    pub code: String,
    pub name: String,
    pub dir: String,
}

impl SiteConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let content =
            fs::read_to_string(path).with_context(|| format!("failed to read site config from {}", path.display()))?;

        toml::from_str(&content).context("failed to parse site config TOML")
    }

    #[must_use]
    pub fn is_rtl(&self, language_code: &str) -> bool {
        self.languages
            .iter()
            .find(|language| language.code == language_code)
            .is_some_and(|language| language.dir.eq_ignore_ascii_case("rtl"))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedirectsFile {
    pub redirect: Vec<RedirectRule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedirectRule {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub preserve_hash: bool,
    #[serde(default)]
    pub preserve_query: bool,
}

pub fn load_redirects(path: &Path) -> Result<Vec<RedirectRule>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read redirects file from {}", path.display()))?;

    let parsed: RedirectsFile = toml::from_str(&content).context("failed to parse redirects TOML")?;
    Ok(parsed.redirect)
}
