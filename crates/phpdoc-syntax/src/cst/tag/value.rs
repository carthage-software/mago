use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::text::Text;
use crate::cst::r#type::Type;
use crate::cst::variable::Variable;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ReturnTagValue<'arena> {
    pub r#type: &'arena Type<'arena>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ThrowsTagValue<'arena> {
    pub r#type: &'arena Type<'arena>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct VarTagValue<'arena> {
    pub r#type: &'arena Type<'arena>,
    pub variable: Option<Variable<'arena>>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct MixinTagValue<'arena> {
    pub r#type: &'arena Type<'arena>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct SelfOutTagValue<'arena> {
    pub r#type: &'arena Type<'arena>,
    pub description: Option<Text<'arena>>,
}

impl HasSpan for ReturnTagValue<'_> {
    fn span(&self) -> Span {
        match &self.description {
            Some(description) => self.r#type.span().join(description.span()),
            None => self.r#type.span(),
        }
    }
}

impl HasSpan for ThrowsTagValue<'_> {
    fn span(&self) -> Span {
        match &self.description {
            Some(description) => self.r#type.span().join(description.span()),
            None => self.r#type.span(),
        }
    }
}

impl HasSpan for VarTagValue<'_> {
    fn span(&self) -> Span {
        let end = self
            .description
            .as_ref()
            .map(HasSpan::span)
            .or_else(|| self.variable.as_ref().map(HasSpan::span))
            .unwrap_or_else(|| self.r#type.span());

        self.r#type.span().join(end)
    }
}

impl HasSpan for MixinTagValue<'_> {
    fn span(&self) -> Span {
        match &self.description {
            Some(description) => self.r#type.span().join(description.span()),
            None => self.r#type.span(),
        }
    }
}

impl HasSpan for SelfOutTagValue<'_> {
    fn span(&self) -> Span {
        match &self.description {
            Some(description) => self.r#type.span().join(description.span()),
            None => self.r#type.span(),
        }
    }
}
