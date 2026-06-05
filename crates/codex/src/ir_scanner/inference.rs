use std::collections::BTreeMap;
use std::sync::Arc;

use mago_database::file::File;
use mago_hir::ir::expression::Access;
use mago_hir::ir::expression::ArrayElement;
use mago_hir::ir::expression::Binary;
use mago_hir::ir::expression::CompositeStringPart;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::expression::MagicConstant;
use mago_hir::ir::expression::UnaryPrefix;
use mago_hir::ir::expression::definition::DefinitionExpressionKind;
use mago_hir::ir::expression::operator::BinaryOperator;
use mago_hir::ir::expression::operator::UnaryPrefixOperator;
use mago_hir::ir::expression::selector::ConstantSelector;
use mago_hir::ir::identifier::Identifier;
use mago_hir::ir::literal::Literal;
use mago_hir::ir::literal::LiteralKind;
use mago_word::Word;
use mago_word::WordMap;
use mago_word::ascii_lowercase_constant_name_word;
use mago_word::concat_word;
use mago_word::word;

use crate::identifier::function_like::FunctionLikeIdentifier;
use crate::metadata::constant::ConstantMetadata;
use crate::scanner::inference::get_literal_constant_type;
use crate::scanner::inference::get_platform_constant_type;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::array::keyed::TKeyedArray;
use crate::ttype::atomic::array::list::TList;
use crate::ttype::atomic::callable::TCallable;
use crate::ttype::atomic::reference::TReference;
use crate::ttype::atomic::reference::TReferenceMemberSelector;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeString;
use crate::ttype::atomic::scalar::string::TString;
use crate::ttype::atomic::scalar::string::TStringCasing;
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
use crate::ttype::get_null;
use crate::ttype::get_object;
use crate::ttype::get_string;
use crate::ttype::get_true;
use crate::ttype::get_void;
use crate::ttype::union::TUnion;
use crate::ttype::wrap_atomic;
use crate::utils::str_is_numeric;

#[must_use]
pub fn infer(
    expression: &Expression<'_, (), (), ()>,
    classname: Option<Word>,
    file: &File,
    constants: &WordMap<ConstantMetadata>,
) -> Option<TUnion> {
    match &expression.kind {
        ExpressionKind::Literal(literal) => Some(infer_literal(literal)),
        ExpressionKind::CompositeString(parts) => Some(infer_composite_string(parts)),
        ExpressionKind::UnaryPrefix(unary) => infer_unary(unary, classname, file, constants),
        ExpressionKind::Binary(binary) => infer_binary(binary, classname, file, constants),
        ExpressionKind::Empty(_) | ExpressionKind::Isset(_) => Some(get_bool()),
        ExpressionKind::Print(_) => Some(get_literal_int(1)),
        ExpressionKind::Constant(identifier) => infer_constant(identifier, constants),
        ExpressionKind::MagicConstant(magic) => Some(infer_magic_constant(*magic, expression.span.start.offset, file)),
        ExpressionKind::Array(elements) | ExpressionKind::List(elements) => {
            infer_array(elements, classname, file, constants)
        }
        ExpressionKind::Access(access) => infer_access(access, classname),
        ExpressionKind::Definition(definition) => match &definition.kind {
            DefinitionExpressionKind::Closure(_) | DefinitionExpressionKind::ArrowFunction(_) => {
                let name = crate::build_synthetic_name("closure", file, expression.span);

                Some(wrap_atomic(TAtomic::Callable(TCallable::Alias(FunctionLikeIdentifier::Closure(name)))))
            }
            _ => None,
        },
        _ => None,
    }
}

