use mago_syntax::ast::Node;

/// A precomputed tree node for the AST.
///
/// `AstNode` wraps a [`Node`] from the AST and precomputes its children into a vector of [`AstNode`]
/// structures. This avoids multiple calls to [`Node::children()`] during linting, thereby optimizing
/// traversal performance.
#[derive(Debug)]
pub(crate) struct AstNode<'a> {
    /// The wrapped AST node.
    pub node: Node<'a>,
    /// The precomputed child nodes.
    pub children: Vec<AstNode<'a>>,
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
