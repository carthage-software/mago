use mago_interner::ThreadedInterner;
use mago_names::ResolvedNames;
use mago_php_version::PHPVersion;
use mago_reporting::IssueCollection;
use mago_source::Source;
use mago_syntax::ast::Program;
use mago_syntax::walker::Walker;

use crate::internal::CheckingWalker;
use crate::internal::context::Context;

mod internal;

/// The main entry point for performing semantic analysis on a PHP program's AST.
///
/// This checker is responsible for traversing the Abstract Syntax Tree (AST)
/// and validating the code against a set of semantic rules, such as type correctness,
/// variable usage, and adherence to language features for a specific PHP version.
#[derive(Debug, Clone, Copy)]
pub struct SemanticsChecker<'a> {
    version: &'a PHPVersion,
    interner: &'a ThreadedInterner,
}

impl<'a> SemanticsChecker<'a> {
    /// Creates a new `SemanticsChecker`.
    ///
    /// # Arguments
    ///
    /// - `version`: The target PHP version to check against.
    /// - `interner`: A reference to the string interner for efficient string handling.
    pub fn new(version: &'a PHPVersion, interner: &'a ThreadedInterner) -> Self {
        Self { version, interner }
    }

    /// Analyzes the given program AST for semantic issues.
    ///
    /// This method walks the entire AST, applying semantic rules and collecting any
    /// violations it finds.
    ///
    /// # Arguments
    ///
    /// - `source`: The source file being analyzed.
    /// - `program`: The root of the AST for the program.
    /// - `names`: The resolved names (e.g., fully qualified class names) from the name resolution pass.
    ///
    /// # Returns
    ///
    /// An `IssueCollection` containing all semantic issues discovered during the check.
    pub fn check(&self, source: &Source, program: &Program, names: &ResolvedNames) -> IssueCollection {
        let mut context = Context::new(self.interner, self.version, program, names, source);

        CheckingWalker.walk_program(program, &mut context);

        context.issues
    }
}
