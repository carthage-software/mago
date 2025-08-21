use mago_span::*;
use mago_syntax::ast::*;

use crate::internal::consts::*;
use crate::internal::context::Context;

pub mod access;
pub mod argument;
pub mod array;
pub mod assignment;
pub mod attribute;
pub mod call;
pub mod class_like;
pub mod closure_creation;
pub mod constant;
pub mod control_flow;
pub mod expression;
pub mod function_like;
pub mod hint;
pub mod literal;
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
const MAGIC_METHOD_SEMANTICS: &[(&str, Option<usize>, bool, bool, bool)] = &[
    (CONSTRUCTOR_MAGIC_METHOD, None, false, false, false),
    (DESTRUCTOR_MAGIC_METHOD, None, false, false, false),
    (CLONE_MAGIC_METHOD, None, false, false, true),
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
            let symbol = context.get_name(&identifier.span().start);

            "generator".eq_ignore_ascii_case(symbol)
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
