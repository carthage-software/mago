use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_span::Span;

/// Type parsing precedence levels.
///
/// Lower ordinal values = lower precedence = binds more loosely.
/// Higher ordinal values = higher precedence = binds more tightly.
///
/// For example, in `Closure(): int|string`:
/// - With `Lowest` precedence: parses as `Union(Closure(): int, string)` (correct PHPStan/Psalm behavior)
/// - Callable return types use `Callable` precedence, which stops before `|`, `&`, and `is`
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TypePrecedence {
    /// Lowest precedence - parses everything including unions, intersections, conditionals
    Lowest,
    /// Conditional types: `T is U ? V : W`
    Conditional,
    /// Union types: `T|U`
    Union,
    /// Intersection types: `T&U`
    Intersection,
    /// Postfix operations: `T[]`, `T[K]`
    Postfix,
    /// Callable return type context - stops before `|`, `&`, `is`
    Callable,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
pub enum TypeTokenKind {
    Int,
    Integer,
    String,
    Float,
    Real,
    Double,
    Bool,
    Boolean,
    False,
    True,
    Object,
    Callable,
    Array,
    NonEmptyArray,
    NonEmptyString,
    NonEmptyLowercaseString,
    NonFalsyString,
    LowercaseString,
    TruthyString,
    Iterable,
    Null,
    Mixed,
    NonEmptyMixed,
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
    UnspecifiedLiteralFloat,
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
    IntMask,
    IntMaskOf,
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
    Exclamation,
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
pub struct TypeToken<'input> {
    pub kind: TypeTokenKind,
    pub value: &'input str,
    pub span: Span,
}

impl TypeTokenKind {
    #[inline]
    #[must_use]
    pub const fn is_trivia(&self) -> bool {
        matches!(self, Self::SingleLineComment | Self::Whitespace)
    }

    #[inline]
    #[must_use]
    pub const fn is_simple_identifier(&self) -> bool {
        matches!(self, Self::Identifier)
    }

    #[inline]
    #[must_use]
    pub const fn is_identifier(&self) -> bool {
        matches!(self, Self::Identifier | Self::QualifiedIdentifier | Self::FullyQualifiedIdentifier)
    }

    #[inline]
    #[must_use]
    pub const fn is_keyword(&self) -> bool {
        matches!(
            self,
            Self::Int
                | Self::Integer
                | Self::Double
                | Self::String
                | Self::Float
                | Self::Real
                | Self::Bool
                | Self::Boolean
                | Self::False
                | Self::True
                | Self::Object
                | Self::Callable
                | Self::Array
                | Self::NonEmptyArray
                | Self::NonEmptyString
                | Self::NonEmptyLowercaseString
                | Self::LowercaseString
                | Self::TruthyString
                | Self::NonFalsyString
                | Self::Iterable
                | Self::Null
                | Self::Mixed
                | Self::NonEmptyMixed
                | Self::NumericString
                | Self::ClassString
                | Self::InterfaceString
                | Self::TraitString
                | Self::EnumString
                | Self::StringableObject
                | Self::PureCallable
                | Self::PureClosure
                | Self::UnspecifiedLiteralString
                | Self::UnspecifiedLiteralFloat
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
                | Self::IntMask
                | Self::IntMaskOf
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
    #[must_use]
    pub const fn is_array_like(&self) -> bool {
        matches!(self, Self::Array | Self::NonEmptyArray | Self::AssociativeArray | Self::List | Self::NonEmptyList)
    }
}
