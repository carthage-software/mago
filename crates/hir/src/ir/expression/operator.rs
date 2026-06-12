#[cfg(feature = "serde")]
use serde::Serialize;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum AssignmentOperator {
    Addition,       // `+=`
    Subtraction,    // `-=`
    Multiplication, // `*=`
    Division,       // `/=`
    Modulo,         // `%=`
    Exponentiation, // `**=`
    Concat,         // `.=`
    BitwiseAnd,     // `&=`
    BitwiseOr,      // `|=`
    BitwiseXor,     // `^=`
    LeftShift,      // `<<=`
    RightShift,     // `>>=`
    Coalesce,       // `??=`
}

/// Represents a PHP binary operator.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum BinaryOperator {
    Addition,           // `+`
    Subtraction,        // `-`
    Multiplication,     // `*`
    Division,           // `/`
    Modulo,             // `%`
    Exponentiation,     // `**`
    BitwiseAnd,         // `&`
    BitwiseOr,          // `|`
    BitwiseXor,         // `^`
    LeftShift,          // `<<`
    RightShift,         // `>>`
    NullCoalesce,       // `??`
    Equal,              // `==`
    NotEqual,           // `!=`
    Identical,          // `===`
    NotIdentical,       // `!==`
    LessThan,           // `<`
    LessThanOrEqual,    // `<=`
    GreaterThan,        // `>`
    GreaterThanOrEqual, // `>=`
    Spaceship,          // `<=>`
    StringConcat,       // `.`
    Instanceof,         // `instanceof`
    And,                // `&&`
    Or,                 // `||`
    Xor,                // `xor`
    Pipe,               // `|>`
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum UnaryPrefixOperator {
    ErrorControl, // `@$expr`
    Reference,    // `&$expr`
    ArrayCast,    // `(array) $expr`
    BoolCast,     // `(bool) $expr`
    FloatCast,    // `(float) $expr`
    IntCast,      // `(int) $expr`
    ObjectCast,   // `(object) $expr`
    UnsetCast,    // `(unset) $expr`
    StringCast,   // `(string) $expr`
    VoidCast,     // `(void) $expr`
    BitwiseNot,   // `~$expr`
    Not,          // `!$expr`
    PreIncrement, // `++$expr`
    PreDecrement, // `--$expr`
    Plus,         // `+$expr`
    Negation,     // `-$expr`
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum UnaryPostfixOperator {
    PostIncrement, // `$expr++`
    PostDecrement, // `$expr--`
}
