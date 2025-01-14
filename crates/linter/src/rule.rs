use std::fmt::Debug;

use mago_ast::Program;
use mago_reporting::Level;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::settings::RuleSettings;

/// A `ConfiguredRule` is created after the user’s configuration is applied. If the user
/// chooses to enable a rule and set certain options or a custom severity level, those
/// details are stored here. The linter then uses this configured instance to run the rule
/// and produce issues accordingly.
#[derive(Debug)]
pub struct ConfiguredRule {
    /// The unique identifier for this rule in the format `"plugin_slug/rule_slug"`.
    ///
    /// For example, if the plugin is `"analysis"` and the rule is `"instantiation"`,
    /// the slug might be `"analysis/instantiation"`. This is often used to look up
    /// the rule’s settings or to reference it in messages and CLI commands.
    pub slug: String,

    /// The **effective level** at which this rule is enforced, such as [`Level::Error`],
    /// [`Level::Warning`], [`Level::Help`], or [`Level::Note`].
    ///
    /// This is determined by either the rule’s default level or an override from
    /// user configuration. If a rule is effectively disabled, it will not appear
    /// as a `ConfiguredRule` in the linter (thus it doesn’t run at all).
    pub level: Level,

    /// The complete **configuration settings** for this rule, as specified or overridden
    /// by the user (e.g., in a TOML file). These settings might include:
    pub settings: RuleSettings,

    /// The **actual rule implementation**, containing the detection and reporting logic.
    ///
    /// This is typically your struct that implements [`Rule`], including methods
    /// to walk the AST and generate [`Issue`](mago_reporting::Issue) objects if
    /// a violation is found. The linter invokes [`Rule::lint`] on this object
    /// when applying the rule.
    pub rule: Box<dyn Rule>,
}

/// A trait representing a single linting rule.
///
/// A `Rule` defines the logic for checking a program or individual nodes within the AST (Abstract Syntax Tree)
/// for specific patterns or issues and reporting diagnostics if any are found.
///
/// Implementors of this trait should provide the rule's name and the logic for checking programs and nodes.
pub trait Rule: for<'a> Walker<LintContext<'a>> + Send + Sync + Debug {
    fn get_definition(&self) -> RuleDefinition;

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
