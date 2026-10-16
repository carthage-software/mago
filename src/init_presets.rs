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
    /// Whether this preset enables the overly wide `@throws` check by default.
    pub const fn find_overly_wide_throw_types(self) -> bool {
        matches!(self, Self::Strict | Self::Maximum)
    }
}