fn infer_access(access: &Access<'_, (), (), ()>, classname: Option<Word>) -> Option<TUnion> {
    let Access::ClassConstant(class, ConstantSelector::Name(constant)) = access else {
        return None;
    };

    let accessed_class = match &class.kind {
        ExpressionKind::Identifier(identifier) => word(identifier.value),
        ExpressionKind::Self_ | ExpressionKind::Static => classname?,
        ExpressionKind::Parent => word("parent"),
        _ => return None,
    };

    if constant.value.eq_ignore_ascii_case(b"class") {
        return Some(wrap_atomic(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::literal(accessed_class)))));
    }

    Some(wrap_atomic(TAtomic::Reference(TReference::Member {
        class_like_name: accessed_class,
        member_selector: TReferenceMemberSelector::Identifier(word(constant.value)),
    })))
}

fn infer_constant(identifier: &Identifier<'_>, constants: &WordMap<ConstantMetadata>) -> Option<TUnion> {
    let short = identifier.last_segment();
    if let Some(union) = get_literal_constant_type(short) {
        return Some(union);
    }

    if let Some(union) = get_platform_constant_type(short) {
        return Some(union);
    }

    constants
        .get(&ascii_lowercase_constant_name_word(identifier.value))
        .and_then(|constant| constant.inferred_type.clone())
}

fn infer_magic_constant(magic: MagicConstant, offset: u32, file: &File) -> TUnion {
    match magic {
        MagicConstant::Line => get_literal_int(i64::from(file.line_number(offset)) + 1),
        MagicConstant::File => match file.path.as_deref().and_then(|path| path.to_str()) {
            Some(path) => get_literal_string(word(path)),
            None => get_non_empty_string(),
        },
        MagicConstant::Directory => {
            match file.path.as_deref().and_then(std::path::Path::parent).and_then(|path| path.to_str()) {
                Some(path) => get_literal_string(word(path)),
                None => get_non_empty_string(),
            }
        }
        MagicConstant::Namespace
        | MagicConstant::Trait
        | MagicConstant::Class
        | MagicConstant::Function
        | MagicConstant::Method
        | MagicConstant::Property => get_string(),
    }
}

