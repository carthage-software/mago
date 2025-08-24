use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::LazyLock;

use mago_atom::atom;
use mago_atom::concat_atom;
use mago_names::ResolvedNames;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::flags::attribute::AttributeFlags;
use crate::identifier::function_like::FunctionLikeIdentifier;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::array::keyed::TKeyedArray;
use crate::ttype::atomic::array::list::TList;
use crate::ttype::atomic::callable::TCallable;
use crate::ttype::atomic::reference::TReference;
use crate::ttype::atomic::reference::TReferenceMemberSelector;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeString;
use crate::ttype::atomic::scalar::float::TFloat;
use crate::ttype::atomic::scalar::int::TInteger;
use crate::ttype::atomic::scalar::string::TString;
use crate::ttype::atomic::scalar::string::TStringLiteral;
use crate::ttype::get_bool;
use crate::ttype::get_empty_string;
use crate::ttype::get_false;
use crate::ttype::get_float;
use crate::ttype::get_int;
use crate::ttype::get_int_or_float;
use crate::ttype::get_literal_int;
use crate::ttype::get_never;
use crate::ttype::get_non_empty_string;
use crate::ttype::get_non_negative_int;
use crate::ttype::get_null;
use crate::ttype::get_open_resource;
use crate::ttype::get_positive_int;
use crate::ttype::get_string;
use crate::ttype::get_true;
use crate::ttype::union::TUnion;
use crate::ttype::wrap_atomic;
use crate::utils::str_is_numeric;

