//! Preset configurations for the `init` command.
//!
//! This module provides predefined formatter configurations that can be used to quickly
//! set up `mago.toml` with common coding standards. Each preset is a complete
//! `[formatter]` TOML section.

use strum_macros::EnumString;

/// Enum representing the available formatter presets.
#[derive(Debug, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum FormatterPreset {
    Laravel,
    Psr12,
    Default,
}

/// Returns the TOML configuration string for the given formatter preset.
///
/// # Arguments
///
/// * `preset` - The `FormatterPreset` to get the configuration for.
///
/// # Returns
///
/// A static string slice containing the TOML configuration for the formatter.
pub fn get_preset_config(preset: FormatterPreset) -> &'static str {
    match preset {
        FormatterPreset::Laravel => LARAVEL_PRESET,
        FormatterPreset::Psr12 => PSR12_PRESET,
        FormatterPreset::Default => DEFAULT_PRESET,
    }
}

/// The default formatter preset, based on PER-CS.
const DEFAULT_PRESET: &str = r#"[formatter]
print-width = 120
tab-width = 4
use-tabs = false
single-quote = true
trailing-comma = true
control-brace-style = "same-line"
empty-line-before-return = true
empty-line-after-opening-brace = false
"#;

/// The PSR-12 formatter preset.
const PSR12_PRESET: &str = r#"[formatter]
print-width = 120
tab-width = 4
use-tabs = false
single-quote = true
trailing-comma = false
control-brace-style = "psr2"
empty-line-before-return = false
empty-line-after-opening-brace = false
"#;

/// The Laravel formatter preset.
const LARAVEL_PRESET: &str = r#"[formatter]
print-width = 120
tab-width = 4
use-tabs = false
single-quote = true
trailing-comma = false
control-brace-style = "same-line"
empty-line-before-return = true
empty-line-after-opening-brace = false
"#;
