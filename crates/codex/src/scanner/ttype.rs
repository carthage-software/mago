use mago_atom::Atom;
use mago_atom::atom;
use mago_docblock::tag::TypeString;
use mago_names::scope::NamespaceScope;
use mago_span::HasSpan;
use mago_syntax::ast::Hint;
use mago_syntax::ast::Identifier;
use mago_syntax::ast::UnionHint;

use crate::metadata::ttype::TypeMetadata;
use crate::scanner::Context;
use crate::ttype::TType;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::callable::TCallable;
use crate::ttype::atomic::callable::TCallableSignature;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::object::named::TNamedObject;
use crate::ttype::atomic::reference::TReference;
use crate::ttype::builder;
use crate::ttype::error::TypeError;
use crate::ttype::get_bool;
use crate::ttype::get_false;
use crate::ttype::get_float;
use crate::ttype::get_int;
use crate::ttype::get_mixed;
use crate::ttype::get_mixed_callable;
use crate::ttype::get_mixed_iterable;
use crate::ttype::get_mixed_keyed_array;
use crate::ttype::get_never;
use crate::ttype::get_null;
use crate::ttype::get_nullable_float;
use crate::ttype::get_nullable_int;
use crate::ttype::get_nullable_object;
use crate::ttype::get_nullable_string;
use crate::ttype::get_object;
use crate::ttype::get_string;
use crate::ttype::get_true;
use crate::ttype::get_void;
use crate::ttype::resolution::TypeResolutionContext;
use crate::ttype::union::TUnion;
use crate::ttype::wrap_atomic;

#[inline]
pub fn get_type_metadata_from_hint<'arena>(
    hint: &'arena Hint<'arena>,
    classname: Option<Atom>,
    context: &mut Context<'_, 'arena>,
) -> TypeMetadata {
    let type_union = get_union_from_hint(hint, classname, context);

    let mut type_metadata = TypeMetadata::new(type_union, hint.span());
    type_metadata.from_docblock = false;
    type_metadata
}

#[inline]
pub fn get_type_metadata_from_type_string(
    ttype: &TypeString,
    classname: Option<Atom>,
    type_context: &TypeResolutionContext,
    scope: &NamespaceScope,
) -> Result<TypeMetadata, TypeError> {
    builder::get_type_from_string(&ttype.value, ttype.span, scope, type_context, classname).map(|type_union| {
        let mut type_metadata = TypeMetadata::new(type_union, ttype.span);
        type_metadata.from_docblock = true;
        type_metadata
    })
}

