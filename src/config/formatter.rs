use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_formatter::presets::FormatterPreset;
use mago_formatter::settings::*;

/// Configuration options for formatting source code.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case", from = "RawFormatterConfiguration", deny_unknown_fields)]
#[schemars(!from)]
pub struct FormatterConfiguration {
    /// A list of patterns to exclude from formatting.
    ///
    /// Defaults to `[]`.
    pub excludes: Vec<String>,

    /// The resolved formatter settings, already merged with preset if one was specified.
    ///
    /// This field is not serialized/deserialized directly - it's computed from `preset` and
    /// individual settings during deserialization.
    #[serde(skip_serializing)]
    pub settings: FormatSettings,
}

/// Intermediate struct used for deserialization before merging with presets.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct RawFormatterConfiguration {
    #[serde(default)]
    excludes: Vec<String>,
    #[serde(default)]
    preset: Option<FormatterPreset>,
    #[serde(flatten)]
    settings: RawFormatSettings,
}

impl From<RawFormatterConfiguration> for FormatterConfiguration {
    fn from(raw: RawFormatterConfiguration) -> Self {
        let base = raw.preset.map(|p| p.settings()).unwrap_or_default();
        let settings = raw.settings.merge_with(base);

        Self { excludes: raw.excludes, settings }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mago_formatter::presets::FormatterPreset;

    #[test]
    fn test_deserialize_without_preset() {
        let toml = r#"
            print-width = 100
            tab-width = 2
            use-tabs = true
        "#;

        let config: FormatterConfiguration = toml::from_str(toml).unwrap();
        assert_eq!(config.settings.print_width, 100);
        assert_eq!(config.settings.tab_width, 2);
        assert!(config.settings.use_tabs);
    }

    #[test]
    fn test_deserialize_with_preset() {
        let toml = r#"
            preset = "default"
        "#;

        let config: FormatterConfiguration = toml::from_str(toml).unwrap();
        let default_preset = FormatterPreset::Default.settings();

        assert_eq!(config.settings.print_width, default_preset.print_width);
        assert_eq!(config.settings.tab_width, default_preset.tab_width);
        assert_eq!(config.settings.use_tabs, default_preset.use_tabs);
    }

    #[test]
    fn test_deserialize_with_preset_and_override() {
        let toml = r#"
            preset = "default"
            print-width = 100
            tab-width = 2
        "#;

        let config: FormatterConfiguration = toml::from_str(toml).unwrap();
        let default_preset = FormatterPreset::Default.settings();

        // Overridden values should be used
        assert_eq!(config.settings.print_width, 100);
        assert_eq!(config.settings.tab_width, 2);

        // Non-overridden values should use preset
        assert_eq!(config.settings.use_tabs, default_preset.use_tabs);
        assert_eq!(config.settings.single_quote, default_preset.single_quote);
    }

    #[test]
    fn test_deserialize_with_preset_override_matching_default() {
        // Tests that when a value is explicitly set (even if it matches FormatSettings::default),
        // it should override the preset value
        let toml = r#"
            preset = "default"
            print-width = 80
        "#;

        let config: FormatterConfiguration = toml::from_str(toml).unwrap();
        let default_preset = FormatterPreset::Default.settings();

        // The preset has print_width = 120, but we set it to 80
        assert_eq!(config.settings.print_width, 80);
        assert_ne!(config.settings.print_width, default_preset.print_width);
    }

    #[test]
    fn test_deserialize_with_invalid_preset() {
        let toml = r#"
            preset = "invalid-preset"
        "#;

        let result: Result<FormatterConfiguration, _> = toml::from_str(toml);
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.contains("unknown preset") || error.contains("invalid-preset"));
    }

    #[test]
    fn test_deserialize_with_excludes() {
        let toml = r#"
            preset = "default"
            excludes = ["vendor/**", "node_modules/**"]
        "#;

        let config: FormatterConfiguration = toml::from_str(toml).unwrap();
        assert_eq!(config.excludes.len(), 2);
        assert_eq!(config.excludes[0], "vendor/**");
        assert_eq!(config.excludes[1], "node_modules/**");
    }

    #[test]
    fn test_deserialize_psr12_preset() {
        let toml = r#"
            preset = "psr-12"
        "#;

        let config: FormatterConfiguration = toml::from_str(toml).unwrap();
        let psr12_preset = FormatterPreset::Psr12.settings();

        assert_eq!(config.settings.print_width, psr12_preset.print_width);
        assert_eq!(config.settings.tab_width, psr12_preset.tab_width);
    }

    #[test]
    fn test_deserialize_pint_preset() {
        let toml = r#"
            preset = "pint"
        "#;

        let config: FormatterConfiguration = toml::from_str(toml).unwrap();
        let pint_preset = FormatterPreset::Pint.settings();

        assert_eq!(config.settings.print_width, pint_preset.print_width);
        assert_eq!(config.settings.tab_width, pint_preset.tab_width);
    }

    #[test]
    fn test_deserialize_tempest_preset() {
        let toml = r#"
            preset = "tempest"
        "#;

        let config: FormatterConfiguration = toml::from_str(toml).unwrap();
        let tempest_preset = FormatterPreset::Tempest.settings();

        assert_eq!(config.settings.print_width, tempest_preset.print_width);
        assert_eq!(config.settings.print_width, 180);
        assert!(config.settings.space_after_logical_not_unary_prefix_operator);
        assert!(config.settings.preserve_breaking_member_access_chain);
        assert!(!config.settings.empty_line_after_opening_tag);
    }

    #[test]
    fn test_deserialize_pint_preset_with_override() {
        let toml = r#"
            preset = "pint"
            print-width = 100
            use-tabs = true
        "#;

        let config: FormatterConfiguration = toml::from_str(toml).unwrap();
        let pint_preset = FormatterPreset::Pint.settings();

        // Overridden values should be used
        assert_eq!(config.settings.print_width, 100);
        assert!(config.settings.use_tabs);

        // Verify these differ from Pint preset
        assert_ne!(config.settings.print_width, pint_preset.print_width);
        assert_ne!(config.settings.use_tabs, pint_preset.use_tabs);

        // Non-overridden values should use Pint preset
        assert_eq!(config.settings.single_quote, pint_preset.single_quote);
        assert_eq!(config.settings.trailing_comma, pint_preset.trailing_comma);
    }

    #[test]
    fn test_explicit_value_overrides_preset_even_if_matching_default() {
        // When a user explicitly sets a value (even if it matches FormatSettings::default()),
        // it should override the preset value.
        let toml = r#"
            preset = "pint"
            end-of-line = "auto"
        "#;

        let config: FormatterConfiguration = toml::from_str(toml).unwrap();
        let pint_preset = FormatterPreset::Pint.settings();

        // User's explicit value "auto" should be used, not the preset value "lf"
        assert_eq!(config.settings.end_of_line, EndOfLine::Auto);
        assert_ne!(config.settings.end_of_line, pint_preset.end_of_line);

        // Other Pint preset values should still be used for non-overridden fields
        assert_eq!(config.settings.print_width, pint_preset.print_width);
        assert_eq!(config.settings.tab_width, pint_preset.tab_width);
    }

    #[test]
    fn test_issue_1010_explicit_default_value_overrides_preset() {
        let toml = r#"
            preset = "hack"
            print-width = 120
        "#;

        let config: FormatterConfiguration = toml::from_str(toml).unwrap();
        assert_eq!(config.settings.print_width, 120);
    }
}
