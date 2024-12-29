use std::fmt::Debug;

use mago_ast::Program;
use mago_reporting::Level;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::settings::RuleSettings;

#[derive(Debug)]
pub struct ConfiguredRule {
    pub level: Level,
    pub settings: RuleSettings,
    pub plugin: String,
    pub rule: Box<dyn Rule>,
}

/// A trait representing a single linting rule.
///
/// A `Rule` defines the logic for checking a program or individual nodes within the AST (Abstract Syntax Tree)
/// for specific patterns or issues and reporting diagnostics if any are found.
///
/// Implementors of this trait should provide the rule's name and the logic for checking programs and nodes.
pub trait Rule: for<'a> Walker<LintContext<'a>> + Send + Sync + Debug {
    /// Returns the name of this rule.
    ///
    /// # Returns
    ///
    /// A static string slice representing the name of this rule.
    ///
    /// This name is used in configurations to enable or disable the rule.
    fn get_name(&self) -> &'static str;

    #[inline]
    fn get_default_level(&self) -> Option<Level> {
        Some(Level::Error)
    }

    /// Lint the entire program for this rule.
    ///
    /// This method is called to apply the rule to the whole [`Program`] AST.
    ///
    /// Note: the default implementation skips linting for non-user-defined programs, if a rule needs to lint
    /// non-user-defined programs, it should override this method.
    ///
    /// # Arguments
    ///
    /// * `program` - The abstract syntax tree (AST) of the program to be linted.
    /// * `configuration` - The configuration for this specific rule.
    /// * `context` - The context for the linting process, which may contain shared state.
    fn lint(&self, program: &Program, context: &mut LintContext<'_>) {
        if !program.source.category().is_user_defined() {
            // Skip linting for non-user-defined programs by default
            //
            // Rules that need to lint non-user-defined programs should override this method
            return;
        }

        self.walk_program(program, context);
    }
}
