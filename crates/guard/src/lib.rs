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
    /// * `settings` - The guard settings containing architectural rules
    #[must_use]
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
    /// A `FortressReport` with all violations found and skip status.
    #[must_use]
    pub fn check<'ast, 'arena>(
        &self,
        codebase: &CodebaseMetadata,
        program: &'ast Program<'arena>,
        resolved_names: &'ast ResolvedNames<'arena>,
    ) -> FortressReport {
        let mut context = GuardContext::new(resolved_names, &self.settings, codebase);

        let mut missing_perimeter_configuration = false;
        let mut missing_structural_configuration = false;

        // Run perimeter guard based on settings
        match self.settings.should_run_perimeter() {
            Some(true) => DependenciesGuardWalker.walk_program(program, &mut context),
            Some(false) => missing_perimeter_configuration = true,
            None => {
                // Mode doesn't allow perimeter
            }
        }

        // Run structural guard based on settings
        match self.settings.should_run_structural() {
            Some(true) => StructuralGuardWalker.walk_program(program, &mut context),
            Some(false) => missing_structural_configuration = true,
            None => {
                // Mode doesn't allow structural
            }
        }

        let mut report = context.report();
        report.missing_perimeter_configuration = missing_perimeter_configuration;
        report.missing_structural_configuration = missing_structural_configuration;
        report
    }
}
