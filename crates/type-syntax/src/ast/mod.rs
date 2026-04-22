use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

pub use crate::ast::alias_reference::*;
pub use crate::ast::array::*;
pub use crate::ast::callable::*;
pub use crate::ast::class_like_string::*;
pub use crate::ast::composite::*;
pub use crate::ast::conditional::*;
pub use crate::ast::generics::*;
pub use crate::ast::identifier::*;
pub use crate::ast::index_access::*;
pub use crate::ast::int_mask::*;
pub use crate::ast::int_range::*;
pub use crate::ast::iterable::*;
pub use crate::ast::key_of::*;
pub use crate::ast::keyword::*;
pub use crate::ast::literal::*;
pub use crate::ast::new::*;
use crate::ast::object::ObjectType;
pub use crate::ast::properties_of::*;
pub use crate::ast::reference::*;
pub use crate::ast::shape::*;
pub use crate::ast::slice::*;
pub use crate::ast::template_type::*;
pub use crate::ast::unary::*;
pub use crate::ast::value_of::*;
pub use crate::ast::variable::*;
pub use crate::ast::wildcard::*;

pub mod alias_reference;
pub mod array;
pub mod callable;
pub mod class_like_string;
pub mod composite;
pub mod conditional;
pub mod generics;
pub mod identifier;
pub mod index_access;
pub mod int_mask;
pub mod int_range;
pub mod iterable;
pub mod key_of;
pub mod keyword;
pub mod literal;
pub mod new;
pub mod object;
pub mod properties_of;
pub mod reference;
pub mod shape;
pub mod slice;
pub mod template_type;
pub mod unary;
pub mod value_of;
pub mod variable;
pub mod wildcard;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
#[non_exhaustive]
pub enum Type<'arena> {
    Parenthesized(ParenthesizedType<'arena>),
    Union(UnionType<'arena>),
    Intersection(IntersectionType<'arena>),
    Nullable(NullableType<'arena>),
    Array(ArrayType<'arena>),
    NonEmptyArray(NonEmptyArrayType<'arena>),
    AssociativeArray(AssociativeArrayType<'arena>),
    List(ListType<'arena>),
    NonEmptyList(NonEmptyListType<'arena>),
    Iterable(IterableType<'arena>),
    ClassString(ClassStringType<'arena>),
    InterfaceString(InterfaceStringType<'arena>),
    EnumString(EnumStringType<'arena>),
    TraitString(TraitStringType<'arena>),
    Reference(ReferenceType<'arena>),
    Mixed(Keyword<'arena>),
    NonEmptyMixed(Keyword<'arena>),
    Null(Keyword<'arena>),
    Void(Keyword<'arena>),
    Never(Keyword<'arena>),
    Resource(Keyword<'arena>),
    ClosedResource(Keyword<'arena>),
    OpenResource(Keyword<'arena>),
    True(Keyword<'arena>),
    False(Keyword<'arena>),
    Bool(Keyword<'arena>),
    Float(Keyword<'arena>),
    Int(Keyword<'arena>),
    PositiveInt(Keyword<'arena>),
    NegativeInt(Keyword<'arena>),
    NonPositiveInt(Keyword<'arena>),
    NonNegativeInt(Keyword<'arena>),
    NonZeroInt(Keyword<'arena>),
    String(Keyword<'arena>),
    StringableObject(Keyword<'arena>),
    ArrayKey(Keyword<'arena>),
    Object(ObjectType<'arena>),
    Numeric(Keyword<'arena>),
    Scalar(Keyword<'arena>),
    CallableString(Keyword<'arena>),
    LowercaseCallableString(Keyword<'arena>),
    UppercaseCallableString(Keyword<'arena>),
    NumericString(Keyword<'arena>),
    NonEmptyString(Keyword<'arena>),
    NonEmptyLowercaseString(Keyword<'arena>),
    LowercaseString(Keyword<'arena>),
    NonEmptyUppercaseString(Keyword<'arena>),
    UppercaseString(Keyword<'arena>),
    TruthyString(Keyword<'arena>),
    NonFalsyString(Keyword<'arena>),
    UnspecifiedLiteralInt(Keyword<'arena>),
    UnspecifiedLiteralString(Keyword<'arena>),
    UnspecifiedLiteralFloat(Keyword<'arena>),
    NonEmptyUnspecifiedLiteralString(Keyword<'arena>),
    LiteralFloat(LiteralFloatType<'arena>),
    LiteralInt(LiteralIntType<'arena>),
    LiteralString(LiteralStringType<'arena>),
    MemberReference(MemberReferenceType<'arena>),
    AliasReference(AliasReferenceType<'arena>),
    Shape(ShapeType<'arena>),
    Callable(CallableType<'arena>),
    Variable(VariableType<'arena>),
    Conditional(ConditionalType<'arena>),
    KeyOf(KeyOfType<'arena>),
    ValueOf(ValueOfType<'arena>),
    IntMask(IntMaskType<'arena>),
    IntMaskOf(IntMaskOfType<'arena>),
    New(NewType<'arena>),
    TemplateType(TemplateTypeType<'arena>),
    IndexAccess(IndexAccessType<'arena>),
    Negated(NegatedType<'arena>),
    Posited(PositedType<'arena>),
    IntRange(IntRangeType<'arena>),
    PropertiesOf(PropertiesOfType<'arena>),
    Slice(SliceType<'arena>),
    Wildcard(WildcardType),
    TrailingPipe(TrailingPipeType<'arena>),
    GlobalWildcardReference(GlobalWildcardType<'arena>),
}

impl HasSpan for Type<'_> {
    fn span(&self) -> Span {
        match self {
            Type::Parenthesized(ty) => ty.span(),
            Type::Union(ty) => ty.span(),
            Type::Intersection(ty) => ty.span(),
            Type::Nullable(ty) => ty.span(),
            Type::Array(ty) => ty.span(),
            Type::NonEmptyArray(ty) => ty.span(),
            Type::AssociativeArray(ty) => ty.span(),
            Type::List(ty) => ty.span(),
            Type::NonEmptyList(ty) => ty.span(),
            Type::Iterable(ty) => ty.span(),
            Type::ClassString(ty) => ty.span(),
            Type::InterfaceString(ty) => ty.span(),
            Type::EnumString(ty) => ty.span(),
            Type::TraitString(ty) => ty.span(),
            Type::Reference(ty) => ty.span(),
            Type::Mixed(ty) => ty.span(),
            Type::NonEmptyMixed(ty) => ty.span(),
            Type::Null(ty) => ty.span(),
            Type::Void(ty) => ty.span(),
            Type::Never(ty) => ty.span(),
            Type::Resource(ty) => ty.span(),
            Type::ClosedResource(ty) => ty.span(),
            Type::OpenResource(ty) => ty.span(),
            Type::True(ty) => ty.span(),
            Type::False(ty) => ty.span(),
            Type::Bool(ty) => ty.span(),
            Type::Float(ty) => ty.span(),
            Type::Int(ty) => ty.span(),
            Type::PositiveInt(ty) => ty.span(),
            Type::NegativeInt(ty) => ty.span(),
            Type::NonPositiveInt(ty) => ty.span(),
            Type::NonNegativeInt(ty) => ty.span(),
            Type::NonZeroInt(ty) => ty.span(),
            Type::String(ty) => ty.span(),
            Type::ArrayKey(ty) => ty.span(),
            Type::Scalar(ty) => ty.span(),
            Type::Object(ty) => ty.span(),
            Type::Numeric(ty) => ty.span(),
            Type::CallableString(ty) => ty.span(),
            Type::LowercaseCallableString(ty) => ty.span(),
            Type::UppercaseCallableString(ty) => ty.span(),
            Type::NumericString(ty) => ty.span(),
            Type::StringableObject(ty) => ty.span(),
            Type::NonEmptyString(ty) => ty.span(),
            Type::NonEmptyLowercaseString(ty) => ty.span(),
            Type::LowercaseString(ty) => ty.span(),
            Type::NonEmptyUppercaseString(ty) => ty.span(),
            Type::UppercaseString(ty) => ty.span(),
            Type::TruthyString(ty) => ty.span(),
            Type::NonFalsyString(ty) => ty.span(),
            Type::UnspecifiedLiteralInt(ty) => ty.span(),
            Type::UnspecifiedLiteralString(ty) => ty.span(),
            Type::UnspecifiedLiteralFloat(ty) => ty.span(),
            Type::NonEmptyUnspecifiedLiteralString(ty) => ty.span(),
            Type::LiteralFloat(ty) => ty.span(),
            Type::LiteralInt(ty) => ty.span(),
            Type::LiteralString(ty) => ty.span(),
            Type::MemberReference(ty) => ty.span(),
            Type::AliasReference(ty) => ty.span(),
            Type::Shape(ty) => ty.span(),
            Type::Callable(ty) => ty.span(),
            Type::Conditional(ty) => ty.span(),
            Type::Variable(ty) => ty.span(),
            Type::KeyOf(ty) => ty.span(),
            Type::ValueOf(ty) => ty.span(),
            Type::IntMask(ty) => ty.span(),
            Type::IntMaskOf(ty) => ty.span(),
            Type::New(ty) => ty.span(),
            Type::TemplateType(ty) => ty.span(),
            Type::IndexAccess(ty) => ty.span(),
            Type::Negated(ty) => ty.span(),
            Type::Posited(ty) => ty.span(),
            Type::IntRange(ty) => ty.span(),
            Type::PropertiesOf(ty) => ty.span(),
            Type::Slice(ty) => ty.span(),
            Type::Wildcard(ty) => ty.span(),
            Type::TrailingPipe(ty) => ty.span(),
            Type::GlobalWildcardReference(ty) => ty.span(),
        }
    }
}

impl std::fmt::Display for Type<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Parenthesized(ty) => write!(f, "{ty}"),
            Type::Union(ty) => write!(f, "{ty}"),
            Type::Intersection(ty) => write!(f, "{ty}"),
            Type::Nullable(ty) => write!(f, "{ty}"),
            Type::Array(ty) => write!(f, "{ty}"),
            Type::NonEmptyArray(ty) => write!(f, "{ty}"),
            Type::AssociativeArray(ty) => write!(f, "{ty}"),
            Type::List(ty) => write!(f, "{ty}"),
            Type::NonEmptyList(ty) => write!(f, "{ty}"),
            Type::Iterable(ty) => write!(f, "{ty}"),
            Type::ClassString(ty) => write!(f, "{ty}"),
            Type::InterfaceString(ty) => write!(f, "{ty}"),
            Type::EnumString(ty) => write!(f, "{ty}"),
            Type::TraitString(ty) => write!(f, "{ty}"),
            Type::Reference(ty) => write!(f, "{ty}"),
            Type::Mixed(ty) => write!(f, "{ty}"),
            Type::NonEmptyMixed(ty) => write!(f, "{ty}"),
            Type::Null(ty) => write!(f, "{ty}"),
            Type::Void(ty) => write!(f, "{ty}"),
            Type::Never(ty) => write!(f, "{ty}"),
            Type::Resource(ty) => write!(f, "{ty}"),
            Type::ClosedResource(ty) => write!(f, "{ty}"),
            Type::OpenResource(ty) => write!(f, "{ty}"),
            Type::True(ty) => write!(f, "{ty}"),
            Type::False(ty) => write!(f, "{ty}"),
            Type::Bool(ty) => write!(f, "{ty}"),
            Type::Float(ty) => write!(f, "{ty}"),
            Type::Int(ty) => write!(f, "{ty}"),
            Type::PositiveInt(ty) => write!(f, "{ty}"),
            Type::NegativeInt(ty) => write!(f, "{ty}"),
            Type::NonPositiveInt(ty) => write!(f, "{ty}"),
            Type::NonNegativeInt(ty) => write!(f, "{ty}"),
            Type::NonZeroInt(ty) => write!(f, "{ty}"),
            Type::String(ty) => write!(f, "{ty}"),
            Type::ArrayKey(ty) => write!(f, "{ty}"),
            Type::Scalar(ty) => write!(f, "{ty}"),
            Type::Object(ty) => write!(f, "{ty}"),
            Type::Numeric(ty) => write!(f, "{ty}"),
            Type::CallableString(ty) => write!(f, "{ty}"),
            Type::LowercaseCallableString(ty) => write!(f, "{ty}"),
            Type::UppercaseCallableString(ty) => write!(f, "{ty}"),
            Type::NumericString(ty) => write!(f, "{ty}"),
            Type::StringableObject(ty) => write!(f, "{ty}"),
            Type::NonEmptyString(ty) => write!(f, "{ty}"),
            Type::NonEmptyLowercaseString(ty) => write!(f, "{ty}"),
            Type::LowercaseString(ty) => write!(f, "{ty}"),
            Type::NonEmptyUppercaseString(ty) => write!(f, "{ty}"),
            Type::UppercaseString(ty) => write!(f, "{ty}"),
            Type::TruthyString(ty) => write!(f, "{ty}"),
            Type::NonFalsyString(ty) => write!(f, "{ty}"),
            Type::UnspecifiedLiteralInt(ty) => write!(f, "{ty}"),
            Type::UnspecifiedLiteralString(ty) => write!(f, "{ty}"),
            Type::UnspecifiedLiteralFloat(ty) => write!(f, "{ty}"),
            Type::NonEmptyUnspecifiedLiteralString(ty) => write!(f, "{ty}"),
            Type::LiteralFloat(ty) => write!(f, "{ty}"),
            Type::LiteralInt(ty) => write!(f, "{ty}"),
            Type::LiteralString(ty) => write!(f, "{ty}"),
            Type::MemberReference(ty) => write!(f, "{ty}"),
            Type::AliasReference(ty) => write!(f, "{ty}"),
            Type::Shape(ty) => write!(f, "{ty}"),
            Type::Callable(ty) => write!(f, "{ty}"),
            Type::Conditional(ty) => write!(f, "{ty}"),
            Type::Variable(ty) => write!(f, "{ty}"),
            Type::KeyOf(ty) => write!(f, "{ty}"),
            Type::ValueOf(ty) => write!(f, "{ty}"),
            Type::IntMask(ty) => write!(f, "{ty}"),
            Type::IntMaskOf(ty) => write!(f, "{ty}"),
            Type::New(ty) => write!(f, "{ty}"),
            Type::TemplateType(ty) => write!(f, "{ty}"),
            Type::IndexAccess(ty) => write!(f, "{ty}"),
            Type::Negated(ty) => write!(f, "{ty}"),
            Type::Posited(ty) => write!(f, "{ty}"),
            Type::IntRange(ty) => write!(f, "{ty}"),
            Type::PropertiesOf(ty) => write!(f, "{ty}"),
            Type::Slice(ty) => write!(f, "{ty}"),
            Type::Wildcard(ty) => write!(f, "{ty}"),
            Type::TrailingPipe(ty) => write!(f, "{ty}"),
            Type::GlobalWildcardReference(ty) => write!(f, "{ty}"),
        }
    }
}
