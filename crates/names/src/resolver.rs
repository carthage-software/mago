use bumpalo::Bump;
use mago_syntax::ast::Program;
use mago_syntax::walker::MutWalker;

use crate::ResolvedNames;
use crate::internal::context::NameResolutionContext;
use crate::internal::walker::NameWalker;

/// Orchestrates the process of resolving names within a PHP Abstract Syntax Tree (AST).
///
/// This struct acts as the main entry point for the name resolution pass.
/// It requires an arena to store resolved names.
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct NameResolver<'arena> {
    arena: &'arena Bump,
}

impl<'arena> NameResolver<'arena> {
    /// Creates a new `NameResolver` instance.
    pub fn new(arena: &'arena Bump) -> Self {
        NameResolver { arena }
    }

    /// Resolves names within the provided PHP AST `Program`.
    ///
    /// # Arguments
    ///
    /// * `program` - A reference to the root `Program` AST node. The lifetime `'ast`
    ///   ensures the AST outlives the borrowing done within this method.
    ///
    /// # Returns
    ///
    /// A `ResolvedNames` struct containing the mapping of original names/nodes
    /// to their resolved fully qualified names.
    pub fn resolve<'ast>(&self, program: &'ast Program<'arena>) -> ResolvedNames<'arena> {
        let mut context = NameResolutionContext::new(self.arena);
        let mut walker = NameWalker::default();

        walker.walk_program(program, &mut context);

        walker.resolved_names
    }
}
