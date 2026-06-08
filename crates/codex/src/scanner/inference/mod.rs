use mago_allocator::Arena;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::LazyLock;

use mago_names::ResolvedNames;
use mago_names::scope::NamespaceScope;
use mago_span::HasPosition;
use mago_span::HasSpan;
use mago_syntax::ast::Access;
use mago_syntax::ast::Array;
use mago_syntax::ast::ArrayElement;
use mago_syntax::ast::Binary;
use mago_syntax::ast::BinaryOperator;
use mago_syntax::ast::ClassConstantAccess;
use mago_syntax::ast::ClassLikeConstantSelector;
use mago_syntax::ast::Construct;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Identifier;
use mago_syntax::ast::LegacyArray;
use mago_syntax::ast::Literal;
use mago_syntax::ast::MagicConstant;
use mago_syntax::ast::StringPart;
use mago_syntax::ast::UnaryPrefix;
use mago_syntax::ast::UnaryPrefixOperator;
use mago_word::Word;
use mago_word::WordMap;
use mago_word::ascii_lowercase_constant_name_word;
use mago_word::concat_word;
use mago_word::word;

use crate::flags::attribute::AttributeFlags;
use crate::identifier::function_like::FunctionLikeIdentifier;
use crate::metadata::constant::ConstantMetadata;
use crate::scanner::Context;
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
use crate::ttype::atomic::scalar::string::TStringCasing;
use crate::ttype::atomic::scalar::string::TStringLiteral;
use crate::ttype::get_arraykey;
use crate::ttype::get_bool;
use crate::ttype::get_empty_string;
use crate::ttype::get_false;
use crate::ttype::get_float;
use crate::ttype::get_int;
use crate::ttype::get_int_or_float;
use crate::ttype::get_literal_int;
use crate::ttype::get_literal_string;
use crate::ttype::get_mixed;
use crate::ttype::get_mixed_keyed_array;
use crate::ttype::get_never;
use crate::ttype::get_non_empty_string;
use crate::ttype::get_non_negative_int;
use crate::ttype::get_null;
use crate::ttype::get_object;
use crate::ttype::get_open_resource;
use crate::ttype::get_positive_int;
use crate::ttype::get_string;
use crate::ttype::get_true;
use crate::ttype::get_void;
use crate::ttype::union::TUnion;
use crate::ttype::wrap_atomic;
use crate::utils::str_is_numeric;

/// Returns the type for a predefined literal constant, if known.
///
/// These constants (`true`, `false`, `null`) are parsed as `Literal` nodes when bare,
/// but become `ConstantAccess` nodes when accessed via FQN (e.g. `\true`).
#[inline]
#[must_use]
pub fn get_literal_constant_type(name: &[u8]) -> Option<TUnion> {
    let name = name.strip_prefix(b"\\").unwrap_or(name);

    if name.eq_ignore_ascii_case(b"true") {
        Some(get_true())
    } else if name.eq_ignore_ascii_case(b"false") {
        Some(get_false())
    } else if name.eq_ignore_ascii_case(b"null") {
        Some(get_null())
    } else {
        None
    }
}

