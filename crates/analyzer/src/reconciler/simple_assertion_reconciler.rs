use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Arc;

use mago_atom::Atom;
use mago_atom::atom;
use mago_codex::assertion::Assertion;
use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::generic::TGenericParameter;
use mago_codex::ttype::atomic::iterable::TIterable;
use mago_codex::ttype::atomic::mixed::truthiness::TMixedTruthiness;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::atomic::resource::TResource;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::bool::TBool;
use mago_codex::ttype::atomic::scalar::float::TFloat;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::atomic_comparator;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::get_arraykey;
use mago_codex::ttype::get_closed_resource;
use mago_codex::ttype::get_float;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_mixed_iterable;
use mago_codex::ttype::get_mixed_keyed_array;
use mago_codex::ttype::get_mixed_list;
use mago_codex::ttype::get_mixed_maybe_from_loop;
use mago_codex::ttype::get_never;
use mago_codex::ttype::get_non_empty_string;
use mago_codex::ttype::get_null;
use mago_codex::ttype::get_numeric;
use mago_codex::ttype::get_object;
use mago_codex::ttype::get_open_resource;
use mago_codex::ttype::get_resource;
use mago_codex::ttype::get_scalar;
use mago_codex::ttype::get_string_with_props;
use mago_codex::ttype::get_union_from_integer;
use mago_codex::ttype::intersect_union_types;
use mago_codex::ttype::shared::MIXED_KEYED_ARRAY_ATOMIC;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::wrap_atomic;
use mago_span::Span;

use crate::intersect_simple;
use crate::reconciler::Context;
use crate::reconciler::map_concrete_generic_constraint;
use crate::reconciler::map_generic_constraint_or_else;
use crate::reconciler::refine_array_key;
use crate::reconciler::simple_negated_assertion_reconciler::subtract_null;
use crate::reconciler::trigger_issue_for_impossible;

// This performs type intersections and more general reconciliations
pub(crate) fn reconcile(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    span: Option<&Span>,
    negated: bool,
    inside_loop: bool,
) -> Option<TUnion> {
    if let Some(assertion_type) = assertion.get_type() {
        // `mixed is T` -> `T`, always
        if existing_var_type.is_mixed() {
            return Some(wrap_atomic(assertion_type.clone()));
        }

        match assertion_type {
            TAtomic::Scalar(TScalar::Generic) => {
                return intersect_simple!(
                    TAtomic::Scalar(scalar) if !scalar.is_generic(),
                    TAtomic::Mixed(_),
                    context,
                    get_scalar(),
                    assertion,
                    existing_var_type,
                    key,
                    negated,
                    span,
                    assertion.has_equality(),
                );
            }
            TAtomic::Null => {
                return Some(intersect_null(context, assertion, existing_var_type, key, negated, span));
            }
            TAtomic::Resource(resource_to_intersect) => {
                return Some(intersect_resource(
                    context,
                    assertion,
                    existing_var_type,
                    key,
                    negated,
                    span,
                    *resource_to_intersect,
                ));
            }
            TAtomic::Mixed(mixed) if mixed.is_non_null() => {
                return Some(subtract_null(context, assertion, existing_var_type, key, !negated, span));
            }
            TAtomic::Object(TObject::Any) => {
                return Some(intersect_object(
                    context,
                    assertion,
                    existing_var_type,
                    key,
                    negated,
                    span,
                    assertion.has_equality(),
                ));
            }
            TAtomic::Object(TObject::HasMethod(has_method)) => {
                return Some(reconcile_has_method(
                    context,
                    assertion,
                    existing_var_type,
                    key,
                    has_method.method,
                    negated,
                    span,
                ));
            }
            TAtomic::Object(TObject::HasProperty(has_property)) => {
                return Some(reconcile_has_property(
                    context,
                    assertion,
                    existing_var_type,
                    key,
                    has_property.property,
                    negated,
                    span,
                ));
            }
            TAtomic::Iterable(TIterable { key_type, value_type, intersection_types: None }) => {
                if (key_type.is_mixed() || key_type.is_array_key()) && value_type.is_mixed() {
                    return Some(intersect_iterable(
                        context,
                        assertion,
                        existing_var_type,
                        key,
                        negated,
                        span,
                        assertion.has_equality(),
                    ));
                }
            }
            TAtomic::Array(TArray::List(TList { known_elements: None, non_empty, element_type, .. }))
                if element_type.is_placeholder() || element_type.is_mixed() =>
            {
                return Some(intersect_array_list(
                    context,
                    assertion,
                    existing_var_type,
                    key,
                    negated,
                    span,
                    assertion.has_equality(),
                    *non_empty,
                ));
            }
            TAtomic::Array(TArray::Keyed(TKeyedArray { known_items: None, parameters: Some(parameters), .. })) => {
                if (parameters.0.is_placeholder() || parameters.0.is_array_key())
                    && (parameters.1.is_placeholder() || parameters.1.is_mixed())
                {
                    return Some(intersect_keyed_array(
                        context,
                        assertion,
                        existing_var_type,
                        key,
                        negated,
                        span,
                        assertion.has_equality(),
                    ));
                }
            }
            TAtomic::Scalar(TScalar::ArrayKey) => {
                return Some(intersect_arraykey(
                    context,
                    assertion,
                    existing_var_type,
                    key,
                    negated,
                    span,
                    assertion.has_equality(),
                ));
            }
            TAtomic::Scalar(TScalar::Numeric) => {
                return Some(intersect_numeric(
                    context,
                    assertion,
                    existing_var_type,
                    key,
                    negated,
                    span,
                    assertion.has_equality(),
                ));
            }
            TAtomic::Scalar(TScalar::String(str)) if str.is_general() => {
                return Some(intersect_string(
                    context,
                    assertion,
                    existing_var_type,
                    key,
                    negated,
                    span,
                    assertion.has_equality(),
                    str.is_non_empty,
                    str.is_truthy,
                    str.is_numeric,
                    str.is_lowercase,
                ));
            }
            TAtomic::Scalar(TScalar::Bool(bool)) => {
                return Some(intersect_bool(
                    context,
                    assertion,
                    existing_var_type,
                    key,
                    negated,
                    span,
                    assertion.has_equality(),
                    *bool,
                ));
            }
            TAtomic::Scalar(TScalar::Float(float)) if float.is_general() => {
                return Some(intersect_float(
                    context,
                    assertion,
                    existing_var_type,
                    key,
                    negated,
                    span,
                    assertion.has_equality(),
                    float,
                ));
            }
            TAtomic::Scalar(TScalar::Integer(i)) if !i.is_literal() => {
                return Some(intersect_int(
                    context,
                    assertion,
                    existing_var_type,
                    key,
                    negated,
                    span,
                    assertion.has_equality(),
                    i,
                ));
            }
            TAtomic::Mixed(mixed) if mixed.is_vanilla() || mixed.is_isset_from_loop() => {
                if existing_var_type.is_mixed() {
                    return Some(existing_var_type.clone());
                }
            }
            _ => {}
        }
    }

    match assertion {
        Assertion::Any => Some(existing_var_type.clone()),
        Assertion::Truthy | Assertion::NonEmpty => {
            Some(reconcile_truthy_or_non_empty(context, assertion, existing_var_type, key, negated, span))
        }
        Assertion::IsEqualIsset | Assertion::IsIsset => {
            Some(reconcile_isset(context, assertion, existing_var_type, key, negated, span, inside_loop))
        }
        Assertion::HasStringArrayAccess => {
            Some(reconcile_array_access(context, assertion, existing_var_type, key, negated, span, false))
        }
        Assertion::HasIntOrStringArrayAccess => {
            Some(reconcile_array_access(context, assertion, existing_var_type, key, negated, span, true))
        }
        Assertion::ArrayKeyExists => {
            let mut existing_var_type = existing_var_type.clone();
            if existing_var_type.is_never() {
                existing_var_type = get_mixed_maybe_from_loop(inside_loop);
            }
            Some(existing_var_type)
        }
        Assertion::InArray(typed_value) => {
            Some(reconcile_in_array(context, assertion, existing_var_type, key, negated, span, typed_value))
        }
        Assertion::HasArrayKey(key_name) => {
            Some(reconcile_has_array_key(context, assertion, existing_var_type, key, key_name, negated, span))
        }
        Assertion::HasNonnullEntryForKey(key_name) => Some(reconcile_has_nonnull_entry_for_key(
            context,
            assertion,
            existing_var_type,
            key,
            key_name,
            negated,
            span,
        )),
        Assertion::NonEmptyCountable(_) => {
            Some(reconcile_non_empty_countable(context, assertion, existing_var_type, key, negated, span, false))
        }
        Assertion::HasExactCount(count) => {
            Some(reconcile_exactly_countable(context, assertion, existing_var_type, key, negated, span, false, *count))
        }
        Assertion::HasAtLeastCount(count) => {
            Some(reconcile_at_least_countable(context, assertion, existing_var_type, key, negated, span, false, *count))
        }
        Assertion::IsLessThan(less_than) => {
            Some(reconcile_less_than(context, assertion, existing_var_type, key, negated, span, *less_than))
        }
        Assertion::IsGreaterThan(greater_than) => {
            Some(reconcile_greater_than(context, assertion, existing_var_type, key, negated, span, *greater_than))
        }
        Assertion::IsLessThanOrEqual(less_than_or_equal) => Some(reconcile_less_than_or_equal(
            context,
            assertion,
            existing_var_type,
            key,
            negated,
            span,
            *less_than_or_equal,
        )),
        Assertion::IsGreaterThanOrEqual(greater_than_or_equal) => Some(reconcile_greater_than_or_equal(
            context,
            assertion,
            existing_var_type,
            key,
            negated,
            span,
            *greater_than_or_equal,
        )),
        Assertion::Countable => Some(reconcile_countable(context, assertion, existing_var_type, key, negated, span)),
        _ => None,
    }
}

