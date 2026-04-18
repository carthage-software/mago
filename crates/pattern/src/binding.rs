//! [`Binding`] implementation for Mago nodes.
//!
//! A binding is the "pointer to a thing" grit passes around during matching. For Mago,
//! a binding is either a reference to an AST node, a constant, a byte range in source, or
//! a file path. Matching cares about nodes; rewriting cares about ranges/text.

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;

use grit_pattern_matcher::binding::Binding;
use grit_pattern_matcher::constant::Constant;
use grit_pattern_matcher::context::QueryContext;
use grit_pattern_matcher::effects::Effect;
use grit_pattern_matcher::pattern::FileRegistry;
use grit_util::AnalysisLogs;
use grit_util::AstNode;
use grit_util::ByteRange;
use grit_util::CodeRange;
use grit_util::Range;
use grit_util::error::GritResult;

use crate::node::MagoNode;
use crate::query_context::MagoQueryContext;

/// Binding variants. Mirrors `MarzanoBinding` but without tree-sitter specifics.
#[derive(Debug, Clone)]
pub enum MagoBinding<'a> {
    Node(MagoNode<'a>),
    String(&'a str, ByteRange),
    Range(ByteRange, &'a str),
    FileName(&'a Path),
    Constant(&'a Constant),
    /// An "empty" binding slot, used when a pattern expects a child that doesn't exist
    /// on the target. Carries the parent node so rewrites can attach edits near where the
    /// child would have lived.
    Empty(MagoNode<'a>, u32),
    /// A sequence of nodes captured by a named `^...name` dots metavariable. Rendered as
    /// their source texts joined by `, ` so templates can splice the sequence back in.
    NodeList(Vec<MagoNode<'a>>),
}

impl<'a> PartialEq for MagoBinding<'a> {
    fn eq(&self, other: &Self) -> bool {
        use MagoBinding::*;
        match (self, other) {
            (Node(a), Node(b)) => a.id() == b.id(),
            (String(a1, r1), String(a2, r2)) => r1 == r2 && std::ptr::eq(*a1, *a2),
            (Range(r1, a1), Range(r2, a2)) => r1 == r2 && std::ptr::eq(*a1, *a2),
            (FileName(a), FileName(b)) => a == b,
            (Constant(a), Constant(b)) => std::ptr::eq(*a, *b),
            (Empty(n1, i1), Empty(n2, i2)) => n1.id() == n2.id() && i1 == i2,
            (NodeList(a), NodeList(b)) => a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.id() == y.id()),
            _ => false,
        }
    }
}

impl<'a> Binding<'a, MagoQueryContext> for MagoBinding<'a> {
    fn from_constant(constant: &'a Constant) -> Self {
        MagoBinding::Constant(constant)
    }

    fn from_node(node: MagoNode<'a>) -> Self {
        MagoBinding::Node(node)
    }

    fn from_path(path: &'a Path) -> Self {
        MagoBinding::FileName(path)
    }

    fn from_range(range: ByteRange, source: &'a str) -> Self {
        MagoBinding::Range(range, source)
    }

    fn singleton(&self) -> Option<MagoNode<'a>> {
        match self {
            MagoBinding::Node(node) => Some(*node),
            _ => None,
        }
    }

    fn get_sexp(&self) -> Option<String> {
        None
    }

    fn position(&self, _language: &<MagoQueryContext as QueryContext>::Language<'a>) -> Option<Range> {
        None
    }

    fn range(&self, _language: &<MagoQueryContext as QueryContext>::Language<'a>) -> Option<ByteRange> {
        match self {
            MagoBinding::Node(node) => Some(node.byte_range()),
            MagoBinding::Range(range, _) => Some(*range),
            MagoBinding::String(_, range) => Some(*range),
            _ => None,
        }
    }

    fn code_range(&self, _language: &<MagoQueryContext as QueryContext>::Language<'a>) -> Option<CodeRange> {
        match self {
            MagoBinding::Node(node) => Some(node.code_range()),
            _ => None,
        }
    }

    fn is_equivalent_to(&self, other: &Self, _language: &<MagoQueryContext as QueryContext>::Language<'a>) -> bool {
        match (self, other) {
            (MagoBinding::Node(a), MagoBinding::Node(b)) => {
                a.kind() == b.kind() && a.text().ok().map(|s| s.into_owned()) == b.text().ok().map(|s| s.into_owned())
            }
            _ => self == other,
        }
    }

    fn is_suppressed(
        &self,
        _language: &<MagoQueryContext as QueryContext>::Language<'a>,
        _current_name: Option<&str>,
    ) -> bool {
        // We don't model suppression through grit yet; rely on Mago's own pragma system.
        false
    }

    fn get_insertion_padding(
        &self,
        _text: &str,
        _is_first: bool,
        _language: &<MagoQueryContext as QueryContext>::Language<'a>,
    ) -> Option<String> {
        None
    }

    fn linearized_text(
        &self,
        language: &<MagoQueryContext as QueryContext>::Language<'a>,
        _effects: &[Effect<'a, MagoQueryContext>],
        _files: &FileRegistry<'a, MagoQueryContext>,
        _memo: &mut HashMap<CodeRange, Option<String>>,
        _distributed_indent: Option<usize>,
        _logs: &mut AnalysisLogs,
    ) -> GritResult<Cow<'a, str>> {
        // We materialise rewrites outside of grit's linearization (the query driver reads
        // `State.effects` directly and emits `Rewrite { range, replacement }` pairs).
        // Return the binding's source text unchanged.
        self.text(language).map(|c| Cow::Owned(c.into_owned()))
    }

    fn text(&self, _language: &<MagoQueryContext as QueryContext>::Language<'a>) -> GritResult<Cow<'_, str>> {
        match self {
            MagoBinding::Node(node) => node.text(),
            MagoBinding::String(src, range) => {
                let end = range.end.min(src.len());
                let start = range.start.min(end);
                Ok(Cow::Borrowed(&src[start..end]))
            }
            MagoBinding::Range(range, src) => {
                let end = range.end.min(src.len());
                let start = range.start.min(end);
                Ok(Cow::Borrowed(&src[start..end]))
            }
            MagoBinding::FileName(path) => Ok(Cow::Owned(path.to_string_lossy().into_owned())),
            MagoBinding::Constant(c) => Ok(Cow::Owned(c.to_string())),
            MagoBinding::Empty(_, _) => Ok(Cow::Borrowed("")),
            MagoBinding::NodeList(nodes) => {
                let mut out = String::new();
                for (i, node) in nodes.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    out.push_str(&node.text()?);
                }
                Ok(Cow::Owned(out))
            }
        }
    }

    fn source(&self) -> Option<&'a str> {
        match self {
            MagoBinding::Node(node) => Some(node.source()),
            MagoBinding::String(src, _) => Some(*src),
            MagoBinding::Range(_, src) => Some(*src),
            MagoBinding::NodeList(nodes) => nodes.first().map(|n| n.source()),
            _ => None,
        }
    }

    fn as_constant(&self) -> Option<&Constant> {
        match self {
            MagoBinding::Constant(c) => Some(*c),
            _ => None,
        }
    }

    fn as_filename(&self) -> Option<&Path> {
        match self {
            MagoBinding::FileName(p) => Some(*p),
            _ => None,
        }
    }

    fn as_node(&self) -> Option<MagoNode<'a>> {
        self.singleton()
    }

    fn is_list(&self) -> bool {
        false
    }

    fn list_items(&self) -> Option<impl Iterator<Item = MagoNode<'a>> + Clone> {
        None::<std::iter::Empty<MagoNode<'a>>>
    }

    fn parent_node(&self) -> Option<MagoNode<'a>> {
        match self {
            MagoBinding::Node(node) => node.parent(),
            MagoBinding::Empty(parent, _) => Some(*parent),
            _ => None,
        }
    }

    fn is_truthy(&self) -> bool {
        match self {
            MagoBinding::Node(_) => true,
            MagoBinding::String(_, r) | MagoBinding::Range(r, _) => r.end > r.start,
            MagoBinding::FileName(_) => true,
            MagoBinding::Constant(c) => c.is_truthy(),
            MagoBinding::Empty(_, _) => false,
            MagoBinding::NodeList(nodes) => !nodes.is_empty(),
        }
    }

    fn log_empty_field_rewrite_error(
        &self,
        _language: &<MagoQueryContext as QueryContext>::Language<'a>,
        _logs: &mut AnalysisLogs,
    ) -> GritResult<()> {
        Ok(())
    }
}
