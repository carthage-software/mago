use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_span::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[repr(C)]
pub enum TypeTokenKind {
    Int,
    String,
    Float,
    Bool,
    False,
    True,
    Object,
    Callable,
    Array,
    NonEmptyArray,
    NonEmptyString,
    LowercaseString,
    TruthyString,
    Iterable,
    Null,
    Mixed,
    NumericString,
    ClassString,
    InterfaceString,
    TraitString,
    EnumString,
    StringableObject,
    PureCallable,
    PureClosure,
    UnspecifiedLiteralString,
    UnspecifiedLiteralInt,
    NonEmptyUnspecifiedLiteralString,
    Resource,
    Void,
    Scalar,
    Numeric,
    NoReturn,
    NeverReturn,
    NeverReturns,
    Never,
    Nothing,
    ArrayKey,
    List,
    NonEmptyList,
    OpenResource,
    ClosedResource,
    AssociativeArray,
    KeyOf,
    ValueOf,
    Min,
    Max,
    PropertiesOf,
    PublicPropertiesOf,
    PrivatePropertiesOf,
    ProtectedPropertiesOf,
    PositiveInt,
    NegativeInt,
    NonPositiveInt,
    NonNegativeInt,
    As,
    Is,
    Not,
    Identifier,
    QualifiedIdentifier,
    FullyQualifiedIdentifier,
    Plus,
    Minus,
    LessThan,
    GreaterThan,
    Pipe,
    Ampersand,
    Question,
    Comma,
    Colon,
    ColonColon,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    LeftParenthesis,
    RightParenthesis,
    Equals,
    Ellipsis,
    PartialLiteralString,
    LiteralString,
    LiteralInteger,
    LiteralFloat,
    Variable,
    Whitespace,
    SingleLineComment,
    Asterisk,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct TypeToken<'input> {
    pub kind: TypeTokenKind,
    pub value: &'input str,
    pub span: Span,
}

impl TypeTokenKind {
    #[inline]
    pub const fn is_trivia(&self) -> bool {
        matches!(self, Self::SingleLineComment | Self::Whitespace)
    }

    #[inline]
    pub const fn is_simple_identifier(&self) -> bool {
        matches!(self, Self::Identifier)
    }

    #[inline]
    pub const fn is_identifier(&self) -> bool {
        matches!(self, Self::Identifier | Self::QualifiedIdentifier | Self::FullyQualifiedIdentifier)
    }

    #[inline]
    pub const fn is_keyword(&self) -> bool {
        matches!(
            self,
            Self::Int
                | Self::String
                | Self::Float
                | Self::Bool
                | Self::False
                | Self::True
                | Self::Object
                | Self::Callable
                | Self::Array
                | Self::NonEmptyArray
                | Self::NonEmptyString
                | Self::LowercaseString
                | Self::TruthyString
                | Self::Iterable
                | Self::Null
                | Self::Mixed
                | Self::NumericString
                | Self::ClassString
                | Self::InterfaceString
                | Self::TraitString
                | Self::EnumString
                | Self::StringableObject
                | Self::PureCallable
                | Self::PureClosure
                | Self::UnspecifiedLiteralString
                | Self::NonEmptyUnspecifiedLiteralString
                | Self::Resource
                | Self::Void
                | Self::Scalar
                | Self::Numeric
                | Self::NoReturn
                | Self::NeverReturn
                | Self::NeverReturns
                | Self::Never
                | Self::Nothing
                | Self::ArrayKey
                | Self::List
                | Self::NonEmptyList
                | Self::OpenResource
                | Self::ClosedResource
                | Self::AssociativeArray
                | Self::Is
                | Self::As
                | Self::Not
                | Self::KeyOf
                | Self::ValueOf
                | Self::Min
                | Self::Max
                | Self::UnspecifiedLiteralInt
                | Self::PropertiesOf
                | Self::PublicPropertiesOf
                | Self::PrivatePropertiesOf
                | Self::ProtectedPropertiesOf
                | Self::PositiveInt
                | Self::NegativeInt
                | Self::NonPositiveInt
                | Self::NonNegativeInt
        )
    }

    #[inline]
    pub const fn is_array_like(&self) -> bool {
        matches!(self, Self::Array | Self::NonEmptyArray | Self::AssociativeArray | Self::List | Self::NonEmptyList)
    }
}
