use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::text::Text;
use crate::cst::r#type::Type;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExtendsTagValue<'arena> {
    pub r#type: &'arena Type<'arena>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ImplementsTagValue<'arena> {
    pub r#type: &'arena Type<'arena>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct UseTagValue<'arena> {
    pub r#type: &'arena Type<'arena>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RequireExtendsTagValue<'arena> {
    pub r#type: &'arena Type<'arena>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RequireImplementsTagValue<'arena> {
    pub r#type: &'arena Type<'arena>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SealedTagValue<'arena> {
    pub r#type: &'arena Type<'arena>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct InheritorsTagValue<'arena> {
    pub r#type: &'arena Type<'arena>,
    pub description: Option<Text<'arena>>,
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
    UseTagValue,
    RequireExtendsTagValue,
    RequireImplementsTagValue,
    SealedTagValue,
    InheritorsTagValue,
);
