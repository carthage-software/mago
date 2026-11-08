use schemars::JsonSchema;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct PrinterSettings {
    pub print_width: usize,
    pub tab_width: usize,
    pub use_tabs: bool,
    pub end_of_line: EndOfLine,
}

impl Default for PrinterSettings {
    fn default() -> Self {
        Self { print_width: 120, tab_width: 4, use_tabs: false, end_of_line: EndOfLine::default() }
    }
}

/// The line endings used for generated line breaks.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum EndOfLine {
    #[default]
    #[cfg_attr(feature = "serde", serde(alias = "Auto"))]
    Auto,
    #[cfg_attr(feature = "serde", serde(alias = "Lf"))]
    Lf,
    #[cfg_attr(feature = "serde", serde(alias = "Crlf"))]
    Crlf,
    #[cfg_attr(feature = "serde", serde(alias = "Cr"))]
    Cr,
}

impl EndOfLine {
    #[inline]
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Crlf => "\r\n",
            Self::Cr => "\r",
            Self::Lf | Self::Auto => "\n",
        }
    }
}
