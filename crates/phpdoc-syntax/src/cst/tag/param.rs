use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::text::Text;
use crate::cst::r#type::Type;
use crate::cst::variable::Variable;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ParamTagValue<'arena> {
    pub r#type: &'arena Type<'arena>,
    pub ampersand: Option<Span>,
    pub ellipsis: Option<Span>,
    pub parameter: Option<Variable<'arena>>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TypelessParamTagValue<'arena> {
    pub ampersand: Option<Span>,
    pub ellipsis: Option<Span>,
    pub parameter: Variable<'arena>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ParamOutTagValue<'arena> {
    pub r#type: &'arena Type<'arena>,
    pub parameter: Variable<'arena>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ParamClosureThisTagValue<'arena> {
    pub r#type: &'arena Type<'arena>,
    pub parameter: Variable<'arena>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ParamImmediatelyInvokedCallableTagValue<'arena> {
    pub parameter: Variable<'arena>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ParamLaterInvokedCallableTagValue<'arena> {
    pub parameter: Variable<'arena>,
    pub description: Option<Text<'arena>>,
}

impl ParamTagValue<'_> {
    #[inline]
    #[must_use]
    pub const fn is_by_reference(&self) -> bool {
        self.ampersand.is_some()
    }

    #[inline]
    #[must_use]
    pub const fn is_variadic(&self) -> bool {
        self.ellipsis.is_some()
    }
}

impl TypelessParamTagValue<'_> {
    #[inline]
    #[must_use]
    pub const fn is_by_reference(&self) -> bool {
        self.ampersand.is_some()
    }

    #[inline]
    #[must_use]
    pub const fn is_variadic(&self) -> bool {
        self.ellipsis.is_some()
    }
}

impl HasSpan for ParamTagValue<'_> {
    fn span(&self) -> Span {
        let end = self
            .description
            .as_ref()
            .map(HasSpan::span)
            .or_else(|| self.parameter.as_ref().map(HasSpan::span))
            .or(self.ellipsis)
            .or(self.ampersand)
            .unwrap_or_else(|| self.r#type.span());

        self.r#type.span().join(end)
    }
}

impl HasSpan for TypelessParamTagValue<'_> {
    fn span(&self) -> Span {
        let start = self.ampersand.or(self.ellipsis).unwrap_or_else(|| self.parameter.span());
        let end = self.description.as_ref().map_or_else(|| self.parameter.span(), HasSpan::span);

        start.join(end)
    }
}

impl HasSpan for ParamOutTagValue<'_> {
    fn span(&self) -> Span {
        let end = self.description.as_ref().map_or_else(|| self.parameter.span(), HasSpan::span);

        self.r#type.span().join(end)
    }
}

impl HasSpan for ParamClosureThisTagValue<'_> {
    fn span(&self) -> Span {
        let end = self.description.as_ref().map_or_else(|| self.parameter.span(), HasSpan::span);

        self.r#type.span().join(end)
    }
}

impl HasSpan for ParamImmediatelyInvokedCallableTagValue<'_> {
    fn span(&self) -> Span {
        let end = self.description.as_ref().map_or_else(|| self.parameter.span(), HasSpan::span);

        self.parameter.span().join(end)
    }
}

impl HasSpan for ParamLaterInvokedCallableTagValue<'_> {
    fn span(&self) -> Span {
        let end = self.description.as_ref().map_or_else(|| self.parameter.span(), HasSpan::span);

        self.parameter.span().join(end)
    }
}
