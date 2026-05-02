use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::Context;
use anyhow::Result;

pub type I18nMap = BTreeMap<String, String>;

pub fn load_languages(base: &Path, language_codes: &[String]) -> Result<BTreeMap<String, I18nMap>> {
    let mut map = BTreeMap::new();

    for language_code in language_codes {
        let path = base.join(format!("{language_code}.toml"));
        let content =
            fs::read_to_string(&path).with_context(|| format!("failed to read i18n file {}", path.display()))?;

        let parsed: I18nMap =
            toml::from_str(&content).with_context(|| format!("failed to parse i18n file {}", path.display()))?;

        map.insert(language_code.clone(), parsed);
    }

    Ok(map)
}

#[must_use]
pub fn t(i18n: &I18nMap, key: &str, fallback: &str) -> String {
    i18n.get(key).cloned().unwrap_or_else(|| fallback.to_string())
}

#[must_use]
pub fn t_format(i18n: &I18nMap, key: &str, fallback: &str, args: &[(&str, &str)]) -> String {
    let template = i18n.get(key).map(String::as_str).unwrap_or(fallback);
    let mut out = template.to_string();
    for (placeholder, value) in args {
        out = out.replace(&format!("{{{placeholder}}}"), value);
    }
    out
}
