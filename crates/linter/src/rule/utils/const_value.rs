use std::cmp::Ordering;

use ordered_float::OrderedFloat;

use mago_names::ResolvedNames;
use mago_syntax::cst::Access;
use mago_syntax::cst::BinaryOperator;
use mago_syntax::cst::ClassConstantAccess;
use mago_syntax::cst::ClassLikeConstantSelector;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Literal;
use mago_syntax::cst::UnaryPrefix;
use mago_syntax::cst::UnaryPrefixOperator;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConstantValue<'arena> {
    Integer(i64),
    Float(OrderedFloat<f64>),
    String(&'arena [u8]),
    Boolean(bool),
    Null,
    Constant(&'arena [u8]),
    ClassConstant(&'arena [u8], &'arena [u8]),
}

#[derive(Clone, Copy)]
enum Number {
    Integer(i64),
    Float(f64),
}

#[must_use]
pub fn get_constant_value<'arena>(
    resolved_names: &ResolvedNames<'arena>,
    expression: &Expression<'arena>,
) -> Option<ConstantValue<'arena>> {
    match expression {
        Expression::Parenthesized(parenthesized) => get_constant_value(resolved_names, parenthesized.expression),
        Expression::Literal(literal) => get_literal_value(literal),
        Expression::UnaryPrefix(unary) => get_unary_value(resolved_names, unary),
        Expression::Binary(binary) => {
            let left = get_constant_value(resolved_names, binary.lhs)?;
            let right = get_constant_value(resolved_names, binary.rhs)?;

            get_binary_value(binary.operator, left, right)
        }
        Expression::ConstantAccess(constant_access) => {
            resolved_names.resolve(&constant_access.name).map(ConstantValue::Constant)
        }
        Expression::Access(Access::ClassConstant(access)) => get_class_constant_value(resolved_names, access),
        _ => None,
    }
}

#[must_use]
fn get_literal_value<'arena>(literal: &Literal<'arena>) -> Option<ConstantValue<'arena>> {
    match literal {
        Literal::Integer(integer) => integer.value.map(integer_to_value),
        Literal::Float(float) => Some(ConstantValue::Float(float.value)),
        Literal::String(string) => string.value.map(ConstantValue::String),
        Literal::True(_) => Some(ConstantValue::Boolean(true)),
        Literal::False(_) => Some(ConstantValue::Boolean(false)),
        Literal::Null(_) => Some(ConstantValue::Null),
    }
}

#[must_use]
fn get_unary_value<'arena>(
    resolved_names: &ResolvedNames<'arena>,
    unary: &UnaryPrefix<'arena>,
) -> Option<ConstantValue<'arena>> {
    let operand = get_constant_value(resolved_names, unary.operand)?;

    match unary.operator {
        UnaryPrefixOperator::Negation(_) => match operand.to_number()? {
            Number::Integer(value) => Some(match value.checked_neg() {
                Some(value) => ConstantValue::Integer(value),
                None => ConstantValue::Float(OrderedFloat(-(value as f64))),
            }),
            Number::Float(value) => number_to_value(Number::Float(-value)),
        },
        UnaryPrefixOperator::Plus(_) => number_to_value(operand.to_number()?),
        UnaryPrefixOperator::Not(_) => Some(ConstantValue::Boolean(!operand.to_boolean()?)),
        UnaryPrefixOperator::BitwiseNot(_) => Some(ConstantValue::Integer(!operand.to_integer()?)),
        UnaryPrefixOperator::BoolCast(..) | UnaryPrefixOperator::BooleanCast(..) => {
            Some(ConstantValue::Boolean(operand.to_boolean()?))
        }
        UnaryPrefixOperator::IntCast(..) | UnaryPrefixOperator::IntegerCast(..) => {
            Some(ConstantValue::Integer(operand.to_integer()?))
        }
        UnaryPrefixOperator::FloatCast(..)
        | UnaryPrefixOperator::DoubleCast(..)
        | UnaryPrefixOperator::RealCast(..) => number_to_value(Number::Float(operand.to_float()?)),
        _ => None,
    }
}

