use bumpalo::Bump;
use bumpalo::collections::Vec;

use mago_collector::Collector;
use mago_database::file::File;
use mago_names::ResolvedNames;
use mago_php_version::PHPVersion;
use mago_span::HasPosition;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::registry::RuleRegistry;
use crate::scope::ScopeStack;

#[derive(Debug)]
pub struct LintContext<'ctx, 'arena> {
    pub php_version: PHPVersion,
    pub arena: &'arena Bump,
    pub registry: &'ctx RuleRegistry,
    pub source_file: &'ctx File,
    pub resolved_names: &'ctx ResolvedNames<'arena>,
    pub collector: Collector<'ctx, 'arena>,
    pub scope: ScopeStack<'arena>,
    pub constant_expression_depth: usize,
    ancestors: Vec<'arena, Node<'ctx, 'arena>>,
}

impl<'ctx, 'arena> LintContext<'ctx, 'arena> {
    pub fn new(
        php_version: PHPVersion,
        arena: &'arena Bump,
        registry: &'ctx RuleRegistry,
        source_file: &'ctx File,
        resolved_names: &'ctx ResolvedNames<'arena>,
        collector: Collector<'ctx, 'arena>,
    ) -> Self {
        Self {
            php_version,
            arena,
            registry,
            source_file,
            resolved_names,
            collector,
            scope: ScopeStack::new_in(arena),
            constant_expression_depth: 0,
            ancestors: Vec::with_capacity_in(32, arena),
        }
    }

    /// Checks if we are currently inside a constant expression context.
    ///
    /// Constant expression contexts include attribute arguments, parameter default values,
    /// property default values, and constant values.
    pub fn is_in_constant_expression(&self) -> bool {
        self.constant_expression_depth > 0
    }

    /// Checks if a name at a given position is imported.
    pub fn is_name_imported(&self, position: &impl HasPosition) -> bool {
        self.resolved_names.is_imported(&position.position())
    }

    /// Retrieves the name associated with a given position in the code.
    ///
    /// # Panics
    ///
    /// Panics if no name is found at the specified position.
    pub fn lookup_name(&self, position: &impl HasPosition) -> &'arena str {
        self.resolved_names.get(&position.position())
    }

    /// Called by the walker on node entry. Rules must not call this.
    #[doc(hidden)]
    pub(crate) fn push_ancestor(&mut self, node: Node<'ctx, 'arena>) {
        self.ancestors.push(node);
    }

    /// Called by the walker on node exit. Rules must not call this.
    #[doc(hidden)]
    pub(crate) fn pop_ancestor(&mut self) {
        self.ancestors.pop();
    }

    /// Returns the immediate parent node, or `None` if the current node is
    /// the root.
    ///
    /// This is equivalent to `get_nth_parent(0)`.
    #[must_use]
    pub fn get_parent(&self) -> Option<Node<'ctx, 'arena>> {
        self.get_nth_parent(0)
    }

    /// Returns the `NodeKind` of the immediate parent.
    ///
    /// Shorthand for `get_parent().map(|n| n.kind())`.
    #[must_use]
    pub fn get_parent_kind(&self) -> Option<NodeKind> {
        self.get_parent().map(|n| n.kind())
    }

    /// Returns the `n`-th ancestor node of the current node.
    ///
    /// `n = 0` is the immediate parent, `n = 1` is the grandparent, etc.
    /// Returns `None` if the ancestor stack is not deep enough.
    #[must_use]
    pub fn get_nth_parent(&self, n: usize) -> Option<Node<'ctx, 'arena>> {
        // ancestors: [..., grandparent, parent, current]
        // parent = len - 2, grandparent = len - 3, etc.
        let len = self.ancestors.len();
        let index = n + 2; // skip current (len-1) + 1 for the offset
        if len >= index { Some(self.ancestors[len - index]) } else { None }
    }

    /// Returns the `NodeKind` of the `n`-th ancestor.
    ///
    /// Shorthand for `get_nth_parent(n).map(|n| n.kind())`.
    #[must_use]
    pub fn get_nth_parent_kind(&self, n: usize) -> Option<NodeKind> {
        self.get_nth_parent(n).map(|n| n.kind())
    }

    /// Returns `true` if any ancestor of the current node has the given kind.
    #[must_use]
    pub fn is_child_of(&self, kind: NodeKind) -> bool {
        let len = self.ancestors.len();
        if len < 2 {
            return false;
        }

        self.ancestors[..len - 1].iter().any(|n| n.kind() == kind)
    }
}