pub(crate) fn intersect_null(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    negated: bool,
    span: Option<&Span>,
) -> TUnion {
    if existing_var_type.is_mixed() {
        return get_null();
    }

    let mut acceptable_types = Vec::new();
    let mut did_remove_type = false;

    for atomic in existing_var_type.types.as_ref() {
        match atomic {
            TAtomic::Null => {
                acceptable_types.push(TAtomic::Null);
            }
            TAtomic::Mixed(mixed) if !mixed.is_isset_from_loop() && (mixed.is_vanilla() || !mixed.is_non_null()) => {
                acceptable_types.push(TAtomic::Null);
                did_remove_type = true;
            }
            TAtomic::GenericParameter(generic_parameter) => {
                did_remove_type = true;
                if let Some(atomic) = map_generic_constraint_or_else(generic_parameter, get_null, |constraint| {
                    intersect_null(context, assertion, constraint, None, false, None)
                }) {
                    acceptable_types.push(atomic);
                }
            }
            TAtomic::Variable(_) => {
                if !existing_var_type.is_nullable() {
                    acceptable_types.push(atomic.clone());
                }

                did_remove_type = true;
            }
            TAtomic::Object(TObject::Named(named_object)) if !named_object.has_type_parameters() => {
                did_remove_type = true;
            }
            _ => {
                did_remove_type = true;
            }
        }
    }

    if (acceptable_types.is_empty() || !did_remove_type)
        && let Some(key) = key
        && let Some(span) = span
    {
        let old_var_type_atom = existing_var_type.get_id();

        trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, !did_remove_type, negated, span);
    }

    if !acceptable_types.is_empty() {
        return TUnion::from_vec(acceptable_types);
    }

    get_never()
}

pub(crate) fn intersect_resource(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    negated: bool,
    span: Option<&Span>,
    resource_to_intersection: TResource,
) -> TUnion {
    if existing_var_type.is_mixed() {
        return match resource_to_intersection.closed {
            None => get_resource(),
            Some(true) => get_closed_resource(),
            Some(false) => get_open_resource(),
        };
    }

    let mut acceptable_types = Vec::new();
    let mut did_remove_type = false;

    for atomic in existing_var_type.types.as_ref() {
        match atomic {
            TAtomic::Resource(existing_resource) => match (existing_resource.closed, resource_to_intersection.closed) {
                (Some(true), Some(true)) | (Some(false), Some(false)) | (None | Some(_), None) => {
                    acceptable_types.push(TAtomic::Resource(*existing_resource));
                }
                (None, Some(true | false)) => {
                    did_remove_type = true;

                    acceptable_types.push(TAtomic::Resource(resource_to_intersection));
                }
                (Some(true), Some(false)) | (Some(false), Some(true)) => {
                    did_remove_type = true;
                }
            },
            TAtomic::GenericParameter(generic_parameter) => {
                did_remove_type = true;
                if let Some(atomic) = map_generic_constraint_or_else(generic_parameter, get_null, |constraint| {
                    intersect_resource(context, assertion, constraint, None, false, None, resource_to_intersection)
                }) {
                    acceptable_types.push(atomic);
                }
            }
            _ => {
                did_remove_type = true;
            }
        }
    }

    if (acceptable_types.is_empty() || !did_remove_type)
        && let Some(key) = key
        && let Some(span) = span
    {
        let old_var_type_atom = existing_var_type.get_id();

        trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, !did_remove_type, negated, span);
    }

    if !acceptable_types.is_empty() {
        return TUnion::from_vec(acceptable_types);
    }

    get_never()
}

fn intersect_object(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
) -> TUnion {
    if existing_var_type.is_mixed() {
        return get_object();
    }

    let mut object_types = Vec::new();
    let mut did_remove_type = false;

    for atomic in existing_var_type.types.as_ref() {
        if atomic.is_object_type() {
            object_types.push(atomic.clone());
        } else if let TAtomic::GenericParameter(generic_parameter) = atomic {
            did_remove_type = true;

            if let Some(atomic) = map_generic_constraint_or_else(generic_parameter, get_object, |constraint| {
                intersect_object(context, assertion, constraint, None, false, None, is_equality)
            }) {
                object_types.push(atomic);
            }
        } else {
            did_remove_type = true;
        }
    }

    if (object_types.is_empty() || (!did_remove_type && !is_equality))
        && let Some(key) = key
        && let Some(span) = span
    {
        let old_var_type_atom = existing_var_type.get_id();

        trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, !did_remove_type, negated, span);
    }

    if !object_types.is_empty() {
        return TUnion::from_vec(object_types);
    }

    get_never()
}

fn intersect_iterable(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
) -> TUnion {
    if existing_var_type.is_mixed() {
        return get_mixed_iterable();
    }

    let mut acceptable_types = Vec::new();
    let mut did_remove_type = false;

    for atomic in existing_var_type.types.as_ref() {
        if atomic.is_array_or_traversable(context.codebase) {
            acceptable_types.push(atomic.clone());

            continue;
        }

        match atomic {
            TAtomic::GenericParameter(generic_parameter) => {
                did_remove_type = true;

                if let Some(atomic) =
                    map_generic_constraint_or_else(generic_parameter, get_mixed_iterable, |constraint| {
                        intersect_iterable(context, assertion, constraint, None, false, span, is_equality)
                    })
                {
                    acceptable_types.push(atomic);
                }
            }
            TAtomic::Variable(_) => {
                acceptable_types.push(atomic.clone());
                did_remove_type = true;
            }
            _ => {
                did_remove_type = true;
            }
        }
    }

    if (acceptable_types.is_empty() || (!did_remove_type && !is_equality))
        && let Some(key) = key
        && let Some(span) = span
    {
        trigger_issue_for_impossible(
            context,
            existing_var_type.get_id(),
            key,
            assertion,
            !did_remove_type,
            negated,
            span,
        );
    }

    if !acceptable_types.is_empty() {
        return TUnion::from_vec(acceptable_types);
    }

    get_never()
}

