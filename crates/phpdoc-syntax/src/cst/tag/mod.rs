use serde::Serialize;

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

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Tag<'arena> {
    pub name: Identifier<'arena>,
    pub vendor: Option<TagVendor>,
    pub value: TagValue<'arena>,
}

impl HasSpan for Tag<'_> {
    fn span(&self) -> Span {
        self.name.span().join(self.value.span())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "kind", content = "value")]
pub enum TagValue<'arena> {
    Param(ParamTagValue<'arena>),
    TypelessParam(TypelessParamTagValue<'arena>),
    ParamOut(ParamOutTagValue<'arena>),
    ParamClosureThis(ParamClosureThisTagValue<'arena>),
    ParamImmediatelyInvokedCallable(ParamImmediatelyInvokedCallableTagValue<'arena>),
    ParamLaterInvokedCallable(ParamLaterInvokedCallableTagValue<'arena>),
    Return(ReturnTagValue<'arena>),
    Var(VarTagValue<'arena>),
    Throws(ThrowsTagValue<'arena>),
    Mixin(MixinTagValue<'arena>),
    SelfOut(SelfOutTagValue<'arena>),
    Template(TemplateTagValue<'arena>),
    Extends(ExtendsTagValue<'arena>),
    Implements(ImplementsTagValue<'arena>),
    Uses(UsesTagValue<'arena>),
    RequireExtends(RequireExtendsTagValue<'arena>),
    RequireImplements(RequireImplementsTagValue<'arena>),
    Sealed(SealedTagValue<'arena>),
    Inheritors(InheritorsTagValue<'arena>),
    Method(MethodTagValue<'arena>),
    Property(PropertyTagValue<'arena>),
    Assert(AssertTagValue<'arena>),
    AssertMethod(AssertTagMethodValue<'arena>),
    AssertProperty(AssertTagPropertyValue<'arena>),
    Where(WhereTagValue<'arena>),
    Deprecated(DeprecatedTagValue<'arena>),
    InheritDoc(InheritDocTagValue<'arena>),
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
            TagValue::Var(value) => value.span(),
            TagValue::Throws(value) => value.span(),
            TagValue::Mixin(value) => value.span(),
            TagValue::SelfOut(value) => value.span(),
            TagValue::Template(value) => value.span(),
            TagValue::Extends(value) => value.span(),
            TagValue::Implements(value) => value.span(),
            TagValue::Uses(value) => value.span(),
            TagValue::RequireExtends(value) => value.span(),
            TagValue::RequireImplements(value) => value.span(),
            TagValue::Sealed(value) => value.span(),
            TagValue::Inheritors(value) => value.span(),
            TagValue::Method(value) => value.span(),
            TagValue::Property(value) => value.span(),
            TagValue::Assert(value) => value.span(),
            TagValue::AssertMethod(value) => value.span(),
            TagValue::AssertProperty(value) => value.span(),
            TagValue::Where(value) => value.span(),
            TagValue::Deprecated(value) => value.span(),
            TagValue::InheritDoc(value) => value.span(),
            TagValue::TypeAlias(value) => value.span(),
            TagValue::TypeAliasImport(value) => value.span(),
            TagValue::PureUnlessCallableIsImpure(value) => value.span(),
            TagValue::Generic(value) => value.span(),
            TagValue::Invalid(value) => value.span(),
        }
    }
}
