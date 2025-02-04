/// A directive that instructs the linter on how to proceed after inspecting a node.
///
/// When a rule inspects a node using the `lint_node` method, it returns a `LintDirective`
/// to control the subsequent linting flow:
///
/// - **`Continue`**:
///   Process the current node as usual and then inspect its children. This is the default behavior.
///
/// - **`Prune`**:
///   Process the current node but do not inspect its children. Use this when the rule has fully
///   handled the node and further linting of its descendants is unnecessary.
///
/// - **`Abort`**:
///   Immediately stop the entire linting process. Use this when a critical condition is met and
///   no further analysis is required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LintDirective {
    /// Continue processing the node and its children.
    Continue,
    /// Process the node but prune (skip) its children.
    Prune,
    /// Abort the entire linting process immediately.
    Abort,
}

impl LintDirective {
    /// Returns `true` if the directive is `Continue`.
    pub fn is_continue(self) -> bool {
        matches!(self, LintDirective::Continue)
    }

    /// Returns `true` if the directive is `Prune`.
    pub fn is_prune(self) -> bool {
        matches!(self, LintDirective::Prune)
    }

    /// Returns `true` if the directive is `Abort`.
    pub fn is_abort(self) -> bool {
        matches!(self, LintDirective::Abort)
    }
}

impl Default for LintDirective {
    #[inline]
    fn default() -> Self {
        LintDirective::Continue
    }
}