fn intersect_array_list(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
    is_non_empty: bool,
) -> TUnion {
    if existing_var_type.is_mixed() {
        return wrap_atomic(if is_non_empty {
            TAtomic::Array(TArray::List(TList::new_non_empty(Arc::new(get_mixed()))))
        } else {
            TAtomic::Array(TArray::List(TList::new(Arc::new(get_mixed()))))
        });
    }

    let mut acceptable_types = Vec::new();
    let mut did_remove_type = false;

    'outer: for atomic in existing_var_type.types.as_ref() {
        match atomic {
            TAtomic::Array(TArray::Keyed(TKeyedArray { known_items, parameters, non_empty })) => {
                if let Some(known_items) = known_items {
                    for k in known_items.keys() {
                        if !k.is_integer() {
                            did_remove_type = true;
                            continue 'outer;
                        }
                    }
                }

                let element_type = if let Some((key_parameter, value_parameter)) = parameters {
                    if !key_parameter.has_int() {
                        did_remove_type = true;
                        continue 'outer;
                    }

                    value_parameter.clone()
                } else {
                    Arc::new(get_mixed())
                };

                did_remove_type = true;
                acceptable_types.push(if is_non_empty || *non_empty {
                    TAtomic::Array(TArray::List(TList::new_non_empty(element_type)))
                } else {
                    TAtomic::Array(TArray::List(TList::new(element_type)))
                });
            }
            TAtomic::Array(TArray::List(_)) => {
                acceptable_types.push(atomic.clone());
            }
            TAtomic::Iterable(iterable) => {
                let element_type = iterable.get_value_type();

                acceptable_types.push(if is_non_empty {
                    TAtomic::Array(TArray::List(TList::new_non_empty(Arc::new(element_type.clone()))))
                } else {
                    TAtomic::Array(TArray::List(TList::new(Arc::new(element_type.clone()))))
                });
            }
            TAtomic::GenericParameter(generic_parameter) => {
                did_remove_type = true;

                if let Some(atomic) = map_generic_constraint_or_else(generic_parameter, get_mixed_list, |constraint| {
                    intersect_array_list(context, assertion, constraint, None, false, span, is_equality, is_non_empty)
                }) {
                    acceptable_types.push(atomic);
                }
            }
            TAtomic::Variable(_) => {
                acceptable_types.push(atomic.clone());
                did_remove_type = true;
            }
            TAtomic::Object(TObject::Named(_)) => {
                did_remove_type = true;
            }
            TAtomic::Callable(_) => {
                did_remove_type = true;

                let mut known_items = BTreeMap::new();

                known_items.insert(
                    ArrayKey::Integer(0),
                    (
                        false,
                        TUnion::from_vec(vec![TAtomic::Object(TObject::Any), TAtomic::Scalar(TScalar::class_string())]),
                    ),
                );
                known_items.insert(ArrayKey::Integer(1), (false, get_non_empty_string()));

                acceptable_types.push(TAtomic::Array(TArray::Keyed(
                    TKeyedArray::new().with_known_items(known_items).with_non_empty(true),
                )));
            }
            _ => {
                did_remove_type = true;
            }
        }
    }

    if (acceptable_types.is_empty() || (!did_remove_type && !is_equality))
        && let Some(key) = key
        && let Some(span) = span
    {
        trigger_issue_for_impossible(
            context,
            existing_var_type.get_id(),
            key,
            assertion,
            !did_remove_type,
            negated,
            span,
        );
    }

    if !acceptable_types.is_empty() {
        return TUnion::from_vec(acceptable_types);
    }

    get_never()
}

fn intersect_keyed_array(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
) -> TUnion {
    let assertion_type = assertion.get_type();

    if existing_var_type.is_mixed() {
        return if let Some(assertion_type) = assertion_type {
            wrap_atomic(assertion_type.clone())
        } else {
            get_mixed_keyed_array()
        };
    }

    let mut acceptable_types = Vec::new();
    let mut did_remove_type = false;

    for atomic in existing_var_type.types.as_ref() {
        match atomic {
            TAtomic::Array(TArray::Keyed(keyed_array)) if !keyed_array.has_known_items() => {
                let mut non_empty = keyed_array.is_non_empty();

                if let Some(TAtomic::Array(assertion_array)) = assertion_type
                    && assertion_array.is_non_empty()
                {
                    non_empty = true;
                }

                acceptable_types.push(TAtomic::Array(TArray::Keyed(keyed_array.as_non_empty_array(non_empty))));
            }
            TAtomic::Array(TArray::Keyed(keyed_array)) => {
                acceptable_types.push(TAtomic::Array(TArray::Keyed(keyed_array.clone())));
            }
            TAtomic::Array(TArray::List(list)) => {
                acceptable_types.push(TAtomic::Array(TArray::List(list.clone())));
            }
            TAtomic::Iterable(iterable) => {
                let key_type = refine_array_key(iterable.get_key_type());
                let value_type = iterable.get_value_type();

                acceptable_types.push(TAtomic::Array(TArray::Keyed(TKeyedArray::new_with_parameters(
                    Arc::new(key_type),
                    Arc::new(value_type.clone()),
                ))));
            }
            TAtomic::GenericParameter(generic_parameter) => {
                did_remove_type = true;

                if let Some(atomic) =
                    map_generic_constraint_or_else(generic_parameter, get_mixed_keyed_array, |constraint| {
                        intersect_keyed_array(context, assertion, constraint, None, false, None, is_equality)
                    })
                {
                    acceptable_types.push(atomic);
                }
            }
            TAtomic::Variable(_) => {
                acceptable_types.push(atomic.clone());
                did_remove_type = true;
            }
            TAtomic::Object(TObject::Named(_)) => {
                did_remove_type = true;
            }
            TAtomic::Callable(_) => {
                did_remove_type = true;

                let mut known_items = BTreeMap::new();

                known_items.insert(
                    ArrayKey::Integer(0),
                    (
                        false,
                        TUnion::from_vec(vec![TAtomic::Object(TObject::Any), TAtomic::Scalar(TScalar::class_string())]),
                    ),
                );
                known_items.insert(ArrayKey::Integer(1), (false, get_non_empty_string()));

                acceptable_types.push(TAtomic::Array(TArray::Keyed(
                    TKeyedArray::new().with_known_items(known_items).with_non_empty(true),
                )));
            }
            _ => {
                did_remove_type = true;
            }
        }
    }

    if (acceptable_types.is_empty() || (!did_remove_type && !is_equality))
        && let Some(key) = key
        && let Some(span) = span
    {
        trigger_issue_for_impossible(
            context,
            existing_var_type.get_id(),
            key,
            assertion,
            !did_remove_type,
            negated,
            span,
        );
    }

    if !acceptable_types.is_empty() {
        return TUnion::from_vec(acceptable_types);
    }

    get_never()
}

fn intersect_arraykey(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
) -> TUnion {
    if existing_var_type.is_mixed() {
        return get_arraykey();
    }

    let mut acceptable_types = Vec::new();
    let mut did_remove_type = false;

    for atomic in existing_var_type.types.as_ref() {
        match atomic {
            TAtomic::Mixed(_) | TAtomic::Scalar(TScalar::Generic) => {
                return get_arraykey();
            }
            TAtomic::Scalar(TScalar::Numeric) => {
                did_remove_type = true; // removed `float`

                acceptable_types.push(TAtomic::Scalar(TScalar::String(TString::numeric())));
                acceptable_types.push(TAtomic::Scalar(TScalar::Integer(TInteger::Unspecified)));
            }
            TAtomic::Scalar(TScalar::Integer(integer)) => {
                acceptable_types.push(TAtomic::Scalar(TScalar::Integer(*integer)));
            }
            TAtomic::Scalar(TScalar::String(string)) => {
                acceptable_types.push(TAtomic::Scalar(TScalar::String(string.clone())));
            }
            TAtomic::Scalar(TScalar::ClassLikeString(class_like_string)) => {
                acceptable_types.push(TAtomic::Scalar(TScalar::ClassLikeString(class_like_string.clone())));
            }
            TAtomic::Scalar(TScalar::ArrayKey) => {
                acceptable_types.push(TAtomic::Scalar(TScalar::ArrayKey));
            }
            TAtomic::GenericParameter(generic_parameter) => {
                did_remove_type = true;

                if let Some(atomic) = map_generic_constraint_or_else(generic_parameter, get_arraykey, |constraint| {
                    intersect_arraykey(context, assertion, constraint, None, false, None, is_equality)
                }) {
                    acceptable_types.push(atomic);
                }
            }
            TAtomic::Variable(_) => {
                acceptable_types.push(atomic.clone());
                did_remove_type = true;
            }
            _ => {
                did_remove_type = true;
            }
        }
    }

    if (acceptable_types.is_empty() || (!did_remove_type && !is_equality))
        && let Some(key) = key
        && let Some(span) = span
    {
        trigger_issue_for_impossible(
            context,
            existing_var_type.get_id(),
            key,
            assertion,
            !did_remove_type,
            negated,
            span,
        );
    }

    if !acceptable_types.is_empty() {
        return TUnion::from_vec(acceptable_types);
    }

    get_never()
}