#[must_use]
fn get_binary_value<'arena>(
    operator: BinaryOperator<'arena>,
    left: ConstantValue<'arena>,
    right: ConstantValue<'arena>,
) -> Option<ConstantValue<'arena>> {
    match operator {
        BinaryOperator::Addition(_) => number_to_value(add(left.to_number()?, right.to_number()?)),
        BinaryOperator::Subtraction(_) => number_to_value(subtract(left.to_number()?, right.to_number()?)),
        BinaryOperator::Multiplication(_) => number_to_value(multiply(left.to_number()?, right.to_number()?)),
        BinaryOperator::Division(_) => number_to_value(divide(left.to_number()?, right.to_number()?)?),
        BinaryOperator::Modulo(_) => modulo(left.to_integer()?, right.to_integer()?),
        BinaryOperator::Exponentiation(_) => number_to_value(power(left.to_number()?, right.to_number()?)),
        BinaryOperator::BitwiseAnd(_) => Some(ConstantValue::Integer(left.to_integer()? & right.to_integer()?)),
        BinaryOperator::BitwiseOr(_) => Some(ConstantValue::Integer(left.to_integer()? | right.to_integer()?)),
        BinaryOperator::BitwiseXor(_) => Some(ConstantValue::Integer(left.to_integer()? ^ right.to_integer()?)),
        BinaryOperator::LeftShift(_) => shift_left(left.to_integer()?, right.to_integer()?),
        BinaryOperator::RightShift(_) => shift_right(left.to_integer()?, right.to_integer()?),
        BinaryOperator::NullCoalesce(_) => match left {
            ConstantValue::Null => Some(right),
            ConstantValue::Constant(_) | ConstantValue::ClassConstant(..) => None,
            other => Some(other),
        },
        BinaryOperator::Equal(_) => loose_equal(left, right).map(ConstantValue::Boolean),
        BinaryOperator::NotEqual(_) | BinaryOperator::AngledNotEqual(_) => {
            loose_equal(left, right).map(|value| ConstantValue::Boolean(!value))
        }
        BinaryOperator::Identical(_) => identical(left, right).map(ConstantValue::Boolean),
        BinaryOperator::NotIdentical(_) => identical(left, right).map(|value| ConstantValue::Boolean(!value)),
        BinaryOperator::LessThan(_) => {
            compare(left, right).map(|order| ConstantValue::Boolean(order == Ordering::Less))
        }
        BinaryOperator::LessThanOrEqual(_) => {
            compare(left, right).map(|order| ConstantValue::Boolean(order != Ordering::Greater))
        }
        BinaryOperator::GreaterThan(_) => {
            compare(left, right).map(|order| ConstantValue::Boolean(order == Ordering::Greater))
        }
        BinaryOperator::GreaterThanOrEqual(_) => {
            compare(left, right).map(|order| ConstantValue::Boolean(order != Ordering::Less))
        }
        BinaryOperator::Spaceship(_) => compare(left, right).map(|order| {
            ConstantValue::Integer(match order {
                Ordering::Less => -1,
                Ordering::Equal => 0,
                Ordering::Greater => 1,
            })
        }),
        BinaryOperator::And(_) | BinaryOperator::LowAnd(_) => {
            if left.to_boolean()? {
                Some(ConstantValue::Boolean(right.to_boolean()?))
            } else {
                Some(ConstantValue::Boolean(false))
            }
        }
        BinaryOperator::Or(_) | BinaryOperator::LowOr(_) => {
            if left.to_boolean()? {
                Some(ConstantValue::Boolean(true))
            } else {
                Some(ConstantValue::Boolean(right.to_boolean()?))
            }
        }
        BinaryOperator::LowXor(_) => Some(ConstantValue::Boolean(left.to_boolean()? ^ right.to_boolean()?)),
        _ => None,
    }
}

