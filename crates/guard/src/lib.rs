use mago_codex::metadata::CodebaseMetadata;
use mago_names::ResolvedNames;
use mago_syntax::ast::Program;
use mago_syntax::walker::MutWalker;

use crate::context::GuardContext;
use crate::perimeter::DependenciesGuardWalker;
use crate::report::FortressReport;
use crate::settings::Settings;
use crate::structural::StructuralGuardWalker;

pub mod path;
pub mod report;
pub mod settings;

mod context;
mod matcher;
mod perimeter;
mod structural;

#[derive(Debug)]
pub struct ArchitecturalGuard {
    settings: Settings,
}

impl ArchitecturalGuard {
    /// Creates a new Guard instance.
    ///
    /// # Arguments
    ///
    /// * `arena` - The bump allocator to use for memory management
    /// * `settings` - The guard settings containing architectural rules
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    /// Performs architectural boundary checking on a program.
    ///
    /// # Arguments
    ///
    /// * `codebase` - The codebase metadata for symbol lookups
    /// * `program` - The AST of the program
    /// * `resolved_names` - The resolved names for the program
    ///
    /// # Returns
    ///
    /// A `GuardResult` containing all violations found.
    pub fn check<'ast, 'arena>(
        &self,
        codebase: &CodebaseMetadata,
        program: &'ast Program<'arena>,
        resolved_names: &'ast ResolvedNames<'arena>,
    ) -> FortressReport {
        let mut context = GuardContext::new(resolved_names, &self.settings, codebase);

        DependenciesGuardWalker.walk_program(program, &mut context);
        StructuralGuardWalker.walk_program(program, &mut context);

        context.report()
    }
}