#[inline]
fn get_union_from_hint<'arena>(
    hint: &'arena Hint<'arena>,
    classname: Option<Atom>,
    context: &mut Context<'_, 'arena>,
) -> TUnion {
    match hint {
        Hint::Parenthesized(parenthesized_hint) => get_union_from_hint(parenthesized_hint.hint, classname, context),
        Hint::Identifier(identifier) => get_union_from_identifier_hint(identifier, context),
        Hint::Nullable(nullable_hint) => match nullable_hint.hint {
            Hint::Null(_) => get_null(),
            Hint::String(_) => get_nullable_string(),
            Hint::Integer(_) => get_nullable_int(),
            Hint::Float(_) => get_nullable_float(),
            Hint::Object(_) => get_nullable_object(),
            _ => get_union_from_hint(nullable_hint.hint, classname, context).as_nullable(),
        },
        Hint::Union(UnionHint { left: Hint::Null(_), right, .. }) => match right {
            Hint::Null(_) => get_null(),
            Hint::String(_) => get_nullable_string(),
            Hint::Integer(_) => get_nullable_int(),
            Hint::Float(_) => get_nullable_float(),
            Hint::Object(_) => get_nullable_object(),
            _ => get_union_from_hint(right, classname, context).as_nullable(),
        },
        Hint::Union(UnionHint { left, right: Hint::Null(_), .. }) => match left {
            Hint::Null(_) => get_null(),
            Hint::String(_) => get_nullable_string(),
            Hint::Integer(_) => get_nullable_int(),
            Hint::Float(_) => get_nullable_float(),
            Hint::Object(_) => get_nullable_object(),
            _ => get_union_from_hint(left, classname, context).as_nullable(),
        },
        Hint::Union(union_hint) => {
            let left = get_union_from_hint(union_hint.left, classname, context);
            let right = get_union_from_hint(union_hint.right, classname, context);

            let combined_types: Vec<TAtomic> = left.types.iter().chain(right.types.iter()).cloned().collect();

            TUnion::from_vec(combined_types)
        }
        Hint::Null(_) => get_null(),
        Hint::True(_) => get_true(),
        Hint::False(_) => get_false(),
        Hint::Array(_) => get_mixed_keyed_array(),
        Hint::Callable(_) => get_mixed_callable(),
        Hint::Static(_) => {
            let classname = classname.unwrap_or_else(|| atom("static"));

            wrap_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_this(classname))))
        }
        Hint::Self_(_) => {
            let classname = classname.unwrap_or_else(|| atom("static"));

            wrap_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(classname))))
        }
        Hint::Void(_) => get_void(),
        Hint::Never(_) => get_never(),
        Hint::Float(_) => get_float(),
        Hint::Bool(_) => get_bool(),
        Hint::Integer(_) => get_int(),
        Hint::String(_) => get_string(),
        Hint::Object(_) => get_object(),
        Hint::Mixed(_) => get_mixed(),
        Hint::Parent(k) => {
            tracing::trace!("Unsupported parent hint in {} at {}", context.file.id, k.span.start,);

            get_mixed()
        }
        Hint::Intersection(intersection) => {
            let left = get_union_from_hint(intersection.left, classname, context);
            let right = get_union_from_hint(intersection.right, classname, context);

            let left_types = left.types;
            let right_types = right.types;
            let mut intersection_types = vec![];
            for left_type in left_types.into_owned() {
                if !left_type.can_be_intersected() {
                    // should be an error.
                    continue;
                }

                for right_type in right_types.as_ref() {
                    if !right_type.can_be_intersected() {
                        // should be an error.
                        continue;
                    }

                    let mut intersection = left_type.clone();
                    if let Some(nested_intersections) = right_type.get_intersection_types() {
                        let mut right_base = right_type.clone();
                        if let Some(intersections) = right_base.get_intersection_types_mut() {
                            intersections.clear();
                        }

                        intersection.add_intersection_type(right_base);
                        for nested in nested_intersections {
                            intersection.add_intersection_type(nested.clone());
                        }
                    } else {
                        intersection.add_intersection_type(right_type.clone());
                    }

                    intersection_types.push(intersection);
                }
            }

            TUnion::from_vec(intersection_types)
        }
        Hint::Iterable(_) => get_mixed_iterable(),
    }
}

#[inline]
fn get_union_from_identifier_hint<'arena>(
    identifier: &'arena Identifier<'arena>,
    context: &mut Context<'_, 'arena>,
) -> TUnion {
    let name = context.resolved_names.get(identifier);

    if name.eq_ignore_ascii_case("Generator") {
        return wrap_atomic(TAtomic::Object(TObject::Named(
            TNamedObject::new(atom(name)).with_type_parameters(Some(vec![
                get_mixed(),
                get_mixed(),
                get_mixed(),
                get_mixed(),
            ])),
        )));
    }

    if name.eq_ignore_ascii_case("Closure") {
        return wrap_atomic(TAtomic::Callable(TCallable::Signature(TCallableSignature::mixed(true))));
    }

    wrap_atomic(TAtomic::Reference(TReference::Symbol { name: atom(name), parameters: None, intersection_types: None }))
}

/// Merges a docblock type with a real type, preserving nullability from the real type.
///
/// If the real type is nullable but the docblock type is not, this function makes
/// the docblock type nullable. This ensures that the actual signature's nullability
/// is respected even when a more specific type is provided in the docblock.
///
/// # Examples
///
/// - Real: `?string`, Docblock: `non-empty-string` → Result: `?non-empty-string`
/// - Real: `null|int`, Docblock: `int` → Result: `null|int`
/// - Real: `string`, Docblock: `non-empty-string` → Result: `non-empty-string`
///
/// # Arguments
///
/// * `docblock_type` - The type from the @param, @var, or @return annotation
/// * `real_type` - The actual type from the code signature (if any)
///
/// # Returns
///
/// The docblock type, potentially modified to be nullable if the real type was nullable
#[inline]
pub fn merge_type_preserving_nullability(
    docblock_type: TypeMetadata,
    real_type: Option<&TypeMetadata>,
) -> TypeMetadata {
    if docblock_type.type_union.types.iter().any(|t| t.is_conditional()) {
        return docblock_type;
    }

    if real_type.is_some_and(|tm| tm.type_union.is_nullable()) && !docblock_type.type_union.accepts_null() {
        docblock_type.map_type_union(super::super::ttype::union::TUnion::as_nullable)
    } else {
        docblock_type
    }
}