#[must_use]
fn get_class_constant_value<'arena>(
    resolved_names: &ResolvedNames<'arena>,
    access: &ClassConstantAccess<'arena>,
) -> Option<ConstantValue<'arena>> {
    let class = match access.class {
        Expression::Identifier(identifier) => resolved_names.resolve(identifier)?,
        Expression::Self_(keyword) | Expression::Static(keyword) | Expression::Parent(keyword) => keyword.value,
        _ => return None,
    };

    let ClassLikeConstantSelector::Identifier(constant) = &access.constant else {
        return None;
    };

    Some(ConstantValue::ClassConstant(class, constant.value))
}

impl ConstantValue<'_> {
    fn to_boolean(self) -> Option<bool> {
        match self {
            ConstantValue::Integer(value) => Some(value != 0),
            ConstantValue::Float(value) => Some(value.0 != 0.0),
            ConstantValue::String(value) => Some(!value.is_empty() && value != b"0"),
            ConstantValue::Boolean(value) => Some(value),
            ConstantValue::Null => Some(false),
            ConstantValue::Constant(_) | ConstantValue::ClassConstant(..) => None,
        }
    }

    fn to_number(self) -> Option<Number> {
        match self {
            ConstantValue::Integer(value) => Some(Number::Integer(value)),
            ConstantValue::Float(value) => Some(Number::Float(value.0)),
            ConstantValue::Boolean(value) => Some(Number::Integer(i64::from(value))),
            ConstantValue::Null => Some(Number::Integer(0)),
            _ => None,
        }
    }

    fn to_integer(self) -> Option<i64> {
        match self.to_number()? {
            Number::Integer(value) => Some(value),
            Number::Float(value) => float_to_integer(value),
        }
    }

    fn to_float(self) -> Option<f64> {
        match self.to_number()? {
            Number::Integer(value) => Some(value as f64),
            Number::Float(value) => Some(value),
        }
    }

    fn as_numeric(self) -> Option<Number> {
        match self {
            ConstantValue::Integer(value) => Some(Number::Integer(value)),
            ConstantValue::Float(value) => Some(Number::Float(value.0)),
            _ => None,
        }
    }
}

impl Number {
    fn as_float(self) -> f64 {
        match self {
            Number::Integer(value) => value as f64,
            Number::Float(value) => value,
        }
    }
}

#[must_use]
fn add(left: Number, right: Number) -> Number {
    match (left, right) {
        (Number::Integer(left), Number::Integer(right)) => match left.checked_add(right) {
            Some(value) => Number::Integer(value),
            None => Number::Float(left as f64 + right as f64),
        },
        _ => Number::Float(left.as_float() + right.as_float()),
    }
}

#[must_use]
fn subtract(left: Number, right: Number) -> Number {
    match (left, right) {
        (Number::Integer(left), Number::Integer(right)) => match left.checked_sub(right) {
            Some(value) => Number::Integer(value),
            None => Number::Float(left as f64 - right as f64),
        },
        _ => Number::Float(left.as_float() - right.as_float()),
    }
}

#[must_use]
fn multiply(left: Number, right: Number) -> Number {
    match (left, right) {
        (Number::Integer(left), Number::Integer(right)) => match left.checked_mul(right) {
            Some(value) => Number::Integer(value),
            None => Number::Float(left as f64 * right as f64),
        },
        _ => Number::Float(left.as_float() * right.as_float()),
    }
}

#[must_use]
fn divide(left: Number, right: Number) -> Option<Number> {
    if right.as_float() == 0.0 {
        return None;
    }

    match (left, right) {
        (Number::Integer(left), Number::Integer(right)) => match left.checked_rem(right) {
            Some(0) => match left.checked_div(right) {
                Some(value) => Some(Number::Integer(value)),
                None => Some(Number::Float(left as f64 / right as f64)),
            },
            _ => Some(Number::Float(left as f64 / right as f64)),
        },
        _ => Some(Number::Float(left.as_float() / right.as_float())),
    }
}

#[must_use]
fn power(left: Number, right: Number) -> Number {
    if let (Number::Integer(base), Number::Integer(exponent)) = (left, right)
        && let Ok(exponent) = u32::try_from(exponent)
        && let Some(value) = base.checked_pow(exponent)
    {
        return Number::Integer(value);
    }

    Number::Float(left.as_float().powf(right.as_float()))
}

