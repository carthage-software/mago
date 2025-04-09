use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

pub use crate::ast::array::*;
pub use crate::ast::callable::*;
pub use crate::ast::class_like_string::*;
pub use crate::ast::composite::*;
pub use crate::ast::generics::*;
pub use crate::ast::identifier::*;
pub use crate::ast::iterable::*;
pub use crate::ast::keyword::*;
pub use crate::ast::literal::*;
pub use crate::ast::reference::*;
pub use crate::ast::shape::*;
pub use crate::ast::unary::*;

pub mod array;
pub mod callable;
pub mod class_like_string;
pub mod composite;
pub mod generics;
pub mod identifier;
pub mod iterable;
pub mod keyword;
pub mod literal;
pub mod reference;
pub mod shape;
pub mod unary;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum Type<'input> {
    Parenthesized(ParenthesizedType<'input>),
    Union(UnionType<'input>),
    Intersection(IntersectionType<'input>),
    Nullable(NullableType<'input>),
    Array(ArrayType<'input>),
    NonEmptyArray(NonEmptyArrayType<'input>),
    AssociativeArray(AssociativeArrayType<'input>),
    List(ListType<'input>),
    NonEmptyList(NonEmptyListType<'input>),
    Iterable(IterableType<'input>),
    ClassString(ClassStringType<'input>),
    InterfaceString(InterfaceStringType<'input>),
    EnumString(EnumStringType<'input>),
    TraitString(TraitStringType<'input>),
    Reference(ReferenceType<'input>),
    Mixed(Keyword<'input>),
    Null(Keyword<'input>),
    Void(Keyword<'input>),
    Never(Keyword<'input>),
    Resource(Keyword<'input>),
    ClosedResource(Keyword<'input>),
    OpenResource(Keyword<'input>),
    True(Keyword<'input>),
    False(Keyword<'input>),
    Bool(Keyword<'input>),
    Float(Keyword<'input>),
    Int(Keyword<'input>),
    String(Keyword<'input>),
    StringableObject(Keyword<'input>),
    ArrayKey(Keyword<'input>),
    Object(Keyword<'input>),
    Numeric(Keyword<'input>),
    Scalar(Keyword<'input>),
    NumericString(Keyword<'input>),
    NonEmptyString(Keyword<'input>),
    TruthyString(Keyword<'input>),
    UnspecifiedLiteralString(Keyword<'input>),
    NonEmptyUnspecifiedLiteralString(Keyword<'input>),
    LiteralFloat(LiteralFloatType<'input>),
    LiteralInt(LiteralIntType<'input>),
    LiteralString(LiteralStringType<'input>),
    MemberReference(MemberReferenceType<'input>),
    Shape(ShapeType<'input>),
    Callable(CallableType<'input>),
    Negated(NegatedType<'input>),
    Posited(PositedType<'input>),
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
            Type::String(ty) => ty.span(),
            Type::ArrayKey(ty) => ty.span(),
            Type::Scalar(ty) => ty.span(),
            Type::Object(ty) => ty.span(),
            Type::Numeric(ty) => ty.span(),
            Type::NumericString(ty) => ty.span(),
            Type::StringableObject(ty) => ty.span(),
            Type::NonEmptyString(ty) => ty.span(),
            Type::TruthyString(ty) => ty.span(),
            Type::UnspecifiedLiteralString(ty) => ty.span(),
            Type::NonEmptyUnspecifiedLiteralString(ty) => ty.span(),
            Type::LiteralFloat(ty) => ty.span(),
            Type::LiteralInt(ty) => ty.span(),
            Type::LiteralString(ty) => ty.span(),
            Type::MemberReference(ty) => ty.span(),
            Type::Shape(ty) => ty.span(),
            Type::Callable(ty) => ty.span(),
            Type::Negated(ty) => ty.span(),
            Type::Posited(ty) => ty.span(),
        }
    }
}
