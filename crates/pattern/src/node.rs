//! Mago AST node adapter for the grit pattern engine.
//!
//! The grit engine requires each node to support `parent` / `children` / siblings, but
//! Mago's [`mago_syntax::ast::Node`] enum doesn't carry parent pointers. We solve this by
//! pre-indexing the whole [`Program`] in [`MagoIndex`]: a flat DFS walk assigns each node a
//! `u32` id and records its parent + children slots. `MagoNode` is then a cheap
//! `(id, &MagoIndex)` pair that satisfies [`grit_util::AstNode`] without new allocations
//! per traversal.
//!
//! Lifetimes: Mago's `Node<'ast, 'arena>` has two: the arena the whole AST is
//! bump-allocated in, and the references into that arena. For matcher use we collapse both
//! to a single `'a`, which is fine as long as the arena outlives the index (and it does:
//! the query driver holds the arena for the whole session).

use std::borrow::Cow;

use grit_util::AstCursor;
use grit_util::AstNode;
use grit_util::ByteRange;
use grit_util::CodeRange;
use grit_util::Position;

use mago_span::HasSpan;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Identifier;
use mago_syntax::ast::LocalIdentifier;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::Program;
use mago_syntax::ast::Variable;

/// Opaque identifier for a node inside a [`MagoIndex`].
pub type NodeId = u32;

/// ID reserved for the root program node.
pub const ROOT_NODE_ID: NodeId = 0;

/// Pre-computed flat index of a parsed [`Program`].
///
/// One of these is built for each target file we want to query. Construction is O(N) in
/// the number of AST nodes. Subsequent traversals (parent / children / siblings) are O(1)
/// against the stored tables.
pub struct MagoIndex<'a> {
    /// The original source text, needed for [`AstNode::text`] and range conversions.
    source: &'a str,
    /// Every node in DFS pre-order. `nodes[0]` is the program root.
    nodes: Vec<Node<'a, 'a>>,
    /// `parents[i]` = parent of node `i`, or `None` for the root.
    parents: Vec<Option<NodeId>>,
    /// `child_ranges[i]` = `(start, end)` into [`Self::child_indices`] for node `i`'s children.
    child_ranges: Vec<(u32, u32)>,
    /// Flat list of child node ids, grouped by parent (via [`Self::child_ranges`]).
    child_indices: Vec<NodeId>,
    /// For each node, its position within its parent's child list. Root's is 0.
    position_in_parent: Vec<u32>,
}

impl<'a> MagoIndex<'a> {
    /// Builds an index for the given parsed program and its source.
    ///
    /// The two lifetimes of `Node<'ast, 'arena>` are collapsed to `'a` on the assumption
    /// that the caller holds both the arena and the source for at least `'a`.
    pub fn new(program: &'a Program<'a>, source: &'a str) -> Self {
        let mut idx = MagoIndex {
            source,
            nodes: Vec::new(),
            parents: Vec::new(),
            child_ranges: Vec::new(),
            child_indices: Vec::new(),
            position_in_parent: Vec::new(),
        };

        idx.visit(Node::Program(program), None, 0);
        idx
    }

    fn visit(&mut self, node: Node<'a, 'a>, parent: Option<NodeId>, position_in_parent: u32) -> NodeId {
        let id = self.nodes.len() as NodeId;
        self.nodes.push(node);
        self.parents.push(parent);
        self.position_in_parent.push(position_in_parent);
        // Reserve our own child-range slot so slot index == node id; the actual range is
        // backfilled after recursion so every descendant has already taken its own slot.
        self.child_ranges.push((0, 0));

        let mut children: Vec<NodeId> = Vec::new();
        let mut child_position: u32 = 0;
        node.visit_children(|child| {
            let child_id = self.visit(child, Some(id), child_position);
            children.push(child_id);
            child_position += 1;
        });

        let range_start = self.child_indices.len() as u32;
        self.child_indices.extend(&children);
        let range_end = self.child_indices.len() as u32;
        self.child_ranges[id as usize] = (range_start, range_end);

        id
    }

