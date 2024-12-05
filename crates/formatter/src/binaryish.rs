use strum::Display;

use fennec_ast::ArithmeticInfixOperator;
use fennec_ast::BitwiseInfixOperator;
use fennec_ast::ComparisonOperator;
use fennec_ast::LogicalInfixOperator;
use fennec_span::HasSpan;
use fennec_span::Span;
use fennec_token::GetPrecedence;
use fennec_token::Precedence;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Display)]
pub enum BinaryishOperator {
    Bitwise(BitwiseInfixOperator),
    Logical(LogicalInfixOperator),
    Arithmetic(ArithmeticInfixOperator),
    Comparison(ComparisonOperator),
    Concat(Span),
    Coalesce(Span),
}

impl BinaryishOperator {
    #[inline]
    pub fn is_bitwise(&self) -> bool {
        matches!(self, Self::Bitwise(_))
    }

    #[inline]
    pub fn is_bitwise_shift(&self) -> bool {
        matches!(self, Self::Bitwise(op) if op.is_shift())
    }

    #[inline]
    pub fn is_logical(&self) -> bool {
        matches!(self, Self::Logical(_))
    }

    #[inline]
    pub fn is_arithmetic(&self) -> bool {
        matches!(self, Self::Arithmetic(_))
    }

    #[inline]
    pub fn is_comparison(&self) -> bool {
        matches!(self, Self::Comparison(_))
    }

    #[inline]
    pub fn is_same_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bitwise(a), Self::Bitwise(b)) => a.is_same_as(b),
            (Self::Logical(a), Self::Logical(b)) => a.is_same_as(b),
            (Self::Arithmetic(a), Self::Arithmetic(b)) => a.is_same_as(b),
            (Self::Comparison(a), Self::Comparison(b)) => a.is_same_as(b),
            (Self::Concat(_), Self::Concat(_)) => true,
            (Self::Coalesce(_), Self::Coalesce(_)) => true,
            _ => false,
        }
    }
}

impl From<BitwiseInfixOperator> for BinaryishOperator {
    fn from(op: BitwiseInfixOperator) -> Self {
        Self::Bitwise(op)
    }
}
impl From<LogicalInfixOperator> for BinaryishOperator {
    fn from(op: LogicalInfixOperator) -> Self {
        Self::Logical(op)
    }
}

impl From<ArithmeticInfixOperator> for BinaryishOperator {
    fn from(op: ArithmeticInfixOperator) -> Self {
        Self::Arithmetic(op)
    }
}

impl From<ComparisonOperator> for BinaryishOperator {
    fn from(op: ComparisonOperator) -> Self {
        Self::Comparison(op)
    }
}

impl GetPrecedence for BinaryishOperator {
    fn precedence(&self) -> Precedence {
        match self {
            Self::Bitwise(op) => op.precedence(),
            Self::Logical(op) => op.precedence(),
            Self::Arithmetic(op) => op.precedence(),
            Self::Comparison(op) => op.precedence(),
            Self::Concat(_) => Precedence::Concat,
            Self::Coalesce(_) => Precedence::NullCoalesce,
        }
    }
}

impl HasSpan for BinaryishOperator {
    fn span(&self) -> Span {
        match self {
            Self::Bitwise(op) => op.span(),
            Self::Logical(op) => op.span(),
            Self::Arithmetic(op) => op.span(),
            Self::Comparison(op) => op.span(),
            Self::Concat(span) => *span,
            Self::Coalesce(span) => *span,
        }
    }
}

impl BinaryishOperator {
    pub fn should_flatten(self, parent_op: Self) -> bool {
        let self_precedence = self.precedence();
        let parent_precedence = parent_op.precedence();

        if self_precedence != parent_precedence {
            // Do not flatten if operators have different precedence
            return false;
        }

        match (self, parent_op) {
            // Handle concatenation operator
            (BinaryishOperator::Concat(_), BinaryishOperator::Concat(_)) => true,

            // Arithmetic operators
            (BinaryishOperator::Arithmetic(a), BinaryishOperator::Arithmetic(p)) => {
                // Prevent flattening for non-associative operators
                if a.is_exponentiation() || p.is_exponentiation() {
                    return false;
                }

                if a.is_subtraction() || a.is_division() || p.is_subtraction() || p.is_division() {
                    return false;
                }

                // Do not flatten if operators are different
                if !a.is_same_as(&p) {
                    return false;
                }
                // Allow flattening for addition and multiplication
                true
            }

            // Bitwise operators
            (BinaryishOperator::Bitwise(a), BinaryishOperator::Bitwise(p)) => {
                // Prevent flattening for shifts
                if a.is_shift() || p.is_shift() {
                    return false;
                }

                // Do not flatten if operators are different
                if !a.is_same_as(&p) {
                    return false;
                }

                true
            }
            (a, b) => {
                // Allow flattening if operators have the same
                a.is_same_as(&b)
            }
        }
    }
}

impl BinaryishOperator {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bitwise(op) => match op {
                BitwiseInfixOperator::And(_) => "&",
                BitwiseInfixOperator::Or(_) => "|",
                BitwiseInfixOperator::Xor(_) => "^",
                BitwiseInfixOperator::LeftShift(_) => "<<",
                BitwiseInfixOperator::RightShift(_) => ">>",
            },
            Self::Logical(op) => match op {
                LogicalInfixOperator::And(_) => "&&",
                LogicalInfixOperator::Or(_) => "||",
                LogicalInfixOperator::LowPrecedenceAnd(_) => "and",
                LogicalInfixOperator::LowPrecedenceOr(_) => "or",
                LogicalInfixOperator::LowPrecedenceXor(_) => "xor",
            },
            Self::Arithmetic(op) => match op {
                ArithmeticInfixOperator::Addition(_) => "+",
                ArithmeticInfixOperator::Subtraction(_) => "-",
                ArithmeticInfixOperator::Multiplication(_) => "*",
                ArithmeticInfixOperator::Division(_) => "/",
                ArithmeticInfixOperator::Modulo(_) => "%",
                ArithmeticInfixOperator::Exponentiation(_) => "**",
            },
            Self::Comparison(op) => match op {
                ComparisonOperator::Equal(_) => "==",
                ComparisonOperator::NotEqual(_) => "!=",
                ComparisonOperator::Identical(_) => "===",
                ComparisonOperator::NotIdentical(_) => "!==",
                ComparisonOperator::AngledNotEqual(_) => "<>",
                ComparisonOperator::LessThan(_) => "<",
                ComparisonOperator::LessThanOrEqual(_) => "<=",
                ComparisonOperator::GreaterThan(_) => ">",
                ComparisonOperator::GreaterThanOrEqual(_) => ">=",
                ComparisonOperator::Spaceship(_) => "<=>",
            },
            Self::Concat(_) => ".",
            Self::Coalesce(_) => "??",
        }
    }
}
