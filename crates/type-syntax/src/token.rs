use serde::Deserialize;
use serde::Serialize;

use mago_span::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
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
    As,
    Is,
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
    Whitespace,
    SingleLineComment,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct TypeToken<'input> {
    pub kind: TypeTokenKind,
    pub value: &'input str,
    pub span: Span,
}

impl TypeTokenKind {
    #[inline(always)]
    pub const fn is_trivia(&self) -> bool {
        matches!(self, Self::SingleLineComment | Self::Whitespace)
    }

    #[inline(always)]
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
        )
    }

    #[inline(always)]
    pub const fn is_array_like(&self) -> bool {
        matches!(self, Self::Array | Self::NonEmptyArray | Self::AssociativeArray | Self::List | Self::NonEmptyList)
    }
}