    /// Returns the source text backing the indexed program.
    pub fn source(&self) -> &'a str {
        self.source
    }

    /// Returns the underlying [`Node`] for the given id.
    pub fn node(&self, id: NodeId) -> Node<'a, 'a> {
        self.nodes[id as usize]
    }

    /// Returns the parent id for `id`, or `None` for the root.
    pub fn parent_of(&self, id: NodeId) -> Option<NodeId> {
        self.parents[id as usize]
    }

    /// Returns a slice of the direct child ids for the given node.
    pub fn children_of(&self, id: NodeId) -> &[NodeId] {
        let (s, e) = self.child_ranges[id as usize];
        &self.child_indices[s as usize..e as usize]
    }

    /// Position of the given node within its parent's child list. Root is `0`.
    pub fn position_in_parent(&self, id: NodeId) -> u32 {
        self.position_in_parent[id as usize]
    }

    /// Returns an iterator over every indexed node in DFS pre-order.
    pub fn iter_ids(&self) -> std::ops::Range<NodeId> {
        0..self.nodes.len() as NodeId
    }

    /// Returns the total number of indexed nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if the index is empty. (Should never be: the program root is always
    /// indexed.)
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl std::fmt::Debug for MagoIndex<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MagoIndex").field("nodes", &self.nodes.len()).finish()
    }
}

/// Cheap handle into a [`MagoIndex`]: just a node id plus an index reference.
#[derive(Clone, Copy)]
pub struct MagoNode<'a> {
    id: NodeId,
    index: &'a MagoIndex<'a>,
}

impl<'a> MagoNode<'a> {
    pub fn new(index: &'a MagoIndex<'a>, id: NodeId) -> Self {
        Self { index, id }
    }

    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn index(&self) -> &'a MagoIndex<'a> {
        self.index
    }

    pub fn source(&self) -> &'a str {
        self.index.source()
    }

    pub fn inner(&self) -> Node<'a, 'a> {
        self.index.node(self.id)
    }

    pub fn kind(&self) -> NodeKind {
        self.inner().kind()
    }

    /// Returns `true` if this node is a comment. Mago stores comments in program trivia,
    /// not as AST nodes, so this is always `false` for the MVP. When we wire comments into
    /// the engine's trivia model we'll revisit this.
    pub fn is_comment(&self) -> bool {
        false
    }

    /// Returns `true` if this node is a statement-shaped node per [`Node::is_statement`].
    pub fn is_statement_kind(&self) -> bool {
        self.inner().is_statement()
    }

    /// Returns `true` if this node is a metavariable produced by snippet preprocessing:
    /// an identifier (or identifier-like node) whose name starts with the substitute prefix
    /// [`crate::language::MagoLanguage`] uses (`µ`).
    pub fn is_metavariable(&self) -> bool {
        metavariable_name(self).is_some()
    }

    /// Returns the metavariable name (without any prefix) if this node is a metavariable.
    pub fn metavariable_name(&self) -> Option<&'a str> {
        metavariable_name(self)
    }
}

/// Look for a `µ`-prefixed identifier nested anywhere inside `node` at the leaf-like level.
/// We have to unwrap through several AST layers because the same conceptual identifier can
/// appear as `Expression::Identifier(Identifier::Local(id))` or `Node::LocalIdentifier(id)`
/// or as a `ConstantAccess` wrapping one, depending on whether the snippet parser treated
/// it as a call target, a bare constant, a variable, etc.
fn metavariable_name<'a>(node: &MagoNode<'a>) -> Option<&'a str> {
    match node.inner() {
        Node::LocalIdentifier(id) => strip_prefix(id.value),
        Node::Identifier(Identifier::Local(id)) => strip_prefix(id.value),
        Node::Expression(Expression::Identifier(Identifier::Local(id))) => strip_prefix(id.value),
        Node::Expression(Expression::ConstantAccess(access)) => match access.name {
            Identifier::Local(LocalIdentifier { value, .. }) => strip_prefix(value),
            _ => None,
        },
        Node::ConstantAccess(access) => match access.name {
            Identifier::Local(LocalIdentifier { value, .. }) => strip_prefix(value),
            _ => None,
        },
        Node::Expression(Expression::Variable(Variable::Direct(var))) => var.name.strip_prefix("$µ"),
        Node::DirectVariable(var) => var.name.strip_prefix("$µ"),
        Node::Variable(Variable::Direct(var)) => var.name.strip_prefix("$µ"),
        _ => None,
    }
}

fn strip_prefix(value: &str) -> Option<&str> {
    value.strip_prefix('µ')
}

impl std::fmt::Debug for MagoNode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MagoNode").field("id", &self.id).field("kind", &self.kind()).finish()
    }
}

