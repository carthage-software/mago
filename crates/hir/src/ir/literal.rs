use ordered_float::OrderedFloat;
#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_span::Span;

use mago_allocator::copy::CopyInto;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Literal<'arena> {
    pub span: Span,
    pub kind: LiteralKind<'arena>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum LiteralKind<'arena> {
    String(LiteralString<'arena>),
    Integer(LiteralInteger<'arena>),
    Float(LiteralFloat<'arena>),
    True,
    False,
    Null,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum LiteralStringKind {
    SingleQuoted,
    DoubleQuoted,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct LiteralString<'arena> {
    pub span: Span,
    pub kind: LiteralStringKind,
    pub raw: &'arena [u8],
    pub value: Option<&'arena [u8]>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct LiteralInteger<'arena> {
    pub span: Span,
    pub raw: &'arena [u8],
    pub value: Option<u64>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct LiteralFloat<'arena> {
    pub span: Span,
    pub raw: &'arena [u8],
    pub value: OrderedFloat<f64>,
}

impl HasSpan for Literal<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for LiteralString<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for LiteralInteger<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for LiteralFloat<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl CopyInto for Literal<'_> {
    type Output<'arena> = Literal<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Literal { span: self.span, kind: self.kind.copy_into(arena) }
    }
}

impl CopyInto for LiteralKind<'_> {
    type Output<'arena> = LiteralKind<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match *self {
            LiteralKind::String(string) => LiteralKind::String(string.copy_into(arena)),
            LiteralKind::Integer(integer) => LiteralKind::Integer(integer.copy_into(arena)),
            LiteralKind::Float(float) => LiteralKind::Float(float.copy_into(arena)),
            LiteralKind::True => LiteralKind::True,
            LiteralKind::False => LiteralKind::False,
            LiteralKind::Null => LiteralKind::Null,
        }
    }
}

impl CopyInto for LiteralString<'_> {
    type Output<'arena> = LiteralString<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        LiteralString {
            span: self.span,
            kind: self.kind,
            raw: arena.alloc_slice_copy(self.raw),
            value: self.value.map(|value| &*arena.alloc_slice_copy(value)),
        }
    }
}

impl CopyInto for LiteralInteger<'_> {
    type Output<'arena> = LiteralInteger<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        LiteralInteger { span: self.span, raw: arena.alloc_slice_copy(self.raw), value: self.value }
    }
}

impl CopyInto for LiteralFloat<'_> {
    type Output<'arena> = LiteralFloat<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        LiteralFloat { span: self.span, raw: arena.alloc_slice_copy(self.raw), value: self.value }
    }
}
