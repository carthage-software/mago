use mago_span::HasSpan;
use mago_span::Span;
#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;

use mago_allocator::copy::CopyInto;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct AssignmentOperator {
    pub span: Span,
    pub kind: AssignmentOperatorKind,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum AssignmentOperatorKind {
    Assign,         // `=`
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

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct BinaryOperator {
    pub span: Span,
    pub kind: BinaryOperatorKind,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum BinaryOperatorKind {
    Addition,                             // `+`
    Subtraction,                          // `-`
    Multiplication,                       // `*`
    Division,                             // `/`
    Modulo,                               // `%`
    Exponentiation,                       // `**`
    BitwiseAnd,                           // `&`
    BitwiseOr,                            // `|`
    BitwiseXor,                           // `^`
    LeftShift,                            // `<<`
    RightShift,                           // `>>`
    NullCoalesce,                         // `??`
    Equal,                                // `==`
    NotEqual(NotEqualBinaryOperatorKind), // `!=`, `<>`
    Identical,                            // `===`
    NotIdentical,                         // `!==`
    LessThan,                             // `<`
    LessThanOrEqual,                      // `<=`
    GreaterThan,                          // `>`
    GreaterThanOrEqual,                   // `>=`
    Spaceship,                            // `<=>`
    StringConcat,                         // `.`
    Instanceof,                           // `instanceof`
    And(AndBinaryOperatorKind),           // `&&`, `and`
    Or(OrBinaryOperatorKind),             // `||`, `or`
    Xor,                                  // `xor`
    Pipe,                                 // `|>`
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum NotEqualBinaryOperatorKind {
    BangEqual,   // `!=`
    LessGreater, // `<>`
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum AndBinaryOperatorKind {
    AmpersandAmpersand, // `&&`
    KeywordAnd,         // `and`
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum OrBinaryOperatorKind {
    PipePipe,  // `||`
    KeywordOr, // `or`
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct UnaryPrefixOperator {
    pub span: Span,
    pub kind: UnaryPrefixOperatorKind,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum UnaryPrefixOperatorKind {
    ErrorControl,                                  // `@$expr`
    Reference,                                     // `&$expr`
    ArrayCast,                                     // `(array) $expr`
    BoolCast(BoolCastUnaryPrefixOperatorKind),     // `(bool) $expr`, `(boolean) $expr`
    FloatCast(FloatCastUnaryPrefixOperatorKind),   // `(float) $expr`, `(double) $expr`, `(real) $expr`
    IntCast(IntCastUnaryPrefixOperatorKind),       // `(int) $expr`, `(integer) $expr`
    ObjectCast,                                    // `(object) $expr`
    UnsetCast,                                     // `(unset) $expr`
    StringCast(StringCastUnaryPrefixOperatorKind), // `(string) $expr`, `(binary) $expr`
    VoidCast,                                      // `(void) $expr`
    BitwiseNot,                                    // `~$expr`
    Not,                                           // `!$expr`
    PreIncrement,                                  // `++$expr`
    PreDecrement,                                  // `--$expr`
    Plus,                                          // `+$expr`
    Negation,                                      // `-$expr`
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum BoolCastUnaryPrefixOperatorKind {
    Bool,    // `(bool) $expr`
    Boolean, // `(boolean) $expr`
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum FloatCastUnaryPrefixOperatorKind {
    Float,  // `(float) $expr`
    Double, // `(double) $expr`
    Real,   // `(real) $expr`
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum IntCastUnaryPrefixOperatorKind {
    Int,     // `(int) $expr`
    Integer, // `(integer) $expr`
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum StringCastUnaryPrefixOperatorKind {
    String, // `(string) $expr`
    Binary, // `(binary) $expr`
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct UnaryPostfixOperator {
    pub span: Span,
    pub kind: UnaryPostfixOperatorKind,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum UnaryPostfixOperatorKind {
    PostIncrement, // `$expr++`
    PostDecrement, // `$expr--`
}

impl CopyInto for AssignmentOperator {
    type Output<'arena> = AssignmentOperator;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}

impl CopyInto for BinaryOperator {
    type Output<'arena> = BinaryOperator;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}

impl CopyInto for UnaryPrefixOperator {
    type Output<'arena> = UnaryPrefixOperator;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}

impl CopyInto for UnaryPostfixOperator {
    type Output<'arena> = UnaryPostfixOperator;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}

impl CopyInto for AssignmentOperatorKind {
    type Output<'arena> = AssignmentOperatorKind;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}

impl CopyInto for BinaryOperatorKind {
    type Output<'arena> = BinaryOperatorKind;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}

impl CopyInto for UnaryPrefixOperatorKind {
    type Output<'arena> = UnaryPrefixOperatorKind;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}

impl CopyInto for UnaryPostfixOperatorKind {
    type Output<'arena> = UnaryPostfixOperatorKind;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}

impl CopyInto for NotEqualBinaryOperatorKind {
    type Output<'arena> = NotEqualBinaryOperatorKind;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}

impl CopyInto for AndBinaryOperatorKind {
    type Output<'arena> = AndBinaryOperatorKind;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}

impl CopyInto for OrBinaryOperatorKind {
    type Output<'arena> = OrBinaryOperatorKind;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}

impl CopyInto for BoolCastUnaryPrefixOperatorKind {
    type Output<'arena> = BoolCastUnaryPrefixOperatorKind;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}

impl CopyInto for FloatCastUnaryPrefixOperatorKind {
    type Output<'arena> = FloatCastUnaryPrefixOperatorKind;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}

impl CopyInto for IntCastUnaryPrefixOperatorKind {
    type Output<'arena> = IntCastUnaryPrefixOperatorKind;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}

impl CopyInto for StringCastUnaryPrefixOperatorKind {
    type Output<'arena> = StringCastUnaryPrefixOperatorKind;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}

impl HasSpan for AssignmentOperator {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for BinaryOperator {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for UnaryPrefixOperator {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for UnaryPostfixOperator {
    fn span(&self) -> Span {
        self.span
    }
}