impl<'a> AstNode for MagoNode<'a> {
    fn ancestors(&self) -> impl Iterator<Item = Self> {
        let mut current = self.index.parent_of(self.id);
        std::iter::from_fn(move || {
            let id = current?;
            current = self.index.parent_of(id);
            Some(MagoNode::new(self.index, id))
        })
    }

    fn children(&self) -> impl Iterator<Item = Self> {
        let ids = self.index.children_of(self.id);
        let index = self.index;
        ids.iter().copied().map(move |id| MagoNode::new(index, id))
    }

    fn parent(&self) -> Option<Self> {
        self.index.parent_of(self.id).map(|id| MagoNode::new(self.index, id))
    }

    fn next_named_node(&self) -> Option<Self> {
        self.next_sibling()
    }

    fn previous_named_node(&self) -> Option<Self> {
        self.previous_sibling()
    }

    fn next_sibling(&self) -> Option<Self> {
        let parent_id = self.index.parent_of(self.id)?;
        let siblings = self.index.children_of(parent_id);
        let pos = self.index.position_in_parent(self.id) as usize;
        siblings.get(pos + 1).copied().map(|id| MagoNode::new(self.index, id))
    }

    fn previous_sibling(&self) -> Option<Self> {
        let parent_id = self.index.parent_of(self.id)?;
        let siblings = self.index.children_of(parent_id);
        let pos = self.index.position_in_parent(self.id) as usize;
        if pos == 0 {
            return None;
        }
        siblings.get(pos - 1).copied().map(|id| MagoNode::new(self.index, id))
    }

    fn text(&self) -> grit_util::error::GritResult<Cow<'_, str>> {
        let span = self.inner().span();
        let start = span.start.offset as usize;
        let end = span.end.offset as usize;
        let src = self.index.source();
        let end = end.min(src.len());
        let start = start.min(end);
        Ok(Cow::Borrowed(&src[start..end]))
    }

    fn byte_range(&self) -> ByteRange {
        let span = self.inner().span();
        ByteRange::new(span.start.offset as usize, span.end.offset as usize)
    }

    fn code_range(&self) -> CodeRange {
        let span = self.inner().span();
        CodeRange::new(span.start.offset, span.end.offset, self.index.source())
    }

    fn walk(&self) -> impl AstCursor<Node = Self> {
        MagoCursor::new(*self)
    }
}

/// Simple cursor that walks a [`MagoNode`] subtree in DFS pre-order.
pub struct MagoCursor<'a> {
    root: MagoNode<'a>,
    /// Stack of (node, next_child_index) frames being walked.
    stack: Vec<(MagoNode<'a>, usize)>,
    current: MagoNode<'a>,
}

impl<'a> MagoCursor<'a> {
    pub fn new(root: MagoNode<'a>) -> Self {
        Self { root, stack: Vec::new(), current: root }
    }

    /// The cursor's root: the node the traversal started from.
    pub fn root(&self) -> MagoNode<'a> {
        self.root
    }
}

impl<'a> AstCursor for MagoCursor<'a> {
    type Node = MagoNode<'a>;

    fn goto_first_child(&mut self) -> bool {
        let idx = self.current.index();
        let children = idx.children_of(self.current.id());
        if let Some(&first) = children.first() {
            self.stack.push((self.current, 0));
            self.current = MagoNode::new(idx, first);
            true
        } else {
            false
        }
    }

    fn goto_parent(&mut self) -> bool {
        if let Some((parent, _)) = self.stack.pop() {
            self.current = parent;
            true
        } else {
            false
        }
    }

    fn goto_next_sibling(&mut self) -> bool {
        let Some((parent, cursor)) = self.stack.last_mut() else {
            return false;
        };
        let next_index = *cursor + 1;
        let idx = parent.index();
        let children = idx.children_of(parent.id());
        if let Some(&next) = children.get(next_index) {
            *cursor = next_index;
            self.current = MagoNode::new(idx, next);
            true
        } else {
            false
        }
    }

    fn node(&self) -> Self::Node {
        self.current
    }
}

/// Computes a best-effort [`Position`] (1-based line, 0-based byte column) from a byte offset
/// in the given source.
pub fn position_of(source: &str, offset: u32) -> Position {
    let mut line: u32 = 1;
    let mut column: u32 = 0;
    let end = (offset as usize).min(source.len());
    for &b in source.as_bytes()[..end].iter() {
        if b == b'\n' {
            line += 1;
            column = 0;
        } else {
            column += 1;
        }
    }
    Position::new(line, column)
}
