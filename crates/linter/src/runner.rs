use mago_codex::metadata::CodebaseMetadata;
use mago_interner::ThreadedInterner;
use mago_names::ResolvedNames;
use mago_php_version::PHPVersion;
use mago_reporting::IssueCollection;
use mago_source::Source;
use mago_syntax::ast::Node;
use mago_syntax::ast::Program;

use crate::ast::AstNode;
use crate::context::LintContext;
use crate::pragma::Pragma;
use crate::pragma::get_pragmas;
use crate::rule::ConfiguredRule;

/// The `Runner` is responsible for executing a lint rule on the AST of a PHP program.
///
/// It holds contextual data such as the PHP version, interner, codebase, source, ast, names and a collection
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
    codebase: &'a CodebaseMetadata,
    source: &'a Source,
    resolved_names: &'a ResolvedNames,
    issues: IssueCollection,
    ast: AstNode<'a>,
    pragmas: Vec<Pragma<'a>>,
}

impl<'a> Runner<'a> {
    /// Creates a new `Runner` instance.
    ///
    /// This method converts the program AST into a precomputed tree
    /// representation to avoid repeated calls to [`Node::children()`] during linting.
    ///
    /// # Parameters
    ///
    /// - `php_version`: The PHP version used during linting.
    /// - `interner`: A reference to the threaded interner for resolving interned strings.
    /// - `codebase`: A reference to the codebase metadata, providing additional context.
    /// - `source`: The source of the program to be linted.
    /// - `program`: The program AST to be linted.
    /// - `resolved_names`: The resolved names for the program, used for name resolution during linting.
    ///
    /// # Returns
    ///
    /// A new `Runner` instance.
    pub fn new(
        php_version: PHPVersion,
        interner: &'a ThreadedInterner,
        codebase: &'a CodebaseMetadata,
        source: &'a Source,
        program: &'a Program,
        resolved_names: &'a ResolvedNames,
    ) -> Self {
        Self {
            php_version,
            interner,
            codebase,
            source,
            resolved_names,
            ast: AstNode::from(Node::Program(program)),
            pragmas: get_pragmas(source, program, interner),
            issues: IssueCollection::default(),
        }
    }

    /// Executes the specified lint rule on the precomputed AST.
    ///
    /// # Parameters
    ///
    /// - `configured_rule`: The lint rule configuration to execute.
    pub fn run(&mut self, configured_rule: &ConfiguredRule) {
        let mut context = LintContext::new(
            self.php_version,
            configured_rule,
            self.interner,
            self.codebase,
            self.source,
            self.resolved_names,
            // Filter the pragmas to only those that are relevant to this rule.
            self.pragmas
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
