use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax_core::cst::Sequence;

use crate::cst::Keyword;
use crate::cst::expression::ConstantExpression;
use crate::cst::identifier::Identifier;
use crate::cst::tag::template::TemplateTagValue;
use crate::cst::text::Text;
use crate::cst::r#type::Type;
use crate::cst::variable::Variable;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MethodTagValue<'arena> {
    pub r#static: Option<Span>,
    pub visibility: Option<Visibility<'arena>>,
    pub return_type: Option<&'arena Type<'arena>>,
    pub name: Identifier<'arena>,
    pub templates: Option<&'arena MethodTemplateParameterList<'arena>>,
    pub parameters: &'arena MethodParameterList<'arena>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
pub enum Visibility<'arena> {
    Public(Keyword<'arena>),
    Protected(Keyword<'arena>),
    Private(Keyword<'arena>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MethodTemplateParameterList<'arena> {
    pub less_than: Span,
    pub entries: Sequence<'arena, MethodTemplateParameter<'arena>>,
    pub greater_than: Span,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MethodTemplateParameter<'arena> {
    pub template: TemplateTagValue<'arena>,
    pub comma: Option<Span>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MethodParameterList<'arena> {
    pub left_parenthesis: Span,
    pub entries: Sequence<'arena, MethodTagValueParameter<'arena>>,
    pub right_parenthesis: Span,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MethodTagValueParameter<'arena> {
    pub r#type: Option<&'arena Type<'arena>>,
    pub ampersand: Option<Span>,
    pub ellipsis: Option<Span>,
    pub parameter: Variable<'arena>,
    pub default: Option<MethodTagValueParameterDefault<'arena>>,
    pub comma: Option<Span>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MethodTagValueParameterDefault<'arena> {
    pub equals: Span,
    pub value: &'arena ConstantExpression<'arena>,
}

impl MethodTagValue<'_> {
    #[inline]
    #[must_use]
    pub const fn is_static(&self) -> bool {
        self.r#static.is_some()
    }
}

impl MethodTagValueParameter<'_> {
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

    #[inline]
    #[must_use]
    pub const fn is_optional(&self) -> bool {
        self.default.is_some()
    }
}

impl HasSpan for MethodTagValue<'_> {
    fn span(&self) -> Span {
        let start = self.r#static.or_else(|| self.return_type.map(HasSpan::span)).unwrap_or_else(|| self.name.span());
        let end = self.description.as_ref().map_or_else(|| self.parameters.span(), HasSpan::span);

        start.join(end)
    }
}

impl HasSpan for MethodTemplateParameterList<'_> {
    fn span(&self) -> Span {
        self.less_than.join(self.greater_than)
    }
}

impl HasSpan for MethodTemplateParameter<'_> {
    fn span(&self) -> Span {
        match &self.comma {
            Some(comma) => self.template.span().join(*comma),
            None => self.template.span(),
        }
    }
}

impl HasSpan for MethodParameterList<'_> {
    fn span(&self) -> Span {
        self.left_parenthesis.join(self.right_parenthesis)
    }
}

impl HasSpan for MethodTagValueParameter<'_> {
    fn span(&self) -> Span {
        let start = match &self.r#type {
            Some(r#type) => r#type.span(),
            None => self.ampersand.or(self.ellipsis).unwrap_or_else(|| self.parameter.span()),
        };

        let end =
            self.comma.or_else(|| self.default.as_ref().map(HasSpan::span)).unwrap_or_else(|| self.parameter.span());

        start.join(end)
    }
}

impl HasSpan for MethodTagValueParameterDefault<'_> {
    fn span(&self) -> Span {
        self.equals.join(self.value.span())
    }
}
