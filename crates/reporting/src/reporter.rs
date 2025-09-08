use std::str::FromStr;

use pager::Pager;
use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use strum::VariantNames;

use mago_database::ReadDatabase;

use crate::Issue;
use crate::IssueCollection;
use crate::Level;
use crate::error::ReportingError;
use crate::internal::emitter::Emitter;
use crate::internal::writer::ReportWriter;

/// Defines the output target for the `ReportWriter`.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, VariantNames)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ReportingTarget {
    /// Direct output to standard output (stdout).
    #[default]
    Stdout,
    /// Direct output to standard error (stderr).
    Stderr,
}

/// The format to use when writing the report.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, VariantNames)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ReportingFormat {
    #[default]
    Rich,
    Medium,
    Short,
    Ariadne,
    Github,
    Gitlab,
    Json,
    Count,
    CodeCount,
    Checkstyle,
    Emacs,
}

impl ReportingFormat {
    /// Returns `true` if the reporting format supports being displayed in a pager.
    pub fn supports_paging(&self) -> bool {
        match self {
            // These formats are meant for human consumption in a terminal.
            Self::Rich | Self::Medium | Self::Short | Self::Ariadne | Self::Emacs => true,

            // These formats are for CI/CD, machine-readable, or produce short summaries.
            Self::Github | Self::Gitlab | Self::Json | Self::Count | Self::CodeCount | Self::Checkstyle => false,
        }
    }
}

#[derive(Debug)]
pub struct Reporter {
    database: ReadDatabase,
    target: ReportingTarget,
    with_colors: bool,
    use_pager: bool,
    pager: Option<String>,
}

impl Reporter {
    pub fn new(
        manager: ReadDatabase,
        target: ReportingTarget,
        with_colors: bool,
        use_pager: bool,
        pager: Option<String>,
    ) -> Self {
        Self { database: manager, target, with_colors, use_pager, pager }
    }

    pub fn report(
        &self,
        issues: impl IntoIterator<Item = Issue>,
        format: ReportingFormat,
    ) -> Result<Option<Level>, ReportingError> {
        let issues = IssueCollection::from(issues);
        if issues.is_empty() {
            return Ok(None);
        }

        let highest_level = issues.get_highest_level();

        let mut pager =
            if let Some(pager_name) = &self.pager { Pager::with_default_pager(pager_name) } else { Pager::new() };

        if self.use_pager && self.target == ReportingTarget::Stdout && format.supports_paging() {
            pager.setup();
        }

        let writer = ReportWriter::new(self.target, self.with_colors);
        match format.emit(&mut writer.lock(), &self.database, issues) {
            Ok(_) => Ok(highest_level),
            Err(error) if pager.is_on() && error.is_broken_pipe() => {
                // Silently ignore broken pipe errors when using a pager.
                // This usually means the user quit the pager before reading all the output.
                Ok(highest_level)
            }
            Err(error) => Err(error),
        }
    }
}

impl FromStr for ReportingTarget {
    type Err = ReportingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "stdout" | "out" => Ok(Self::Stdout),
            "stderr" | "err" => Ok(Self::Stderr),
            _ => Err(ReportingError::InvalidTarget(s.to_string())),
        }
    }
}

impl FromStr for ReportingFormat {
    type Err = ReportingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rich" => Ok(Self::Rich),
            "medium" => Ok(Self::Medium),
            "short" => Ok(Self::Short),
            "ariadne" => Ok(Self::Ariadne),
            "github" => Ok(Self::Github),
            "gitlab" => Ok(Self::Gitlab),
            "json" => Ok(Self::Json),
            "count" => Ok(Self::Count),
            "codecode" | "code-count" => Ok(Self::CodeCount),
            "checkstyle" => Ok(Self::Checkstyle),
            "emacs" => Ok(Self::Emacs),
            _ => Err(ReportingError::InvalidFormat(s.to_string())),
        }
    }
}