#[inline]
pub fn infer<'arena>(resolved_names: &ResolvedNames<'arena>, expression: &Expression<'arena>) -> Option<TUnion> {
    match expression {
        Expression::Literal(literal) => match literal {
            Literal::String(literal_string) => {
                Some(match literal_string.value {
                    Some(value) => {
                        if value.is_empty() {
                            get_empty_string()
                        } else if value.len() < 1000 {
                            wrap_atomic(TAtomic::Scalar(TScalar::String(TString::known_literal(atom(value)))))
                        } else {
                            wrap_atomic(TAtomic::Scalar(TScalar::String(TString::unspecified_literal_with_props(
                                str_is_numeric(value),
                                true, // truthy
                                true, // not empty
                                value.chars().all(|c| c.is_lowercase()),
                            ))))
                        }
                    }
                    None => get_string(),
                })
            }
            Literal::Integer(literal_integer) => Some(match literal_integer.value {
                Some(value) => get_literal_int(value as i64),
                None => get_int_or_float(),
            }),
            Literal::Float(_) => Some(get_float()),
            Literal::True(_) => Some(get_true()),
            Literal::False(_) => Some(get_false()),
            Literal::Null(_) => Some(get_null()),
        },
        Expression::CompositeString(composite_string) => {
            let mut contains_content = false;
            for part in composite_string.parts().iter() {
                match part {
                    StringPart::Literal(literal_string_part) => {
                        if !literal_string_part.value.is_empty() {
                            contains_content = true;
                            break;
                        }
                    }
                    _ => {
                        continue;
                    }
                }
            }

            if contains_content { Some(get_non_empty_string()) } else { Some(get_string()) }
        }
        Expression::UnaryPrefix(UnaryPrefix { operator, operand }) => {
            let operand_type = infer(resolved_names, operand)?;

            match operator {
                UnaryPrefixOperator::Plus(_) => {
                    Some(if let Some(operand_value) = operand_type.get_single_literal_int_value() {
                        get_literal_int(operand_value)
                    } else if let Some(operand_value) = operand_type.get_single_literal_float_value() {
                        TUnion::from_single(Cow::Owned(TAtomic::Scalar(TScalar::Float(TFloat::literal(operand_value)))))
                    } else {
                        operand_type
                    })
                }
                UnaryPrefixOperator::Negation(_) => {
                    Some(if let Some(operand_value) = operand_type.get_single_literal_int_value() {
                        get_literal_int(operand_value.saturating_mul(-1))
                    } else if let Some(operand_value) = operand_type.get_single_literal_float_value() {
                        TUnion::from_single(Cow::Owned(TAtomic::Scalar(TScalar::Float(TFloat::literal(
                            -operand_value,
                        )))))
                    } else {
                        operand_type
                    })
                }
                _ => None,
            }
        }
        Expression::Binary(Binary { operator: BinaryOperator::StringConcat(_), lhs, rhs }) => {
            let Some(lhs_type) = infer(resolved_names, lhs) else { return Some(get_string()) };
            let Some(rhs_type) = infer(resolved_names, rhs) else { return Some(get_string()) };

            let lhs_string = match lhs_type.get_single_owned() {
                TAtomic::Scalar(TScalar::String(s)) => s.clone(),
                _ => return Some(get_string()),
            };

            let rhs_string = match rhs_type.get_single_owned() {
                TAtomic::Scalar(TScalar::String(s)) => s.clone(),
                _ => return Some(get_string()),
            };

            if let (Some(left_val), Some(right_val)) =
                (lhs_string.get_known_literal_value(), rhs_string.get_known_literal_value())
            {
                return Some(wrap_atomic(TAtomic::Scalar(TScalar::String(TString::known_literal(concat_atom!(
                    left_val, right_val
                ))))));
            }

            let is_non_empty = lhs_string.is_non_empty() || rhs_string.is_non_empty();
            let is_truthy = lhs_string.is_truthy() || rhs_string.is_truthy();
            let is_literal_origin = lhs_string.is_literal_origin() && rhs_string.is_literal_origin();
            let is_lowercase = lhs_string.is_lowercase() && rhs_string.is_lowercase();

            let final_string_type = if is_literal_origin {
                TString::unspecified_literal_with_props(false, is_truthy, is_non_empty, is_lowercase)
            } else {
                TString::general_with_props(false, is_truthy, is_non_empty, is_lowercase)
            };

            Some(wrap_atomic(TAtomic::Scalar(TScalar::String(final_string_type))))
        }
        Expression::Binary(Binary { operator, lhs, rhs }) if operator.is_bitwise() => {
            let lhs = infer(resolved_names, lhs);
            let rhs = infer(resolved_names, rhs);

            Some(wrap_atomic(
                match (
                    lhs.and_then(|v| v.get_single_literal_int_value()),
                    rhs.and_then(|v| v.get_single_literal_int_value()),
                ) {
                    (Some(lhs), Some(rhs)) => {
                        let value = match operator {
                            BinaryOperator::BitwiseAnd(_) => lhs & rhs,
                            BinaryOperator::BitwiseOr(_) => lhs | rhs,
                            BinaryOperator::BitwiseXor(_) => lhs ^ rhs,
                            BinaryOperator::LeftShift(_) => lhs << rhs,
                            BinaryOperator::RightShift(_) => lhs >> rhs,
                            _ => {
                                unreachable!("unexpected bitwise operator: {:?}", operator);
                            }
                        };

                        TAtomic::Scalar(TScalar::literal_int(value))
                    }
                    _ => TAtomic::Scalar(TScalar::int()),
                },
            ))
        }
        Expression::Construct(construct) => match construct {
            Construct::Isset(_) => Some(get_bool()),
            Construct::Empty(_) => Some(get_bool()),
            Construct::Print(_) => Some(get_literal_int(1)),
            _ => None,
        },
        Expression::ConstantAccess(access) => infer_constant(resolved_names, &access.name),
        Expression::Access(Access::ClassConstant(ClassConstantAccess {
            class,
            constant: ClassLikeConstantSelector::Identifier(identifier),
            ..
        })) => {
            let class_name_str = if let Expression::Identifier(identifier) = class {
                resolved_names.get(identifier)
            } else {
                return None;
            };

            Some(wrap_atomic(if identifier.value.eq_ignore_ascii_case("class") {
                TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::literal(atom(class_name_str))))
            } else if class_name_str.eq_ignore_ascii_case("Attribute") {
                let bits = match identifier.value {
                    "TARGET_CLASS" => Some(AttributeFlags::TARGET_CLASS.bits()),
                    "TARGET_FUNCTION" => Some(AttributeFlags::TARGET_FUNCTION.bits()),
                    "TARGET_METHOD" => Some(AttributeFlags::TARGET_METHOD.bits()),
                    "TARGET_PROPERTY" => Some(AttributeFlags::TARGET_PROPERTY.bits()),
                    "TARGET_CLASS_CONSTANT" => Some(AttributeFlags::TARGET_CLASS_CONSTANT.bits()),
                    "TARGET_PARAMETER" => Some(AttributeFlags::TARGET_PARAMETER.bits()),
                    "TARGET_CONSTANT" => Some(AttributeFlags::TARGET_CONSTANT.bits()),
                    "TARGET_ALL" => Some(AttributeFlags::TARGET_ALL.bits()),
                    "IS_REPEATABLE" => Some(AttributeFlags::IS_REPEATABLE.bits()),
                    _ => None,
                };

                match bits {
                    Some(bits) => return Some(get_literal_int(bits as i64)),
                    None => TAtomic::Reference(TReference::Member {
                        class_like_name: atom(class_name_str),
                        member_selector: TReferenceMemberSelector::Identifier(atom(identifier.value)),
                    }),
                }
            } else {
                TAtomic::Reference(TReference::Member {
                    class_like_name: atom(class_name_str),
                    member_selector: TReferenceMemberSelector::Identifier(atom(identifier.value)),
                })
            }))
        }
        Expression::Array(Array { elements, .. }) | Expression::LegacyArray(LegacyArray { elements, .. })
            if is_list_array_expression(expression) =>
        {
            let mut entries = BTreeMap::new();

            for (i, element) in elements.iter().enumerate() {
                let ArrayElement::Value(element) = element else {
                    return None;
                };

                entries.insert(i, (false, infer(resolved_names, element.value)?));
            }

            Some(wrap_atomic(TAtomic::Array(TArray::List(TList {
                known_count: Some(entries.len()),
                known_elements: Some(entries),
                element_type: Box::new(get_never()),
                non_empty: true,
            }))))
        }
        Expression::Array(Array { elements, .. }) | Expression::LegacyArray(LegacyArray { elements, .. })
            if is_keyed_array_expression(expression) =>
        {
            let mut known_items = BTreeMap::new();
            for element in elements.iter() {
                let ArrayElement::KeyValue(element) = element else {
                    return None;
                };

                let key_type = infer(resolved_names, element.key).and_then(|v| v.get_single_array_key())?;
                known_items.insert(key_type, (false, infer(resolved_names, element.value)?));

                if known_items.len() > 100 {
                    return None;
                }
            }

            let mut keyed_array = TKeyedArray::new();
            keyed_array.non_empty = !known_items.is_empty();
            keyed_array.known_items = Some(known_items);

            Some(TUnion::from_single(Cow::Owned(TAtomic::Array(TArray::Keyed(keyed_array)))))
        }
        Expression::Closure(closure) => {
            let span = closure.span();

            Some(wrap_atomic(TAtomic::Callable(TCallable::Alias(FunctionLikeIdentifier::Closure(
                span.file_id,
                span.start,
            )))))
        }
        Expression::ArrowFunction(arrow_func) => {
            let span = arrow_func.span();

            Some(wrap_atomic(TAtomic::Callable(TCallable::Alias(FunctionLikeIdentifier::Closure(
                span.file_id,
                span.start,
            )))))
        }
        _ => None,
    }
}

