use mago_syntax_core::cst::TokenSeparatedSequence;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::identifier::Identifier;
use crate::cst::text::Text;
use crate::cst::r#type::Type;
use crate::token::Token;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ExtendsTagValue<'arena> {
    pub r#type: &'arena Type<'arena>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ImplementsTagValue<'arena> {
    pub r#type: &'arena Type<'arena>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct UsesTagValue<'arena> {
    pub r#type: &'arena Type<'arena>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct RequireExtendsTagValue<'arena> {
    pub r#type: &'arena Type<'arena>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct RequireImplementsTagValue<'arena> {
    pub r#type: &'arena Type<'arena>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct SealedTagValue<'arena> {
    pub r#type: &'arena Type<'arena>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct InheritorsTagValue<'arena> {
    pub span: Span,
    pub inheritors: TokenSeparatedSequence<'arena, Identifier<'arena>, Token<'arena>>,
}

impl HasSpan for InheritorsTagValue<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

macro_rules! single_type_with_description {
    ($($name:ident),+ $(,)?) => {
        $(
            impl HasSpan for $name<'_> {
                fn span(&self) -> Span {
                    match &self.description {
                        Some(description) => self.r#type.span().join(description.span()),
                        None => self.r#type.span(),
                    }
                }
            }
        )+
    };
}

single_type_with_description!(
    ExtendsTagValue,
    ImplementsTagValue,
    UsesTagValue,
    RequireExtendsTagValue,
    RequireImplementsTagValue,
    SealedTagValue,
);