/// Returns the platform-aware type for a predefined constant, if known.
///
/// These constants have values that vary across platforms (e.g. 32-bit vs 64-bit),
/// so their types should be ranges or unions rather than host-specific literals.
#[inline]
pub fn get_platform_constant_type(name: &[u8]) -> Option<TUnion> {
    static DIR_SEPARATOR_SLICE: LazyLock<[TAtomic; 2]> = LazyLock::new(|| {
        [
            TAtomic::Scalar(TScalar::String(TString {
                literal: Some(TStringLiteral::Value(word("/"))),
                is_numeric: false,
                is_truthy: true,
                is_non_empty: true,
                is_callable: false,
                casing: TStringCasing::Lowercase,
            })),
            TAtomic::Scalar(TScalar::String(TString {
                literal: Some(TStringLiteral::Value(word("\\"))),
                is_numeric: false,
                is_truthy: true,
                is_non_empty: true,
                is_callable: false,
                casing: TStringCasing::Lowercase,
            })),
        ]
    });

    const PHP_INT_MAX_SLICE: &[TAtomic] = &[
        TAtomic::Scalar(TScalar::Integer(TInteger::Literal(i64::MAX))),
        TAtomic::Scalar(TScalar::Integer(TInteger::Literal(i32::MAX as i64))),
    ];

    const PHP_INT_MIN_SLICE: &[TAtomic] = &[
        TAtomic::Scalar(TScalar::Integer(TInteger::Literal(i64::MIN))),
        TAtomic::Scalar(TScalar::Integer(TInteger::Literal(i32::MIN as i64))),
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

    let name = name.strip_prefix(b"\\").unwrap_or(name);

    match name {
        b"PHP_MAXPATHLEN"
        | b"PHP_WINDOWS_VERSION_BUILD"
        | b"LIBXML_VERSION"
        | b"OPENSSL_VERSION_NUMBER"
        | b"PHP_FLOAT_DIG" => Some(get_int()),
        b"PHP_EXTRA_VERSION" => Some(get_string()),
        b"PHP_BUILD_DATE"
        | b"PEAR_EXTENSION_DIR"
        | b"PEAR_INSTALL_DIR"
        | b"PHP_BINARY"
        | b"PHP_BINDIR"
        | b"PHP_CONFIG_FILE_PATH"
        | b"PHP_CONFIG_FILE_SCAN_DIR"
        | b"PHP_DATADIR"
        | b"PHP_EXTENSION_DIR"
        | b"PHP_LIBDIR"
        | b"PHP_LOCALSTATEDIR"
        | b"PHP_MANDIR"
        | b"PHP_OS"
        | b"PHP_OS_FAMILY"
        | b"PHP_PREFIX"
        | b"PHP_EOL"
        | b"PATH_SEPARATOR"
        | b"PHP_VERSION"
        | b"PHP_SAPI"
        | b"PHP_SYSCONFDIR"
        | b"ICONV_IMPL"
        | b"LIBXML_DOTTED_VERSION"
        | b"PCRE_VERSION" => Some(get_non_empty_string()),
        b"STDIN" | b"STDOUT" | b"STDERR" => Some(get_open_resource()),
        b"NAN" | b"PHP_FLOAT_EPSILON" | b"INF" => Some(get_float()),
        b"PHP_VERSION_ID" => Some(get_positive_int()),
        b"PHP_RELEASE_VERSION" | b"PHP_MINOR_VERSION" => Some(get_non_negative_int()),
        b"PHP_MAJOR_VERSION" => Some(TUnion::from_single(Cow::Borrowed(PHP_MAJOR_VERSION_ATOMIC))),
        b"PHP_ZTS" => Some(TUnion::from_single(Cow::Borrowed(PHP_ZTS_ATOMIC))),
        b"PHP_DEBUG" => Some(TUnion::from_single(Cow::Borrowed(PHP_DEBUG_ATOMIC))),
        b"PHP_INT_SIZE" => Some(TUnion::from_single(Cow::Borrowed(PHP_INT_SIZE_ATOMIC))),
        b"PHP_WINDOWS_VERSION_MAJOR" => Some(TUnion::from_single(Cow::Borrowed(PHP_WINDOWS_VERSION_MAJOR_ATOMIC))),
        b"DIRECTORY_SEPARATOR" => Some(TUnion::new(Cow::Borrowed(DIR_SEPARATOR_SLICE.as_slice()))),
        b"PHP_INT_MAX" => Some(TUnion::new(Cow::Borrowed(PHP_INT_MAX_SLICE))),
        b"PHP_INT_MIN" => Some(TUnion::new(Cow::Borrowed(PHP_INT_MIN_SLICE))),
        b"PHP_WINDOWS_VERSION_MINOR" => Some(TUnion::new(Cow::Borrowed(PHP_WINDOWS_VERSION_MINOR_SLICE))),
        _ => None,
    }
}

#[inline]
pub(super) fn infer<'arena, A>(
    context: &Context<'_, 'arena, A>,
    scope: &NamespaceScope,
    expression: &'arena Expression<'arena>,
    enclosing_class: Option<Word>,
) -> Option<TUnion>
where
    A: Arena,
{
    infer_with_constants(context, scope, expression, enclosing_class, None)
}

