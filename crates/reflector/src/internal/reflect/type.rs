use fennec_ast::*;
use fennec_reflection::class_like::ClassLikeReflection;
use fennec_reflection::r#type::*;
use fennec_span::*;
use kind::TypeKind;

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
        Hint::Identifier(identifier) => TypeKind::Identifier(context.semantics.names.get(identifier)),
        Hint::Parenthesized(parenthesized_hint) => build_kind(parenthesized_hint.hint.as_ref(), context, scope),
        Hint::Nullable(nullable) => match build_kind(nullable.hint.as_ref(), context, scope) {
            TypeKind::Union(mut inner) => {
                inner.insert(0, TypeKind::Null);

                TypeKind::Union(inner)
            }
            kind => TypeKind::Union(vec![TypeKind::Null, kind]),
        },
        Hint::Union(union_hint) => {
            let mut kinds = vec![];

            match build_kind(&union_hint.left.as_ref(), context, scope) {
                TypeKind::Union(left_kinds) => kinds.extend(left_kinds),
                kind => {
                    kinds.push(kind);
                }
            }

            match build_kind(&union_hint.right.as_ref(), context, scope) {
                TypeKind::Union(right_kinds) => kinds.extend(right_kinds),
                kind => {
                    kinds.push(kind);
                }
            }

            TypeKind::Union(kinds)
        }
        Hint::Intersection(intersection_hint) => {
            let mut kinds = vec![];

            match build_kind(&intersection_hint.left.as_ref(), context, scope) {
                TypeKind::Intersection(left_kinds) => kinds.extend(left_kinds),
                kind => {
                    kinds.push(kind);
                }
            }

            match build_kind(&intersection_hint.right.as_ref(), context, scope) {
                TypeKind::Intersection(right_kinds) => kinds.extend(right_kinds),
                kind => {
                    kinds.push(kind);
                }
            }

            TypeKind::Intersection(kinds)
        }
        Hint::Null(_) => TypeKind::Null,
        Hint::True(_) => TypeKind::True,
        Hint::False(_) => TypeKind::False,
        Hint::Array(_) => TypeKind::Array,
        Hint::Callable(_) => TypeKind::Callable,
        Hint::Void(_) => TypeKind::Void,
        Hint::Never(_) => TypeKind::Never,
        Hint::Float(_) => TypeKind::Float,
        Hint::Bool(_) => TypeKind::Bool,
        Hint::Integer(_) => TypeKind::Integer,
        Hint::String(_) => TypeKind::String,
        Hint::Object(_) => TypeKind::Object,
        Hint::Mixed(_) => TypeKind::Mixed,
        Hint::Iterable(_) => TypeKind::Iterable,
        Hint::Static(_) => {
            if let Some(scope) = scope {
                TypeKind::Static(scope.name)
            } else {
                TypeKind::Unknown
            }
        }
        Hint::Self_(_) => {
            if let Some(scope) = scope {
                TypeKind::Self_(scope.name)
            } else {
                TypeKind::Unknown
            }
        }
        Hint::Parent(_) => {
            if let Some(scope) = scope {
                TypeKind::Parent(scope.name)
            } else {
                TypeKind::Unknown
            }
        }
    }
}
