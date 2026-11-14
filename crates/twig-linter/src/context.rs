use mago_allocator::prelude::*;

use mago_collector::Collector;
use mago_database::file::File;
use mago_twig_syntax::cst::Node;

use crate::registry::RuleRegistry;
use crate::scope::ScopeStack;

#[derive(Debug)]
#[allow(clippy::partial_pub_fields)]
pub struct LintContext<'ctx, 'arena, A>
where
    A: Arena,
{
    pub arena: &'arena A,
    pub registry: &'ctx RuleRegistry,
    pub source_file: &'ctx File,
    pub collector: Collector<'ctx, 'arena, A>,
    pub scope: ScopeStack<'arena, A>,
    pub conditional_depth: usize,
    ancestors: Vec<'arena, Node<'ctx, 'arena>, A>,
}

impl<'ctx, 'arena, A> LintContext<'ctx, 'arena, A>
where
    A: Arena,
{
    pub fn new(
        arena: &'arena A,
        registry: &'ctx RuleRegistry,
        source_file: &'ctx File,
        collector: Collector<'ctx, 'arena, A>,
    ) -> Self {
        Self {
            arena,
            registry,
            source_file,
            collector,
            scope: ScopeStack::new_in(arena),
            conditional_depth: 0,
            ancestors: Vec::with_capacity_in(32, arena),
        }
    }

    /// Checks if we are currently inside a conditional (`? :`) expression.
    ///
    /// An arrow function body starts a new context, so a conditional inside it
    /// is not nested in a conditional that surrounds the arrow function.
    #[must_use]
    pub fn is_in_conditional(&self) -> bool {
        self.conditional_depth > 0
    }

    /// Checks whether `name` calls a macro imported with `{% from %}` rather than a Twig function.
    #[must_use]
    pub fn is_imported(&self, name: &[u8]) -> bool {
        self.scope.is_imported(name)
    }

    /// Called by the walker on node entry. Rules must not call this.
    #[doc(hidden)]
    pub(crate) fn push_ancestor(&mut self, node: Node<'ctx, 'arena>) {
        if let Node::Statement(_) = node
            && let Some(Node::Embed(_)) = self.ancestors.last()
        {
            self.scope.enter_embed_body();
        }

        self.ancestors.push(node);
    }

    /// Called by the walker on node exit. Rules must not call this.
    #[doc(hidden)]
    pub(crate) fn pop_ancestor(&mut self) {
        if let Some(Node::From(from)) = self.ancestors.pop() {
            for name in from.names.iter() {
                self.scope.import(name.to.unwrap_or(name.from).value);
            }
        }
    }

    /// Returns the immediate parent node, or `None` if the current node is the root.
    #[must_use]
    pub fn get_parent(&self) -> Option<Node<'ctx, 'arena>> {
        self.ancestors.len().checked_sub(2).map(|index| self.ancestors[index])
    }
}
