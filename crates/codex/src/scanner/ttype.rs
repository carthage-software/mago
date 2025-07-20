use mago_docblock::tag::TypeString;
use mago_interner::StringIdentifier;
use mago_names::scope::NamespaceScope;
use mago_span::HasSpan;
use mago_syntax::ast::Hint;
use mago_syntax::ast::Identifier;

use crate::metadata::ttype::TypeMetadata;
use crate::scanner::Context;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::callable::TCallable;
use crate::ttype::atomic::callable::TCallableSignature;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::object::named::TNamedObject;
use crate::ttype::atomic::reference::TReference;
use crate::ttype::error::TypeError;
use crate::ttype::resolution::TypeResolutionContext;
use crate::ttype::union::TUnion;
use crate::ttype::*;

#[inline]
pub fn get_type_metadata_from_hint<'ast>(
    hint: &'ast Hint,
    classname: Option<&StringIdentifier>,
    context: &'ast mut Context<'_>,
) -> TypeMetadata {
    let type_union = get_union_from_hint(hint, classname, context);

    let mut type_metadata = TypeMetadata::new(type_union, hint.span());
    type_metadata.from_docblock = false;
    type_metadata
}

#[inline]
pub fn get_type_metadata_from_type_string(
    ttype: &TypeString,
    classname: Option<&StringIdentifier>,
    type_context: &TypeResolutionContext,
    context: &mut Context<'_>,
    scope: &NamespaceScope,
) -> Result<TypeMetadata, TypeError> {
    builder::get_type_from_string(&ttype.value, ttype.span, scope, type_context, classname, context.interner).map(
        |type_union| {
            let mut type_metadata = TypeMetadata::new(type_union, ttype.span);
            type_metadata.from_docblock = true;
            type_metadata
        },
    )
}

#[inline]
fn get_union_from_hint<'ast>(
    hint: &'ast Hint,
    classname: Option<&StringIdentifier>,
    context: &'ast mut Context<'_>,
) -> TUnion {
    match hint {
        Hint::Parenthesized(parenthesized_hint) => get_union_from_hint(&parenthesized_hint.hint, classname, context),
        Hint::Identifier(identifier) => get_union_from_identifier_hint(identifier, context),
        Hint::Nullable(nullable_hint) => {
            let mut tunion = get_union_from_hint(&nullable_hint.hint, classname, context);

            tunion.types.push(TAtomic::Null);
            tunion
        }
        Hint::Union(union_hint) => {
            let mut all_atomics = vec![];

            all_atomics.extend(get_union_from_hint(&union_hint.left, classname, context).types);
            all_atomics.extend(get_union_from_hint(&union_hint.right, classname, context).types);

            TUnion::new(all_atomics)
        }
        Hint::Null(_) => get_null(),
        Hint::True(_) => get_true(),
        Hint::False(_) => get_false(),
        Hint::Array(_) => get_mixed_keyed_array(),
        Hint::Callable(_) => get_mixed_callable(),
        Hint::Static(_) => {
            let classname = if let Some(classname) = classname { *classname } else { context.interner.intern("this") };

            wrap_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_this(classname))))
        }
        Hint::Self_(_) => {
            let classname = if let Some(classname) = classname { *classname } else { context.interner.intern("this") };

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
            tracing::trace!(
                "Unsupported parent hint in {} at {}",
                context.interner.lookup(&context.source.identifier.0),
                k.span.start,
            );

            get_mixed()
        }
        Hint::Intersection(intersection) => {
            let left = get_union_from_hint(&intersection.left, classname, context);
            let right = get_union_from_hint(&intersection.right, classname, context);

            let left_types = left.types;
            let right_types = right.types;
            let mut intersection_types = vec![];
            for left_type in left_types {
                if !left_type.can_be_intersected() {
                    // should be an error.
                    continue;
                }

                for right_type in &right_types {
                    if !right_type.can_be_intersected() {
                        // should be an error.
                        continue;
                    }

                    let mut intersection = left_type.clone();
                    intersection.add_intersection_type(right_type.clone());
                    intersection_types.push(intersection);
                }
            }

            TUnion::new(intersection_types)
        }
        Hint::Iterable(_) => get_mixed_iterable(),
    }
}

#[inline]
fn get_union_from_identifier_hint<'ast>(identifier: &'ast Identifier, context: &'ast mut Context<'_>) -> TUnion {
    let name = context.resolved_names.get(identifier);
    let name_str = context.interner.lookup(name);

    if name_str.eq_ignore_ascii_case("Generator") {
        return wrap_atomic(TAtomic::Object(TObject::Named(
            TNamedObject::new(*name).with_type_parameters(Some(vec![get_mixed(), get_mixed(), get_mixed()])),
        )));
    }

    if name_str.eq_ignore_ascii_case("Closure") {
        return wrap_atomic(TAtomic::Callable(TCallable::Signature(TCallableSignature::mixed(true))));
    }

    wrap_atomic(TAtomic::Reference(TReference::Symbol { name: *name, parameters: None, intersection_types: None }))
}
