use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::identifier::Identifier;

pub use crate::cst::tag::assert::*;
pub use crate::cst::tag::inheritance::*;
pub use crate::cst::tag::meta::*;
pub use crate::cst::tag::method::*;
pub use crate::cst::tag::param::*;
pub use crate::cst::tag::property::*;
pub use crate::cst::tag::template::*;
pub use crate::cst::tag::type_alias::*;
pub use crate::cst::tag::value::*;
pub use crate::cst::tag::vendor::*;
pub use crate::cst::tag::where_clause::*;

pub mod assert;
pub mod inheritance;
pub mod meta;
pub mod method;
pub mod param;
pub mod property;
pub mod template;
pub mod type_alias;
pub mod value;
pub mod vendor;
pub mod where_clause;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Tag<'arena> {
    pub at: Span,
    pub name: Identifier<'arena>,
    pub vendor: Option<TagVendor>,
    pub value: TagValue<'arena>,
}

impl HasSpan for Tag<'_> {
    fn span(&self) -> Span {
        self.at.span().join(self.value.span())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
pub enum TagValue<'arena> {
    Param(ParamTagValue<'arena>),
    TypelessParam(TypelessParamTagValue<'arena>),
    ParamOut(ParamOutTagValue<'arena>),
    ParamClosureThis(ParamClosureThisTagValue<'arena>),
    ParamImmediatelyInvokedCallable(ParamImmediatelyInvokedCallableTagValue<'arena>),
    ParamLaterInvokedCallable(ParamLaterInvokedCallableTagValue<'arena>),
    Return(ReturnTagValue<'arena>),
    RealReturn(ReturnTagValue<'arena>),
    Var(VarTagValue<'arena>),
    Throws(ThrowsTagValue<'arena>),
    Mixin(MixinTagValue<'arena>),
    SelfOut(SelfOutTagValue<'arena>),
    Template(TemplateTagValue<'arena>),
    Extends(ExtendsTagValue<'arena>),
    Implements(ImplementsTagValue<'arena>),
    Use(UseTagValue<'arena>),
    RequireExtends(RequireExtendsTagValue<'arena>),
    RequireImplements(RequireImplementsTagValue<'arena>),
    Sealed(SealedTagValue<'arena>),
    Inheritors(InheritorsTagValue<'arena>),
    Method(MethodTagValue<'arena>),
    Property(PropertyTagValue<'arena>),
    PropertyRead(PropertyTagValue<'arena>),
    PropertyWrite(PropertyTagValue<'arena>),
    Assert(AssertTagValue<'arena>),
    AssertIfTrue(AssertTagValue<'arena>),
    AssertIfFalse(AssertTagValue<'arena>),
    Where(WhereTagValue<'arena>),
    Deprecated(DeprecatedTagValue<'arena>),
    Final(FinalTagValue<'arena>),
    Internal(InternalTagValue<'arena>),
    Api(ApiTagValue<'arena>),
    Experimental(ExperimentalTagValue<'arena>),
    Pure(PureTagValue<'arena>),
    Impure(ImpureTagValue<'arena>),
    Readonly(ReadonlyTagValue<'arena>),
    MustUse(MustUseTagValue<'arena>),
    NoNamedArguments(NoNamedArgumentsTagValue<'arena>),
    NotDeprecated(NotDeprecatedTagValue<'arena>),
    EnumInterface(EnumInterfaceTagValue<'arena>),
    ConsistentConstructor(ConsistentConstructorTagValue<'arena>),
    ConsistentTemplates(ConsistentTemplatesTagValue<'arena>),
    SealProperties(SealPropertiesTagValue<'arena>),
    NoSealProperties(NoSealPropertiesTagValue<'arena>),
    SealMethods(SealMethodsTagValue<'arena>),
    NoSealMethods(NoSealMethodsTagValue<'arena>),
    MutationFree(MutationFreeTagValue<'arena>),
    ExternalMutationFree(ExternalMutationFreeTagValue<'arena>),
    SuspendsFiber(SuspendsFiberTagValue<'arena>),
    IgnoreNullableReturn(IgnoreNullableReturnTagValue<'arena>),
    IgnoreFalsableReturn(IgnoreFalsableReturnTagValue<'arena>),
    InheritDoc(InheritDocTagValue<'arena>),
    Trace(TraceTagValue<'arena>),
    TypeAlias(TypeAliasTagValue<'arena>),
    TypeAliasImport(TypeAliasImportTagValue<'arena>),
    PureUnlessCallableIsImpure(PureUnlessCallableIsImpureTagValue<'arena>),
    Generic(GenericTagValue<'arena>),
    Invalid(InvalidTagValue<'arena>),
}

impl HasSpan for TagValue<'_> {
    fn span(&self) -> Span {
        match self {
            TagValue::Param(value) => value.span(),
            TagValue::TypelessParam(value) => value.span(),
            TagValue::ParamOut(value) => value.span(),
            TagValue::ParamClosureThis(value) => value.span(),
            TagValue::ParamImmediatelyInvokedCallable(value) => value.span(),
            TagValue::ParamLaterInvokedCallable(value) => value.span(),
            TagValue::Return(value) => value.span(),
            TagValue::RealReturn(value) => value.span(),
            TagValue::Var(value) => value.span(),
            TagValue::Throws(value) => value.span(),
            TagValue::Mixin(value) => value.span(),
            TagValue::SelfOut(value) => value.span(),
            TagValue::Template(value) => value.span(),
            TagValue::Extends(value) => value.span(),
            TagValue::Implements(value) => value.span(),
            TagValue::Use(value) => value.span(),
            TagValue::RequireExtends(value) => value.span(),
            TagValue::RequireImplements(value) => value.span(),
            TagValue::Sealed(value) => value.span(),
            TagValue::Inheritors(value) => value.span(),
            TagValue::Method(value) => value.span(),
            TagValue::Property(value) => value.span(),
            TagValue::PropertyRead(value) => value.span(),
            TagValue::PropertyWrite(value) => value.span(),
            TagValue::Assert(value) => value.span(),
            TagValue::AssertIfTrue(value) => value.span(),
            TagValue::AssertIfFalse(value) => value.span(),
            TagValue::Where(value) => value.span(),
            TagValue::Deprecated(value) => value.span(),
            TagValue::Final(value) => value.span(),
            TagValue::Internal(value) => value.span(),
            TagValue::Api(value) => value.span(),
            TagValue::Experimental(value) => value.span(),
            TagValue::Pure(value) => value.span(),
            TagValue::Impure(value) => value.span(),
            TagValue::Readonly(value) => value.span(),
            TagValue::MustUse(value) => value.span(),
            TagValue::NoNamedArguments(value) => value.span(),
            TagValue::NotDeprecated(value) => value.span(),
            TagValue::EnumInterface(value) => value.span(),
            TagValue::ConsistentConstructor(value) => value.span(),
            TagValue::ConsistentTemplates(value) => value.span(),
            TagValue::SealProperties(value) => value.span(),
            TagValue::NoSealProperties(value) => value.span(),
            TagValue::SealMethods(value) => value.span(),
            TagValue::NoSealMethods(value) => value.span(),
            TagValue::MutationFree(value) => value.span(),
            TagValue::ExternalMutationFree(value) => value.span(),
            TagValue::SuspendsFiber(value) => value.span(),
            TagValue::IgnoreNullableReturn(value) => value.span(),
            TagValue::IgnoreFalsableReturn(value) => value.span(),
            TagValue::InheritDoc(value) => value.span(),
            TagValue::Trace(value) => value.span(),
            TagValue::TypeAlias(value) => value.span(),
            TagValue::TypeAliasImport(value) => value.span(),
            TagValue::PureUnlessCallableIsImpure(value) => value.span(),
            TagValue::Generic(value) => value.span(),
            TagValue::Invalid(value) => value.span(),
        }
    }
}
