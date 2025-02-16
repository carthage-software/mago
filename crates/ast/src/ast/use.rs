use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Position;
use mago_span::Span;

use crate::ast::identifier::Identifier;
use crate::ast::identifier::LocalIdentifier;
use crate::ast::keyword::Keyword;
use crate::ast::terminator::Terminator;
use crate::sequence::TokenSeparatedSequence;

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Use<'a> {
    pub r#use: Keyword,
    pub items: UseItems<'a>,
    pub terminator: Terminator,
}

#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum UseItems<'a> {
    Sequence(UseItemSequence<'a>),
    TypedSequence(TypedUseItemSequence<'a>),
    TypedList(TypedUseItemList<'a>),
    MixedList(MixedUseItemList<'a>),
}

#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum UseType {
    Function(Keyword),
    Const(Keyword),
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct UseItemSequence<'a> {
    pub start: Position,
    pub items: TokenSeparatedSequence<'a, UseItem>,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct TypedUseItemSequence<'a> {
    pub r#type: UseType,
    pub items: TokenSeparatedSequence<'a, UseItem>,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct TypedUseItemList<'a> {
    pub r#type: UseType,
    pub namespace: Identifier,
    pub namespace_separator: Span,
    pub left_brace: Span,
    pub items: TokenSeparatedSequence<'a, UseItem>,
    pub right_brace: Span,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct MixedUseItemList<'a> {
    pub namespace: Identifier,
    pub namespace_separator: Span,
    pub left_brace: Span,
    pub items: TokenSeparatedSequence<'a, MaybeTypedUseItem>,
    pub right_brace: Span,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct MaybeTypedUseItem {
    pub r#type: Option<UseType>,
    pub item: UseItem,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct UseItem {
    pub name: Identifier,
    pub alias: Option<UseItemAlias>,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct UseItemAlias {
    pub r#as: Keyword,
    pub identifier: LocalIdentifier,
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

impl HasSpan for UseType {
    fn span(&self) -> Span {
        match self {
            UseType::Function(keyword) => keyword.span(),
            UseType::Const(keyword) => keyword.span(),
        }
    }
}

impl HasSpan for UseItemSequence<'_> {
    fn span(&self) -> Span {
        self.items.span(self.start)
    }
}

impl HasSpan for TypedUseItemSequence<'_> {
    fn span(&self) -> Span {
        self.r#type.span().join(self.items.span(self.r#type.span().end))
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

impl HasSpan for MaybeTypedUseItem {
    fn span(&self) -> Span {
        if let Some(r#type) = &self.r#type {
            r#type.span().join(self.item.span())
        } else {
            self.item.span()
        }
    }
}

impl HasSpan for UseItem {
    fn span(&self) -> Span {
        if let Some(alias) = &self.alias {
            self.name.span().join(alias.span())
        } else {
            self.name.span()
        }
    }
}

impl HasSpan for UseItemAlias {
    fn span(&self) -> Span {
        self.r#as.span().join(self.identifier.span())
    }
}
