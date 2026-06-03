use serde::Serialize;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
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
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum PrefixUnaryOperator {
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum PostfixUnaryOperator {
    PostIncrement, // `$expr++`
    PostDecrement, // `$expr--`
}
