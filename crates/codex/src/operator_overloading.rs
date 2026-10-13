use mago_syntax::cst::BinaryOperator;
use mago_word::word;

use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::object::named::TNamedObject;
use crate::ttype::atomic::scalar::TScalar;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OverloadedOperator {
    Arithmetic,
    Bitwise,
    Shift,
    BitwiseNot,
    Negation,
    IncrementDecrement,
    Comparison,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperandSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperandAcceptance {
    Accepted,
    PartlyAccepted,
    Rejected,
}

#[derive(Debug)]
pub struct OperatorOverloadingClass {
    name: &'static str,
    operators: &'static [OverloadedOperator],
    string_grammar: StringOperandGrammar,
    exponent_or_shift_amount: ExponentOrShiftAmount,
    compares_with_any_value: bool,
}

#[derive(Debug)]
enum StringOperandGrammar {
    Decimal,
    GmpInteger,
}

#[derive(Debug)]
enum ExponentOrShiftAmount {
    WholeOperand,
    NonNegativeInt,
}

struct NumericLiteral {
    is_zero: bool,
    has_fractional_part: bool,
}

static BCMATH_NUMBER: OperatorOverloadingClass = OperatorOverloadingClass {
    name: "BcMath\\Number",
    operators: &[
        OverloadedOperator::Arithmetic,
        OverloadedOperator::Negation,
        OverloadedOperator::IncrementDecrement,
        OverloadedOperator::Comparison,
    ],
    string_grammar: StringOperandGrammar::Decimal,
    exponent_or_shift_amount: ExponentOrShiftAmount::WholeOperand,
    compares_with_any_value: true,
};

static GMP: OperatorOverloadingClass = OperatorOverloadingClass {
    name: "GMP",
    operators: &[
        OverloadedOperator::Arithmetic,
        OverloadedOperator::Bitwise,
        OverloadedOperator::Shift,
        OverloadedOperator::BitwiseNot,
        OverloadedOperator::Negation,
        OverloadedOperator::IncrementDecrement,
        OverloadedOperator::Comparison,
    ],
    string_grammar: StringOperandGrammar::GmpInteger,
    exponent_or_shift_amount: ExponentOrShiftAmount::NonNegativeInt,
    compares_with_any_value: false,
};

static OPERATOR_OVERLOADING_CLASSES: &[&OperatorOverloadingClass] = &[&BCMATH_NUMBER, &GMP];

#[must_use]
pub fn get_operator_overloading_class(atomic: &TAtomic) -> Option<&'static OperatorOverloadingClass> {
    let TAtomic::Object(TObject::Named(named_object)) = atomic else {
        return None;
    };

    let name = named_object.name.as_bytes().strip_prefix(b"\\").unwrap_or(named_object.name.as_bytes());

    OPERATOR_OVERLOADING_CLASSES.iter().copied().find(|class| name.eq_ignore_ascii_case(class.name.as_bytes()))
}

#[must_use]
pub fn get_class_overloading(
    atomic: &TAtomic,
    operator: OverloadedOperator,
) -> Option<&'static OperatorOverloadingClass> {
    get_operator_overloading_class(atomic).filter(|class| class.overloads(operator))
}

#[must_use]
pub fn get_operation_handler(
    operator: &BinaryOperator<'_>,
    left: &TAtomic,
    right: &TAtomic,
) -> Option<(&'static OperatorOverloadingClass, OperandSide)> {
    let overloaded_operator = if operator.is_arithmetic() {
        OverloadedOperator::Arithmetic
    } else if operator.is_bit_shift() {
        OverloadedOperator::Shift
    } else if operator.is_bitwise() {
        OverloadedOperator::Bitwise
    } else {
        return None;
    };

    get_class_overloading(left, overloaded_operator)
        .map(|class| (class, OperandSide::Left))
        .or_else(|| get_class_overloading(right, overloaded_operator).map(|class| (class, OperandSide::Right)))
}

#[must_use]
pub fn get_comparison_handler(
    operator: &BinaryOperator<'_>,
    left: &TAtomic,
    right: &TAtomic,
) -> Option<(&'static OperatorOverloadingClass, OperandSide)> {
    if !operator.is_comparison() || operator.is_identity() {
        return None;
    }

    if matches!(operator, BinaryOperator::Equal(_) | BinaryOperator::NotEqual(_) | BinaryOperator::AngledNotEqual(_))
        && left.is_object_type()
        && right.is_object_type()
    {
        return None;
    }

    let ((first, first_side), (second, second_side)) = match operator {
        BinaryOperator::GreaterThan(_) | BinaryOperator::GreaterThanOrEqual(_) => {
            ((right, OperandSide::Right), (left, OperandSide::Left))
        }
        _ => ((left, OperandSide::Left), (right, OperandSide::Right)),
    };

    if first.is_object_type() {
        return get_class_overloading(first, OverloadedOperator::Comparison).map(|class| (class, first_side));
    }

    get_class_overloading(second, OverloadedOperator::Comparison).map(|class| (class, second_side))
}

