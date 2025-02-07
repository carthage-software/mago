use mago_ast::Node;
use mago_interner::ThreadedInterner;
use mago_php_version::PHPVersion;
use mago_reflection::CodebaseReflection;
use mago_reporting::IssueCollection;
use mago_semantics::Semantics;

use crate::ast::AstNode;
use crate::context::LintContext;
use crate::ignore::get_ignores;
use crate::ignore::IgnoreDirective;
use crate::rule::ConfiguredRule;

/// The `Runner` is responsible for executing a lint rule on the AST of a PHP program.
///
/// It holds contextual data such as the PHP version, interner, codebase, semantics, and a collection
/// of issues discovered during linting. To optimize repeated calls to [`Node::children()`], the runner
/// precomputes a tree representation of the AST using the [`AstNode`] structure.
///
/// # Usage
///
/// 1. Create a new runner via [`Runner::new`].
/// 2. For each configured lint rule, call [`Runner::run`].
/// 3. After processing all rules, call [`Runner::finish`] to retrieve the reported issues.
pub struct Runner<'a> {
    php_version: PHPVersion,
    interner: &'a ThreadedInterner,
    codebase: &'a CodebaseReflection,
    semantics: &'a Semantics,
    issues: IssueCollection,
    ast: AstNode<'a>,
    ignores: Vec<IgnoreDirective<'a>>,
}

impl<'a> Runner<'a> {
    /// Creates a new `Runner` instance.
    ///
    /// This method converts the program AST (found in `semantics`) into a precomputed tree
    /// representation to avoid repeated calls to [`Node::children()`] during linting.
    ///
    /// # Parameters
    ///
    /// - `php_version`: The PHP version used during linting.
    /// - `interner`: A reference to the threaded interner for resolving interned strings.
    /// - `codebase`: A reference to the codebase reflection, providing additional context.
    /// - `semantics`: The semantics of the program to be linted.
    ///
    /// # Returns
    ///
    /// A new `Runner` instance.
    pub fn new(
        php_version: PHPVersion,
        interner: &'a ThreadedInterner,
        codebase: &'a CodebaseReflection,
        semantics: &'a Semantics,
    ) -> Self {
        Self {
            php_version,
            interner,
            codebase,
            semantics,
            ast: AstNode::from(Node::Program(&semantics.program)),
            ignores: get_ignores(semantics, interner),
            issues: IssueCollection::default(),
        }
    }

    /// Executes the specified lint rule on the precomputed AST and reports unused ignore comments.
    ///
    /// This method creates a [`LintContext`] for the given rule and recursively lints the AST starting
    /// from the root node. During the linting process, the rule's `lint_node` method is applied to each node,
    /// and any issues discovered are added to the runner's issue collection. Prior to linting, the method filters
    /// the list of available ignore comments to include only those whose directives match the rule's slug (case-insensitively).
    ///
    /// After linting, any ignore comments that remain in the context (i.e. they were not consumed by any node)
    /// are reported as unused ignores. This helps users identify ignore directives that have no effect and may be
    /// safely removed.
    ///
    /// # Parameters
    ///
    /// - `configured_rule`: The lint rule configuration to execute. This configuration contains the rule's slug,
    ///   which is used to match against ignore directive rules.
    ///
    /// # Behavior
    ///
    /// 1. **Filtering Ignores:**
    ///    The method first filters the runner's list of ignore comments to retain only those whose directives
    ///    match the rule's slug. These are then passed into the [`LintContext`].
    ///
    /// 2. **Linting the AST:**
    ///    The AST is recursively traversed. If an ignore comment applies to a node, it is consumed and removed
    ///    from the context to ensure it is not applied multiple times.
    ///
    /// 3. **Reporting Unused Ignores:**
    ///    After linting, any ignore comments remaining in the context (i.e. unused) are reported as issues.
    ///    These issues include annotations pointing to the ignore's source span along with help notes suggesting
    ///    that the ignore directive be removed.
    ///
    /// # Example
    ///
    /// ```rust
    /// // Assume `runner` is an instance of the lint runner,
    /// // and `configured_rule` is a configured lint rule.
    /// runner.run(&configured_rule);
    /// // After running, any unused ignore comments will have been added to the issue collection.
    /// ```
    pub fn run(&mut self, configured_rule: &ConfiguredRule) {
        let mut context = LintContext::new(
            self.php_version,
            configured_rule,
            self.interner,
            self.codebase,
            self.semantics,
            // Filter the ignores to only those that are relevant to this rule.
            self.ignores
                .iter()
                .filter(|directive| configured_rule.slug.eq_ignore_ascii_case(directive.rule))
                .collect::<Vec<_>>(),
        );

        context.lint(&self.ast);

        self.issues.extend(context.finish());
    }

    /// Finalizes the linting process and returns the collection of reported issues.
    ///
    /// # Returns
    ///
    /// An [`IssueCollection`] containing all issues reported during linting.
    pub fn finish(self) -> IssueCollection {
        self.issues
    }
}
