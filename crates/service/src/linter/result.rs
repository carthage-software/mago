use fennec_reporting::Issue;
use fennec_reporting::Level;
use serde::Deserialize;
use serde::Serialize;

use fennec_reporting::IssueCollection;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct LintResult {
    pub issues: IssueCollection,
}

impl LintResult {
    pub fn new(issues: IssueCollection) -> Self {
        Self { issues }
    }

    pub fn has_errors(&self) -> bool {
        self.issues.iter().any(|issue| issue.level >= Level::Error)
    }

    pub fn only_fixable(self) -> impl Iterator<Item = Issue> {
        self.issues.into_iter().filter(|issue| !issue.suggestions.is_empty())
    }
}

impl IntoIterator for LintResult {
    type Item = Issue;

    type IntoIter = std::vec::IntoIter<Issue>;

    fn into_iter(self) -> Self::IntoIter {
        self.issues.into_iter()
    }
}
