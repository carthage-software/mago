use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::text::Text;
use crate::cst::variable::Variable;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct DeprecatedTagValue<'arena> {
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct PureUnlessCallableIsImpureTagValue<'arena> {
    pub parameter: Variable<'arena>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct GenericTagValue<'arena> {
    pub value: Text<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct InvalidTagValue<'arena> {
    pub value: Text<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct InheritDocTagValue<'arena> {
    pub description: Text<'arena>,
}

impl HasSpan for DeprecatedTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for InheritDocTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for PureUnlessCallableIsImpureTagValue<'_> {
    fn span(&self) -> Span {
        let end = self.description.as_ref().map_or_else(|| self.parameter.span(), HasSpan::span);

        self.parameter.span().join(end)
    }
}

impl HasSpan for GenericTagValue<'_> {
    fn span(&self) -> Span {
        self.value.span()
    }
}

impl HasSpan for InvalidTagValue<'_> {
    fn span(&self) -> Span {
        self.value.span()
    }
}
