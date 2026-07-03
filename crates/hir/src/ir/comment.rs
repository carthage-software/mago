use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_span::HasSpan;
use mago_span::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Comment<'arena> {
    pub span: Span,
    pub kind: CommentKind,
    pub value: &'arena [u8],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum CommentKind {
    SingleLine,
    MultiLine,
    Hash,
    DocBlock,
}

impl CommentKind {
    #[inline]
    #[must_use]
    pub const fn is_docblock(&self) -> bool {
        matches!(self, CommentKind::DocBlock)
    }

    #[inline]
    #[must_use]
    pub const fn is_block_comment(&self) -> bool {
        matches!(self, CommentKind::MultiLine | CommentKind::DocBlock)
    }

    #[inline]
    #[must_use]
    pub const fn is_single_line_comment(&self) -> bool {
        matches!(self, CommentKind::Hash | CommentKind::SingleLine)
    }
}

impl HasSpan for Comment<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl CopyInto for Comment<'_> {
    type Output<'arena> = Comment<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Comment { span: self.span, kind: self.kind, value: arena.alloc_slice_copy(self.value) }
    }
}
