use mago_ast::Node;
use mago_interner::ThreadedInterner;
use mago_php_version::PHPVersion;
use mago_reflection::CodebaseReflection;
use mago_reporting::IssueCollection;
use mago_semantics::Semantics;

use crate::context::LintContext;
use crate::directive::LintDirective;
use crate::rule::ConfiguredRule;
use crate::rule::Rule;

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
    ast: AstNode<'a>,
    issues: IssueCollection,
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
        // Precompute the AST tree from the semantics program.
        let ast = AstNode::from(Node::Program(&semantics.program));

        Self { php_version, interner, codebase, semantics, issues: IssueCollection::default(), ast }
    }

    /// Executes the specified lint rule on the precomputed AST.
    ///
    /// This method creates a [`LintContext`] for the given rule and recursively lints the AST starting
    /// from the root node. The rule's `lint_node` method is applied at each node, and any issues reported
    /// are added to the runner's issue collection.
    ///
    /// # Parameters
    ///
    /// - `configured_rule`: The lint rule configuration to be executed.
    pub fn run(&mut self, configured_rule: &ConfiguredRule) {
        let mut context = LintContext {
            php_version: self.php_version,
            rule: configured_rule,
            interner: self.interner,
            codebase: self.codebase,
            semantics: self.semantics,
            issues: &mut self.issues,
        };

        lint_ast_node(&configured_rule.rule, &self.ast, &mut context);
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

/// Recursively applies a lint rule to the precomputed AST.
///
/// For each node, this function calls the rule's [`Rule::lint_node`] method and then recurses
/// into the node's children based on the returned [`LintDirective`]:
///
/// - **`LintDirective::Continue`**: Lint the children of the node.
/// - **`LintDirective::Prune`**: Skip linting the node's children.
/// - **`LintDirective::Abort`**: Immediately stop linting the current branch.
///
/// # Parameters
///
/// - `rule`: The lint rule to apply.
/// - `ast_node`: The current node in the precomputed AST.
/// - `context`: The linting context for reporting issues and providing additional data.
///
/// # Returns
///
/// Returns `true` if the linting process should be aborted for the current branch;
/// otherwise, returns `false`.
#[inline]
fn lint_ast_node<'a, R: Rule>(rule: &R, ast_node: &AstNode<'a>, context: &mut LintContext<'a>) -> bool {
    let directive = rule.lint_node(ast_node.node, context);

    match directive {
        LintDirective::Continue => {
            // Recurse into each child node.
            for child in &ast_node.children {
                if lint_ast_node(rule, child, context) {
                    return true;
                }
            }
            false
        }
        LintDirective::Prune => false, // Skip children.
        LintDirective::Abort => true,  // Abort the current branch.
    }
}

/// A precomputed tree node for the AST.
///
/// `AstNode` wraps a [`Node`] from the AST and precomputes its children into a vector of [`AstNode`]
/// structures. This avoids multiple calls to [`Node::children()`] during linting, thereby optimizing
/// traversal performance.
#[derive(Debug)]
struct AstNode<'a> {
    /// The wrapped AST node.
    node: Node<'a>,
    /// The precomputed child nodes.
    children: Vec<AstNode<'a>>,
}

impl<'a> From<Node<'a>> for AstNode<'a> {
    /// Recursively converts a [`Node`] into an [`AstNode`], precomputing its children.
    ///
    /// # Parameters
    ///
    /// - `node`: The AST node to be converted.
    ///
    /// # Returns
    ///
    /// An [`AstNode`] representing the given node and its descendants.
    fn from(node: Node<'a>) -> Self {
        let node_children = node.children();
        let mut children = Vec::with_capacity(node_children.len());
        for child in node_children {
            children.push(AstNode::from(child));
        }

        Self { node, children }
    }
}
