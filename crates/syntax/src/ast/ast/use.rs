use serde::Serialize;
use strum::Display;

use mago_database::file::FileId;
use mago_span::HasSpan;
use mago_span::Position;
use mago_span::Span;

use crate::ast::ast::identifier::Identifier;
use crate::ast::ast::identifier::LocalIdentifier;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::terminator::Terminator;
use crate::ast::sequence::TokenSeparatedSequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Use<'arena> {
    pub r#use: Keyword<'arena>,
    pub items: UseItems<'arena>,
    pub terminator: Terminator<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum UseItems<'arena> {
    Sequence(UseItemSequence<'arena>),
    TypedSequence(TypedUseItemSequence<'arena>),
    TypedList(TypedUseItemList<'arena>),
    MixedList(MixedUseItemList<'arena>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum UseType<'arena> {
    Function(Keyword<'arena>),
    Const(Keyword<'arena>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct UseItemSequence<'arena> {
    pub file_id: FileId,
    pub start: Position,
    pub items: TokenSeparatedSequence<'arena, UseItem<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TypedUseItemSequence<'arena> {
    pub r#type: UseType<'arena>,
    pub items: TokenSeparatedSequence<'arena, UseItem<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TypedUseItemList<'arena> {
    pub r#type: UseType<'arena>,
    pub namespace: Identifier<'arena>,
    pub namespace_separator: Span,
    pub left_brace: Span,
    pub items: TokenSeparatedSequence<'arena, UseItem<'arena>>,
    pub right_brace: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct MixedUseItemList<'arena> {
    pub namespace: Identifier<'arena>,
    pub namespace_separator: Span,
    pub left_brace: Span,
    pub items: TokenSeparatedSequence<'arena, MaybeTypedUseItem<'arena>>,
    pub right_brace: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct MaybeTypedUseItem<'arena> {
    pub r#type: Option<UseType<'arena>>,
    pub item: UseItem<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct UseItem<'arena> {
    pub name: Identifier<'arena>,
    pub alias: Option<UseItemAlias<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct UseItemAlias<'arena> {
    pub r#as: Keyword<'arena>,
    pub identifier: LocalIdentifier<'arena>,
}

impl UseType<'_> {
    #[inline]
    pub const fn is_function(&self) -> bool {
        matches!(self, UseType::Function(_))
    }

    #[inline]
    pub const fn is_const(&self) -> bool {
        matches!(self, UseType::Const(_))
    }
}

impl HasSpan for Use<'_> {
    fn span(&self) -> Span {
        self.r#use.span().join(self.terminator.span())
    }
}

impl HasSpan for UseItems<'_> {
    fn span(&self) -> Span {
        match self {
            UseItems::Sequence(items) => items.span(),
            UseItems::TypedSequence(items) => items.span(),
            UseItems::TypedList(items) => items.span(),
            UseItems::MixedList(items) => items.span(),
        }
    }
}

impl HasSpan for UseType<'_> {
    fn span(&self) -> Span {
        match self {
            UseType::Function(keyword) => keyword.span(),
            UseType::Const(keyword) => keyword.span(),
        }
    }
}

impl HasSpan for UseItemSequence<'_> {
    fn span(&self) -> Span {
        self.items.span(self.file_id, self.start)
    }
}

impl HasSpan for TypedUseItemSequence<'_> {
    fn span(&self) -> Span {
        let types_span = self.r#type.span();

        types_span.join(self.items.span(types_span.file_id, types_span.end))
    }
}

impl HasSpan for TypedUseItemList<'_> {
    fn span(&self) -> Span {
        self.r#type.span().join(self.right_brace)
    }
}

impl HasSpan for MixedUseItemList<'_> {
    fn span(&self) -> Span {
        self.namespace.span().join(self.right_brace)
    }
}

impl HasSpan for MaybeTypedUseItem<'_> {
    fn span(&self) -> Span {
        if let Some(r#type) = &self.r#type { r#type.span().join(self.item.span()) } else { self.item.span() }
    }
}

impl HasSpan for UseItem<'_> {
    fn span(&self) -> Span {
        if let Some(alias) = &self.alias { self.name.span().join(alias.span()) } else { self.name.span() }
    }
}

impl HasSpan for UseItemAlias<'_> {
    fn span(&self) -> Span {
        self.r#as.span().join(self.identifier.span())
    }
}
