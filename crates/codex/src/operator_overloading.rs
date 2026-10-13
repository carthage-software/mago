use mago_syntax::cst::BinaryOperator;
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
#[derive(Debug)]
pub struct OperatorOverloadingClass {
    name: &'static str,
    operators: &'static [OverloadedOperator],
    string_grammar: StringOperandGrammar,
    exponent_or_shift_amount: ExponentOrShiftAmount,
    compares_with_any_value: bool,
}

/// How a class reads a string operand, which is stricter than PHP's own numeric strings.
#[derive(Debug)]
enum StringOperandGrammar {
    /// BCMath: an optional sign, then digits with at most one `.`; no whitespace, no exponent.
    Decimal,
    /// GMP's `mpz_set_str` after PHP strips `0x`, `0o` and `0b` prefixes.
    GmpInteger,
}

/// How a class reads the right operand of `**`, `<<` and `>>`.
#[derive(Debug)]
enum ExponentOrShiftAmount {
    /// Like any other operand, as long as it has no fractional part.
    WholeOperand,
    /// As a PHP `int` that is not negative: PHP's own integer strings rather than GMP's, and no `bool`.
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

/// Returns the operator-overloading class `atomic` is an instance of, if any.
#[must_use]
pub fn get_operator_overloading_class(atomic: &TAtomic) -> Option<&'static OperatorOverloadingClass> {
    let TAtomic::Object(TObject::Named(named_object)) = atomic else {
        return None;
    };

    let name = named_object.name.as_bytes().strip_prefix(b"\\").unwrap_or(named_object.name.as_bytes());

    OPERATOR_OVERLOADING_CLASSES.iter().copied().find(|class| name.eq_ignore_ascii_case(class.name.as_bytes()))
}