fn intersect_numeric(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
) -> TUnion {
    let mut acceptable_types = Vec::new();
    let mut did_remove_type = false;

    for atomic in existing_var_type.types.as_ref() {
        match atomic {
            TAtomic::Mixed(_) | TAtomic::Scalar(TScalar::Generic) => {
                return get_numeric();
            }
            TAtomic::Scalar(TScalar::Float(float)) => {
                acceptable_types.push(TAtomic::Scalar(TScalar::Float(*float)));
            }
            TAtomic::Scalar(TScalar::Integer(integer)) => {
                acceptable_types.push(TAtomic::Scalar(TScalar::Integer(*integer)));
            }
            TAtomic::Scalar(TScalar::String(existing_string)) if existing_string.is_numeric => {
                acceptable_types.push(atomic.clone());
            }
            TAtomic::Scalar(TScalar::ArrayKey) => {
                did_remove_type = true; // we remove the `non-numeric` string part

                acceptable_types.push(TAtomic::Scalar(TScalar::int()));
                acceptable_types.push(TAtomic::Scalar(TScalar::numeric_string()));
            }
            TAtomic::Scalar(TScalar::String(existing_string)) => {
                did_remove_type = true; // we remove the `non-numeric` string part

                acceptable_types.push(TAtomic::Scalar(TScalar::String(existing_string.as_numeric(false))));
            }
            TAtomic::GenericParameter(generic_parameter) => {
                did_remove_type = true;
                if let Some(atomic) = map_generic_constraint_or_else(generic_parameter, get_numeric, |constraint| {
                    intersect_numeric(context, assertion, constraint, None, false, None, is_equality)
                }) {
                    acceptable_types.push(atomic);
                }
            }
            TAtomic::Variable(_) => {
                acceptable_types.push(atomic.clone());
                did_remove_type = true;
            }
            _ => {
                did_remove_type = true;
            }
        }
    }

    if (acceptable_types.is_empty() || (!did_remove_type && !is_equality))
        && let Some(key) = key
        && let Some(span) = span
    {
        trigger_issue_for_impossible(
            context,
            existing_var_type.get_id(),
            key,
            assertion,
            !did_remove_type,
            negated,
            span,
        );
    }

    if !acceptable_types.is_empty() {
        return TUnion::from_vec(acceptable_types);
    }

    get_never()
}

fn intersect_string(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
    is_non_empty: bool,
    is_truthy: bool,
    is_numeric: bool,
    is_lowercase: bool,
) -> TUnion {
    let mut acceptable_types = Vec::new();
    let mut did_remove_type = false;

    for atomic in existing_var_type.types.as_ref() {
        match atomic {
            TAtomic::Scalar(TScalar::String(existing_string)) => {
                acceptable_types.push(
                    get_string_with_props(
                        is_numeric || existing_string.is_numeric,
                        is_truthy || existing_string.is_truthy,
                        is_non_empty || existing_string.is_non_empty,
                        is_lowercase || existing_string.is_lowercase,
                    )
                    .get_single_owned(),
                );
            }
            TAtomic::Scalar(TScalar::ClassLikeString(_)) if !is_numeric => {
                acceptable_types.push(atomic.clone());
            }
            TAtomic::Mixed(_) | TAtomic::Scalar(TScalar::Generic | TScalar::ArrayKey) => {
                return get_string_with_props(is_numeric, is_truthy, is_non_empty, is_lowercase);
            }
            TAtomic::GenericParameter(generic_parameter) => {
                did_remove_type = true;

                if let Some(atomic) = map_generic_constraint_or_else(
                    generic_parameter,
                    || get_string_with_props(is_numeric, is_truthy, is_non_empty, is_lowercase),
                    |constraint| {
                        intersect_string(
                            context,
                            assertion,
                            constraint,
                            None,
                            false,
                            None,
                            is_equality,
                            is_non_empty,
                            is_truthy,
                            is_numeric,
                            is_lowercase,
                        )
                    },
                ) {
                    acceptable_types.push(atomic);
                }
            }
            TAtomic::Variable(_) => {
                acceptable_types.push(atomic.clone());
                did_remove_type = true;
            }
            TAtomic::Object(_) => {
                did_remove_type = true;
            }
            _ => {
                if atomic_comparator::is_contained_by(
                    context.codebase,
                    atomic,
                    get_string_with_props(is_numeric, is_truthy, is_non_empty, is_lowercase).get_single(),
                    false,
                    &mut ComparisonResult::new(),
                ) {
                    acceptable_types.push(atomic.clone());
                } else {
                    did_remove_type = true;
                }
            }
        }
    }

    if (acceptable_types.is_empty() || (!did_remove_type && !is_equality))
        && let Some(key) = key
        && let Some(span) = span
    {
        trigger_issue_for_impossible(
            context,
            existing_var_type.get_id(),
            key,
            assertion,
            !did_remove_type,
            negated,
            span,
        );
    }

    if !acceptable_types.is_empty() {
        return TUnion::from_vec(acceptable_types);
    }

    get_never()
}

fn intersect_bool(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
    boolean: TBool,
) -> TUnion {
    // Treat specific boolean values (true/false literals) as equality checks
    // even if the assertion is IsType rather than IsIdentical
    let is_equality = is_equality || !boolean.is_general();

    let mut acceptable_types = Vec::new();
    let mut did_remove_type = false;

    for atomic in existing_var_type.types.as_ref() {
        match atomic {
            TAtomic::Scalar(TScalar::Bool(existing_bool)) => {
                if existing_bool.is_general() || *existing_bool == boolean {
                    acceptable_types.push(TAtomic::Scalar(TScalar::Bool(boolean)));
                } else {
                    did_remove_type = true;
                }
            }
            TAtomic::Mixed(_) | TAtomic::Scalar(TScalar::Generic | TScalar::ArrayKey) => {
                return TUnion::from_atomic(TAtomic::Scalar(TScalar::Bool(boolean)));
            }
            TAtomic::GenericParameter(generic_parameter) => {
                did_remove_type = true;

                if let Some(atomic) = map_generic_constraint_or_else(
                    generic_parameter,
                    || TUnion::from_atomic(TAtomic::Scalar(TScalar::Bool(boolean))),
                    |constraint| {
                        intersect_bool(context, assertion, constraint, None, false, None, is_equality, boolean)
                    },
                ) {
                    acceptable_types.push(atomic);
                }
            }
            TAtomic::Variable(_) => {
                acceptable_types.push(atomic.clone());
                did_remove_type = true;
            }
            _ => {
                if atomic_comparator::is_contained_by(
                    context.codebase,
                    atomic,
                    &TAtomic::Scalar(TScalar::Bool(boolean)),
                    false,
                    &mut ComparisonResult::new(),
                ) {
                    acceptable_types.push(atomic.clone());
                } else {
                    did_remove_type = true;
                }
            }
        }
    }

    if (acceptable_types.is_empty() || (!did_remove_type && !is_equality))
        && let Some(key) = key
        && let Some(span) = span
    {
        trigger_issue_for_impossible(
            context,
            existing_var_type.get_id(),
            key,
            assertion,
            !did_remove_type,
            negated,
            span,
        );
    }

    if !acceptable_types.is_empty() {
        return TUnion::from_vec(acceptable_types);
    }

    get_never()
}

fn intersect_float(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
    float: &TFloat,
) -> TUnion {
    let mut acceptable_types = Vec::new();
    let mut did_remove_type = false;

    for atomic in existing_var_type.types.as_ref() {
        match atomic {
            TAtomic::Scalar(TScalar::Float(_)) => {
                acceptable_types.push(TAtomic::Scalar(TScalar::Float(*float)));
            }
            TAtomic::Mixed(_) | TAtomic::Scalar(TScalar::Generic | TScalar::Numeric) => {
                return get_float();
            }
            TAtomic::GenericParameter(generic_parameter) => {
                did_remove_type = true;

                if let Some(atomic) = map_generic_constraint_or_else(generic_parameter, get_float, |constraint| {
                    intersect_float(context, assertion, constraint, None, false, None, is_equality, float)
                }) {
                    acceptable_types.push(atomic);
                }
            }
            TAtomic::Variable(_) => {
                acceptable_types.push(atomic.clone());
                did_remove_type = true;
            }
            _ => {
                if atomic_comparator::is_contained_by(
                    context.codebase,
                    atomic,
                    &TAtomic::Scalar(TScalar::Float(*float)),
                    false,
                    &mut ComparisonResult::new(),
                ) {
                    acceptable_types.push(atomic.clone());
                } else {
                    did_remove_type = true;
                }
            }
        }
    }

    if (acceptable_types.is_empty() || (!did_remove_type && !is_equality))
        && let Some(key) = key
        && let Some(span) = span
    {
        trigger_issue_for_impossible(
            context,
            existing_var_type.get_id(),
            key,
            assertion,
            !did_remove_type,
            negated,
            span,
        );
    }

    if !acceptable_types.is_empty() {
        return TUnion::from_vec(acceptable_types);
    }

    get_never()
}

