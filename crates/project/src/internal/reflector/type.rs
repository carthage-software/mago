use mago_ast::*;
use mago_interner::StringIdentifier;
use mago_reflection::class_like::ClassLikeReflection;
use mago_reflection::r#type::kind::*;
use mago_reflection::r#type::*;
use mago_span::*;

use crate::internal::context::Context;

#[inline]
pub fn maybe_reflect_hint<'ast>(
    hint: &'ast Option<Hint>,
    context: &'ast mut Context<'_>,
    scope: Option<&ClassLikeReflection>,
) -> Option<TypeReflection> {
    let Some(hint) = hint else {
        return None;
    };

    Some(TypeReflection { kind: build_kind(hint, context, scope), inferred: false, span: hint.span() })
}

#[inline]
pub fn reflect_hint<'ast>(
    hint: &'ast Hint,
    context: &'ast mut Context<'_>,
    scope: Option<&ClassLikeReflection>,
) -> TypeReflection {
    TypeReflection { kind: build_kind(hint, context, scope), inferred: false, span: hint.span() }
}

#[inline]
fn build_kind<'ast>(hint: &'ast Hint, context: &'ast mut Context<'_>, scope: Option<&ClassLikeReflection>) -> TypeKind {
    match &hint {
        Hint::Identifier(identifier) => named_object_kind(*context.names.get(identifier), vec![]),
        Hint::Parenthesized(parenthesized_hint) => build_kind(parenthesized_hint.hint.as_ref(), context, scope),
        Hint::Nullable(nullable) => match build_kind(nullable.hint.as_ref(), context, scope) {
            TypeKind::Union { mut kinds } => {
                kinds.insert(0, null_kind());

                TypeKind::Union { kinds }
            }
            kind => union_kind(vec![null_kind(), kind]),
        },
        Hint::Union(union_hint) => {
            let mut kinds = vec![];

            match build_kind(union_hint.left.as_ref(), context, scope) {
                TypeKind::Union { kinds: left_kinds } => kinds.extend(left_kinds),
                kind => {
                    kinds.push(kind);
                }
            }

            match build_kind(union_hint.right.as_ref(), context, scope) {
                TypeKind::Union { kinds: right_kinds } => kinds.extend(right_kinds),
                kind => {
                    kinds.push(kind);
                }
            }

            union_kind(kinds)
        }
        Hint::Intersection(intersection_hint) => {
            let mut kinds = vec![];

            match build_kind(intersection_hint.left.as_ref(), context, scope) {
                TypeKind::Intersection { kinds: left_kinds } => kinds.extend(left_kinds),
                kind => {
                    kinds.push(kind);
                }
            }

            match build_kind(intersection_hint.right.as_ref(), context, scope) {
                TypeKind::Intersection { kinds: right_kinds } => kinds.extend(right_kinds),
                kind => {
                    kinds.push(kind);
                }
            }

            intersection_kind(kinds)
        }
        Hint::Null(_) => null_kind(),
        Hint::True(_) => true_kind(),
        Hint::False(_) => false_kind(),
        Hint::Array(_) => array_kind(array_key_kind(), mixed_kind(true), None),
        Hint::Callable(_) => any_callable_kind(),
        Hint::Void(_) => void_kind(),
        Hint::Never(_) => never_kind(),
        Hint::Float(_) => float_kind(),
        Hint::Bool(_) => bool_kind(),
        Hint::Integer(_) => integer_kind(),
        Hint::String(_) => string_kind(),
        Hint::Object(_) => any_object_kind(),
        Hint::Mixed(_) => mixed_kind(true),
        Hint::Iterable(_) => iterable_kind(mixed_kind(true), mixed_kind(true)),
        Hint::Static(_) => {
            let scope = match &scope {
                Some(scope) => context.interner.intern(scope.name.get_key(context.interner)),
                None => StringIdentifier::empty(),
            };

            static_kind(scope)
        }
        Hint::Self_(_) => {
            let scope = match &scope {
                Some(scope) => context.interner.intern(scope.name.get_key(context.interner)),
                None => StringIdentifier::empty(),
            };

            self_kind(scope)
        }
        Hint::Parent(_) => {
            let scope = match &scope {
                Some(scope) => context.interner.intern(scope.name.get_key(context.interner)),
                None => StringIdentifier::empty(),
            };

            parent_kind(scope)
        }
    }
}
