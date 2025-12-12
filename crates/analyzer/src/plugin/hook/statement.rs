//! Statement hooks for intercepting statement analysis.

use mago_syntax::ast::Statement;

use crate::plugin::context::HookContext;
use crate::plugin::hook::HookAction;
use crate::plugin::hook::HookResult;
use crate::plugin::provider::Provider;

/// Hook trait for intercepting statement analysis.
///
/// This hook receives the real AST statement and full mutable context,
/// allowing hooks to inspect statements, report issues, and modify analysis state.
pub trait StatementHook: Provider {
    /// Called before a statement is analyzed.
    ///
    /// Return `HookAction::Continue` to proceed with normal analysis, or
    /// `HookAction::Skip` to skip analysis of this statement.
    fn before_statement<'ast, 'arena>(
        &self,
        _stmt: &'ast Statement<'arena>,
        _context: &mut HookContext<'_, '_>,
    ) -> HookResult<HookAction> {
        Ok(HookAction::Continue)
    }

    /// Called after a statement has been analyzed.
    fn after_statement<'ast, 'arena>(
        &self,
        _stmt: &'ast Statement<'arena>,
        _context: &mut HookContext<'_, '_>,
    ) -> HookResult<()> {
        Ok(())
    }
}