fn intersect_int(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
    integer: &TInteger,
) -> TUnion {
    let mut acceptable_types = Vec::new();
    let mut did_remove_type = false;

    for atomic in existing_var_type.types.as_ref() {
        match atomic {
            TAtomic::Scalar(TScalar::Integer(_)) => {
                acceptable_types.push(TAtomic::Scalar(TScalar::Integer(*integer)));
            }
            TAtomic::Mixed(_) | TAtomic::Scalar(TScalar::Generic | TScalar::ArrayKey | TScalar::Numeric) => {
                return get_union_from_integer(integer);
            }
            TAtomic::GenericParameter(generic_parameter) => {
                did_remove_type = true;

                if let Some(atomic) = map_generic_constraint_or_else(
                    generic_parameter,
                    || get_union_from_integer(integer),
                    |constraint| intersect_int(context, assertion, constraint, None, false, None, is_equality, integer),
                ) {
                    acceptable_types.push(atomic);
                }
            }
            TAtomic::Variable(_) => {
                acceptable_types.push(atomic.clone());
                did_remove_type = true;
            }
            _ => {
                if atomic_comparator::is_contained_by(
                    context.codebase,
                    atomic,
                    &TAtomic::Scalar(TScalar::Integer(*integer)),
                    false,
                    &mut ComparisonResult::new(),
                ) {
                    acceptable_types.push(atomic.clone());
                } else {
                    did_remove_type = true;
                }
            }
        }
    }

    if (acceptable_types.is_empty() || (!did_remove_type && !is_equality))
        && let Some(key) = key
        && let Some(span) = span
    {
        trigger_issue_for_impossible(
            context,
            existing_var_type.get_id(),
            key,
            assertion,
            !did_remove_type,
            negated,
            span,
        );
    }

    if !acceptable_types.is_empty() {
        return TUnion::from_vec(acceptable_types);
    }

    get_never()
}

fn reconcile_truthy_or_non_empty(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    negated: bool,
    span: Option<&Span>,
) -> TUnion {
    let mut did_remove_type = existing_var_type.possibly_undefined() || existing_var_type.possibly_undefined_from_try();
    let mut new_var_type = existing_var_type.clone();
    let mut acceptable_types = vec![];

    let is_non_empty_assertion = matches!(assertion, Assertion::NonEmpty);
    let possibly_undefined_from_try = new_var_type.possibly_undefined_from_try();

    for atomic in new_var_type.types.to_mut().drain(..) {
        if atomic.is_falsy() {
            did_remove_type = true;
        } else if !atomic.is_truthy() || possibly_undefined_from_try {
            did_remove_type = true;

            match atomic {
                TAtomic::GenericParameter(generic_parameter) => {
                    if let Some(atomic) = map_concrete_generic_constraint(&generic_parameter, |constraint| {
                        reconcile_truthy_or_non_empty(context, assertion, constraint, None, false, None)
                    }) {
                        acceptable_types.push(atomic);
                    }
                }
                TAtomic::Variable { .. } => {
                    did_remove_type = true;
                    acceptable_types.push(atomic);
                }
                TAtomic::Scalar(TScalar::Bool(bool)) if bool.is_general() => {
                    acceptable_types.push(TAtomic::Scalar(TScalar::r#true()));
                }
                TAtomic::Array(TArray::List(mut list)) => {
                    list.non_empty = true;
                    acceptable_types.push(TAtomic::Array(TArray::List(list)));
                }
                TAtomic::Array(TArray::Keyed(mut keyed_array)) => {
                    keyed_array.non_empty = true;
                    acceptable_types.push(TAtomic::Array(TArray::Keyed(keyed_array)));
                }
                TAtomic::Mixed(mixed) => {
                    acceptable_types.push(TAtomic::Mixed(
                        mixed.with_is_isset_from_loop(false).with_truthiness(TMixedTruthiness::Truthy),
                    ));
                }
                TAtomic::Scalar(TScalar::String(mut str)) if !str.is_known_literal() => {
                    str.is_truthy = true;
                    str.is_non_empty = true;

                    acceptable_types.push(TAtomic::Scalar(TScalar::String(str)));
                }
                _ => {
                    acceptable_types.push(atomic);
                }
            }
        } else {
            acceptable_types.push(atomic);
        }
    }

    new_var_type.set_possibly_undefined_from_try(false);
    new_var_type.set_possibly_undefined(false, None);

    get_acceptable_type(
        context,
        acceptable_types,
        did_remove_type,
        key,
        span,
        existing_var_type,
        assertion,
        negated,
        !is_non_empty_assertion,
        new_var_type,
    )
}

fn reconcile_isset(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    negated: bool,
    span: Option<&Span>,
    inside_loop: bool,
) -> TUnion {
    let mut did_remove_type = existing_var_type.possibly_undefined() || existing_var_type.possibly_undefined_from_try();

    if existing_var_type.possibly_undefined() {
        did_remove_type = true;
    }

    let mut new_var_type = existing_var_type.clone();

    let existing_var_types = new_var_type.types.to_mut().drain(..).collect::<Vec<_>>();

    let mut acceptable_types = vec![];

    for atomic in existing_var_types {
        if let TAtomic::Null = atomic {
            did_remove_type = true;
        } else if let TAtomic::Mixed(mixed) = atomic {
            if mixed.is_non_null() {
                acceptable_types.push(TAtomic::Mixed(mixed));
            } else {
                acceptable_types.push(TAtomic::Mixed(mixed.with_is_non_null(true)));
                did_remove_type = true;
            }
        } else {
            acceptable_types.push(atomic);
        }
    }

    // every type was removed, this is an impossible assertion
    if let Some(key) = key
        && let Some(span) = span
        && (!did_remove_type || acceptable_types.is_empty())
    {
        let old_var_type_atom = existing_var_type.get_id();

        trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, !did_remove_type, negated, span);
    }

    if acceptable_types.is_empty() {
        return get_mixed_maybe_from_loop(inside_loop);
    }

    new_var_type.set_possibly_undefined_from_try(false);
    new_var_type.types = Cow::Owned(acceptable_types);

    if new_var_type.is_never() {
        return get_mixed_maybe_from_loop(inside_loop);
    }

    new_var_type
}

fn reconcile_non_empty_countable(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    negated: bool,
    span: Option<&Span>,
    recursive_check: bool,
) -> TUnion {
    let mut did_remove_type = false;
    let mut new_var_type = existing_var_type.clone();
    let mut acceptable_types = vec![];

    for atomic in new_var_type.types.to_mut().drain(..) {
        match atomic {
            TAtomic::Array(TArray::List(TList { non_empty, element_type, known_elements, known_count })) => {
                if !non_empty {
                    did_remove_type = true;
                }

                acceptable_types.push(TAtomic::Array(TArray::List(TList {
                    non_empty: true,
                    element_type,
                    known_elements,
                    known_count,
                })));
            }
            TAtomic::Array(TArray::Keyed(TKeyedArray { non_empty, parameters, known_items })) => {
                if !non_empty {
                    did_remove_type = true;
                }

                acceptable_types.push(TAtomic::Array(TArray::Keyed(TKeyedArray {
                    non_empty: true,
                    parameters,
                    known_items,
                })));
            }
            _ => {
                acceptable_types.push(atomic);
            }
        }
    }

    if let Some(key) = key
        && let Some(span) = span
        && !recursive_check
        && (!did_remove_type || acceptable_types.is_empty())
    {
        let old_var_type_atom = existing_var_type.get_id();

        trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, !did_remove_type, negated, span);
    }

    if acceptable_types.is_empty() {
        return get_never();
    }

    new_var_type.types = Cow::Owned(acceptable_types);
    new_var_type
}