impl OperatorOverloadingClass {
    #[must_use]
    pub fn get_name(&self) -> &'static str {
        self.name
    }

    #[must_use]
    pub fn overloads(&self, operator: OverloadedOperator) -> bool {
        self.operators.contains(&operator)
    }

    #[must_use]
    pub fn accepts_operand(&self, atomic: &TAtomic) -> OperandAcceptance {
        let accepted = match atomic {
            TAtomic::Scalar(TScalar::Integer(_)) => true,
            TAtomic::Scalar(TScalar::String(string)) => match string.get_known_literal_value() {
                Some(value) => self.string_grammar.parse(value).is_some(),
                None => string.is_numeric,
            },
            TAtomic::Scalar(TScalar::Float(_) | TScalar::Bool(_)) => return OperandAcceptance::PartlyAccepted,
            _ if includes_ints_or_any_object(atomic) => return OperandAcceptance::PartlyAccepted,
            _ => self.is_class_of(atomic),
        };

        OperandAcceptance::from(accepted)
    }

    #[must_use]
    pub fn accepts_exponent_or_shift_amount(&self, atomic: &TAtomic) -> OperandAcceptance {
        match self.exponent_or_shift_amount {
            ExponentOrShiftAmount::WholeOperand => {
                let has_fractional_part = atomic
                    .get_literal_string_value()
                    .and_then(|value| self.string_grammar.parse(value))
                    .is_some_and(|literal| literal.has_fractional_part);

                if has_fractional_part { OperandAcceptance::Rejected } else { self.accepts_operand(atomic) }
            }
            ExponentOrShiftAmount::NonNegativeInt => {
                let accepted = match atomic {
                    TAtomic::Scalar(TScalar::Integer(integer)) => !integer.is_negative(),
                    TAtomic::Scalar(TScalar::String(string)) => match string.get_known_literal_value() {
                        Some(value) => parse_php_integer(value).is_some_and(|value| value >= 0),
                        None => string.is_numeric,
                    },
                    TAtomic::Scalar(TScalar::Float(_)) => return OperandAcceptance::PartlyAccepted,
                    _ if includes_ints_or_any_object(atomic) => return OperandAcceptance::PartlyAccepted,
                    _ => self.is_class_of(atomic),
                };

                OperandAcceptance::from(accepted)
            }
        }
    }

    #[must_use]
    pub fn accepts_comparison_operand(&self, atomic: &TAtomic) -> OperandAcceptance {
        if self.compares_with_any_value {
            return OperandAcceptance::Accepted;
        }

        match atomic {
            TAtomic::Null | TAtomic::Scalar(TScalar::Bool(_)) => OperandAcceptance::Accepted,
            _ => self.accepts_operand(atomic),
        }
    }

    #[must_use]
    pub fn get_instance_atomic(&self) -> TAtomic {
        TAtomic::Object(TObject::Named(TNamedObject::new(word(self.name))))
    }

    #[must_use]
    pub fn is_literal_zero(&self, atomic: &TAtomic) -> bool {
        match atomic.get_literal_string_value() {
            Some(value) => self.string_grammar.parse(value).is_some_and(|literal| literal.is_zero),
            None => atomic.get_literal_int_value() == Some(0),
        }
    }

    fn is_class_of(&self, atomic: &TAtomic) -> bool {
        get_operator_overloading_class(atomic).is_some_and(|class| std::ptr::eq(class, self))
    }
}

impl From<bool> for OperandAcceptance {
    fn from(accepted: bool) -> Self {
        if accepted { OperandAcceptance::Accepted } else { OperandAcceptance::Rejected }
    }
}

fn includes_ints_or_any_object(atomic: &TAtomic) -> bool {
    matches!(
        atomic,
        TAtomic::Scalar(TScalar::Generic | TScalar::Numeric | TScalar::ArrayKey) | TAtomic::Object(TObject::Any)
    )
}

impl StringOperandGrammar {
    fn parse(&self, value: &[u8]) -> Option<NumericLiteral> {
        match self {
            StringOperandGrammar::Decimal => parse_decimal(value),
            StringOperandGrammar::GmpInteger => parse_gmp_integer(value),
        }
    }
}

