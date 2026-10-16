use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

/// Analyzer strictness levels offered by `mago init`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnalyzerStrictnessPreset {
    /// Minimal checks for legacy codebases or gradual adoption.
    Relaxed,
    /// Sensible defaults for most projects.
    #[default]
    Balanced,
    /// Most checks for clean, well-maintained codebases.
    Strict,
    /// All checks enabled.
    Maximum,
}

/// Analyzer settings selected by an initialization strictness preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AnalyzerStrictnessPresetSettings {
    /// Whether the preset enables checking whether documented `@throws` types can escape.
    pub find_overly_wide_throw_types: bool,
}

impl AnalyzerStrictnessPresetSettings {
    /// Returns the TOML assignment for this setting.
    #[must_use]
    pub fn to_toml_assignment(self) -> String {
        format!("find-overly-wide-throw-types = {}", self.find_overly_wide_throw_types)
    }
}

impl Display for AnalyzerStrictnessPreset {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Relaxed => write!(f, "Relaxed - Minimal checks, good for legacy codebases"),
            Self::Balanced => write!(f, "Balanced - Sensible defaults for most projects"),
            Self::Strict => write!(f, "Strict - Enable most checks for clean codebases"),
            Self::Maximum => write!(f, "Maximum - All checks enabled, strictest settings"),
        }
    }
}

impl AnalyzerStrictnessPreset {
    /// Returns the analyzer settings selected by this preset.
    #[must_use]
    pub const fn settings(self) -> AnalyzerStrictnessPresetSettings {
        AnalyzerStrictnessPresetSettings { find_overly_wide_throw_types: matches!(self, Self::Strict | Self::Maximum) }
    }
}
