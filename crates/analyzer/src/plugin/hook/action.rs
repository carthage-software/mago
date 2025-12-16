//! Hook action and result types.

use mago_codex::ttype::union::TUnion;

use crate::plugin::error::PluginError;

pub type HookResult<T> = Result<T, PluginError>;

/// Action to take after a hook runs (for statement hooks).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HookAction {
    #[default]
    Continue,
    Skip,
}

/// Result type for expression hooks that can provide a custom type when skipping.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ExpressionHookResult {
    /// Continue with normal analysis.
    #[default]
    Continue,
    /// Skip normal analysis (expression type will be `mixed`).
    Skip,
    /// Skip normal analysis and use the provided type.
    SkipWithType(TUnion),
}

impl ExpressionHookResult {
    /// Returns true if this result indicates the hook wants to skip analysis.
    #[inline]
    #[must_use]
    pub fn should_skip(&self) -> bool {
        !matches!(self, ExpressionHookResult::Continue)
    }

    /// Returns the type to use if `SkipWithType`, otherwise `None`.
    #[inline]
    #[must_use]
    pub fn take_type(self) -> Option<TUnion> {
        match self {
            ExpressionHookResult::SkipWithType(ty) => Some(ty),
            _ => None,
        }
    }
}
