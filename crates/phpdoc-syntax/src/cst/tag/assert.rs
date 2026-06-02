use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::identifier::Identifier;
use crate::cst::text::Text;
use crate::cst::r#type::Type;
use crate::cst::variable::Variable;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct AssertTagValue<'arena> {
    pub bang: Option<Span>,
    pub equals: Option<Span>,
    pub r#type: &'arena Type<'arena>,
    pub parameter: Variable<'arena>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct AssertTagMethodValue<'arena> {
    pub bang: Option<Span>,
    pub equals: Option<Span>,
    pub r#type: &'arena Type<'arena>,
    pub parameter: Variable<'arena>,
    pub arrow: Span,
    pub method: Identifier<'arena>,
    pub left_parenthesis: Span,
    pub right_parenthesis: Span,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct AssertTagPropertyValue<'arena> {
    pub bang: Option<Span>,
    pub equals: Option<Span>,
    pub r#type: &'arena Type<'arena>,
    pub parameter: Variable<'arena>,
    pub arrow: Span,
    pub property: Identifier<'arena>,
    pub description: Option<Text<'arena>>,
}

impl AssertTagValue<'_> {
    #[inline]
    #[must_use]
    pub const fn is_negated(&self) -> bool {
        self.bang.is_some()
    }

    #[inline]
    #[must_use]
    pub const fn is_equality(&self) -> bool {
        self.equals.is_some()
    }
}

impl AssertTagMethodValue<'_> {
    #[inline]
    #[must_use]
    pub const fn is_negated(&self) -> bool {
        self.bang.is_some()
    }

    #[inline]
    #[must_use]
    pub const fn is_equality(&self) -> bool {
        self.equals.is_some()
    }
}

impl AssertTagPropertyValue<'_> {
    #[inline]
    #[must_use]
    pub const fn is_negated(&self) -> bool {
        self.bang.is_some()
    }

    #[inline]
    #[must_use]
    pub const fn is_equality(&self) -> bool {
        self.equals.is_some()
    }
}

impl HasSpan for AssertTagValue<'_> {
    fn span(&self) -> Span {
        let start = self.bang.or(self.equals).unwrap_or_else(|| self.r#type.span());
        let end = self.description.as_ref().map_or_else(|| self.parameter.span(), HasSpan::span);

        start.join(end)
    }
}

impl HasSpan for AssertTagMethodValue<'_> {
    fn span(&self) -> Span {
        let start = self.bang.or(self.equals).unwrap_or_else(|| self.r#type.span());
        let end = self.description.as_ref().map_or(self.right_parenthesis, HasSpan::span);

        start.join(end)
    }
}

impl HasSpan for AssertTagPropertyValue<'_> {
    fn span(&self) -> Span {
        let start = self.bang.or(self.equals).unwrap_or_else(|| self.r#type.span());
        let end = self.description.as_ref().map_or_else(|| self.property.span(), HasSpan::span);

        start.join(end)
    }
}
