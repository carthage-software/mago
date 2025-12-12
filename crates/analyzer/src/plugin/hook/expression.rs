//! Expression hooks for intercepting expression analysis.

use mago_syntax::ast::Expression;

use crate::plugin::context::HookContext;
use crate::plugin::hook::ExpressionHookResult;
use crate::plugin::hook::HookResult;
use crate::plugin::provider::Provider;

/// Hook trait for intercepting expression analysis.
///
/// This hook receives the real AST expression and full mutable context,
/// allowing hooks to inspect expressions, report issues, modify analysis state,
/// and optionally skip analysis with a custom type.
pub trait ExpressionHook: Provider {
    /// Called before an expression is analyzed.
    ///
    /// Return `ExpressionHookResult::Continue` to proceed with normal analysis,
    /// `ExpressionHookResult::Skip` to skip analysis (type will be `mixed`), or
    /// `ExpressionHookResult::SkipWithType(ty)` to skip with a custom type.
    fn before_expression<'ast, 'arena>(
        &self,
        _expr: &'ast Expression<'arena>,
        _context: &mut HookContext<'_, '_>,
    ) -> HookResult<ExpressionHookResult> {
        Ok(ExpressionHookResult::Continue)
    }

    /// Called after an expression has been analyzed.
    fn after_expression<'ast, 'arena>(
        &self,
        _expr: &'ast Expression<'arena>,
        _context: &mut HookContext<'_, '_>,
    ) -> HookResult<()> {
        Ok(())
    }
}
