use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::text::Text;
use crate::cst::variable::Variable;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DeprecatedTagValue<'arena> {
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FinalTagValue<'arena> {
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct InternalTagValue<'arena> {
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ApiTagValue<'arena> {
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExperimentalTagValue<'arena> {
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PureTagValue<'arena> {
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ImpureTagValue<'arena> {
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ReadonlyTagValue<'arena> {
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MustUseTagValue<'arena> {
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NoNamedArgumentsTagValue<'arena> {
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NotDeprecatedTagValue<'arena> {
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EnumInterfaceTagValue<'arena> {
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ConsistentConstructorTagValue<'arena> {
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ConsistentTemplatesTagValue<'arena> {
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SealPropertiesTagValue<'arena> {
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NoSealPropertiesTagValue<'arena> {
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SealMethodsTagValue<'arena> {
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NoSealMethodsTagValue<'arena> {
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MutationFreeTagValue<'arena> {
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExternalMutationFreeTagValue<'arena> {
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SuspendsFiberTagValue<'arena> {
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct IgnoreNullableReturnTagValue<'arena> {
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct IgnoreFalsableReturnTagValue<'arena> {
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct InheritDocTagValue<'arena> {
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TraceTagValue<'arena> {
    pub variable: Variable<'arena>,
    pub description: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PureUnlessCallableIsImpureTagValue<'arena> {
    pub parameter: Variable<'arena>,
    pub description: Option<Text<'arena>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct GenericTagValue<'arena> {
    pub value: Text<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct InvalidTagValue<'arena> {
    pub value: Text<'arena>,
}

impl HasSpan for DeprecatedTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for FinalTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for InternalTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for ApiTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for ExperimentalTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for PureTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for ImpureTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for ReadonlyTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for MustUseTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for NoNamedArgumentsTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for NotDeprecatedTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for EnumInterfaceTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for ConsistentConstructorTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for ConsistentTemplatesTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for SealPropertiesTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for NoSealPropertiesTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for SealMethodsTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for NoSealMethodsTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for MutationFreeTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for ExternalMutationFreeTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for SuspendsFiberTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for IgnoreNullableReturnTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for IgnoreFalsableReturnTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for InheritDocTagValue<'_> {
    fn span(&self) -> Span {
        self.description.span()
    }
}

impl HasSpan for TraceTagValue<'_> {
    fn span(&self) -> Span {
        self.variable.span.join(self.description.span)
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