fn reconcile_exactly_countable(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    negated: bool,
    span: Option<&Span>,
    recursive_check: bool,
    count: usize,
) -> TUnion {
    let old_var_type_atom = existing_var_type.get_id();

    let mut did_remove_type = false;

    let existing_var_types = existing_var_type.types.as_ref();
    let mut existing_var_type = existing_var_type.clone();

    for atomic in existing_var_types {
        if let TAtomic::Array(TArray::List(TList { non_empty, known_count, element_type, known_elements })) = atomic {
            let min_under_count = if let Some(known_count) = known_count { *known_count < count } else { false };
            if !non_empty || min_under_count {
                existing_var_type.remove_type(atomic);
                if !element_type.is_never() {
                    existing_var_type.types.to_mut().push(TAtomic::Array(TArray::List(TList {
                        element_type: element_type.clone(),
                        known_elements: known_elements.clone(),
                        known_count: Some(count),
                        non_empty: true,
                    })));
                }

                did_remove_type = true;
            }
        } else if let TAtomic::Array(TArray::Keyed(TKeyedArray { non_empty, parameters, known_items })) = atomic {
            did_remove_type = true;

            if !non_empty {
                existing_var_type.remove_type(atomic);

                existing_var_type.types.to_mut().push(TAtomic::Array(TArray::Keyed(TKeyedArray {
                    known_items: known_items.clone(),
                    parameters: parameters.clone(),
                    non_empty: true,
                })));
            }
        }
    }

    if !did_remove_type || existing_var_type.types.is_empty() {
        // every type was removed, this is an impossible assertion
        if let Some(key) = key
            && let Some(span) = span
            && !recursive_check
        {
            trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, !did_remove_type, negated, span);
        }

        if existing_var_type.types.is_empty() {
            return get_never();
        }
    }

    existing_var_type
}

fn reconcile_at_least_countable(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    negated: bool,
    span: Option<&Span>,
    recursive_check: bool,
    count: usize,
) -> TUnion {
    let old_var_type_atom = existing_var_type.get_id();

    let mut did_remove_type = false;

    let existing_var_types = existing_var_type.types.as_ref();
    let mut existing_var_type = existing_var_type.clone();

    for atomic in existing_var_types {
        if let TAtomic::Array(TArray::List(TList { non_empty, known_count, element_type, known_elements })) = atomic {
            let min_under_count = match known_count {
                Some(kc) => *kc < count,
                None => *non_empty && count > 1,
            };

            if !non_empty || min_under_count {
                existing_var_type.remove_type(atomic);
                if !element_type.is_never() {
                    existing_var_type.types.to_mut().push(TAtomic::Array(TArray::List(TList {
                        element_type: element_type.clone(),
                        known_elements: known_elements.clone(),
                        known_count: Some(count),
                        non_empty: true,
                    })));
                }

                did_remove_type = true;
            }
        } else if let TAtomic::Array(TArray::Keyed(TKeyedArray { non_empty, parameters, known_items })) = atomic {
            did_remove_type = true;

            if !non_empty {
                existing_var_type.remove_type(atomic);

                existing_var_type.types.to_mut().push(TAtomic::Array(TArray::Keyed(TKeyedArray {
                    known_items: known_items.clone(),
                    parameters: parameters.clone(),
                    non_empty: true,
                })));
            }
        }
    }

    if !did_remove_type || existing_var_type.types.is_empty() {
        // every type was removed, this is an impossible assertion
        if let Some(key) = key
            && let Some(span) = span
            && !recursive_check
        {
            trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, !did_remove_type, negated, span);
        }

        if existing_var_type.types.is_empty() {
            return get_never();
        }
    }

    existing_var_type
}

fn reconcile_countable(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    negated: bool,
    span: Option<&Span>,
) -> TUnion {
    if existing_var_type.has_mixed() || existing_var_type.has_template() {
        return TUnion::from_vec(vec![
            TAtomic::Object(TObject::Named(TNamedObject::new(atom("Countable")))),
            MIXED_KEYED_ARRAY_ATOMIC.clone(),
        ]);
    }

    let mut redundant = true;
    let mut countable_types = vec![];

    for atomic in existing_var_type.types.as_ref() {
        if atomic.is_countable(context.codebase) {
            countable_types.push(atomic.clone());
        } else if let TAtomic::Object(TObject::Any) = atomic {
            countable_types.push(TAtomic::Object(TObject::Named(TNamedObject::new(atom("Countable")))));
            redundant = false;
        } else if matches!(atomic, TAtomic::Object(_)) {
            let mut countable = TNamedObject::new(atom("Countable"));
            countable.add_intersection_type(atomic.clone());
            countable_types.push(TAtomic::Object(TObject::Named(countable)));

            redundant = false;
        } else if let TAtomic::Iterable(iterable) = atomic {
            if iterable.key_type.is_array_key() || iterable.key_type.is_int() || iterable.key_type.is_any_string() {
                countable_types.push(TAtomic::Array(TArray::Keyed(TKeyedArray::new_with_parameters(
                    Arc::new(iterable.get_key_type().clone()),
                    Arc::new(iterable.get_value_type().clone()),
                ))));
            }

            let mut object = TNamedObject::new(atom("Traversable"))
                .with_type_parameters(Some(vec![iterable.get_key_type().clone(), iterable.get_value_type().clone()]));

            object.add_intersection_type(TAtomic::Object(TObject::Named(TNamedObject::new(atom("Countable")))));

            countable_types.push(TAtomic::Object(TObject::Named(object)));
            redundant = false;
        } else {
            redundant = false;
        }
    }

    if redundant || countable_types.is_empty() {
        // every type was removed, this is an impossible assertion
        if let Some(key) = key
            && let Some(span) = span
        {
            let old_var_type_atom = existing_var_type.get_id();

            trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, redundant, negated, span);
        }

        if countable_types.is_empty() {
            return get_never();
        }
    }

    existing_var_type.clone_with_types(countable_types)
}

#[inline]
fn reconcile_less_than(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    negated: bool,
    span: Option<&Span>,
    value: i64,
) -> TUnion {
    reconcile_integer_comparison(
        context,
        assertion,
        existing_var_type,
        key,
        negated,
        span,
        value,
        true,  // is_less_than
        false, // or_equal
    )
}

#[inline]
fn reconcile_less_than_or_equal(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    negated: bool,
    span: Option<&Span>,
    value: i64,
) -> TUnion {
    reconcile_integer_comparison(
        context,
        assertion,
        existing_var_type,
        key,
        negated,
        span,
        value,
        true, // is_less_than
        true, // or_equal
    )
}

#[inline]
fn reconcile_greater_than(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    negated: bool,
    span: Option<&Span>,
    value: i64,
) -> TUnion {
    reconcile_integer_comparison(
        context,
        assertion,
        existing_var_type,
        key,
        negated,
        span,
        value,
        false, // is_less_than
        false, // or_equal
    )
}

#[inline]
fn reconcile_greater_than_or_equal(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    negated: bool,
    span: Option<&Span>,
    value: i64,
) -> TUnion {
    reconcile_integer_comparison(
        context,
        assertion,
        existing_var_type,
        key,
        negated,
        span,
        value,
        false, // is_less_than
        true,  // or_equal
    )
}

fn reconcile_integer_comparison(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    negated: bool,
    span: Option<&Span>,
    value: i64,
    is_less_than: bool,
    or_equal: bool,
) -> TUnion {
    let old_var_type_atom = existing_var_type.get_id();

    let existing_var_types = existing_var_type.types.as_ref();
    let mut existing_var_type = existing_var_type.clone();

    let mut redundant = true;

    for atomic in existing_var_types {
        if is_less_than
            && value == 0
            && let TAtomic::Null | TAtomic::Scalar(TScalar::Bool(TBool { value: Some(false) })) = &atomic
        {
            existing_var_type.remove_type(atomic);
        }

        let TAtomic::Scalar(TScalar::Integer(integer)) = atomic else {
            redundant = false;
            continue;
        };

        existing_var_type.remove_type(atomic);

        if integer.is_unspecified() {
            redundant = false;

            if is_less_than {
                existing_var_type.types.to_mut().push(TAtomic::Scalar(TScalar::Integer(TInteger::To(if or_equal {
                    value
                } else {
                    value.saturating_sub(1)
                }))));
            } else {
                existing_var_type.types.to_mut().push(TAtomic::Scalar(TScalar::Integer(TInteger::From(if or_equal {
                    value
                } else {
                    value.saturating_add(1)
                }))));
            }
        } else {
            let new_integer = match (is_less_than, or_equal) {
                (true, false) => integer.to_less_than(value),
                (true, true) => integer.to_less_than_or_equal(value),
                (false, false) => integer.to_greater_than(value),
                (false, true) => integer.to_greater_than_or_equal(value),
            };

            if let Some(new_integer) = new_integer {
                if new_integer != *integer {
                    redundant = false;
                }

                existing_var_type.types.to_mut().push(TAtomic::Scalar(TScalar::Integer(new_integer)));
            } else {
                redundant = false;
            }
        }
    }

    if redundant || existing_var_type.types.is_empty() {
        if let Some(key) = key
            && let Some(span) = span
        {
            trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, redundant, negated, span);
        }

        if existing_var_type.types.is_empty() {
            return get_never();
        }
    }

    existing_var_type
}

