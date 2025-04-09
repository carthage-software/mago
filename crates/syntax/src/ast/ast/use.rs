use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Position;
use mago_span::Span;

use crate::ast::ast::identifier::Identifier;
use crate::ast::ast::identifier::LocalIdentifier;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::terminator::Terminator;
use crate::ast::sequence::TokenSeparatedSequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct Use {
    pub r#use: Keyword,
    pub items: UseItems,
    pub terminator: Terminator,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum UseItems {
    Sequence(UseItemSequence),
    TypedSequence(TypedUseItemSequence),
    TypedList(TypedUseItemList),
    MixedList(MixedUseItemList),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum UseType {
    Function(Keyword),
    Const(Keyword),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct UseItemSequence {
    pub start: Position,
    pub items: TokenSeparatedSequence<UseItem>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct TypedUseItemSequence {
    pub r#type: UseType,
    pub items: TokenSeparatedSequence<UseItem>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct TypedUseItemList {
    pub r#type: UseType,
    pub namespace: Identifier,
    pub namespace_separator: Span,
    pub left_brace: Span,
    pub items: TokenSeparatedSequence<UseItem>,
    pub right_brace: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct MixedUseItemList {
    pub namespace: Identifier,
    pub namespace_separator: Span,
    pub left_brace: Span,
    pub items: TokenSeparatedSequence<MaybeTypedUseItem>,
    pub right_brace: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct MaybeTypedUseItem {
    pub r#type: Option<UseType>,
    pub item: UseItem,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct UseItem {
    pub name: Identifier,
    pub alias: Option<UseItemAlias>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct UseItemAlias {
    pub r#as: Keyword,
    pub identifier: LocalIdentifier,
}

impl UseType {
    #[inline(always)]
    pub const fn is_function(&self) -> bool {
        matches!(self, UseType::Function(_))
    }

    #[inline(always)]
    pub const fn is_const(&self) -> bool {
        matches!(self, UseType::Const(_))
    }
}

impl HasSpan for Use {
    fn span(&self) -> Span {
        self.r#use.span().join(self.terminator.span())
    }
}

impl HasSpan for UseItems {
    fn span(&self) -> Span {
        match self {
            UseItems::Sequence(items) => items.span(),
            UseItems::TypedSequence(items) => items.span(),
            UseItems::TypedList(items) => items.span(),
            UseItems::MixedList(items) => items.span(),
        }
    }
}

impl HasSpan for UseType {
    fn span(&self) -> Span {
        match self {
            UseType::Function(keyword) => keyword.span(),
            UseType::Const(keyword) => keyword.span(),
        }
    }
}

impl HasSpan for UseItemSequence {
    fn span(&self) -> Span {
        self.items.span(self.start)
    }
}

impl HasSpan for TypedUseItemSequence {
    fn span(&self) -> Span {
        self.r#type.span().join(self.items.span(self.r#type.span().end))
    }
}

impl HasSpan for TypedUseItemList {
    fn span(&self) -> Span {
        self.r#type.span().join(self.right_brace)
    }
}

impl HasSpan for MixedUseItemList {
    fn span(&self) -> Span {
        self.namespace.span().join(self.right_brace)
    }
}

impl HasSpan for MaybeTypedUseItem {
    fn span(&self) -> Span {
        if let Some(r#type) = &self.r#type { r#type.span().join(self.item.span()) } else { self.item.span() }
    }
}

impl HasSpan for UseItem {
    fn span(&self) -> Span {
        if let Some(alias) = &self.alias { self.name.span().join(alias.span()) } else { self.name.span() }
    }
}

impl HasSpan for UseItemAlias {
    fn span(&self) -> Span {
        self.r#as.span().join(self.identifier.span())
    }
}