#[must_use]
fn modulo<'arena>(left: i64, right: i64) -> Option<ConstantValue<'arena>> {
    if right == 0 {
        return None;
    }

    Some(ConstantValue::Integer(left.checked_rem(right).unwrap_or(0)))
}

#[must_use]
fn shift_left<'arena>(left: i64, right: i64) -> Option<ConstantValue<'arena>> {
    let bits = u32::try_from(right).ok()?;

    left.checked_shl(bits).map(ConstantValue::Integer)
}

#[must_use]
fn shift_right<'arena>(left: i64, right: i64) -> Option<ConstantValue<'arena>> {
    let bits = u32::try_from(right).ok()?;

    left.checked_shr(bits).map(ConstantValue::Integer)
}

#[must_use]
fn identical(left: ConstantValue<'_>, right: ConstantValue<'_>) -> Option<bool> {
    match (left, right) {
        (ConstantValue::Constant(left), ConstantValue::Constant(right)) => (left == right).then_some(true),
        (
            ConstantValue::ClassConstant(left_class, left_name),
            ConstantValue::ClassConstant(right_class, right_name),
        ) => (left_class == right_class && left_name == right_name).then_some(true),
        (ConstantValue::Constant(_) | ConstantValue::ClassConstant(..), _)
        | (_, ConstantValue::Constant(_) | ConstantValue::ClassConstant(..)) => None,
        _ => Some(left == right),
    }
}

#[must_use]
fn loose_equal(left: ConstantValue<'_>, right: ConstantValue<'_>) -> Option<bool> {
    match (left, right) {
        (ConstantValue::Constant(left), ConstantValue::Constant(right)) => return (left == right).then_some(true),
        (
            ConstantValue::ClassConstant(left_class, left_name),
            ConstantValue::ClassConstant(right_class, right_name),
        ) => {
            return (left_class == right_class && left_name == right_name).then_some(true);
        }
        (ConstantValue::Constant(_) | ConstantValue::ClassConstant(..), _)
        | (_, ConstantValue::Constant(_) | ConstantValue::ClassConstant(..)) => return None,
        _ => {}
    }

    if matches!(left, ConstantValue::Boolean(_)) || matches!(right, ConstantValue::Boolean(_)) {
        return Some(left.to_boolean()? == right.to_boolean()?);
    }

    match (left, right) {
        (ConstantValue::Integer(left), ConstantValue::Integer(right)) => Some(left == right),
        (ConstantValue::String(left), ConstantValue::String(right)) => (left == right).then_some(true),
        (ConstantValue::Null, ConstantValue::Null) => Some(true),
        (ConstantValue::Null, _) | (_, ConstantValue::Null) => None,
        _ => {
            let left = left.as_numeric()?;
            let right = right.as_numeric()?;

            Some(left.as_float() == right.as_float())
        }
    }
}

#[must_use]
fn compare(left: ConstantValue<'_>, right: ConstantValue<'_>) -> Option<Ordering> {
    match (left, right) {
        (ConstantValue::Integer(left), ConstantValue::Integer(right)) => Some(left.cmp(&right)),
        _ => {
            let left = left.as_numeric()?.as_float();
            let right = right.as_numeric()?.as_float();

            left.partial_cmp(&right)
        }
    }
}

#[must_use]
fn integer_to_value<'arena>(value: u64) -> ConstantValue<'arena> {
    match i64::try_from(value) {
        Ok(value) => ConstantValue::Integer(value),
        Err(_) => ConstantValue::Float(OrderedFloat(value as f64)),
    }
}

#[must_use]
fn number_to_value<'arena>(number: Number) -> Option<ConstantValue<'arena>> {
    match number {
        Number::Integer(value) => Some(ConstantValue::Integer(value)),
        Number::Float(value) => value.is_finite().then_some(ConstantValue::Float(OrderedFloat(value))),
    }
}

#[must_use]
fn float_to_integer(value: f64) -> Option<i64> {
    if value.is_finite() && value >= i64::MIN as f64 && value <= i64::MAX as f64 { Some(value as i64) } else { None }
}