fn infer_composite_string(parts: &[CompositeStringPart<'_, (), (), ()>]) -> TUnion {
    let contains_content =
        parts.iter().any(|part| matches!(part, CompositeStringPart::Literal(value) if !value.is_empty()));

    if contains_content { get_non_empty_string() } else { get_string() }
}

fn infer_unary(
    unary: &UnaryPrefix<'_, (), (), ()>,
    classname: Option<Word>,
    file: &File,
    constants: &WordMap<ConstantMetadata>,
) -> Option<TUnion> {
    let operand = infer(unary.operand, classname, file, constants)?;

    Some(match unary.operator {
        UnaryPrefixOperator::Plus => {
            if let Some(value) = operand.get_single_literal_int_value() {
                get_literal_int(value)
            } else {
                operand
            }
        }
        UnaryPrefixOperator::Negation => {
            if let Some(value) = operand.get_single_literal_int_value() {
                get_literal_int(value.wrapping_neg())
            } else {
                operand
            }
        }
        UnaryPrefixOperator::ArrayCast => get_mixed_keyed_array(),
        UnaryPrefixOperator::BoolCast => get_bool(),
        UnaryPrefixOperator::FloatCast => get_float(),
        UnaryPrefixOperator::IntCast => get_int(),
        UnaryPrefixOperator::ObjectCast => get_object(),
        UnaryPrefixOperator::UnsetCast => get_null(),
        UnaryPrefixOperator::StringCast => get_string(),
        UnaryPrefixOperator::VoidCast => get_void(),
        UnaryPrefixOperator::Not => get_bool(),
        _ => return None,
    })
}

fn infer_string_concat(
    binary: &Binary<'_, (), (), ()>,
    classname: Option<Word>,
    file: &File,
    constants: &WordMap<ConstantMetadata>,
) -> TUnion {
    let Some(left_type) = infer(binary.left, classname, file, constants) else {
        return get_string();
    };
    let Some(right_type) = infer(binary.right, classname, file, constants) else {
        return get_string();
    };

    let TAtomic::Scalar(TScalar::String(left_string)) = left_type.get_single_owned() else {
        return get_string();
    };
    let TAtomic::Scalar(TScalar::String(right_string)) = right_type.get_single_owned() else {
        return get_string();
    };

    if let (Some(left_value), Some(right_value)) =
        (left_string.get_known_literal_value(), right_string.get_known_literal_value())
    {
        return wrap_atomic(TAtomic::Scalar(TScalar::String(TString::known_literal(concat_word!(
            left_value,
            right_value
        )))));
    }

    let is_non_empty = left_string.is_non_empty() || right_string.is_non_empty();
    let is_truthy = left_string.is_truthy() || right_string.is_truthy();
    let is_literal_origin = left_string.is_literal_origin() && right_string.is_literal_origin();
    let casing = match (left_string.casing, right_string.casing) {
        (TStringCasing::Lowercase, TStringCasing::Lowercase) => TStringCasing::Lowercase,
        (TStringCasing::Uppercase, TStringCasing::Uppercase) => TStringCasing::Uppercase,
        _ => TStringCasing::Unspecified,
    };

    let string_type = if is_literal_origin {
        TString::unspecified_literal_with_props(false, is_truthy, is_non_empty, false, casing)
    } else {
        TString::general_with_props(false, is_truthy, is_non_empty, false, casing)
    };

    wrap_atomic(TAtomic::Scalar(TScalar::String(string_type)))
}

fn infer_binary(
    binary: &Binary<'_, (), (), ()>,
    classname: Option<Word>,
    file: &File,
    constants: &WordMap<ConstantMetadata>,
) -> Option<TUnion> {
    if binary.operator == BinaryOperator::StringConcat {
        return Some(infer_string_concat(binary, classname, file, constants));
    }

    let left = infer(binary.left, classname, file, constants).and_then(|union| union.get_single_literal_int_value());
    let right = infer(binary.right, classname, file, constants).and_then(|union| union.get_single_literal_int_value());

    if is_bitwise(binary.operator) {
        return Some(match (left, right) {
            (Some(left), Some(right)) => {
                let value = match binary.operator {
                    BinaryOperator::BitwiseAnd => left & right,
                    BinaryOperator::BitwiseOr => left | right,
                    BinaryOperator::BitwiseXor => left ^ right,
                    BinaryOperator::LeftShift => {
                        if right < 0 {
                            return Some(get_int());
                        }
                        u32::try_from(right).ok().and_then(|shift| left.checked_shl(shift)).unwrap_or_default()
                    }
                    BinaryOperator::RightShift => {
                        if right < 0 {
                            return Some(get_int());
                        }
                        match u32::try_from(right).ok().and_then(|shift| left.checked_shr(shift)) {
                            Some(value) => value,
                            None if left >= 0 => 0,
                            None => -1,
                        }
                    }
                    _ => return Some(get_int()),
                };

                wrap_atomic(TAtomic::Scalar(TScalar::literal_int(value)))
            }
            _ => wrap_atomic(TAtomic::Scalar(TScalar::int())),
        });
    }

    if is_arithmetic(binary.operator) {
        return Some(match (left, right) {
            (Some(left), Some(right)) => {
                let result = match binary.operator {
                    BinaryOperator::Addition => left.checked_add(right),
                    BinaryOperator::Subtraction => left.checked_sub(right),
                    BinaryOperator::Multiplication => left.checked_mul(right),
                    BinaryOperator::Modulo => left.checked_rem(right),
                    BinaryOperator::Exponentiation if right >= 0 => {
                        u32::try_from(right).ok().and_then(|exponent| left.checked_pow(exponent))
                    }
                    BinaryOperator::Division if left.checked_rem(right) == Some(0) => left.checked_div(right),
                    _ => None,
                };

                match result {
                    Some(value) => get_literal_int(value),
                    None => get_int_or_float(),
                }
            }
            _ => get_int_or_float(),
        });
    }

    None
}

const fn is_bitwise(operator: BinaryOperator) -> bool {
    matches!(
        operator,
        BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseOr
            | BinaryOperator::BitwiseXor
            | BinaryOperator::LeftShift
            | BinaryOperator::RightShift
    )
}

const fn is_arithmetic(operator: BinaryOperator) -> bool {
    matches!(
        operator,
        BinaryOperator::Addition
            | BinaryOperator::Subtraction
            | BinaryOperator::Multiplication
            | BinaryOperator::Division
            | BinaryOperator::Modulo
            | BinaryOperator::Exponentiation
    )
}

fn infer_array(
    elements: &[ArrayElement<'_, (), (), ()>],
    classname: Option<Word>,
    file: &File,
    constants: &WordMap<ConstantMetadata>,
) -> Option<TUnion> {
    let all_values = elements.iter().all(|element| matches!(element, ArrayElement::Value(_)));
    let all_key_values = elements.iter().all(|element| matches!(element, ArrayElement::KeyValue(_, _)));

    if all_values {
        let mut entries = BTreeMap::new();
        for (index, element) in elements.iter().enumerate() {
            let ArrayElement::Value(value) = element else {
                return None;
            };
            let value_type = infer(value, classname, file, constants).unwrap_or_else(get_mixed);
            entries.insert(index, (false, value_type));
        }

        return Some(wrap_atomic(TAtomic::Array(TArray::List(TList {
            known_count: Some(entries.len()),
            known_elements: Some(entries),
            element_type: Arc::new(get_never()),
            non_empty: !elements.is_empty(),
        }))));
    }

    if all_key_values {
        let mut known_items = BTreeMap::new();
        let mut unknown_key_values = Vec::new();
        for element in elements {
            let ArrayElement::KeyValue(key, value) = element else {
                return None;
            };
            let value_type = infer(value, classname, file, constants).unwrap_or_else(get_mixed);
            let Some(key_type) = infer(key, classname, file, constants).and_then(|union| union.get_single_array_key())
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
            let mut value_parameter_types = Vec::new();
            for value_type in unknown_key_values {
                value_parameter_types.extend(value_type.types.into_owned());
            }
            keyed_array.parameters =
                Some((Arc::new(get_arraykey()), Arc::new(TUnion::from_vec(value_parameter_types))));
        }

        return Some(wrap_atomic(TAtomic::Array(TArray::Keyed(keyed_array))));
    }

    None
}

fn infer_literal(literal: &Literal<'_>) -> TUnion {
    match literal.kind {
        LiteralKind::Integer(integer) => match integer.value {
            Some(value) => get_literal_int(value as i64),
            None => get_int_or_float(),
        },
        LiteralKind::Float(_) => get_float(),
        LiteralKind::String(string) => match string.value {
            Some(value) => infer_literal_string(value),
            None => get_string(),
        },
        LiteralKind::True => get_true(),
        LiteralKind::False => get_false(),
        LiteralKind::Null => get_null(),
    }
}

fn infer_literal_string(value: &[u8]) -> TUnion {
    if value.is_empty() {
        return get_empty_string();
    }

    if value.len() < 1000 {
        return wrap_atomic(TAtomic::Scalar(TScalar::String(TString::known_literal(word(value)))));
    }

    let casing = if value.iter().all(|byte| byte.is_ascii_lowercase() || !byte.is_ascii_alphabetic()) {
        TStringCasing::Lowercase
    } else if value.iter().all(|byte| byte.is_ascii_uppercase() || !byte.is_ascii_alphabetic()) {
        TStringCasing::Uppercase
    } else {
        TStringCasing::Unspecified
    };

    wrap_atomic(TAtomic::Scalar(TScalar::String(TString::unspecified_literal_with_props(
        str_is_numeric(value),
        true,
        true,
        false,
        casing,
    ))))
}
