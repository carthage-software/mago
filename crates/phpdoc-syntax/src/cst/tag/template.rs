use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::identifier::Identifier;
use crate::cst::keyword::Keyword;
use crate::cst::text::Text;
use crate::cst::r#type::Type;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TemplateTagValue<'arena> {
    pub name: Identifier<'arena>,
    pub variance: TemplateTagValueVariance,
    pub bound: Option<TemplateTagValueBound<'arena>>,
    pub default: Option<TemplateTagValueDefault<'arena>>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum TemplateTagValueVariance {
    Invariant,
    Covariant,
    Contravariant,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TemplateTagValueBound<'arena> {
    pub keyword: Keyword<'arena>,
    pub r#type: &'arena Type<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TemplateTagValueDefault<'arena> {
    pub equals: Span,
    pub r#type: &'arena Type<'arena>,
}

impl HasSpan for TemplateTagValue<'_> {
    fn span(&self) -> Span {
        let end = self
            .description
            .as_ref()
            .map(HasSpan::span)
            .or_else(|| self.default.as_ref().map(HasSpan::span))
            .or_else(|| self.bound.as_ref().map(HasSpan::span))
            .unwrap_or_else(|| self.name.span());

        self.name.span().join(end)
    }
}

impl HasSpan for TemplateTagValueBound<'_> {
    fn span(&self) -> Span {
        self.keyword.span().join(self.r#type.span())
    }
}

impl HasSpan for TemplateTagValueDefault<'_> {
    fn span(&self) -> Span {
        self.equals.join(self.r#type.span())
    }
}
