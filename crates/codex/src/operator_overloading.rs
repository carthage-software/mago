use mago_word::word;

use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::object::named::TNamedObject;
use crate::ttype::atomic::scalar::TScalar;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OverloadedOperator {
    /// `+`, `-`, `*`, `/`, `%` and `**`.
    Arithmetic,
    /// `&`, `|` and `^`.
    Bitwise,
    /// `<<` and `>>`.
    Shift,
    /// Unary `~`.
    BitwiseNot,
    /// Unary `-` and `+`.
    Negation,
    /// `++` and `--`.
    IncrementDecrement,
    /// `<`, `<=`, `>`, `>=`, `<=>`, `==`, `!=` and `<>`.
    Comparison,
}

/// A class whose instances PHP lets you use with operators, and which operands it accepts.
#[derive(Debug, PartialEq, Eq)]
pub struct OperatorOverloadingClass {
    pub name: &'static str,
    pub operators: &'static [OverloadedOperator],
    pub allows_negative_exponent: bool,
    compares_with_any_value: bool,
    string_grammar: StringOperandGrammar,
}

/// How a class reads a string operand, which is stricter than PHP's own numeric strings.
#[derive(Debug, PartialEq, Eq)]
enum StringOperandGrammar {
    /// BCMath: an optional sign, then digits with at most one `.`; no whitespace, no exponent.
    Decimal,
    /// GMP's `mpz_set_str` after PHP strips `0x`, `0o` and `0b` prefixes.
    GmpInteger,
}

struct NumericLiteral {
    is_negative: bool,
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
    allows_negative_exponent: true,
    compares_with_any_value: true,
    string_grammar: StringOperandGrammar::Decimal,
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
    allows_negative_exponent: false,
    compares_with_any_value: false,
    string_grammar: StringOperandGrammar::GmpInteger,
};

static OPERATOR_OVERLOADING_CLASSES: &[&OperatorOverloadingClass] = &[&BCMATH_NUMBER, &GMP];

/// Returns the operator-overloading class `atomic` is an instance of, if any.
#[must_use]
pub fn get_operator_overloading_class(atomic: &TAtomic) -> Option<&'static OperatorOverloadingClass> {
    let TAtomic::Object(TObject::Named(named_object)) = atomic else {
        return None;
    };

    let name = named_object.name.as_bytes().strip_prefix(b"\\").unwrap_or(named_object.name.as_bytes());

    OPERATOR_OVERLOADING_CLASSES.iter().copied().find(|class| name.eq_ignore_ascii_case(class.name.as_bytes()))
}

impl OperatorOverloadingClass {
    #[must_use]
    pub fn overloads(&self, operator: OverloadedOperator) -> bool {
        self.operators.contains(&operator)
    }

    /// Whether `atomic` can be the other operand of an arithmetic, bitwise or shift operation on this class.
    ///
    /// `float` is rejected: PHP truncates it to `int` with a deprecation instead of keeping its precision.
    #[must_use]
    pub fn accepts_operand(&self, atomic: &TAtomic) -> bool {
        match atomic {
            TAtomic::Scalar(TScalar::Integer(_) | TScalar::Bool(_)) => true,
            TAtomic::Scalar(TScalar::String(string)) => match string.get_known_literal_value() {
                Some(value) => self.string_grammar.parse(value).is_some(),
                None => string.is_numeric,
            },
            _ => self.is_class_of(atomic),
        }
    }

    #[must_use]
    pub fn accepts_comparison_operand(&self, atomic: &TAtomic) -> bool {
        self.compares_with_any_value
            || matches!(atomic, TAtomic::Null | TAtomic::Scalar(TScalar::Float(_)))
            || self.accepts_operand(atomic)
    }

    #[must_use]
    pub fn get_instance_atomic(&self) -> TAtomic {
        TAtomic::Object(TObject::Named(TNamedObject::new(word(self.name))))
    }

