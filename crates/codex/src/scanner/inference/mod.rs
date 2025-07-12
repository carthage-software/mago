use std::collections::BTreeMap;

use mago_interner::ThreadedInterner;
use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::flags::attribute::AttributeFlags;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::array::keyed::TKeyedArray;
use crate::ttype::atomic::array::list::TList;
use crate::ttype::atomic::reference::TReference;
use crate::ttype::atomic::reference::TReferenceMemberSelector;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeString;
use crate::ttype::atomic::scalar::float::TFloat;
use crate::ttype::atomic::scalar::int::TInteger;
use crate::ttype::atomic::scalar::string::TString;
use crate::ttype::get_bool;
use crate::ttype::get_false;
use crate::ttype::get_float;
use crate::ttype::get_int;
use crate::ttype::get_literal_int;
use crate::ttype::get_never;
use crate::ttype::get_non_empty_string;
use crate::ttype::get_null;
use crate::ttype::get_open_resource;
use crate::ttype::get_string;
use crate::ttype::get_true;
use crate::ttype::union::TUnion;
use crate::ttype::wrap_atomic;
use crate::utils::str_is_numeric;

#[inline]
pub fn infer(interner: &ThreadedInterner, resolved_names: &ResolvedNames, expression: &Expression) -> Option<TUnion> {
    match expression {
        Expression::Literal(literal) => match literal {
            Literal::String(literal_string) => {
                Some(match literal_string.value.as_deref() {
                    Some(value) => {
                        if value.len() < 1000 {
                            wrap_atomic(TAtomic::Scalar(TScalar::String(TString::known_literal(value.to_owned()))))
                        } else {
                            wrap_atomic(TAtomic::Scalar(TScalar::String(TString::unspecified_literal_with_props(
                                str_is_numeric(value),
                                true, // truthy
                                true, // not empty
                            ))))
                        }
                    }
                    None => get_string(),
                })
            }
            Literal::Integer(literal_integer) => Some(get_literal_int(literal_integer.value as i64)),
            Literal::Float(_) => Some(get_float()),
            Literal::True(_) => Some(get_true()),
            Literal::False(_) => Some(get_false()),
            Literal::Null(_) => Some(get_null()),
        },
        Expression::UnaryPrefix(UnaryPrefix { operator, operand }) => {
            let operand_type = infer(interner, resolved_names, operand)?;

            match operator {
                UnaryPrefixOperator::Plus(_) => {
                    Some(if let Some(operand_value) = operand_type.get_single_literal_int_value() {
                        get_literal_int(operand_value)
                    } else if let Some(operand_value) = operand_type.get_single_literal_float_value() {
                        TUnion::new(vec![TAtomic::Scalar(TScalar::Float(TFloat::literal(operand_value)))])
                    } else {
                        operand_type
                    })
                }
                UnaryPrefixOperator::Negation(_) => {
                    Some(if let Some(operand_value) = operand_type.get_single_literal_int_value() {
                        get_literal_int(operand_value.saturating_mul(-1))
                    } else if let Some(operand_value) = operand_type.get_single_literal_float_value() {
                        TUnion::new(vec![TAtomic::Scalar(TScalar::Float(TFloat::literal(-operand_value)))])
                    } else {
                        operand_type
                    })
                }
                _ => None,
            }
        }
        Expression::Binary(Binary { operator: BinaryOperator::StringConcat(_), lhs, rhs }) => {
            let lhs_type = infer(interner, resolved_names, lhs);
            let rhs_type = infer(interner, resolved_names, rhs);

            let lhs_string = lhs_type.map_or_else(TString::general, |t| match t.get_single_owned() {
                TAtomic::Scalar(TScalar::String(s)) => s.clone(),
                _ => TString::general(),
            });

            let rhs_string = rhs_type.map_or_else(TString::general, |t| match t.get_single_owned() {
                TAtomic::Scalar(TScalar::String(s)) => s.clone(),
                _ => TString::general(),
            });

            if let (Some(left_val), Some(right_val)) =
                (lhs_string.get_known_literal_value(), rhs_string.get_known_literal_value())
            {
                let combined_value = format!("{left_val}{right_val}");

                return Some(wrap_atomic(TAtomic::Scalar(TScalar::String(TString::known_literal(combined_value)))));
            }

            let is_non_empty = lhs_string.is_non_empty() || rhs_string.is_non_empty();
            let is_truthy = lhs_string.is_truthy() || rhs_string.is_truthy();
            let is_literal_origin = lhs_string.is_literal_origin() && rhs_string.is_literal_origin();

            let final_string_type = if is_literal_origin {
                TString::unspecified_literal_with_props(false, is_truthy, is_non_empty)
            } else {
                TString::general_with_props(false, is_truthy, is_non_empty)
            };

            Some(wrap_atomic(TAtomic::Scalar(TScalar::String(final_string_type))))
        }
        Expression::Binary(Binary { operator, lhs, rhs }) if operator.is_bitwise() => {
            let lhs = infer(interner, resolved_names, lhs);
            let rhs = infer(interner, resolved_names, rhs);

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
        Expression::ConstantAccess(access) => infer_constant(interner, resolved_names, &access.name),
        Expression::Access(Access::ClassConstant(ClassConstantAccess {
            class,
            constant: ClassLikeConstantSelector::Identifier(identifier),
            ..
        })) => {
            let class_name = if let Expression::Identifier(identifier) = class.as_ref() {
                resolved_names.get(identifier)
            } else {
                return None;
            };

            let class_name_str = interner.lookup(class_name);
            let member_name = interner.lookup(&identifier.value);
            Some(wrap_atomic(if member_name.eq_ignore_ascii_case("class") {
                TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::literal(*class_name)))
            } else if class_name_str.eq_ignore_ascii_case("Attribute") {
                let bits = match member_name {
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
                    Some(bits) => TAtomic::Scalar(TScalar::literal_int(bits as i64)),
                    None => TAtomic::Reference(TReference::Member {
                        class_like_name: *class_name,
                        member_selector: TReferenceMemberSelector::Identifier(identifier.value),
                    }),
                }
            } else {
                TAtomic::Reference(TReference::Member {
                    class_like_name: *class_name,
                    member_selector: TReferenceMemberSelector::Identifier(identifier.value),
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

                entries.insert(i, (false, infer(interner, resolved_names, &element.value)?));
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

                let key_type = infer(interner, resolved_names, &element.key).and_then(|v| v.get_single_array_key())?;
                known_items.insert(key_type, (false, infer(interner, resolved_names, &element.value)?));

                if known_items.len() > 100 {
                    return None;
                }
            }

            let mut keyed_array = TKeyedArray::new();
            keyed_array.non_empty = !known_items.is_empty();
            keyed_array.known_items = Some(known_items);

            Some(TUnion::new(vec![TAtomic::Array(TArray::Keyed(keyed_array))]))
        }
        _ => None,
    }
}

#[inline]
fn infer_constant(interner: &ThreadedInterner, names: &ResolvedNames, constant: &Identifier) -> Option<TUnion> {
    let (short_name, _) = if names.is_imported(constant) {
        let name = interner.lookup(names.get(constant));

        (name, name)
    } else {
        let short_name = interner.lookup(constant.value());
        let imported_name = interner.lookup(names.get(constant));

        if let Some(stripped) = short_name.strip_prefix('\\') {
            (stripped, imported_name)
        } else {
            (short_name, imported_name)
        }
    };

    Some(match short_name {
        "PHP_MAXPATHLEN"
        | "PHP_WINDOWS_VERSION_BUILD"
        | "LIBXML_VERSION"
        | "OPENSSL_VERSION_NUMBER"
        | "PHP_FLOAT_DIG" => get_int(),
        "PHP_VERSION_ID" => TUnion::new(vec![TAtomic::Scalar(TScalar::Integer(TInteger::positive()))]),
        "PHP_RELEASE_VERSION" | "PHP_MINOR_VERSION" => {
            TUnion::new(vec![TAtomic::Scalar(TScalar::Integer(TInteger::non_negative()))])
        }
        "PHP_EXTRA_VERSION" => get_string(),
        "PEAR_EXTENSION_DIR"
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
        "DIRECTORY_SEPARATOR" => TUnion::new(vec![
            TAtomic::Scalar(TScalar::String(TString::known_literal("\\".to_string()))),
            TAtomic::Scalar(TScalar::String(TString::known_literal("/".to_string()))),
        ]),
        "PHP_INT_MAX" => TUnion::new(vec![
            get_literal_int(9223372036854775807).get_single_owned(),
            get_literal_int(2147483647).get_single_owned(),
        ]),
        "PHP_INT_MIN" => TUnion::new(vec![
            get_literal_int(-9223372036854775808).get_single_owned(),
            get_literal_int(-2147483648).get_single_owned(),
        ]),
        "PHP_MAJOR_VERSION" => TUnion::new(vec![TAtomic::Scalar(TScalar::Integer(TInteger::Range(8, 9)))]),
        "PHP_ZTS" => TUnion::new(vec![TAtomic::Scalar(TScalar::Integer(TInteger::Range(0, 1)))]),
        "PHP_DEBUG" => TUnion::new(vec![TAtomic::Scalar(TScalar::Integer(TInteger::Range(0, 1)))]),
        "PHP_INT_SIZE" => TUnion::new(vec![TAtomic::Scalar(TScalar::Integer(TInteger::Range(4, 8)))]),
        "PHP_WINDOWS_VERSION_MAJOR" => TUnion::new(vec![TAtomic::Scalar(TScalar::Integer(TInteger::Range(4, 6)))]),
        "PHP_WINDOWS_VERSION_MINOR" => TUnion::new(vec![
            get_literal_int(0).get_single_owned(),  // Vista/2008/2000/NT4/95
            get_literal_int(1).get_single_owned(),  // XP
            get_literal_int(2).get_single_owned(),  // 2003 R2/2003/XP x64
            get_literal_int(10).get_single_owned(), // 98
            get_literal_int(90).get_single_owned(), // Me
        ]),
        "STDIN" | "STDOUT" | "STDERR" => get_open_resource(),
        "NAN" | "PHP_FLOAT_EPSILON" | "INF" => get_float(),
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