/// Returns the class whose compare handler PHP runs for `left operator right`, with the operand that
/// handler has to accept, or `None` when no listed class's handler runs.
///
/// PHP runs the handler of the first operand when it is an object, else that of the second, and
/// compiles `>` and `>=` as `<` and `<=` with the operands swapped.
#[must_use]
pub fn get_comparison_handler<'operand>(
    operator: &BinaryOperator<'_>,
    left: &'operand TAtomic,
    right: &'operand TAtomic,
) -> Option<(&'static OperatorOverloadingClass, &'operand TAtomic)> {
    if !operator.is_comparison() || operator.is_identity() {
        return None;
    }

    let (first, second) = match operator {
        BinaryOperator::GreaterThan(_) | BinaryOperator::GreaterThanOrEqual(_) => (right, left),
        _ => (left, right),
    };

    if first.is_object_type() {
        return get_operator_overloading_class(first).map(|class| (class, second));
    }

    get_operator_overloading_class(second).map(|class| (class, first))
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

    /// Whether `atomic` can be the other operand of an operator this class overloads, except the right
    /// operand of `**`, `<<` and `>>` after an instance, which [`Self::accepts_exponent_or_shift_amount`] checks.
    ///
    /// `float` is rejected: PHP truncates it to `int` with a deprecation.
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

    /// Whether `atomic` can be the right operand of `**`, `<<` or `>>` when the left one is an instance.
    ///
    /// `float` is rejected: GMP truncates it to `int` silently, BCMath with a deprecation.
    #[must_use]
    pub fn accepts_exponent_or_shift_amount(&self, atomic: &TAtomic) -> bool {
        match self.exponent_or_shift_amount {
            ExponentOrShiftAmount::WholeOperand => {
                self.accepts_operand(atomic)
                    && !atomic
                        .get_literal_string_value()
                        .and_then(|value| self.string_grammar.parse(value))
                        .is_some_and(|literal| literal.has_fractional_part)
            }
            ExponentOrShiftAmount::NonNegativeInt => match atomic {
                TAtomic::Scalar(TScalar::Integer(integer)) => !integer.is_negative(),
                TAtomic::Scalar(TScalar::String(string)) => match string.get_known_literal_value() {
                    Some(value) => parse_php_integer(value).is_some_and(|value| value >= 0),
                    None => string.is_numeric,
                },
                _ => self.is_class_of(atomic),
            },
        }
    }

    /// Whether `atomic` can be the other operand of a comparison this class's compare handler runs.
    ///
    /// `float` is rejected: PHP truncates it to `int` with a deprecation here too.
    #[must_use]
    pub fn accepts_comparison_operand(&self, atomic: &TAtomic) -> bool {
        match atomic {
            TAtomic::Scalar(TScalar::Float(_)) => false,
            TAtomic::Null => true,
            _ => self.compares_with_any_value || self.accepts_operand(atomic),
        }
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
            None => atomic.get_literal_int_value() == Some(0),
        }
    }

    fn is_class_of(&self, atomic: &TAtomic) -> bool {
        get_operator_overloading_class(atomic).is_some_and(|class| std::ptr::eq(class, self))
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

/// Mirrors GMP's `mpz_set_str`: no `+` sign, whitespace allowed after the first digit, and without
/// an explicit radix a `0x`, `0b` or `0` prefix picks hexadecimal, binary or octal.
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

/// PHP's integer strings: whitespace around an optional sign and decimal digits that fit in an `int`.
fn parse_php_integer(value: &[u8]) -> Option<i64> {
    let start = value.iter().position(|byte| !is_c_whitespace(*byte))?;
    let end = value.iter().rposition(|byte| !is_c_whitespace(*byte))?;

    std::str::from_utf8(&value[start..=end]).ok()?.parse().ok()
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
    use mago_span::Span;

    use super::*;
    use crate::ttype::atomic::scalar::int::TInteger;
    use crate::ttype::atomic::scalar::string::TString;
    use crate::ttype::get_mixed_keyed_array;

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

    fn comparison_operator(operator: &str) -> BinaryOperator<'static> {
        let span = Span::dummy(0, 1);

        match operator {
            "<" => BinaryOperator::LessThan(span),
            "<=" => BinaryOperator::LessThanOrEqual(span),
            ">" => BinaryOperator::GreaterThan(span),
            ">=" => BinaryOperator::GreaterThanOrEqual(span),
            "==" => BinaryOperator::Equal(span),
            "!=" => BinaryOperator::NotEqual(span),
            "<=>" => BinaryOperator::Spaceship(span),
            "===" => BinaryOperator::Identical(span),
            "+" => BinaryOperator::Addition(span),
            _ => unreachable!("no test uses {operator}"),
        }
    }

    fn comparison_handler_name(operator: &str, left: &TAtomic, right: &TAtomic) -> Option<&'static str> {
        get_comparison_handler(&comparison_operator(operator), left, right).map(|(class, _)| class.get_name())
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
    fn instance_atomic_is_the_class_itself() {
        for class in [bcmath_number(), gmp()] {
            assert!(class.accepts_operand(&class.get_instance_atomic()));
            assert!(
                get_operator_overloading_class(&class.get_instance_atomic())
                    .is_some_and(|found| std::ptr::eq(found, class))
            );
        }
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
            for float in [TAtomic::Scalar(TScalar::float()), TAtomic::Scalar(TScalar::literal_float(1.0))] {
                assert!(!class.accepts_operand(&float));
                assert!(!class.accepts_exponent_or_shift_amount(&float));
                assert!(!class.accepts_comparison_operand(&float));
            }
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
    fn gmp_reads_exponents_and_shift_amounts_as_non_negative_php_ints() {
        for atomic in [
            literal_int(2),
            literal_int(0),
            TAtomic::Scalar(TScalar::int()),
            TAtomic::Scalar(TScalar::numeric_string()),
            named("GMP"),
            literal_string("2"),
            literal_string("010"),
            literal_string(" \t2 \r"),
            literal_string("+2"),
            literal_string("-0"),
        ] {
            assert!(gmp().accepts_exponent_or_shift_amount(&atomic), "should accept {atomic:?}");
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
            TAtomic::Scalar(TScalar::literal_float(1.5)),
            TAtomic::Scalar(TScalar::bool()),
            TAtomic::Null,
            named("BcMath\\Number"),
        ] {
            assert!(!gmp().accepts_exponent_or_shift_amount(&atomic), "should reject {atomic:?}");
        }
    }

    #[test]
    fn bcmath_number_takes_negative_exponents_but_not_fractional_strings() {
        for atomic in [
            literal_int(-1),
            literal_string("-1"),
            literal_string("2.0"),
            TAtomic::Scalar(TScalar::r#true()),
            named("BcMath\\Number"),
        ] {
            assert!(bcmath_number().accepts_exponent_or_shift_amount(&atomic), "should accept {atomic:?}");
        }

        for atomic in [
            literal_string("1.5"),
            literal_string("-0.01"),
            literal_string("0x2"),
            literal_string(" 2"),
            TAtomic::Scalar(TScalar::literal_float(1.5)),
            named("GMP"),
        ] {
            assert!(!bcmath_number().accepts_exponent_or_shift_amount(&atomic), "should reject {atomic:?}");
        }
    }

    #[test]
    fn bcmath_number_compares_with_anything_but_floats() {
        for atomic in [
            named("GMP"),
            named("stdClass"),
            literal_string("abc"),
            get_mixed_keyed_array().get_single().clone(),
            TAtomic::Null,
        ] {
            assert!(bcmath_number().accepts_comparison_operand(&atomic), "should accept {atomic:?}");
        }
    }

    #[test]
    fn gmp_compares_with_ints_bools_null_and_integer_strings_only() {
        for atomic in [
            named("GMP"),
            literal_int(1),
            TAtomic::Scalar(TScalar::bool()),
            TAtomic::Null,
            literal_string(" 5 "),
            literal_string("0x10"),
        ] {
            assert!(gmp().accepts_comparison_operand(&atomic), "should accept {atomic:?}");
        }

        for atomic in [
            named("BcMath\\Number"),
            named("stdClass"),
            get_mixed_keyed_array().get_single().clone(),
            literal_string("abc"),
            literal_string("1.5"),
            literal_string("08"),
        ] {
            assert!(!gmp().accepts_comparison_operand(&atomic), "should reject {atomic:?}");
        }
    }

    #[test]
    fn comparisons_run_the_first_objects_handler_with_greater_than_swapping_operands() {
        let gmp = named("GMP");
        let number = named("BcMath\\Number");
        let std_class = named("stdClass");
        let int = literal_int(5);

        assert_eq!(comparison_handler_name("==", &gmp, &number), Some("GMP"));
        assert_eq!(comparison_handler_name("==", &number, &gmp), Some("BcMath\\Number"));
        assert_eq!(comparison_handler_name("<", &gmp, &number), Some("GMP"));
        assert_eq!(comparison_handler_name(">", &gmp, &number), Some("BcMath\\Number"));
        assert_eq!(comparison_handler_name(">=", &number, &gmp), Some("GMP"));
        assert_eq!(comparison_handler_name("<=>", &number, &gmp), Some("BcMath\\Number"));
        assert_eq!(comparison_handler_name("<", &std_class, &gmp), None);
        assert_eq!(comparison_handler_name(">", &gmp, &std_class), None);
        assert_eq!(comparison_handler_name("<", &gmp, &std_class), Some("GMP"));
        assert_eq!(comparison_handler_name("<", &int, &gmp), Some("GMP"));
        assert_eq!(comparison_handler_name(">", &int, &gmp), Some("GMP"));
        assert_eq!(comparison_handler_name("<", &int, &int), None);
    }

    #[test]
    fn identity_and_non_comparison_operators_run_no_compare_handler() {
        let gmp = named("GMP");
        let abc = literal_string("abc");

        assert_eq!(comparison_handler_name("===", &gmp, &abc), None);
        assert_eq!(comparison_handler_name("+", &gmp, &abc), None);
    }

    #[test]
    fn comparison_handlers_reject_operands_where_php_throws() {
        let gmp = named("GMP");
        let number = named("BcMath\\Number");
        let std_class = named("stdClass");
        let array = get_mixed_keyed_array().get_single().clone();
        let abc = literal_string("abc");
        let int = literal_int(5);
        let null = TAtomic::Null;

        let cases: [(&TAtomic, &str, &TAtomic, bool); 14] = [
            (&gmp, "==", &number, true),
            (&number, "==", &gmp, false),
            (&gmp, "<", &number, true),
            (&gmp, ">", &number, false),
            (&number, ">", &gmp, true),
            (&number, "<=>", &gmp, false),
            (&array, "<", &gmp, true),
            (&int, "<", &gmp, false),
            (&abc, "==", &gmp, true),
            (&gmp, "!=", &abc, true),
            (&std_class, "<", &gmp, false),
            (&gmp, "<", &std_class, true),
            (&null, "<=>", &gmp, false),
            (&number, "<", &abc, false),
        ];

        for (left, operator, right, throws) in cases {
            let rejects = get_comparison_handler(&comparison_operator(operator), left, right)
                .is_some_and(|(class, operand)| !class.accepts_comparison_operand(operand));

            assert_eq!(rejects, throws, "{left:?} {operator} {right:?}");
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