fn reconcile_array_access(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    negated: bool,
    span: Option<&Span>,
    allow_int_key: bool,
) -> TUnion {
    let mut new_var_type = existing_var_type.clone();

    if new_var_type.is_mixed() {
        let mut result = get_mixed_keyed_array();
        result.set_possibly_undefined(
            new_var_type.possibly_undefined(),
            Some(new_var_type.possibly_undefined_from_try()),
        );
        return result;
    }

    if new_var_type.has_template() {
        return new_var_type;
    }

    new_var_type.types.to_mut().retain(|atomic| {
        (allow_int_key && atomic.is_array_accessible_with_int_or_string_key())
            || (!allow_int_key && atomic.is_array_accessible_with_string_key())
    });

    if new_var_type.types.is_empty() {
        // every type was removed, this is an impossible assertion
        if let Some(key) = key
            && let Some(span) = span
        {
            let old_var_type_atom = existing_var_type.get_id();

            trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, false, negated, span);
        }

        if new_var_type.types.is_empty() {
            return get_never();
        }
    }

    new_var_type
}

fn reconcile_in_array(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    negated: bool,
    span: Option<&Span>,
    typed_value: &TUnion,
) -> TUnion {
    let intersection = intersect_union_types(typed_value, existing_var_type, context.codebase);

    if let Some(intersection) = intersection {
        return intersection;
    }

    if let Some(key) = key
        && let Some(span) = span
    {
        trigger_issue_for_impossible(context, existing_var_type.get_id(), key, assertion, true, negated, span);
    }

    get_mixed()
}

fn reconcile_has_array_key(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    key_name: &ArrayKey,
    negated: bool,
    span: Option<&Span>,
) -> TUnion {
    let mut did_remove_type = existing_var_type.possibly_undefined();
    let mut new_var_type = existing_var_type.clone();
    let mut acceptable_types = vec![];
    let existing_var_types = new_var_type.types.to_mut().drain(..).collect::<Vec<_>>();

    for mut atomic in existing_var_types {
        match &mut atomic {
            TAtomic::Array(TArray::Keyed(TKeyedArray { known_items, parameters, non_empty, .. })) => {
                did_remove_type = true;
                if let Some(known_items) = known_items {
                    if let Some(known_item) = known_items.get_mut(key_name) {
                        if known_item.0 {
                            *non_empty = true;
                            *known_item = (false, known_item.1.clone());
                        }
                    } else if let Some((_, value_param)) = parameters {
                        *non_empty = true;
                        known_items.insert(*key_name, (false, (**value_param).clone()));
                    } else {
                        continue;
                    }
                } else if let Some((key_param, value_param)) = parameters {
                    if union_comparator::can_expression_types_be_identical(
                        context.codebase,
                        &key_name.to_general_union(),
                        key_param.as_ref(),
                        false,
                        false,
                    ) {
                        *non_empty = true;
                        *known_items = Some(BTreeMap::from([(*key_name, (false, (**value_param).clone()))]));
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }

                acceptable_types.push(atomic);
            }
            TAtomic::Array(TArray::List(TList { known_elements, element_type, non_empty, .. })) => {
                did_remove_type = true;
                if let ArrayKey::Integer(i) = key_name {
                    if let Some(known_elements) = known_elements {
                        if let Some(known_element) = known_elements.get_mut(&(*i as usize)) {
                            if known_element.0 {
                                *non_empty = true;
                                *known_element = (false, known_element.1.clone());
                            }
                        } else if !element_type.is_never() {
                            *non_empty = true;
                            known_elements.insert(*i as usize, (false, (**element_type).clone()));
                        } else {
                            continue;
                        }
                    } else if !element_type.is_never() {
                        *non_empty = true;
                        *known_elements = Some(BTreeMap::from([(*i as usize, (false, (**element_type).clone()))]));
                    }

                    acceptable_types.push(atomic);
                }
            }
            TAtomic::GenericParameter(TGenericParameter {
                parameter_name,
                defining_entity,
                intersection_types,
                constraint,
            }) => {
                if constraint.is_mixed() {
                    acceptable_types.push(atomic);
                } else {
                    let acceptable_atomic = TAtomic::GenericParameter(TGenericParameter {
                        constraint: Arc::new(reconcile_has_array_key(
                            context, assertion, constraint, None, key_name, negated, None,
                        )),
                        parameter_name: *parameter_name,
                        defining_entity: *defining_entity,
                        intersection_types: intersection_types.clone(),
                    });

                    acceptable_types.push(acceptable_atomic);
                }
                did_remove_type = true;
            }
            TAtomic::Variable { .. } => {
                did_remove_type = true;
                acceptable_types.push(atomic);
            }
            TAtomic::Mixed(_) => {
                did_remove_type = true;
                acceptable_types.push(atomic);
            }
            TAtomic::Object(TObject::Named(_)) => {
                did_remove_type = true;
                acceptable_types.push(atomic);
            }
            _ => {
                did_remove_type = true;
            }
        }
    }

    // every type was removed, this is an impossible assertion
    if let Some(key) = key
        && let Some(span) = span
        && (!did_remove_type || acceptable_types.is_empty())
    {
        let old_var_type_atom = existing_var_type.get_id();

        trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, !did_remove_type, negated, span);
    }

    if acceptable_types.is_empty() {
        return get_never();
    }

    new_var_type.types = Cow::Owned(acceptable_types);
    new_var_type
}

fn reconcile_has_nonnull_entry_for_key(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    key_name: &ArrayKey,
    negated: bool,
    span: Option<&Span>,
) -> TUnion {
    let mut new_var_type = existing_var_type.clone();

    let existing_var_types = new_var_type.types.to_mut().drain(..).collect::<Vec<_>>();

    let mut acceptable_types = vec![];

    for mut atomic in existing_var_types {
        match &mut atomic {
            TAtomic::Array(TArray::Keyed(TKeyedArray { known_items, parameters, .. })) => {
                if let Some(known_items) = known_items {
                    if let Some(known_item) = known_items.get_mut(key_name) {
                        let nonnull = subtract_null(context, assertion, &known_item.1, None, negated, None);

                        if known_item.0 {
                            *known_item = (false, nonnull);
                        } else if known_item.1 != nonnull {
                            known_item.1 = nonnull;
                        }
                    } else if let Some((_, value_param)) = parameters {
                        let nonnull = subtract_null(context, assertion, value_param, None, negated, None);
                        known_items.insert(*key_name, (false, nonnull));
                    } else {
                        continue;
                    }
                } else if let Some((key_param, value_param)) = parameters {
                    if union_comparator::can_expression_types_be_identical(
                        context.codebase,
                        &key_name.to_general_union(),
                        key_param,
                        false,
                        false,
                    ) {
                        let nonnull = subtract_null(context, assertion, value_param, None, negated, None);
                        *known_items = Some(BTreeMap::from([(*key_name, (false, nonnull))]));
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }

                acceptable_types.push(atomic);
            }
            TAtomic::Array(TArray::List(TList { known_elements, element_type, .. })) => {
                let ArrayKey::Integer(i) = key_name else {
                    continue;
                };

                if let Some(known_elements) = known_elements {
                    if let Some(known_element) = known_elements.get_mut(&(*i as usize)) {
                        let nonnull = subtract_null(context, assertion, &known_element.1, None, negated, None);

                        if known_element.0 {
                            *known_element = (false, nonnull);
                        } else if known_element.1 != nonnull {
                            known_element.1 = nonnull;
                        }
                    } else if !element_type.is_never() {
                        let nonnull = subtract_null(context, assertion, element_type, None, negated, None);
                        known_elements.insert(*i as usize, (false, nonnull));
                    } else {
                        continue;
                    }
                } else if !element_type.is_never() {
                    let nonnull = subtract_null(context, assertion, element_type, None, negated, None);
                    *known_elements = Some(BTreeMap::from([(*i as usize, (false, nonnull))]));
                }

                acceptable_types.push(atomic);
            }
            TAtomic::GenericParameter(generic_parameter) => {
                if let Some(atomic) = map_concrete_generic_constraint(generic_parameter, |constraint| {
                    reconcile_has_nonnull_entry_for_key(context, assertion, constraint, None, key_name, negated, None)
                }) {
                    acceptable_types.push(atomic);
                }
            }
            TAtomic::Variable { .. } => {
                acceptable_types.push(atomic);
            }
            TAtomic::Mixed(_) => {
                // Narrow mixed to a keyed array with the specific key having a non-null value
                let mut keyed_array = get_mixed_keyed_array();
                keyed_array.types.to_mut()[0] = TAtomic::Array(TArray::Keyed(TKeyedArray {
                    known_items: Some(BTreeMap::from([(
                        *key_name,
                        (false, subtract_null(context, assertion, &get_mixed(), None, negated, None)),
                    )])),
                    parameters: Some((Arc::new(get_arraykey()), Arc::new(get_mixed()))),
                    non_empty: false,
                }));
                acceptable_types.extend(keyed_array.types.into_owned());
            }
            TAtomic::Object(TObject::Named(_)) => {
                acceptable_types.push(atomic);
            }
            TAtomic::Scalar(TScalar::String(s)) if !s.is_known_literal() => {
                if let ArrayKey::Integer(_) = key_name {
                    acceptable_types.push(atomic);
                }
            }
            _ => {}
        }
    }

    // Only report issues if all types became incompatible (truly impossible)
    // Don't report redundancy just because the type already satisfied the assertion,
    // as the check might be part of a more specific condition (e.g., is_string after isset)
    if let Some(key) = key
        && let Some(span) = span
        && acceptable_types.is_empty()
    {
        let old_var_type_atom = existing_var_type.get_id();

        trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, false, negated, span);
    }

    if acceptable_types.is_empty() {
        return get_never();
    }

    new_var_type.types = Cow::Owned(acceptable_types);
    new_var_type
}

