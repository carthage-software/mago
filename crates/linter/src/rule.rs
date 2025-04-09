use std::fmt::Debug;

use mago_reporting::Level;
use mago_syntax::ast::Node;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::directive::LintDirective;
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
/// Implementors of this trait should provide the rule's definition and the logic for checking programs and nodes.
pub trait Rule: Send + Sync + Debug {
    /// Retrieves the definition of this rule.
    ///
    /// # Returns
    ///
    /// A [`RuleDefinition`] object representing the rule.
    fn get_definition(&self) -> RuleDefinition;

    /// Inspects a single AST node and determines how the linting process should proceed.
    ///
    /// This method is called for each node encountered during the linting process. The implementation
    /// should analyze the provided node, report any issues via the supplied context, and then return a
    /// [`LintDirective`] that instructs the linter on how to continue:
    ///
    /// - **`LintDirective::Continue`**:
    ///   Process the current node as usual and then inspect its children. This is the default behavior.
    ///
    /// - **`LintDirective::Prune`**:
    ///   Process the current node but do not inspect its children. Use this when the rule has fully
    ///   handled the node and further analysis of its descendants is unnecessary.
    ///
    /// - **`LintDirective::Abort`**:
    ///   Immediately stop the entire linting process. Use this when a critical condition is met and
    ///   no further linting is required.
    ///
    /// # Arguments
    ///
    /// * `node` - The AST node to be inspected.
    /// * `context` - The linting context.
    ///
    /// # Returns
    ///
    /// A [`LintDirective`] that determines how the linter should proceed after processing this node.
    #[allow(unused_variables)]
    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective;
}

impl Rule for Box<dyn Rule> {
    fn get_definition(&self) -> RuleDefinition {
        self.as_ref().get_definition()
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        self.as_ref().lint_node(node, context)
    }
}
