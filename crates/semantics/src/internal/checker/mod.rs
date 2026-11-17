use mago_span::HasSpan;
use mago_syntax::cst::Block;
use mago_syntax::cst::Hint;

use crate::internal::consts::CALL_MAGIC_METHOD;
use crate::internal::consts::CALL_STATIC_MAGIC_METHOD;
use crate::internal::consts::CLONE_MAGIC_METHOD;
use crate::internal::consts::CONSTRUCTOR_MAGIC_METHOD;
use crate::internal::consts::DEBUG_INFO_MAGIC_METHOD;
use crate::internal::consts::DESTRUCTOR_MAGIC_METHOD;
use crate::internal::consts::GET_MAGIC_METHOD;
use crate::internal::consts::INVOKE_MAGIC_METHOD;
use crate::internal::consts::ISSET_MAGIC_METHOD;
use crate::internal::consts::SERIALIZE_MAGIC_METHOD;
use crate::internal::consts::SET_MAGIC_METHOD;
use crate::internal::consts::SET_STATE_MAGIC_METHOD;
use crate::internal::consts::SLEEP_MAGIC_METHOD;
use crate::internal::consts::TO_STRING_MAGIC_METHOD;
use crate::internal::consts::UNSERIALIZE_MAGIC_METHOD;
use crate::internal::consts::UNSET_MAGIC_METHOD;
use crate::internal::consts::WAKEUP_MAGIC_METHOD;
use crate::internal::context::Context;

pub mod access;
pub mod argument;
pub mod array;
pub mod assignment;
pub mod attribute;
pub mod call;
pub mod class_like;
pub mod constant;
pub mod control_flow;
pub mod expression;
pub mod function_like;
pub mod hint;
pub mod literal;
pub mod partial_application;
pub mod pipe;
pub mod statement;
pub mod r#try;

