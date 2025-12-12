//! Issue filter hook for suppressing issues at the end of analysis.

use mago_database::file::File;
use mago_reporting::Issue;

use crate::plugin::hook::HookResult;
use crate::plugin::provider::Provider;

/// Decision for an issue filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IssueFilterDecision {
    /// Keep the issue (include in output).
    #[default]
    Keep,
    /// Remove the issue (suppress from output).
    Remove,
}

/// Hook for filtering issues at the end of analysis.
///
/// Called for each issue after analysis is complete.
/// This allows plugins to suppress issues based on various criteria:
///
/// - Suppress "unused parameter" for methods with `#[Override]`
/// - Suppress issues in generated code
/// - Framework-specific suppression rules
pub trait IssueFilterHook: Provider {
    /// Filter an issue.
    ///
    /// Called for each issue after analysis is complete.
    /// The issue contains the code as a string in `issue.code`.
    /// Return `IssueFilterDecision::Keep` to keep it,
    /// `IssueFilterDecision::Remove` to suppress it.
    fn filter_issue(&self, file: &File, issue: &Issue) -> HookResult<IssueFilterDecision>;
}