    /// Whether `atomic` is a literal this class reads as zero, so dividing by it throws.
    #[must_use]
    pub fn is_literal_zero(&self, atomic: &TAtomic) -> bool {
        match atomic.get_literal_string_value() {
            Some(value) => self.string_grammar.parse(value).is_some_and(|literal| literal.is_zero),
            None => atomic.get_literal_int_value() == Some(0) || atomic.is_false(),
        }
    }

    #[must_use]
    pub fn is_literal_negative(&self, atomic: &TAtomic) -> bool {
        match atomic.get_literal_string_value() {
            Some(value) => self.string_grammar.parse(value).is_some_and(|literal| literal.is_negative),
            None => atomic.get_literal_int_value().is_some_and(|value| value < 0),
        }
    }

    /// Whether `atomic` is a literal string with non-zero digits after the `.`, which BCMath refuses as an exponent.
    #[must_use]
    pub fn is_literal_fractional(&self, atomic: &TAtomic) -> bool {
        atomic
            .get_literal_string_value()
            .and_then(|value| self.string_grammar.parse(value))
            .is_some_and(|literal| literal.has_fractional_part)
    }

    fn is_class_of(&self, atomic: &TAtomic) -> bool {
        get_operator_overloading_class(atomic).is_some_and(|class| class == self)
    }
}

impl StringOperandGrammar {
    fn parse(&self, value: &[u8]) -> Option<NumericLiteral> {
        match self {
            StringOperandGrammar::Decimal => parse_decimal(value),
            StringOperandGrammar::GmpInteger => parse_gmp_integer(value),
        }
    }
}