/// Defines the semantics of magic methods.
///
/// The tuple contains the following elements:
///
/// 1. The name of the magic method.
/// 2. The number of arguments the magic method accepts, or none if it can accept any number of arguments.
/// 3. Whether the magic method has to be public.
/// 4. Whether the magic method has to be static.
/// 5. Whether the magic method can contain a return type.
type MagicMethodSemantic = (&'static [u8], Option<usize>, bool, bool, bool);

const MAGIC_METHOD_SEMANTICS: &[MagicMethodSemantic] = &[
    (CONSTRUCTOR_MAGIC_METHOD, None, false, false, false),
    (DESTRUCTOR_MAGIC_METHOD, Some(0), false, false, false),
    (CLONE_MAGIC_METHOD, Some(0), false, false, true),
    (CALL_MAGIC_METHOD, Some(2), true, false, true),
    (CALL_STATIC_MAGIC_METHOD, Some(2), true, true, true),
    (GET_MAGIC_METHOD, Some(1), true, false, true),
    (SET_MAGIC_METHOD, Some(2), true, false, true),
    (ISSET_MAGIC_METHOD, Some(1), true, false, true),
    (UNSET_MAGIC_METHOD, Some(1), true, false, true),
    (SLEEP_MAGIC_METHOD, Some(0), true, false, true),
    (WAKEUP_MAGIC_METHOD, Some(0), true, false, true),
    (SERIALIZE_MAGIC_METHOD, Some(0), true, false, true),
    (UNSERIALIZE_MAGIC_METHOD, Some(1), true, false, true),
    (TO_STRING_MAGIC_METHOD, Some(0), true, false, true),
    (INVOKE_MAGIC_METHOD, None, true, false, true),
    (SET_STATE_MAGIC_METHOD, Some(1), true, true, true),
    (DEBUG_INFO_MAGIC_METHOD, Some(0), true, false, true),
];

const MAY_BE_NULL: u16 = 1 << 0;
const MAY_BE_FALSE: u16 = 1 << 1;
const MAY_BE_TRUE: u16 = 1 << 2;
const MAY_BE_LONG: u16 = 1 << 3;
const MAY_BE_DOUBLE: u16 = 1 << 4;
const MAY_BE_STRING: u16 = 1 << 5;
const MAY_BE_ARRAY: u16 = 1 << 6;
const MAY_BE_OBJECT: u16 = 1 << 7;
const MAY_BE_CALLABLE: u16 = 1 << 8;
const MAY_BE_VOID: u16 = 1 << 9;
const MAY_BE_STATIC: u16 = 1 << 10;
const MAY_BE_NEVER: u16 = 1 << 11;
const MAY_BE_BOOL: u16 = MAY_BE_FALSE | MAY_BE_TRUE;
const MAY_BE_ANY: u16 =
    MAY_BE_NULL | MAY_BE_BOOL | MAY_BE_LONG | MAY_BE_DOUBLE | MAY_BE_STRING | MAY_BE_ARRAY | MAY_BE_OBJECT;

type MagicMethodType = (u16, &'static str);

const VOID_TYPE: MagicMethodType = (MAY_BE_VOID, "void");
const BOOL_TYPE: MagicMethodType = (MAY_BE_BOOL, "bool");
const STRING_TYPE: MagicMethodType = (MAY_BE_STRING, "string");
const ARRAY_TYPE: MagicMethodType = (MAY_BE_ARRAY, "array");
const NULLABLE_ARRAY_TYPE: MagicMethodType = (MAY_BE_ARRAY | MAY_BE_NULL, "?array");
const OBJECT_TYPE: MagicMethodType = (MAY_BE_OBJECT, "object");

/// Defines the types of magic methods, when the types are declared.
///
/// The tuple contains the following elements:
///
/// 1. The name of the magic method.
/// 2. The type that each of the first parameters must accept.
/// 3. The type that the return type must be part of, or none if any return type is allowed.
type MagicMethodTypes = (&'static [u8], &'static [MagicMethodType], Option<MagicMethodType>);

const MAGIC_METHOD_TYPES: &[MagicMethodTypes] = &[
    (CLONE_MAGIC_METHOD, &[], Some(VOID_TYPE)),
    (GET_MAGIC_METHOD, &[STRING_TYPE], None),
    (SET_MAGIC_METHOD, &[STRING_TYPE], Some(VOID_TYPE)),
    (UNSET_MAGIC_METHOD, &[STRING_TYPE], Some(VOID_TYPE)),
    (ISSET_MAGIC_METHOD, &[STRING_TYPE], Some(BOOL_TYPE)),
    (CALL_MAGIC_METHOD, &[STRING_TYPE, ARRAY_TYPE], None),
    (CALL_STATIC_MAGIC_METHOD, &[STRING_TYPE, ARRAY_TYPE], None),
    (TO_STRING_MAGIC_METHOD, &[], Some(STRING_TYPE)),
    (DEBUG_INFO_MAGIC_METHOD, &[], Some(NULLABLE_ARRAY_TYPE)),
    (SERIALIZE_MAGIC_METHOD, &[], Some(ARRAY_TYPE)),
    (UNSERIALIZE_MAGIC_METHOD, &[ARRAY_TYPE], Some(VOID_TYPE)),
    (SET_STATE_MAGIC_METHOD, &[ARRAY_TYPE], Some(OBJECT_TYPE)),
    (SLEEP_MAGIC_METHOD, &[], Some(ARRAY_TYPE)),
    (WAKEUP_MAGIC_METHOD, &[], Some(VOID_TYPE)),
];

fn hint_type_mask(hint: &Hint) -> (u16, bool) {
    match hint {
        Hint::Identifier(_) | Hint::Self_(_) | Hint::Parent(_) | Hint::Intersection(_) => (0, true),
        Hint::Iterable(_) => (MAY_BE_ARRAY, true),
        Hint::Parenthesized(parenthesized_hint) => hint_type_mask(parenthesized_hint.hint),
        Hint::Nullable(nullable_hint) => {
            let (mask, is_complex) = hint_type_mask(nullable_hint.hint);

            (mask | MAY_BE_NULL, is_complex)
        }
        Hint::Union(union_hint) => {
            let (left_mask, left_is_complex) = hint_type_mask(union_hint.left);
            let (right_mask, right_is_complex) = hint_type_mask(union_hint.right);

            (left_mask | right_mask, left_is_complex || right_is_complex)
        }
        Hint::Null(_) => (MAY_BE_NULL, false),
        Hint::True(_) => (MAY_BE_TRUE, false),
        Hint::False(_) => (MAY_BE_FALSE, false),
        Hint::Bool(_) => (MAY_BE_BOOL, false),
        Hint::Integer(_) => (MAY_BE_LONG, false),
        Hint::Float(_) => (MAY_BE_DOUBLE, false),
        Hint::String(_) => (MAY_BE_STRING, false),
        Hint::Array(_) => (MAY_BE_ARRAY, false),
        Hint::Object(_) => (MAY_BE_OBJECT, false),
        Hint::Callable(_) => (MAY_BE_CALLABLE, false),
        Hint::Mixed(_) => (MAY_BE_ANY, false),
        Hint::Void(_) => (MAY_BE_VOID, false),
        Hint::Never(_) => (MAY_BE_NEVER, false),
        Hint::Static(_) => (MAY_BE_STATIC, false),
    }
}

#[inline]
fn returns_generator<'ast>(context: &mut Context<'_, '_, '_>, block: &'ast Block, hint: &'ast Hint) -> bool {
    if hint_contains_generator(context, hint) {
        return true;
    }

    mago_syntax::utils::block_has_yield(block)
}

#[inline]
fn hint_contains_generator(context: &mut Context<'_, '_, '_>, hint: &Hint) -> bool {
    match hint {
        Hint::Identifier(identifier) => {
            let symbol = context.get_name(identifier.span().start);

            symbol.eq_ignore_ascii_case(b"generator")
        }
        Hint::Parenthesized(parenthesized_hint) => hint_contains_generator(context, parenthesized_hint.hint),
        Hint::Nullable(nullable_hint) => hint_contains_generator(context, nullable_hint.hint),
        Hint::Union(union_hint) => {
            hint_contains_generator(context, union_hint.left) || hint_contains_generator(context, union_hint.right)
        }
        Hint::Intersection(intersection_hint) => {
            hint_contains_generator(context, intersection_hint.left)
                || hint_contains_generator(context, intersection_hint.right)
        }
        _ => false,
    }
}
