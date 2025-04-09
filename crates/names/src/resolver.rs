use mago_interner::ThreadedInterner;
use mago_syntax::ast::Program;
use mago_syntax::walker::MutWalker;

use crate::ResolvedNames;
use crate::internal::context::NameResolutionContext;
use crate::internal::walker::NameWalker;

/// Orchestrates the process of resolving names within a PHP Abstract Syntax Tree (AST).
///
/// This struct acts as the main entry point for the name resolution pass.
/// It requires a string interner to efficiently handle string lookups during
/// the AST traversal.
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct NameResolver<'i> {
    interner: &'i ThreadedInterner,
}

impl<'i> NameResolver<'i> {
    /// Creates a new `NameResolver` instance.
    pub fn new(interner: &'i ThreadedInterner) -> Self {
        Self { interner }
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
    pub fn resolve(&self, program: &Program) -> ResolvedNames {
        let mut context = NameResolutionContext::new(self.interner);
        let mut walker = NameWalker::default();

        walker.walk_program(program, &mut context);

        walker.resolved_names
    }
}
