//! The marker type that ties all adapter impls together as a single [`QueryContext`].

use grit_pattern_matcher::context::QueryContext;

use crate::binding::MagoBinding;
use crate::code_snippet::MagoCodeSnippet;
use crate::context::MagoExecContext;
use crate::file::MagoFile;
use crate::language::MagoLanguage;
use crate::node::MagoNode;
use crate::node_pattern::MagoLeafNodePattern;
use crate::node_pattern::MagoNodePattern;
use crate::resolved_pattern::MagoResolvedPattern;
use crate::tree::MagoTree;

#[derive(Debug, Clone)]
pub struct MagoQueryContext;

impl QueryContext for MagoQueryContext {
    type Node<'a> = MagoNode<'a>;
    type NodePattern = MagoNodePattern;
    type LeafNodePattern = MagoLeafNodePattern;
    type ExecContext<'a> = MagoExecContext<'a>;
    type Binding<'a> = MagoBinding<'a>;
    type CodeSnippet = MagoCodeSnippet;
    type ResolvedPattern<'a> = MagoResolvedPattern<'a>;
    type Language<'a> = MagoLanguage;
    type File<'a> = MagoFile<'a>;
    type Tree<'a> = MagoTree<'a>;
}