#[inline]
pub(super) fn infer_with_constants<'arena, A>(
    context: &Context<'_, 'arena, A>,
    scope: &NamespaceScope,
    expression: &'arena Expression<'arena>,
    enclosing_class: Option<Word>,
    constants: Option<&WordMap<ConstantMetadata>>,
) -> Option<TUnion>
where
    A: Arena,
{
    match expression {
        Expression::MagicConstant(magic_constant) => Some(match magic_constant {
            MagicConstant::Line(_) => {
                get_literal_int(i64::from(context.file.line_number(magic_constant.start_position().offset())) + 1)
            }
            MagicConstant::File(_) => {
                if let Some(path) = context.file.path.as_deref().and_then(|p| p.to_str()) {
                    get_literal_string(word(path))
                } else {
                    get_non_empty_string()
                }
            }
            MagicConstant::Directory(_) => {
                if let Some(path) = context.file.path.as_deref().and_then(|p| p.parent()).and_then(|p| p.to_str()) {
                    get_literal_string(word(path))
                } else {
                    get_non_empty_string()
                }
            }
            MagicConstant::Namespace(_) => {
                if let Some(namespace_name) = scope.namespace_name() {
                    get_literal_string(word(namespace_name))
                } else {
                    get_empty_string()
                }
            }
            MagicConstant::Trait(_) => get_string(),
            MagicConstant::Class(_) => get_string(),
            MagicConstant::Function(_) | MagicConstant::Method(_) => get_string(),
            MagicConstant::Property(_) => get_string(),
        }),
        Expression::Literal(literal) => match literal {
            Literal::String(literal_string) => {
                Some(match literal_string.value {
                    Some(value) => {
                        if value.is_empty() {
                            get_empty_string()
                        } else if value.len() < 1000 {
                            wrap_atomic(TAtomic::Scalar(TScalar::String(TString::known_literal(word(value)))))
                        } else {
                            wrap_atomic(TAtomic::Scalar(TScalar::String(TString::unspecified_literal_with_props(
                                str_is_numeric(value),
                                true,  // truthy
                                true,  // not empty
                                false, // callable, we can't tell here.
                                if value.iter().all(|b| b.is_ascii_lowercase() || !b.is_ascii_alphabetic()) {
                                    TStringCasing::Lowercase
                                } else if value.iter().all(|b| b.is_ascii_uppercase() || !b.is_ascii_alphabetic()) {
                                    TStringCasing::Uppercase
                                } else {
                                    TStringCasing::Unspecified
                                },
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
            for part in composite_string.parts() {
                if let StringPart::Literal(literal_string_part) = part
                    && !literal_string_part.value.is_empty()
                {
                    contains_content = true;
                    break;
                }
            }

            if contains_content { Some(get_non_empty_string()) } else { Some(get_string()) }
        }
        Expression::UnaryPrefix(UnaryPrefix { operator, operand }) => {
            let operand_type = infer_with_constants(context, scope, operand, enclosing_class, constants)?;

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
                        get_literal_int(operand_value.wrapping_neg())
                    } else if let Some(operand_value) = operand_type.get_single_literal_float_value() {
                        TUnion::from_single(Cow::Owned(TAtomic::Scalar(TScalar::Float(TFloat::literal(
                            -operand_value,
                        )))))
                    } else {
                        operand_type
                    })
                }
                UnaryPrefixOperator::ArrayCast(_, _) => Some(get_mixed_keyed_array()),
                UnaryPrefixOperator::BoolCast(_, _) => Some(get_bool()),
                UnaryPrefixOperator::BooleanCast(_, _) => Some(get_bool()),
                UnaryPrefixOperator::DoubleCast(_, _) => Some(get_float()),
                UnaryPrefixOperator::RealCast(_, _) => Some(get_float()),
                UnaryPrefixOperator::FloatCast(_, _) => Some(get_float()),
                UnaryPrefixOperator::IntCast(_, _) => Some(get_int()),
                UnaryPrefixOperator::IntegerCast(_, _) => Some(get_int()),
                UnaryPrefixOperator::ObjectCast(_, _) => Some(get_object()),
                UnaryPrefixOperator::UnsetCast(_, _) => Some(get_null()),
                UnaryPrefixOperator::StringCast(_, _) => Some(get_string()),
                UnaryPrefixOperator::BinaryCast(_, _) => Some(get_string()),
                UnaryPrefixOperator::VoidCast(_, _) => Some(get_void()),
                UnaryPrefixOperator::Not(_) => Some(get_bool()),
                _ => None,
            }
        }
        Expression::Binary(Binary { operator: BinaryOperator::StringConcat(_), lhs, rhs }) => {
            let Some(lhs_type) = infer_with_constants(context, scope, lhs, enclosing_class, constants) else {
                return Some(get_string());
            };
            let Some(rhs_type) = infer_with_constants(context, scope, rhs, enclosing_class, constants) else {
                return Some(get_string());
            };

            let TAtomic::Scalar(TScalar::String(lhs_string)) = lhs_type.get_single_owned() else {
                return Some(get_string());
            };

            let TAtomic::Scalar(TScalar::String(rhs_string)) = rhs_type.get_single_owned() else {
                return Some(get_string());
            };

            if let (Some(left_val), Some(right_val)) =
                (lhs_string.get_known_literal_value(), rhs_string.get_known_literal_value())
            {
                return Some(wrap_atomic(TAtomic::Scalar(TScalar::String(TString::known_literal(concat_word!(
                    left_val, right_val
                ))))));
            }

            let is_non_empty = lhs_string.is_non_empty() || rhs_string.is_non_empty();
            let is_truthy = lhs_string.is_truthy() || rhs_string.is_truthy();
            let is_literal_origin = lhs_string.is_literal_origin() && rhs_string.is_literal_origin();
            let casing = match (lhs_string.casing, rhs_string.casing) {
                (TStringCasing::Lowercase, TStringCasing::Lowercase) => TStringCasing::Lowercase,
                (TStringCasing::Uppercase, TStringCasing::Uppercase) => TStringCasing::Uppercase,
                _ => TStringCasing::Unspecified,
            };

            let final_string_type = if is_literal_origin {
                TString::unspecified_literal_with_props(false, is_truthy, is_non_empty, false, casing)
            } else {
                TString::general_with_props(false, is_truthy, is_non_empty, false, casing)
            };

            Some(wrap_atomic(TAtomic::Scalar(TScalar::String(final_string_type))))
        }
        Expression::Binary(Binary { operator, lhs, rhs }) if operator.is_bitwise() => {
            let lhs = infer_with_constants(context, scope, lhs, enclosing_class, constants);
            let rhs = infer_with_constants(context, scope, rhs, enclosing_class, constants);

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
                            BinaryOperator::LeftShift(_) => {
                                if rhs < 0 {
                                    return Some(get_int());
                                }

                                u32::try_from(rhs).ok().and_then(|s| lhs.checked_shl(s)).unwrap_or_default()
                            }
                            BinaryOperator::RightShift(_) => {
                                if rhs < 0 {
                                    return Some(get_int());
                                }

                                match u32::try_from(rhs).ok().and_then(|s| lhs.checked_shr(s)) {
                                    Some(v) => v,
                                    None => {
                                        if lhs >= 0 {
                                            0
                                        } else {
                                            -1
                                        }
                                    }
                                }
                            }
                            #[allow(clippy::unreachable)]
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
        Expression::Binary(Binary { operator, lhs, rhs }) if operator.is_arithmetic() => {
            let lhs = infer_with_constants(context, scope, lhs, enclosing_class, constants);
            let rhs = infer_with_constants(context, scope, rhs, enclosing_class, constants);

            match (
                lhs.and_then(|v| v.get_single_literal_int_value()),
                rhs.and_then(|v| v.get_single_literal_int_value()),
            ) {
                (Some(lhs_val), Some(rhs_val)) => {
                    let result = match operator {
                        BinaryOperator::Addition(_) => lhs_val.checked_add(rhs_val),
                        BinaryOperator::Subtraction(_) => lhs_val.checked_sub(rhs_val),
                        BinaryOperator::Multiplication(_) => lhs_val.checked_mul(rhs_val),
                        #[allow(clippy::modulo_arithmetic)]
                        BinaryOperator::Modulo(_) if rhs_val != 0 => Some(lhs_val % rhs_val),
                        BinaryOperator::Exponentiation(_) if rhs_val >= 0 => lhs_val.checked_pow(rhs_val as u32),
                        BinaryOperator::Division(_) if rhs_val != 0 && lhs_val % rhs_val == 0 => {
                            Some(lhs_val / rhs_val)
                        }
                        _ => None,
                    };

                    match result {
                        Some(v) => Some(get_literal_int(v)),
                        None => Some(get_int_or_float()),
                    }
                }
                // Can't compute - return int|float
                _ => Some(get_int_or_float()),
            }
        }
        Expression::Construct(construct) => match construct {
            Construct::Isset(_) => Some(get_bool()),
            Construct::Empty(_) => Some(get_bool()),
            Construct::Print(_) => Some(get_literal_int(1)),
            _ => None,
        },
        Expression::ConstantAccess(access) => infer_constant(context.resolved_names, &access.name, constants),
        Expression::Access(Access::ClassConstant(ClassConstantAccess {
            class,
            constant: ClassLikeConstantSelector::Identifier(identifier),
            ..
        })) => {
            let class_name_str: &[u8] = if let Expression::Identifier(identifier) = class {
                context.resolved_names.get(identifier)
            } else if matches!(class, Expression::Self_(_) | Expression::Static(_)) {
                enclosing_class.as_ref().map(Word::as_bytes)?
            } else {
                return None;
            };

            Some(wrap_atomic(if identifier.value.eq_ignore_ascii_case(b"class") {
                TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::literal(word(class_name_str))))
            } else if class_name_str.eq_ignore_ascii_case(b"Attribute") {
                let bits = match identifier.value {
                    b"TARGET_CLASS" => Some(AttributeFlags::TARGET_CLASS.bits()),
                    b"TARGET_FUNCTION" => Some(AttributeFlags::TARGET_FUNCTION.bits()),
                    b"TARGET_METHOD" => Some(AttributeFlags::TARGET_METHOD.bits()),
                    b"TARGET_PROPERTY" => Some(AttributeFlags::TARGET_PROPERTY.bits()),
                    b"TARGET_CLASS_CONSTANT" => Some(AttributeFlags::TARGET_CLASS_CONSTANT.bits()),
                    b"TARGET_PARAMETER" => Some(AttributeFlags::TARGET_PARAMETER.bits()),
                    b"TARGET_CONSTANT" => Some(AttributeFlags::TARGET_CONSTANT.bits()),
                    b"TARGET_ALL" => Some(AttributeFlags::TARGET_ALL.bits()),
                    b"IS_REPEATABLE" => Some(AttributeFlags::IS_REPEATABLE.bits()),
                    _ => None,
                };

                match bits {
                    Some(bits) => return Some(get_literal_int(i64::from(bits))),
                    None => TAtomic::Reference(TReference::Member {
                        class_like_name: word(class_name_str),
                        member_selector: TReferenceMemberSelector::Identifier(word(identifier.value)),
                    }),
                }
            } else {
                TAtomic::Reference(TReference::Member {
                    class_like_name: word(class_name_str),
                    member_selector: TReferenceMemberSelector::Identifier(word(identifier.value)),
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

                let value_type = infer_with_constants(context, scope, element.value, enclosing_class, constants)
                    .unwrap_or_else(get_mixed);

                entries.insert(i, (false, value_type));
            }

            Some(wrap_atomic(TAtomic::Array(TArray::List(TList {
                known_count: Some(entries.len()),
                known_elements: Some(entries),
                element_type: Arc::new(get_never()),
                non_empty: !elements.is_empty(),
            }))))
        }
        Expression::Array(Array { elements, .. }) | Expression::LegacyArray(LegacyArray { elements, .. })
            if is_keyed_array_expression(expression) =>
        {
            let mut known_items = BTreeMap::new();
            let mut unknown_key_values = Vec::new();
            for element in elements {
                let ArrayElement::KeyValue(element) = element else {
                    return None;
                };

                let value_type = infer_with_constants(context, scope, element.value, enclosing_class, constants)
                    .unwrap_or_else(get_mixed);

                let Some(key_type) = infer_with_constants(context, scope, element.key, enclosing_class, constants)
                    .and_then(|v| v.get_single_array_key())
                else {
                    unknown_key_values.push(value_type);
                    continue;
                };

                known_items.insert(key_type, (false, value_type));

                if known_items.len() > 100 {
                    return None;
                }
            }

            let mut keyed_array = TKeyedArray::new();
            keyed_array.non_empty = !known_items.is_empty();
            if !known_items.is_empty() {
                keyed_array.known_items = Some(known_items);
            }

            if !unknown_key_values.is_empty() {
                let mut value_parameter_types = vec![];
                for value_type in unknown_key_values {
                    value_parameter_types.extend(value_type.types.into_owned());
                }

                keyed_array.parameters =
                    Some((Arc::new(get_arraykey()), Arc::new(TUnion::from_vec(value_parameter_types))))
            }

            Some(TUnion::from_single(Cow::Owned(TAtomic::Array(TArray::Keyed(keyed_array)))))
        }
        Expression::Closure(closure) => Some(wrap_atomic(TAtomic::Callable(TCallable::Alias(
            FunctionLikeIdentifier::for_closure(context.file, closure.span()),
        )))),
        Expression::ArrowFunction(arrow_func) => Some(wrap_atomic(TAtomic::Callable(TCallable::Alias(
            FunctionLikeIdentifier::for_closure(context.file, arrow_func.span()),
        )))),
        _ => None,
    }
}

#[inline]
fn infer_constant<'ctx, 'arena>(
    names: &'ctx ResolvedNames<'arena>,
    constant: &'ctx Identifier<'arena>,
    constants_map: Option<&WordMap<ConstantMetadata>>,
) -> Option<TUnion> {
    let (short_name, fqn) = if names.is_imported(constant) {
        (names.get(constant), names.get(constant))
    } else if let Some(stripped) = constant.value().strip_prefix(b"\\") {
        (stripped, names.get(constant))
    } else {
        (constant.value(), names.get(constant))
    };

    if let Some(t) = get_literal_constant_type(short_name) {
        return Some(t);
    }

    if let Some(t) = get_platform_constant_type(short_name) {
        return Some(t);
    }

    if let Some(constants) = constants_map {
        let normalized_name = ascii_lowercase_constant_name_word(fqn);

        if let Some(constant_metadata) = constants.get(&normalized_name)
            && let Some(inferred_type) = &constant_metadata.inferred_type
        {
            return Some(inferred_type.clone());
        }
    }

    None
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