#[inline]
fn infer_constant(names: &ResolvedNames, constant: &Identifier) -> Option<TUnion> {
    static DIR_SEPARATOR_SLICE: LazyLock<[TAtomic; 2]> = LazyLock::new(|| {
        [
            TAtomic::Scalar(TScalar::String(TString {
                literal: Some(TStringLiteral::Value(atom("/"))),
                is_numeric: false,
                is_truthy: true,
                is_non_empty: true,
                is_lowercase: true,
            })),
            TAtomic::Scalar(TScalar::String(TString {
                literal: Some(TStringLiteral::Value(atom("\\"))),
                is_numeric: false,
                is_truthy: true,
                is_non_empty: true,
                is_lowercase: true,
            })),
        ]
    });

    const PHP_INT_MAX_SLICE: &[TAtomic] = &[
        TAtomic::Scalar(TScalar::Integer(TInteger::Literal(9223372036854775807))),
        TAtomic::Scalar(TScalar::Integer(TInteger::Literal(2147483647))),
    ];

    const PHP_INT_MIN_SLICE: &[TAtomic] = &[
        TAtomic::Scalar(TScalar::Integer(TInteger::Literal(-9223372036854775808))),
        TAtomic::Scalar(TScalar::Integer(TInteger::Literal(-2147483648))),
    ];

    const PHP_MAJOR_VERSION_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::Integer(TInteger::Range(8, 9)));
    const PHP_ZTS_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::Integer(TInteger::Range(0, 1)));
    const PHP_DEBUG_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::Integer(TInteger::Range(0, 1)));
    const PHP_INT_SIZE_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::Integer(TInteger::Range(4, 8)));
    const PHP_WINDOWS_VERSION_MAJOR_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::Integer(TInteger::Range(4, 6)));
    const PHP_WINDOWS_VERSION_MINOR_SLICE: &[TAtomic] = &[
        TAtomic::Scalar(TScalar::Integer(TInteger::Literal(0))),
        TAtomic::Scalar(TScalar::Integer(TInteger::Literal(1))),
        TAtomic::Scalar(TScalar::Integer(TInteger::Literal(2))),
        TAtomic::Scalar(TScalar::Integer(TInteger::Literal(10))),
        TAtomic::Scalar(TScalar::Integer(TInteger::Literal(90))),
    ];

    let (short_name, _) = if names.is_imported(constant) {
        (names.get(constant), names.get(constant))
    } else if let Some(stripped) = constant.value().strip_prefix('\\') {
        (stripped, names.get(constant))
    } else {
        (constant.value(), names.get(constant))
    };

    Some(match short_name {
        "PHP_MAXPATHLEN"
        | "PHP_WINDOWS_VERSION_BUILD"
        | "LIBXML_VERSION"
        | "OPENSSL_VERSION_NUMBER"
        | "PHP_FLOAT_DIG" => get_int(),
        "PHP_EXTRA_VERSION" => get_string(),
        "PHP_BUILD_DATE"
        | "PEAR_EXTENSION_DIR"
        | "PEAR_INSTALL_DIR"
        | "PHP_BINARY"
        | "PHP_BINDIR"
        | "PHP_CONFIG_FILE_PATH"
        | "PHP_CONFIG_FILE_SCAN_DIR"
        | "PHP_DATADIR"
        | "PHP_EXTENSION_DIR"
        | "PHP_LIBDIR"
        | "PHP_LOCALSTATEDIR"
        | "PHP_MANDIR"
        | "PHP_OS"
        | "PHP_OS_FAMILY"
        | "PHP_PREFIX"
        | "PHP_EOL"
        | "PATH_SEPARATOR"
        | "PHP_VERSION"
        | "PHP_SAPI"
        | "PHP_SYSCONFDIR"
        | "ICONV_IMPL"
        | "LIBXML_DOTTED_VERSION"
        | "PCRE_VERSION" => get_non_empty_string(),
        "STDIN" | "STDOUT" | "STDERR" => get_open_resource(),
        "NAN" | "PHP_FLOAT_EPSILON" | "INF" => get_float(),
        "PHP_VERSION_ID" => get_positive_int(),
        "PHP_RELEASE_VERSION" | "PHP_MINOR_VERSION" => get_non_negative_int(),
        "PHP_MAJOR_VERSION" => TUnion::from_single(Cow::Borrowed(PHP_MAJOR_VERSION_ATOMIC)),
        "PHP_ZTS" => TUnion::from_single(Cow::Borrowed(PHP_ZTS_ATOMIC)),
        "PHP_DEBUG" => TUnion::from_single(Cow::Borrowed(PHP_DEBUG_ATOMIC)),
        "PHP_INT_SIZE" => TUnion::from_single(Cow::Borrowed(PHP_INT_SIZE_ATOMIC)),
        "PHP_WINDOWS_VERSION_MAJOR" => TUnion::from_single(Cow::Borrowed(PHP_WINDOWS_VERSION_MAJOR_ATOMIC)),
        "DIRECTORY_SEPARATOR" => TUnion::new(Cow::Borrowed(DIR_SEPARATOR_SLICE.as_slice())),
        "PHP_INT_MAX" => TUnion::new(Cow::Borrowed(PHP_INT_MAX_SLICE)),
        "PHP_INT_MIN" => TUnion::new(Cow::Borrowed(PHP_INT_MIN_SLICE)),
        "PHP_WINDOWS_VERSION_MINOR" => TUnion::new(Cow::Borrowed(PHP_WINDOWS_VERSION_MINOR_SLICE)),
        _ => return None,
    })
}

#[inline]
fn is_list_array_expression(expression: &Expression) -> bool {
    match expression {
        Expression::Array(Array { elements, .. }) | Expression::LegacyArray(LegacyArray { elements, .. }) => {
            elements.iter().all(|element| matches!(element, ArrayElement::Value(_)))
        }
        _ => false,
    }
}

#[inline]
fn is_keyed_array_expression(expression: &Expression) -> bool {
    match expression {
        Expression::Array(Array { elements, .. }) | Expression::LegacyArray(LegacyArray { elements, .. }) => {
            elements.iter().all(|element| matches!(element, ArrayElement::KeyValue(_)))
        }
        _ => false,
    }
}
