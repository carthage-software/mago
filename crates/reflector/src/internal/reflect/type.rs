use fennec_ast::*;
use fennec_reflection::class_like::ClassLikeReflection;
use fennec_reflection::r#type::kind::*;
use fennec_reflection::r#type::*;
use fennec_span::*;

use crate::internal::context::Context;

pub fn maybe_reflect_hint<'i, 'ast>(
    hint: &'ast Option<Hint>,
    context: &'ast mut Context<'i>,
    scope: Option<&ClassLikeReflection>,
) -> Option<TypeReflection> {
    let Some(hint) = hint else {
        return None;
    };

    Some(TypeReflection { kind: build_kind(hint, context, scope), inferred: false, span: hint.span() })
}

pub fn reflect_hint<'i, 'ast>(
    hint: &'ast Hint,
    context: &'ast mut Context<'i>,
    scope: Option<&ClassLikeReflection>,
) -> TypeReflection {
    TypeReflection { kind: build_kind(hint, context, scope), inferred: false, span: hint.span() }
}

fn build_kind<'i, 'ast>(
    hint: &'ast Hint,
    context: &'ast mut Context<'i>,
    scope: Option<&ClassLikeReflection>,
) -> TypeKind {
    match &hint {
        Hint::Identifier(identifier) => named_object_kind(context.semantics.names.get(identifier), vec![]),
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

            match build_kind(&union_hint.left.as_ref(), context, scope) {
                TypeKind::Union { kinds: left_kinds } => kinds.extend(left_kinds),
                kind => {
                    kinds.push(kind);
                }
            }

            match build_kind(&union_hint.right.as_ref(), context, scope) {
                TypeKind::Union { kinds: right_kinds } => kinds.extend(right_kinds),
                kind => {
                    kinds.push(kind);
                }
            }

            union_kind(kinds)
        }
        Hint::Intersection(intersection_hint) => {
            let mut kinds = vec![];

            match build_kind(&intersection_hint.left.as_ref(), context, scope) {
                TypeKind::Intersection { kinds: left_kinds } => kinds.extend(left_kinds),
                kind => {
                    kinds.push(kind);
                }
            }

            match build_kind(&intersection_hint.right.as_ref(), context, scope) {
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
        Hint::Array(_) => array_kind(array_key_kind(), mixed_kind(), None),
        Hint::Callable(_) => callable_kind(vec![callable_parameter(mixed_kind(), false, true)], mixed_kind()),
        Hint::Void(_) => void_kind(),
        Hint::Never(_) => never_kind(),
        Hint::Float(_) => float_kind(),
        Hint::Bool(_) => bool_kind(),
        Hint::Integer(_) => integer_kind(),
        Hint::String(_) => string_kind(),
        Hint::Object(_) => any_object_kind(),
        Hint::Mixed(_) => mixed_kind(),
        Hint::Iterable(_) => iterable_kind(mixed_kind(), mixed_kind()),
        Hint::Static(_) => scope.map_or_else(|| mixed_kind(), |scope| static_kind(scope.name)),
        Hint::Self_(_) => scope.map_or_else(|| mixed_kind(), |scope| self_kind(scope.name)),
        Hint::Parent(_) => scope.map_or_else(|| mixed_kind(), |scope| parent_kind(scope.name)),
    }
}
