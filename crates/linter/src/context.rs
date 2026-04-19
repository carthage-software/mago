use bumpalo::Bump;
use bumpalo::collections::Vec;

use mago_collector::Collector;
use mago_database::file::File;
use mago_names::ResolvedNames;
use mago_php_version::PHPVersion;
use mago_span::HasPosition;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::import_tracker::ImportKind;
use crate::import_tracker::ImportResolution;
use crate::import_tracker::ImportTracker;
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
    imports: ImportTracker,
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
            imports: ImportTracker::new(),
            ancestors: Vec::with_capacity_in(32, arena),
        }
    }

    /// Ask the context to synthesise a `use` statement for a class-like FQN.
    ///
    /// Returns `None` when no fix should be offered (we can't import here; no
    /// `<?php` seen yet, we're between braced namespaces, the short name is
    /// already bound to a different symbol in this scope, or the FQN is a
    /// reserved type keyword).
    ///
    /// Returns `Some(resolution)` otherwise. If `resolution.use_statement_edit`
    /// is `None`, the name is already reachable (already imported, possibly
    /// under an alias in `resolution.local_name`, or the FQN is inside the
    /// current namespace) and the caller should just replace the FQN with
    /// `resolution.local_name`.
    ///
    /// # Staggering lets multiple imports land in a single run
    ///
    /// Each subsequent insert at the same anchor is offset by one byte into
    /// the run of whitespace that follows the anchor. Two zero-width inserts
    /// at different offsets don't overlap, so the fixer can apply all of them
    /// in a single pass. When the whitespace run is exhausted later imports
    /// fall back to the base offset and conflict with the first.
    pub fn import_name(&mut self, fqn: &str) -> Option<ImportResolution> {
        self.imports.import(fqn, ImportKind::Name)
    }

    /// Same as [`import_name`](Self::import_name) but emits `use function ...;`.
    pub fn import_function(&mut self, fqn: &str) -> Option<ImportResolution> {
        self.imports.import(fqn, ImportKind::Function)
    }

    /// Same as [`import_name`](Self::import_name) but emits `use const ...;`.
    /// Constant short names are matched case-sensitively (PHP semantics).
    pub fn import_constant(&mut self, fqn: &str) -> Option<ImportResolution> {
        self.imports.import(fqn, ImportKind::Constant)
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
        self.imports.enter_node(node);
    }

    /// Called by the walker on node exit. Rules must not call this.
    #[doc(hidden)]
    pub(crate) fn pop_ancestor(&mut self) {
        if let Some(node) = self.ancestors.pop() {
            self.imports.exit_node(node);
        }
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
