use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::identifier::Identifier;
use crate::cst::text::Text;
use crate::cst::r#type::Type;
use crate::cst::variable::Variable;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum AssertSubject<'arena> {
    Parameter {
        variable: Variable<'arena>,
    },
    Method {
        parameter: Variable<'arena>,
        arrow: Span,
        method: Identifier<'arena>,
        left_parenthesis: Span,
        right_parenthesis: Span,
    },
    Property {
        parameter: Variable<'arena>,
        arrow: Span,
        property: Identifier<'arena>,
    },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum AssertPattern<'arena> {
    Type(&'arena Type<'arena>),
    Truthy(Span),
    Falsy(Span),
    NonEmpty(Span),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AssertTagValue<'arena> {
    pub bang: Option<Span>,
    pub equals: Option<Span>,
    pub pattern: AssertPattern<'arena>,
    pub subject: AssertSubject<'arena>,
    pub description: Option<Text<'arena>>,
}

impl AssertTagValue<'_> {
    #[inline]
    #[must_use]
    pub const fn is_subject_parameter(&self) -> bool {
        matches!(self.subject, AssertSubject::Parameter { .. })
    }

    #[inline]
    #[must_use]
    pub const fn is_subject_method(&self) -> bool {
        matches!(self.subject, AssertSubject::Method { .. })
    }

    #[inline]
    #[must_use]
    pub const fn is_subject_property(&self) -> bool {
        matches!(self.subject, AssertSubject::Property { .. })
    }

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

impl HasSpan for AssertSubject<'_> {
    fn span(&self) -> Span {
        match self {
            AssertSubject::Parameter { variable } => variable.span(),
            AssertSubject::Method { parameter, right_parenthesis, .. } => parameter.span().join(*right_parenthesis),
            AssertSubject::Property { parameter, property, .. } => parameter.span().join(property.span()),
        }
    }
}

impl HasSpan for AssertPattern<'_> {
    fn span(&self) -> Span {
        match self {
            AssertPattern::Type(r#type) => r#type.span(),
            AssertPattern::Truthy(span) | AssertPattern::Falsy(span) | AssertPattern::NonEmpty(span) => *span,
        }
    }
}

impl HasSpan for AssertTagValue<'_> {
    fn span(&self) -> Span {
        let start = self.bang.or(self.equals).unwrap_or_else(|| self.pattern.span());
        let end = self.description.as_ref().map_or_else(|| self.subject.span(), HasSpan::span);

        start.join(end)
    }
}