/// Reconciles a `HasMethod` assertion.
///
/// When `method_exists($obj, 'methodName')` returns true, we know the object has that method.
/// This creates a `HasMethod` type that tracks the known method.
fn reconcile_has_method(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    method_name: Atom,
    negated: bool,
    span: Option<&Span>,
) -> TUnion {
    if existing_var_type.is_mixed() {
        return wrap_atomic(TAtomic::Object(TObject::new_has_method(method_name)));
    }

    let mut acceptable_types = Vec::new();
    let mut did_remove_type = false;

    for atomic in existing_var_type.types.as_ref() {
        match atomic {
            TAtomic::Object(TObject::Any | TObject::WithProperties(_)) => {
                acceptable_types.push(TAtomic::Object(TObject::new_has_method(method_name)));
                did_remove_type = true;
            }
            TAtomic::Object(TObject::Named(named_object)) => {
                let class_name = named_object.get_name();
                let method_id = MethodIdentifier::new(class_name, method_name);
                if context.codebase.method_identifier_exists(&method_id)
                    || context.codebase.get_declaring_method_identifier(&method_id) != method_id
                {
                    acceptable_types.push(atomic.clone());
                } else {
                    let mut new_named = named_object.clone();
                    new_named.add_intersection_type(TAtomic::Object(TObject::new_has_method(method_name)));
                    acceptable_types.push(TAtomic::Object(TObject::Named(new_named)));
                    did_remove_type = true;
                }
            }
            TAtomic::Object(TObject::Enum(_)) => {
                acceptable_types.push(atomic.clone());
            }
            TAtomic::Object(TObject::HasMethod(has_method)) => {
                let mut new_has_method = has_method.clone();
                new_has_method.add_intersection_type(TAtomic::Object(TObject::new_has_method(method_name)));
                acceptable_types.push(TAtomic::Object(TObject::HasMethod(new_has_method)));
            }
            TAtomic::Object(TObject::HasProperty(has_property)) => {
                let mut new_has_property = has_property.clone();
                new_has_property.add_intersection_type(TAtomic::Object(TObject::new_has_method(method_name)));
                acceptable_types.push(TAtomic::Object(TObject::HasProperty(new_has_property)));
            }
            TAtomic::GenericParameter(generic_parameter) => {
                did_remove_type = true;
                if let Some(atomic) = map_concrete_generic_constraint(generic_parameter, |constraint| {
                    reconcile_has_method(context, assertion, constraint, None, method_name, negated, None)
                }) {
                    acceptable_types.push(atomic);
                }
            }
            TAtomic::Variable { .. } => {
                acceptable_types.push(atomic.clone());
                did_remove_type = true;
            }
            TAtomic::Mixed(_) => {
                acceptable_types.push(TAtomic::Object(TObject::new_has_method(method_name)));
                did_remove_type = true;
            }
            _ => {
                did_remove_type = true;
            }
        }
    }

    get_acceptable_type(
        context,
        acceptable_types,
        did_remove_type,
        key,
        span,
        existing_var_type,
        assertion,
        negated,
        true,
        existing_var_type.clone(),
    )
}

/// Reconciles a `HasProperty` assertion.
///
/// When `property_exists($obj, 'propertyName')` returns true, we know the object has that property.
/// This creates a `HasProperty` type that tracks the known property.
fn reconcile_has_property(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&str>,
    property_name: Atom,
    negated: bool,
    span: Option<&Span>,
) -> TUnion {
    if existing_var_type.is_mixed() {
        return wrap_atomic(TAtomic::Object(TObject::new_has_property(property_name)));
    }

    let mut acceptable_types = Vec::new();
    let mut did_remove_type = false;

    for atomic in existing_var_type.types.as_ref() {
        match atomic {
            TAtomic::Object(TObject::Enum(_)) => {
                did_remove_type = true;
            }
            TAtomic::Object(TObject::Any | TObject::WithProperties(_)) => {
                acceptable_types.push(TAtomic::Object(TObject::new_has_property(property_name)));
                did_remove_type = true;
            }
            TAtomic::Object(TObject::Named(named_object)) => {
                let mut new_named = named_object.clone();
                new_named.add_intersection_type(TAtomic::Object(TObject::new_has_property(property_name)));
                acceptable_types.push(TAtomic::Object(TObject::Named(new_named)));
            }
            TAtomic::Object(TObject::HasMethod(has_method)) => {
                let mut new_has_method = has_method.clone();
                new_has_method.add_intersection_type(TAtomic::Object(TObject::new_has_property(property_name)));
                acceptable_types.push(TAtomic::Object(TObject::HasMethod(new_has_method)));
            }
            TAtomic::Object(TObject::HasProperty(has_property)) => {
                let mut new_has_property = has_property.clone();
                new_has_property.add_intersection_type(TAtomic::Object(TObject::new_has_property(property_name)));
                acceptable_types.push(TAtomic::Object(TObject::HasProperty(new_has_property)));
            }
            TAtomic::GenericParameter(generic_parameter) => {
                did_remove_type = true;
                if let Some(atomic) = map_concrete_generic_constraint(generic_parameter, |constraint| {
                    reconcile_has_property(context, assertion, constraint, None, property_name, negated, None)
                }) {
                    acceptable_types.push(atomic);
                }
            }
            TAtomic::Variable { .. } => {
                acceptable_types.push(atomic.clone());
                did_remove_type = true;
            }
            TAtomic::Mixed(_) => {
                acceptable_types.push(TAtomic::Object(TObject::new_has_property(property_name)));
                did_remove_type = true;
            }
            _ => {
                did_remove_type = true;
            }
        }
    }

    get_acceptable_type(
        context,
        acceptable_types,
        did_remove_type,
        key,
        span,
        existing_var_type,
        assertion,
        negated,
        true,
        existing_var_type.clone(),
    )
}

pub(crate) fn get_acceptable_type(
    context: &mut Context<'_, '_>,
    acceptable_types: Vec<TAtomic>,
    did_remove_type: bool,
    key: Option<&str>,
    span: Option<&Span>,
    existing_var_type: &TUnion,
    assertion: &Assertion,
    negated: bool,
    trigger_issue: bool,
    mut new_var_type: TUnion,
) -> TUnion {
    if trigger_issue
        && (acceptable_types.is_empty() || !did_remove_type)
        && let Some(key) = key
        && let Some(span) = span
    {
        let old_var_type_atom = existing_var_type.get_id();

        trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, !did_remove_type, negated, span);
    }

    if acceptable_types.is_empty() {
        return get_never();
    }

    new_var_type.types = Cow::Owned(acceptable_types);
    if new_var_type.has_nullsafe_null() && !new_var_type.is_nullable() {
        new_var_type.set_nullsafe_null(false);
    }

    new_var_type
}
