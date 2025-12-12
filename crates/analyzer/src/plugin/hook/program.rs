//! Program hooks for intercepting program-level analysis.

use mago_database::file::File;
use mago_syntax::ast::Program;

use crate::plugin::context::HookContext;
use crate::plugin::hook::HookAction;
use crate::plugin::hook::HookResult;
use crate::plugin::provider::Provider;

/// Hook trait for intercepting program-level analysis.
///
/// This hook is called before and after analyzing a complete program (file).
/// It receives the source file, full AST, and mutable context, allowing hooks to:
/// - Access file information (id, path, content)
/// - Inspect the entire program structure
/// - Report issues at the program level
/// - Pre-register variables before analysis
/// - Skip analysis entirely
pub trait ProgramHook: Provider {
    /// Called before a program is analyzed.
    ///
    /// Return `HookAction::Continue` to proceed with normal analysis, or
    /// `HookAction::Skip` to skip analysis of this program entirely.
    fn before_program<'ast, 'arena>(
        &self,
        _file: &File,
        _program: &'ast Program<'arena>,
        _context: &mut HookContext<'_, '_>,
    ) -> HookResult<HookAction> {
        Ok(HookAction::Continue)
    }

    /// Called after a program has been analyzed.
    fn after_program<'ast, 'arena>(
        &self,
        _file: &File,
        _program: &'ast Program<'arena>,
        _context: &mut HookContext<'_, '_>,
    ) -> HookResult<()> {
        Ok(())
    }
}