/// An empty string, a lone sign or a lone `.` is valid and reads as zero.
fn parse_decimal(value: &[u8]) -> Option<NumericLiteral> {
    let (is_signed_negative, unsigned) = match value {
        [b'-', rest @ ..] => (true, rest),
        [b'+', rest @ ..] => (false, rest),
        _ => (false, value),
    };

    let (integer_part, fractional_part) = match unsigned.iter().position(|byte| *byte == b'.') {
        Some(position) => (&unsigned[..position], &unsigned[position + 1..]),
        None => (unsigned, &[][..]),
    };

    if !integer_part.iter().chain(fractional_part).all(u8::is_ascii_digit) {
        return None;
    }

    let has_fractional_part = fractional_part.iter().any(|byte| *byte != b'0');
    let is_zero = !has_fractional_part && integer_part.iter().all(|byte| *byte == b'0');

    Some(NumericLiteral { is_negative: is_signed_negative && !is_zero, is_zero, has_fractional_part })
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

/// Mirrors GMP's `mpz_set_str`: no `+` sign, whitespace allowed after the first digit, and without
/// an explicit radix a `0x`, `0b` or `0` prefix picks hexadecimal, binary or octal.
fn parse_mpz_string(value: &[u8], radix: Option<u32>) -> Option<NumericLiteral> {
    let trimmed = trim_c_whitespace_start(value);
    let (is_signed_negative, unsigned) = match trimmed {
        [b'-', rest @ ..] => (true, rest),
        _ => (false, trimmed),
    };

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

    Some(NumericLiteral { is_negative: is_signed_negative && !is_zero, is_zero, has_fractional_part: false })
}

fn trim_c_whitespace_start(value: &[u8]) -> &[u8] {
    let start = value.iter().position(|byte| !is_c_whitespace(*byte)).unwrap_or(value.len());

    &value[start..]
}

/// C's `isspace`, which unlike [`u8::is_ascii_whitespace`] includes the vertical tab.
const fn is_c_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | 0x0B | 0x0C | b'\r')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ttype::atomic::scalar::string::TString;

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
            get_operator_overloading_class(&named("bcmath\\number")).map(|class| class.name),
            Some("BcMath\\Number")
        );
        assert_eq!(
            get_operator_overloading_class(&named("\\BcMath\\Number")).map(|class| class.name),
            Some("BcMath\\Number")
        );
        assert_eq!(get_operator_overloading_class(&named("gmp")).map(|class| class.name), Some("GMP"));
        assert_eq!(get_operator_overloading_class(&named("\\GMP")).map(|class| class.name), Some("GMP"));
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
    fn lists_the_operators_each_class_overloads() {
        let number = bcmath_number();
        assert!(number.overloads(OverloadedOperator::Arithmetic));
        assert!(number.overloads(OverloadedOperator::Negation));
        assert!(number.overloads(OverloadedOperator::IncrementDecrement));
        assert!(number.overloads(OverloadedOperator::Comparison));
        assert!(!number.overloads(OverloadedOperator::Bitwise));
        assert!(!number.overloads(OverloadedOperator::Shift));
        assert!(!number.overloads(OverloadedOperator::BitwiseNot));

        let gmp = gmp();
        for operator in [
            OverloadedOperator::Arithmetic,
            OverloadedOperator::Bitwise,
            OverloadedOperator::Shift,
            OverloadedOperator::BitwiseNot,
            OverloadedOperator::Negation,
            OverloadedOperator::IncrementDecrement,
            OverloadedOperator::Comparison,
        ] {
            assert!(gmp.overloads(operator), "GMP should overload {operator:?}");
        }
    }

    #[test]
    fn instance_atomic_is_the_class_itself() {
        assert_eq!(get_operator_overloading_class(&bcmath_number().get_instance_atomic()), Some(bcmath_number()));
        assert_eq!(get_operator_overloading_class(&gmp().get_instance_atomic()), Some(gmp()));
    }

    #[test]
    fn bcmath_number_accepts_decimal_strings() {
        for value in ["10", "-1.5", "+2", "", ".5", "1.", "007", "-", "."] {
            assert!(bcmath_number().accepts_operand(&literal_string(value)), "should accept {value:?}");
        }
    }

    #[test]
    fn bcmath_number_rejects_strings_outside_its_grammar() {
        for value in [" 5", "5 ", "1e3", "abc", "--1", "+-1", "1.2.3", "0x10"] {
            assert!(!bcmath_number().accepts_operand(&literal_string(value)), "should reject {value:?}");
        }
    }

    #[test]
    fn gmp_accepts_integer_strings() {
        for value in
            ["10", "-0x10", "0X1F", "0b101", "0o17", "010", " 5 ", "-0b1", "\t5\n", "0 5", "0x 1", " -5", "\u{b}5"]
        {
            assert!(gmp().accepts_operand(&literal_string(value)), "should accept {value:?}");
        }
    }

    #[test]
    fn gmp_rejects_strings_outside_its_grammar() {
        for value in ["+5", "", " ", "1.5", "1e3", "0xZZ", "08", "-0o7", "- 5", "0x", "0b102", "1_000", "-"] {
            assert!(!gmp().accepts_operand(&literal_string(value)), "should reject {value:?}");
        }
    }

    #[test]
    fn accepts_integers_bools_and_numeric_strings_of_unknown_value() {
        for class in [bcmath_number(), gmp()] {
            assert!(class.accepts_operand(&TAtomic::Scalar(TScalar::int())));
            assert!(class.accepts_operand(&literal_int(-3)));
            assert!(class.accepts_operand(&TAtomic::Scalar(TScalar::bool())));
            assert!(class.accepts_operand(&TAtomic::Scalar(TScalar::numeric_string())));
            assert!(!class.accepts_operand(&TAtomic::Scalar(TScalar::string())));
            assert!(!class.accepts_operand(&TAtomic::Scalar(TScalar::String(TString::non_empty()))));
        }
    }

    #[test]
    fn rejects_floats() {
        for class in [bcmath_number(), gmp()] {
            assert!(!class.accepts_operand(&TAtomic::Scalar(TScalar::float())));
            assert!(!class.accepts_operand(&TAtomic::Scalar(TScalar::literal_float(1.0))));
        }
    }

    #[test]
    fn accepts_its_own_instances_and_rejects_the_other_class() {
        assert!(bcmath_number().accepts_operand(&named("\\bcmath\\number")));
        assert!(gmp().accepts_operand(&named("GMP")));
        assert!(!bcmath_number().accepts_operand(&named("GMP")));
        assert!(!gmp().accepts_operand(&named("BcMath\\Number")));
        assert!(!gmp().accepts_operand(&named("stdClass")));
        assert!(!gmp().accepts_operand(&TAtomic::Null));
    }

    #[test]
    fn bcmath_number_compares_with_anything() {
        for atomic in
            [named("GMP"), named("stdClass"), literal_string("abc"), TAtomic::Scalar(TScalar::float()), TAtomic::Null]
        {
            assert!(bcmath_number().accepts_comparison_operand(&atomic));
        }
    }

    #[test]
    fn gmp_compares_with_numbers_null_and_integer_strings_only() {
        for atomic in [
            named("GMP"),
            literal_int(1),
            TAtomic::Scalar(TScalar::bool()),
            TAtomic::Null,
            TAtomic::Scalar(TScalar::literal_float(1.5)),
            literal_string(" 5 "),
            literal_string("0x10"),
        ] {
            assert!(gmp().accepts_comparison_operand(&atomic), "should accept {atomic:?}");
        }

        for atomic in [
            named("BcMath\\Number"),
            named("stdClass"),
            literal_string("abc"),
            literal_string("1.5"),
            literal_string("08"),
        ] {
            assert!(!gmp().accepts_comparison_operand(&atomic), "should reject {atomic:?}");
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
            assert!(class.is_literal_zero(&TAtomic::Scalar(TScalar::r#false())));
            assert!(!class.is_literal_zero(&literal_int(1)));
            assert!(!class.is_literal_zero(&TAtomic::Scalar(TScalar::int())));
            assert!(!class.is_literal_zero(&literal_string("1")));
            assert!(!class.is_literal_zero(&literal_string("abc")));
            assert!(!class.is_literal_zero(&TAtomic::Scalar(TScalar::numeric_string())));
        }

        assert!(!bcmath_number().is_literal_zero(&literal_string("0.01")));
        assert!(!gmp().is_literal_zero(&literal_string("0x10")));
    }

    #[test]
    fn detects_literal_negative_values() {
        for class in [bcmath_number(), gmp()] {
            assert!(class.is_literal_negative(&literal_int(-1)));
            assert!(class.is_literal_negative(&literal_string("-1")));
            assert!(!class.is_literal_negative(&literal_int(0)));
            assert!(!class.is_literal_negative(&literal_string("-0")));
            assert!(!class.is_literal_negative(&literal_string("2")));
            assert!(!class.is_literal_negative(&TAtomic::Scalar(TScalar::int())));
        }

        assert!(gmp().is_literal_negative(&literal_string(" -5")));
        assert!(gmp().is_literal_negative(&literal_string("-0x10")));
        assert!(!gmp().is_literal_negative(&literal_string("-0x0")));
        assert!(bcmath_number().is_literal_negative(&literal_string("-0.5")));
    }

    #[test]
    fn only_gmp_refuses_negative_exponents() {
        assert!(bcmath_number().allows_negative_exponent);
        assert!(!gmp().allows_negative_exponent);
    }

    #[test]
    fn detects_literal_fractional_strings() {
        for value in ["1.5", "1.50", "+.5", "-0.01"] {
            assert!(bcmath_number().is_literal_fractional(&literal_string(value)), "{value:?} has a fraction");
        }

        for value in ["2.0", "1.", "2", "", "abc"] {
            assert!(!bcmath_number().is_literal_fractional(&literal_string(value)), "{value:?} has no fraction");
        }

        assert!(!bcmath_number().is_literal_fractional(&literal_int(2)));
        assert!(!gmp().is_literal_fractional(&literal_string("1.5")));
    }
}
