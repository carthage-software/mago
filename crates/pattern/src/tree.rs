//! [`grit_util::Ast`] adapter around Mago's parsed [`Program`].

use std::borrow::Cow;
use std::sync::Arc;

use grit_util::Ast;

use crate::node::MagoIndex;
use crate::node::MagoNode;
use crate::node::ROOT_NODE_ID;

/// Parsed program wrapper used by the grit engine.
///
/// The [`MagoIndex`] is held behind an [`Arc`] so clones are cheap. Grit asks for the
/// tree to be clonable, and sharing the indexed data keeps that cheap even on deep trees.
pub struct MagoTree<'a> {
    pub index: Arc<MagoIndex<'a>>,
}

impl<'a> MagoTree<'a> {
    pub fn new(index: MagoIndex<'a>) -> Self {
        Self { index: Arc::new(index) }
    }

    pub fn from_arc(index: Arc<MagoIndex<'a>>) -> Self {
        Self { index }
    }
}

impl std::fmt::Debug for MagoTree<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MagoTree").field("nodes", &self.index.len()).finish()
    }
}

impl<'a> Clone for MagoTree<'a> {
    fn clone(&self) -> Self {
        Self { index: Arc::clone(&self.index) }
    }
}

impl<'a> PartialEq for MagoTree<'a> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.index, &other.index)
    }
}

impl<'a> Ast for MagoTree<'a> {
    type Node<'b>
        = MagoNode<'b>
    where
        Self: 'b;

    fn root_node(&self) -> Self::Node<'_> {
        MagoNode::new(&self.index, ROOT_NODE_ID)
    }

    fn source(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.index.source())
    }
}
