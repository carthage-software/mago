use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use strum::VariantNames;

use mago_interner::ThreadedInterner;
use mago_source::SourceManager;

use crate::error::ReportingError;
use crate::internal::emitter::Emitter;
use crate::internal::writer::ReportWriter;
use crate::Issue;
use crate::IssueCollection;
use crate::Level;

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
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ReportingFormat {
    #[default]
    Rich,
    Medium,
    Short,
    Github,
    Gitlab,
    Json,
    Count,
    Checkstyle,
    Emacs,
}

#[derive(Clone)]
pub struct Reporter {
    interner: ThreadedInterner,
    manager: SourceManager,
    target: ReportingTarget,
    writer: ReportWriter,
}

impl Reporter {
    pub fn new(interner: ThreadedInterner, manager: SourceManager, target: ReportingTarget) -> Self {
        Self { interner, manager, target, writer: ReportWriter::new(target) }
    }

    pub fn report(
        &self,
        issues: impl IntoIterator<Item = Issue>,
        format: ReportingFormat,
    ) -> Result<Option<Level>, ReportingError> {
        format.emit(&mut self.writer.lock(), &self.manager, &self.interner, IssueCollection::from(issues))
    }
}

unsafe impl Send for Reporter {}
unsafe impl Sync for Reporter {}

impl std::fmt::Debug for Reporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Reporter")
            .field("interner", &self.interner)
            .field("manager", &self.manager)
            .field("target", &self.target)
            .finish_non_exhaustive()
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
            "github" => Ok(Self::Github),
            "gitlab" => Ok(Self::Gitlab),
            "json" => Ok(Self::Json),
            "count" => Ok(Self::Count),
            "checkstyle" => Ok(Self::Checkstyle),
            "emacs" => Ok(Self::Emacs),
            _ => Err(ReportingError::InvalidFormat(s.to_string())),
        }
    }
}