fn parse_decimal(value: &[u8]) -> Option<NumericLiteral> {
    let unsigned = value.strip_prefix(b"-").or_else(|| value.strip_prefix(b"+")).unwrap_or(value);

    let (integer_part, fractional_part) = match unsigned.iter().position(|byte| *byte == b'.') {
        Some(position) => (&unsigned[..position], &unsigned[position + 1..]),
        None => (unsigned, &[][..]),
    };

    if !integer_part.iter().chain(fractional_part).all(u8::is_ascii_digit) {
        return None;
    }

    let has_fractional_part = fractional_part.iter().any(|byte| *byte != b'0');
    let is_zero = !has_fractional_part && integer_part.iter().all(|byte| *byte == b'0');

    Some(NumericLiteral { is_zero, has_fractional_part })
}

fn parse_gmp_integer(value: &[u8]) -> Option<NumericLiteral> {
    let trimmed = trim_c_whitespace_start(value);

    match trimmed {
        [b'0', b'x' | b'X', digits @ ..] => parse_mpz_string(digits, Some(16)),
        [b'0', b'o' | b'O', digits @ ..] => parse_mpz_string(digits, Some(8)),
        [b'0', b'b' | b'B', digits @ ..] => parse_mpz_string(digits, Some(2)),
        _ => parse_mpz_string(trimmed, None),
    }
}

fn parse_mpz_string(value: &[u8], radix: Option<u32>) -> Option<NumericLiteral> {
    let trimmed = trim_c_whitespace_start(value);
    let unsigned = trimmed.strip_prefix(b"-").unwrap_or(trimmed);

    let first_byte = *unsigned.first()?;
    if !char::from(first_byte).is_digit(radix.unwrap_or(10)) {
        return None;
    }

    let (radix, digits) = match (radix, unsigned) {
        (Some(radix), _) => (radix, unsigned),
        (None, [b'0', b'x' | b'X', rest @ ..]) => (16, rest),
        (None, [b'0', b'b' | b'B', rest @ ..]) => (2, rest),
        (None, [b'0', rest @ ..]) => (8, rest),
        (None, _) => (10, unsigned),
    };

    if !digits.iter().all(|byte| is_c_whitespace(*byte) || char::from(*byte).is_digit(radix)) {
        return None;
    }

    let is_zero = digits.iter().all(|byte| *byte == b'0' || is_c_whitespace(*byte));

    Some(NumericLiteral { is_zero, has_fractional_part: false })
}

fn parse_php_integer(value: &[u8]) -> Option<i64> {
    let start = value.iter().position(|byte| !is_c_whitespace(*byte))?;
    let end = value.iter().rposition(|byte| !is_c_whitespace(*byte))?;

    std::str::from_utf8(&value[start..=end]).ok()?.parse().ok()
}

fn trim_c_whitespace_start(value: &[u8]) -> &[u8] {
    let start = value.iter().position(|byte| !is_c_whitespace(*byte)).unwrap_or(value.len());

    &value[start..]
}

const fn is_c_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | 0x0B | 0x0C | b'\r')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ttype::atomic::scalar::int::TInteger;

    fn named(name: &str) -> TAtomic {
        TAtomic::Object(TObject::Named(TNamedObject::new(word(name))))
    }

    fn literal_string(value: &str) -> TAtomic {
        TAtomic::Scalar(TScalar::literal_string(word(value)))
    }

    fn literal_int(value: i64) -> TAtomic {
        TAtomic::Scalar(TScalar::literal_int(value))
    }

    fn bcmath_number() -> &'static OperatorOverloadingClass {
        &BCMATH_NUMBER
    }

    fn gmp() -> &'static OperatorOverloadingClass {
        &GMP
    }

    #[test]
    fn finds_classes_case_insensitively_and_with_leading_backslash() {
        assert_eq!(
            get_operator_overloading_class(&named("bcmath\\number")).map(OperatorOverloadingClass::get_name),
            Some("BcMath\\Number")
        );
        assert_eq!(
            get_operator_overloading_class(&named("\\BcMath\\Number")).map(OperatorOverloadingClass::get_name),
            Some("BcMath\\Number")
        );
        assert_eq!(get_operator_overloading_class(&named("gmp")).map(OperatorOverloadingClass::get_name), Some("GMP"));
        assert_eq!(
            get_operator_overloading_class(&named("\\GMP")).map(OperatorOverloadingClass::get_name),
            Some("GMP")
        );
    }

    #[test]
    fn finds_nothing_for_other_classes_and_scalars() {
        assert!(get_operator_overloading_class(&named("stdClass")).is_none());
        assert!(get_operator_overloading_class(&named("Number")).is_none());
        assert!(get_operator_overloading_class(&named("App\\GMP")).is_none());
        assert!(get_operator_overloading_class(&literal_int(1)).is_none());
        assert!(get_operator_overloading_class(&literal_string("GMP")).is_none());
        assert!(get_operator_overloading_class(&TAtomic::Null).is_none());
    }

    #[test]
    fn bcmath_number_accepts_decimal_strings() {
        for value in ["10", "-1.5", "+2", "", ".5", "1.", "007", "-", "."] {
            assert_eq!(
                bcmath_number().accepts_operand(&literal_string(value)),
                OperandAcceptance::Accepted,
                "should accept {value:?}"
            );
        }
    }

    #[test]
    fn bcmath_number_rejects_strings_outside_its_grammar() {
        for value in [" 5", "5 ", "1e3", "abc", "--1", "+-1", "1.2.3", "0x10"] {
            assert_eq!(
                bcmath_number().accepts_operand(&literal_string(value)),
                OperandAcceptance::Rejected,
                "should reject {value:?}"
            );
        }
    }

    #[test]
    fn gmp_accepts_integer_strings() {
        for value in
            ["10", "-0x10", "0X1F", "0b101", "0o17", "010", " 5 ", "-0b1", "\t5\n", "0 5", "0x 1", " -5", "\u{b}5"]
        {
            assert_eq!(
                gmp().accepts_operand(&literal_string(value)),
                OperandAcceptance::Accepted,
                "should accept {value:?}"
            );
        }
    }

    #[test]
    fn gmp_rejects_strings_outside_its_grammar() {
        for value in ["+5", "", " ", "1.5", "1e3", "0xZZ", "08", "-0o7", "- 5", "0x", "0b102", "1_000", "-"] {
            assert_eq!(
                gmp().accepts_operand(&literal_string(value)),
                OperandAcceptance::Rejected,
                "should reject {value:?}"
            );
        }
    }

    #[test]
    fn gmp_reads_exponents_and_shift_amounts_as_non_negative_php_ints() {
        for atomic in [
            literal_int(2),
            literal_int(0),
            literal_string("2"),
            literal_string("010"),
            literal_string(" \t2 \r"),
            literal_string("+2"),
            literal_string("-0"),
        ] {
            assert_eq!(
                gmp().accepts_exponent_or_shift_amount(&atomic),
                OperandAcceptance::Accepted,
                "should accept {atomic:?}"
            );
        }

        for atomic in [
            literal_int(-1),
            TAtomic::Scalar(TScalar::Integer(TInteger::negative())),
            literal_string("-1"),
            literal_string("0x2"),
            literal_string("0b1"),
            literal_string("0o1"),
            literal_string("1.5"),
            literal_string("1e1"),
            literal_string("2abc"),
            literal_string(""),
            literal_string("99999999999999999999"),
        ] {
            assert_eq!(
                gmp().accepts_exponent_or_shift_amount(&atomic),
                OperandAcceptance::Rejected,
                "should reject {atomic:?}"
            );
        }
    }

    #[test]
    fn bcmath_number_takes_negative_exponents_but_not_fractional_strings() {
        for atomic in [literal_int(-1), literal_string("-1"), literal_string("2.0")] {
            assert_eq!(
                bcmath_number().accepts_exponent_or_shift_amount(&atomic),
                OperandAcceptance::Accepted,
                "should accept {atomic:?}"
            );
        }

        for atomic in [literal_string("1.5"), literal_string("-0.01"), literal_string("0x2"), literal_string(" 2")] {
            assert_eq!(
                bcmath_number().accepts_exponent_or_shift_amount(&atomic),
                OperandAcceptance::Rejected,
                "should reject {atomic:?}"
            );
        }
    }

    #[test]
    fn detects_literal_zero_divisors() {
        for value in ["0", "0.0", "-0", "", "-", ".", "00"] {
            assert!(bcmath_number().is_literal_zero(&literal_string(value)), "BcMath should read {value:?} as zero");
        }

        for value in ["0", "-0", " 0 ", "0x0", "-0x0", "00", "0b0"] {
            assert!(gmp().is_literal_zero(&literal_string(value)), "GMP should read {value:?} as zero");
        }

        for class in [bcmath_number(), gmp()] {
            assert!(class.is_literal_zero(&literal_int(0)));
            assert!(!class.is_literal_zero(&literal_int(1)));
            assert!(!class.is_literal_zero(&TAtomic::Scalar(TScalar::int())));
            assert!(!class.is_literal_zero(&literal_string("1")));
            assert!(!class.is_literal_zero(&literal_string("abc")));
            assert!(!class.is_literal_zero(&TAtomic::Scalar(TScalar::numeric_string())));
        }

        assert!(!bcmath_number().is_literal_zero(&literal_string("0.01")));
        assert!(!gmp().is_literal_zero(&literal_string("0x10")));
    }
}
