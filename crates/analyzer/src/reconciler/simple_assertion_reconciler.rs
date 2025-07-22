use std::collections::BTreeMap;

use mago_codex::assertion::Assertion;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::generic::TGenericParameter;
use mago_codex::ttype::atomic::mixed::TMixed;
use mago_codex::ttype::atomic::mixed::truthiness::TMixedTruthiness;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::atomic::resource::TResource;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::bool::TBool;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::atomic_comparator;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::get_arraykey;
use mago_codex::ttype::get_bool;
use mago_codex::ttype::get_false;
use mago_codex::ttype::get_float;
use mago_codex::ttype::get_int;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_mixed_any;
use mago_codex::ttype::get_mixed_keyed_array;
use mago_codex::ttype::get_mixed_list;
use mago_codex::ttype::get_mixed_maybe_from_loop;
use mago_codex::ttype::get_never;
use mago_codex::ttype::get_null;
use mago_codex::ttype::get_num;
use mago_codex::ttype::get_object;
use mago_codex::ttype::get_scalar;
use mago_codex::ttype::get_string;
use mago_codex::ttype::get_true;
use mago_codex::ttype::intersect_union_types;
use mago_codex::ttype::template::TemplateBound;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::wrap_atomic;
use mago_span::Span;

use super::ReconcilationContext;
use super::refine_array_key;
use crate::intersect_simple;
use crate::reconciler::simple_negated_assertion_reconciler::subtract_null;
use crate::reconciler::trigger_issue_for_impossible;

// This performs type intersections and more general reconciliations
pub(crate) fn reconcile(
    context: &mut ReconcilationContext<'_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    possibly_undefined: bool,
    key: Option<&String>,
    span: Option<&Span>,
    negated: bool,
    inside_loop: bool,
) -> Option<TUnion> {
    if let Some(assertion_type) = assertion.get_type() {
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
            TAtomic::Scalar(TScalar::Bool(bool)) if bool.is_general() => {
                return intersect_simple!(
                    TAtomic::Scalar(TScalar::Bool(_)),
                    TAtomic::Mixed(_) | TAtomic::Scalar(TScalar::Generic),
                    context,
                    get_bool(),
                    assertion,
                    existing_var_type,
                    key,
                    negated,
                    span,
                    assertion.has_equality(),
                );
            }
            TAtomic::Scalar(TScalar::Bool(bool)) if bool.is_false() => {
                return intersect_simple!(
                    TAtomic::Scalar(TScalar::Bool(bool)) if bool.is_false(),
                    atomic if
                      matches!(atomic, TAtomic::Mixed(mixed) if mixed.is_isset_from_loop() || !mixed.is_truthy())
                      || matches!(atomic, TAtomic::Scalar(TScalar::Generic) | TAtomic::Scalar(TScalar::Bool(TBool { value: None }))),
                    context,
                    get_false(),
                    assertion,
                    existing_var_type,
                    key,
                    negated,
                    span,
                    assertion.has_equality(),
                );
            }
            TAtomic::Scalar(TScalar::Bool(bool)) if bool.is_true() => {
                return intersect_simple!(
                    TAtomic::Scalar(TScalar::Bool(bool)) if bool.is_true(),
                    atomic if
                      matches!(atomic, TAtomic::Mixed(mixed) if mixed.is_isset_from_loop() || !mixed.is_falsy())
                      || matches!(atomic, TAtomic::Scalar(TScalar::Generic) | TAtomic::Scalar(TScalar::Bool(TBool { value: None }))),
                    context,
                    get_true(),
                    assertion,
                    existing_var_type,
                    key,
                    negated,
                    span,
                    assertion.has_equality(),
                );
            }
            TAtomic::Scalar(TScalar::Float(float)) if float.is_general() => {
                return intersect_simple!(
                    TAtomic::Scalar(TScalar::Float(float)) if float.is_general(),
                    TAtomic::Mixed(_) | TAtomic::Scalar(TScalar::Generic) | TAtomic::Scalar(TScalar::Number),
                    context,
                    get_float(),
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
                    resource_to_intersect,
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
            TAtomic::Array(TArray::List(TList { known_elements: None, element_type, non_empty, .. })) => {
                if element_type.is_mixed() {
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
            TAtomic::Scalar(TScalar::Number) => {
                return Some(intersect_num(
                    context,
                    assertion,
                    existing_var_type,
                    key,
                    negated,
                    span,
                    assertion.has_equality(),
                ));
            }
            TAtomic::Scalar(TScalar::String(str)) if str.is_boring() => {
                return Some(intersect_string(
                    context,
                    assertion,
                    existing_var_type,
                    key,
                    negated,
                    span,
                    assertion.has_equality(),
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
        Assertion::Truthy | Assertion::NonEmpty => {
            Some(reconcile_truthy_or_non_empty(context, assertion, existing_var_type, key, negated, span))
        }
        Assertion::IsEqualIsset | Assertion::IsIsset => Some(reconcile_isset(
            context,
            assertion,
            existing_var_type,
            possibly_undefined,
            key,
            negated,
            span,
            inside_loop,
        )),
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
        Assertion::HasArrayKey(key_name) => Some(reconcile_has_array_key(
            context,
            assertion,
            existing_var_type,
            key,
            key_name,
            negated,
            possibly_undefined,
            span,
        )),
        Assertion::HasNonnullEntryForKey(key_name) => Some(reconcile_has_nonnull_entry_for_key(
            context,
            assertion,
            existing_var_type,
            key,
            key_name,
            negated,
            possibly_undefined,
            span,
        )),
        Assertion::NonEmptyCountable(_) => {
            Some(reconcile_non_empty_countable(context, assertion, existing_var_type, key, negated, span, false))
        }
        Assertion::HasExactCount(count) => {
            Some(reconcile_exactly_countable(context, assertion, existing_var_type, key, negated, span, false, count))
        }
        Assertion::IsLessThan(less_than) => {
            Some(reconcile_less_than(context, assertion, existing_var_type, key, negated, span, less_than))
        }
        Assertion::IsGreaterThan(greater_than) => {
            Some(reconcile_greater_than(context, assertion, existing_var_type, key, negated, span, greater_than))
        }
        Assertion::IsLessThanOrEqual(less_than_or_equal) => Some(reconcile_less_than_or_equal(
            context,
            assertion,
            existing_var_type,
            key,
            negated,
            span,
            less_than_or_equal,
        )),
        Assertion::IsGreaterThanOrEqual(greater_than_or_equal) => Some(reconcile_greater_than_or_equal(
            context,
            assertion,
            existing_var_type,
            key,
            negated,
            span,
            greater_than_or_equal,
        )),
        Assertion::Countable => Some(reconcile_countable(context, assertion, existing_var_type, key, negated, span)),
        _ => None,
    }
}

pub(crate) fn intersect_null(
    context: &mut ReconcilationContext<'_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
) -> TUnion {
    if existing_var_type.is_mixed() {
        return get_null();
    }

    let mut acceptable_types = Vec::new();
    let mut did_remove_type = false;

    for atomic in &existing_var_type.types {
        match atomic {
            TAtomic::Null => {
                acceptable_types.push(TAtomic::Null);
            }
            TAtomic::Mixed(mixed) if !mixed.is_isset_from_loop() && (mixed.is_vanilla() || !mixed.is_non_null()) => {
                acceptable_types.push(TAtomic::Null);
                did_remove_type = true;
            }
            TAtomic::GenericParameter(TGenericParameter { constraint, .. }) => {
                if constraint.is_mixed() {
                    let atomic = atomic.replace_template_constraint(get_null());

                    acceptable_types.push(atomic);
                } else {
                    let atomic = atomic
                        .replace_template_constraint(intersect_null(context, assertion, constraint, None, false, None));

                    acceptable_types.push(atomic);
                }
                did_remove_type = true;
            }
            TAtomic::Variable(name) => {
                if !existing_var_type.is_nullable() {
                    if let Some(span) = span {
                        let name_str = context.interner.lookup(name);
                        if let Some((lower_bounds, _)) = context.artifacts.type_variable_bounds.get_mut(name_str) {
                            let mut bound = TemplateBound::new(get_null(), 0, None, None);
                            bound.span = Some(*span);
                            lower_bounds.push(bound);
                        }
                    }

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
        let old_var_type_string = existing_var_type.get_id(Some(context.interner));

        trigger_issue_for_impossible(context, &old_var_type_string, key, assertion, !did_remove_type, negated, span);
    }

    if !acceptable_types.is_empty() {
        return TUnion::new(acceptable_types);
    }

    get_never()
}

pub(crate) fn intersect_resource(
    context: &mut ReconcilationContext<'_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    resource_to_intersection: &TResource,
) -> TUnion {
    if existing_var_type.is_mixed() {
        return TUnion::new(vec![TAtomic::Resource(*resource_to_intersection)]);
    }

    let mut acceptable_types = Vec::new();
    let mut did_remove_type = false;

    for atomic in &existing_var_type.types {
        match atomic {
            TAtomic::Resource(existing_resource) => match (existing_resource.closed, resource_to_intersection.closed) {
                (Some(true), Some(true)) | (Some(false), Some(false)) | (None, None) | (Some(_), None) => {
                    acceptable_types.push(TAtomic::Resource(*existing_resource));
                }
                (None, Some(true) | Some(false)) => {
                    did_remove_type = true;

                    acceptable_types.push(TAtomic::Resource(*resource_to_intersection));
                }
                (Some(true), Some(false)) | (Some(false), Some(true)) => {
                    did_remove_type = true;
                }
            },
            TAtomic::Null => {
                acceptable_types.push(TAtomic::Null);
            }
            TAtomic::GenericParameter(TGenericParameter { constraint, .. }) => {
                if constraint.is_mixed() {
                    let atomic = atomic.replace_template_constraint(get_null());

                    acceptable_types.push(atomic);
                } else {
                    let atomic = atomic
                        .replace_template_constraint(intersect_null(context, assertion, constraint, None, false, None));

                    acceptable_types.push(atomic);
                }
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
        let old_var_type_string = existing_var_type.get_id(Some(context.interner));

        trigger_issue_for_impossible(context, &old_var_type_string, key, assertion, !did_remove_type, negated, span);
    }

    if !acceptable_types.is_empty() {
        return TUnion::new(acceptable_types);
    }

    get_never()
}

fn intersect_object(
    context: &mut ReconcilationContext<'_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
) -> TUnion {
    if existing_var_type.is_mixed() {
        return get_object();
    }

    let mut object_types = Vec::new();
    let mut did_remove_type = false;

    for atomic in &existing_var_type.types {
        if atomic.is_object_type() {
            object_types.push(atomic.clone());
        } else if let TAtomic::GenericParameter(TGenericParameter { constraint, .. }) = atomic {
            if constraint.is_mixed() {
                let atomic = atomic.replace_template_constraint(get_object());

                object_types.push(atomic);
            } else if constraint.has_object_type() || constraint.is_mixed() {
                let atomic = atomic.replace_template_constraint(intersect_object(
                    context,
                    assertion,
                    constraint,
                    None,
                    false,
                    None,
                    is_equality,
                ));

                object_types.push(atomic);
            }

            did_remove_type = true;
        } else {
            did_remove_type = true;
        }
    }

    if (object_types.is_empty() || (!did_remove_type && !is_equality))
        && let Some(key) = key
        && let Some(span) = span
    {
        let old_var_type_string = existing_var_type.get_id(Some(context.interner));

        trigger_issue_for_impossible(context, &old_var_type_string, key, assertion, !did_remove_type, negated, span);
    }

    if !object_types.is_empty() {
        return TUnion::new(object_types);
    }

    get_never()
}

fn intersect_array_list(
    context: &mut ReconcilationContext<'_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
    is_non_empty: bool,
) -> TUnion {
    if existing_var_type.is_mixed() {
        return wrap_atomic(if is_non_empty {
            TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(get_mixed()))))
        } else {
            TAtomic::Array(TArray::List(TList::new(Box::new(get_mixed()))))
        });
    }

    let mut acceptable_types = Vec::new();
    let mut did_remove_type = false;

    for atomic in &existing_var_type.types {
        match atomic {
            TAtomic::Array(TArray::List(_)) => {
                acceptable_types.push(atomic.clone());
            }
            TAtomic::Iterable(iterable) => {
                let element_type = iterable.get_value_type();

                acceptable_types.push(if is_non_empty {
                    TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(element_type.clone()))))
                } else {
                    TAtomic::Array(TArray::List(TList::new(Box::new(element_type.clone()))))
                });
            }
            TAtomic::GenericParameter(TGenericParameter { constraint, .. }) => {
                if constraint.is_mixed() {
                    let atomic = atomic.replace_template_constraint(get_mixed_list());

                    acceptable_types.push(atomic);
                } else {
                    let atomic = atomic.replace_template_constraint(intersect_array_list(
                        context,
                        assertion,
                        constraint,
                        None,
                        false,
                        span,
                        is_equality,
                        is_non_empty,
                    ));

                    acceptable_types.push(atomic);
                }

                did_remove_type = true;
            }
            TAtomic::Variable(name) => {
                if let Some(span) = span {
                    let name_str = context.interner.lookup(name);
                    if let Some((lower_bounds, _)) = context.artifacts.type_variable_bounds.get_mut(name_str) {
                        let mut bound = TemplateBound::new(get_mixed_list(), 0, None, None);
                        bound.span = Some(*span);
                        lower_bounds.push(bound);
                    }
                }

                acceptable_types.push(atomic.clone());
                did_remove_type = true;
            }
            TAtomic::Object(TObject::Named(_)) => {
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
            &existing_var_type.get_id(Some(context.interner)),
            key,
            assertion,
            !did_remove_type,
            negated,
            span,
        );
    }

    if !acceptable_types.is_empty() {
        return TUnion::new(acceptable_types);
    }

    get_never()
}

fn intersect_keyed_array(
    context: &mut ReconcilationContext<'_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
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

    for atomic in &existing_var_type.types {
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
            TAtomic::Iterable(iterable) => {
                let key_type = refine_array_key(iterable.get_key_type());
                let value_type = iterable.get_value_type();

                acceptable_types.push(TAtomic::Array(TArray::Keyed(TKeyedArray::new_with_parameters(
                    Box::new(key_type),
                    Box::new(value_type.clone()),
                ))));
            }
            TAtomic::GenericParameter(TGenericParameter { constraint, .. }) => {
                if constraint.is_mixed() {
                    let atomic = atomic.replace_template_constraint(get_mixed_keyed_array());

                    acceptable_types.push(atomic);
                } else {
                    let atomic = atomic.replace_template_constraint(intersect_keyed_array(
                        context,
                        assertion,
                        constraint,
                        None,
                        false,
                        None,
                        is_equality,
                    ));
                    acceptable_types.push(atomic);
                }

                did_remove_type = true;
            }
            TAtomic::Variable(name) => {
                if let Some(span) = span {
                    let name_str = context.interner.lookup(name);
                    if let Some((lower_bounds, _)) = context.artifacts.type_variable_bounds.get_mut(name_str) {
                        let mut bound = TemplateBound::new(get_mixed_keyed_array(), 0, None, None);
                        bound.span = Some(*span);
                        lower_bounds.push(bound);
                    }
                }

                acceptable_types.push(atomic.clone());
                did_remove_type = true;
            }
            TAtomic::Object(TObject::Named(_)) => {
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
            &existing_var_type.get_id(Some(context.interner)),
            key,
            assertion,
            !did_remove_type,
            negated,
            span,
        );
    }

    if !acceptable_types.is_empty() {
        return TUnion::new(acceptable_types);
    }

    get_never()
}

fn intersect_arraykey(
    context: &mut ReconcilationContext<'_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
) -> TUnion {
    if existing_var_type.is_mixed() {
        return get_arraykey();
    }

    let mut acceptable_types = Vec::new();
    let mut did_remove_type = false;

    for atomic in &existing_var_type.types {
        if atomic.is_int() || atomic.is_any_string() || matches!(atomic, TAtomic::Scalar(TScalar::ArrayKey)) {
            acceptable_types.push(atomic.clone());
        } else if matches!(atomic, TAtomic::Scalar(TScalar::Number)) {
            return get_int();
        } else {
            did_remove_type = true;
        }
    }

    if (acceptable_types.is_empty() || (!did_remove_type && !is_equality))
        && let Some(key) = key
        && let Some(span) = span
    {
        trigger_issue_for_impossible(
            context,
            &existing_var_type.get_id(Some(context.interner)),
            key,
            assertion,
            !did_remove_type,
            negated,
            span,
        );
    }

    if !acceptable_types.is_empty() {
        return TUnion::new(acceptable_types);
    }

    get_never()
}

fn intersect_num(
    context: &mut ReconcilationContext<'_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
) -> TUnion {
    if existing_var_type.is_mixed() {
        return get_num();
    }

    let mut acceptable_types = Vec::new();
    let mut did_remove_type = false;

    for atomic in &existing_var_type.types {
        if atomic.is_int() || matches!(atomic, TAtomic::Scalar(TScalar::Float(_))) {
            acceptable_types.push(atomic.clone());
        } else if matches!(atomic, TAtomic::Scalar(TScalar::ArrayKey)) {
            return get_int();
        } else {
            did_remove_type = true;
        }
    }

    if (acceptable_types.is_empty() || (!did_remove_type && !is_equality))
        && let Some(key) = key
        && let Some(span) = span
    {
        trigger_issue_for_impossible(
            context,
            &existing_var_type.get_id(Some(context.interner)),
            key,
            assertion,
            !did_remove_type,
            negated,
            span,
        );
    }

    if !acceptable_types.is_empty() {
        return TUnion::new(acceptable_types);
    }

    get_never()
}

fn intersect_string(
    context: &mut ReconcilationContext<'_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
) -> TUnion {
    let mut acceptable_types = Vec::new();
    let mut did_remove_type = false;

    for atomic in &existing_var_type.types {
        match atomic {
            TAtomic::Scalar(TScalar::String(_)) | TAtomic::Scalar(TScalar::ClassLikeString(_)) => {
                acceptable_types.push(atomic.clone());
            }
            TAtomic::Mixed(_) | TAtomic::Scalar(TScalar::Generic) | TAtomic::Scalar(TScalar::ArrayKey) => {
                return get_string();
            }
            TAtomic::GenericParameter(TGenericParameter { constraint, .. }) => {
                if constraint.is_mixed() {
                    let atomic = atomic.replace_template_constraint(get_string());

                    acceptable_types.push(atomic);
                } else {
                    let atomic = atomic.replace_template_constraint(intersect_string(
                        context,
                        assertion,
                        constraint,
                        None,
                        false,
                        None,
                        is_equality,
                    ));

                    acceptable_types.push(atomic);
                }

                did_remove_type = true;
            }
            TAtomic::Variable(name) => {
                if let Some(span) = span {
                    let name_str = context.interner.lookup(name);
                    if let Some((lower_bounds, _)) = context.artifacts.type_variable_bounds.get_mut(name_str) {
                        let mut bound = TemplateBound::new(get_string(), 0, None, None);
                        bound.span = Some(*span);
                        lower_bounds.push(bound);
                    }
                }

                acceptable_types.push(atomic.clone());
                did_remove_type = true;
            }
            TAtomic::Object(TObject::Named(_)) => {
                did_remove_type = true;
            }
            _ => {
                if atomic_comparator::is_contained_by(
                    context.codebase,
                    context.interner,
                    atomic,
                    &TAtomic::Scalar(TScalar::string()),
                    false,
                    &mut ComparisonResult::new(),
                ) {
                    acceptable_types.push(atomic.clone());

                    if let TAtomic::Object(TObject::Enum(_)) = atomic {
                        did_remove_type = true;
                    }
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
            &existing_var_type.get_id(Some(context.interner)),
            key,
            assertion,
            !did_remove_type,
            negated,
            span,
        );
    }

    if !acceptable_types.is_empty() {
        return TUnion::new(acceptable_types);
    }

    get_never()
}

fn intersect_int(
    context: &mut ReconcilationContext<'_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
) -> TUnion {
    let mut acceptable_types = Vec::new();
    let mut did_remove_type = false;

    for atomic in &existing_var_type.types {
        match atomic {
            TAtomic::Scalar(TScalar::Integer(_)) => {
                acceptable_types.push(atomic.clone());
            }
            TAtomic::Mixed(_)
            | TAtomic::Scalar(TScalar::Generic)
            | TAtomic::Scalar(TScalar::Number)
            | TAtomic::Scalar(TScalar::ArrayKey) => {
                return get_int();
            }
            TAtomic::GenericParameter(TGenericParameter { constraint, .. }) => {
                if constraint.is_mixed() {
                    let atomic = atomic.replace_template_constraint(get_int());

                    acceptable_types.push(atomic);
                } else {
                    let atomic = atomic.replace_template_constraint(intersect_int(
                        context,
                        assertion,
                        constraint,
                        None,
                        false,
                        None,
                        is_equality,
                    ));
                    acceptable_types.push(atomic);
                }

                did_remove_type = true;
            }
            TAtomic::Variable(name) => {
                if let Some(span) = span {
                    let name_str = context.interner.lookup(name);
                    if let Some((lower_bounds, _)) = context.artifacts.type_variable_bounds.get_mut(name_str) {
                        let mut bound = TemplateBound::new(get_int(), 0, None, None);
                        bound.span = Some(*span);
                        lower_bounds.push(bound);
                    }
                }

                acceptable_types.push(atomic.clone());
                did_remove_type = true;
            }
            _ => {
                if atomic_comparator::is_contained_by(
                    context.codebase,
                    context.interner,
                    atomic,
                    &TAtomic::Scalar(TScalar::int()),
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
            &existing_var_type.get_id(Some(context.interner)),
            key,
            assertion,
            !did_remove_type,
            negated,
            span,
        );
    }

    if !acceptable_types.is_empty() {
        return TUnion::new(acceptable_types);
    }

    get_never()
}

fn reconcile_truthy_or_non_empty(
    context: &mut ReconcilationContext<'_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
) -> TUnion {
    let mut did_remove_type = existing_var_type.possibly_undefined_from_try;
    let mut new_var_type = existing_var_type.clone();
    let mut acceptable_types = vec![];

    let is_non_empty_assertion = matches!(assertion, Assertion::NonEmpty);

    for atomic in new_var_type.types.drain(..) {
        if atomic.is_falsy() {
            did_remove_type = true;
        } else if !atomic.is_truthy() || new_var_type.possibly_undefined_from_try {
            did_remove_type = true;

            match atomic {
                TAtomic::GenericParameter(TGenericParameter { ref constraint, .. }) => {
                    if !constraint.is_mixed() {
                        let atomic = atomic.replace_template_constraint(reconcile_truthy_or_non_empty(
                            context, assertion, constraint, None, false, None,
                        ));

                        acceptable_types.push(atomic);
                    } else {
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
                TAtomic::Array(TArray::List(_)) => {
                    acceptable_types.push(atomic.get_non_empty_list(None));
                }
                TAtomic::Array(TArray::Keyed(_)) => {
                    acceptable_types.push(atomic.clone().make_non_empty_keyed_array());
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

    new_var_type.possibly_undefined_from_try = false;

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
    context: &mut ReconcilationContext<'_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    possibly_undefined: bool,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    inside_loop: bool,
) -> TUnion {
    let mut did_remove_type = possibly_undefined || existing_var_type.possibly_undefined_from_try;

    if possibly_undefined {
        did_remove_type = true;
    }

    let mut new_var_type = existing_var_type.clone();

    let existing_var_types = new_var_type.types.drain(..).collect::<Vec<_>>();

    let mut acceptable_types = vec![];

    for atomic in existing_var_types {
        if let TAtomic::Null = atomic {
            did_remove_type = true;
        } else if let TAtomic::Mixed(mixed) = atomic {
            if !mixed.is_non_null() {
                acceptable_types.push(TAtomic::Mixed(mixed.with_is_non_null(true)));
                did_remove_type = true;
            } else {
                acceptable_types.push(TAtomic::Mixed(mixed));
            }
        } else {
            acceptable_types.push(atomic);
        }
    }

    if !did_remove_type || acceptable_types.is_empty() {
        // every type was removed, this is an impossible assertion
        if let Some(key) = key
            && let Some(span) = span
        {
            let old_var_type_string = existing_var_type.get_id(Some(context.interner));

            trigger_issue_for_impossible(
                context,
                &old_var_type_string,
                key,
                assertion,
                !did_remove_type,
                negated,
                span,
            );
        }

        if acceptable_types.is_empty() {
            return get_never();
        }
    }

    new_var_type.possibly_undefined_from_try = false;
    new_var_type.types = acceptable_types;

    if new_var_type.is_never() {
        new_var_type.remove_type(&TAtomic::Never);
        new_var_type.types.push(TAtomic::Mixed(TMixed::maybe_isset_from_loop(inside_loop)));
    }

    new_var_type
}

fn reconcile_non_empty_countable(
    context: &mut ReconcilationContext<'_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    recursive_check: bool,
) -> TUnion {
    let mut did_remove_type = false;
    let mut new_var_type = existing_var_type.clone();
    let mut acceptable_types = vec![];

    for atomic in new_var_type.types.drain(..) {
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

    if !did_remove_type || acceptable_types.is_empty() {
        // every type was removed, this is an impossible assertion
        if let Some(key) = key
            && let Some(span) = span
            && !recursive_check
        {
            let old_var_type_string = existing_var_type.get_id(Some(context.interner));

            trigger_issue_for_impossible(
                context,
                &old_var_type_string,
                key,
                assertion,
                !did_remove_type,
                negated,
                span,
            );
        }

        if acceptable_types.is_empty() {
            return get_never();
        }
    }

    new_var_type.types = acceptable_types;
    new_var_type
}

fn reconcile_exactly_countable(
    context: &mut ReconcilationContext<'_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    recursive_check: bool,
    count: &usize,
) -> TUnion {
    let old_var_type_string = existing_var_type.get_id(Some(context.interner));

    let mut did_remove_type = false;

    let existing_var_types = &existing_var_type.types;
    let mut existing_var_type = existing_var_type.clone();

    for atomic in existing_var_types {
        if let TAtomic::Array(TArray::List(TList { non_empty, known_count, element_type, .. })) = atomic {
            let min_under_count = if let Some(known_count) = known_count { known_count < count } else { false };
            if !non_empty || min_under_count {
                if element_type.is_never() {
                    existing_var_type.remove_type(atomic);
                } else {
                    let non_empty_vec = atomic.get_non_empty_list(Some(*count));

                    existing_var_type.types.push(non_empty_vec);
                }

                did_remove_type = true;
            }
        } else if let TAtomic::Array(TArray::Keyed(TKeyedArray { non_empty, parameters, known_items, .. })) = atomic {
            if !non_empty {
                if parameters.is_none() {
                    existing_var_type.remove_type(atomic);
                } else {
                    let non_empty_dict = atomic.clone().make_non_empty_keyed_array();

                    existing_var_type.types.push(non_empty_dict);
                }

                did_remove_type = true;
            } else if let Some(known_items) = known_items {
                for (u, _) in known_items.values() {
                    if *u {
                        did_remove_type = true;
                    }
                }
            }
        }
    }

    if !did_remove_type || existing_var_type.types.is_empty() {
        // every type was removed, this is an impossible assertion
        if let Some(key) = key
            && let Some(span) = span
            && !recursive_check
        {
            trigger_issue_for_impossible(
                context,
                &old_var_type_string,
                key,
                assertion,
                !did_remove_type,
                negated,
                span,
            );
        }

        if existing_var_type.types.is_empty() {
            return get_never();
        }
    }

    existing_var_type
}

fn reconcile_countable(
    context: &mut ReconcilationContext<'_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
) -> TUnion {
    if existing_var_type.has_mixed() || existing_var_type.has_template() {
        return TUnion::new(vec![
            TAtomic::Object(TObject::Named(TNamedObject::new(context.interner.intern("Countable")))),
            TAtomic::Array(TArray::Keyed(TKeyedArray::new_with_parameters(
                Box::new(get_arraykey()),
                Box::new(get_mixed()),
            ))),
        ]);
    }

    let mut redundant = true;
    let mut countable_types = vec![];

    for atomic in &existing_var_type.types {
        if atomic.is_countable(context.codebase, context.interner) {
            countable_types.push(atomic.clone());
        } else if let TAtomic::Object(TObject::Any) = atomic {
            countable_types
                .push(TAtomic::Object(TObject::Named(TNamedObject::new(context.interner.intern("Countable")))));
            redundant = false;
        } else if matches!(atomic, TAtomic::Object(_)) {
            let mut countable = TNamedObject::new(context.interner.intern("Countable"));
            countable.add_intersection_type(atomic.clone());
            countable_types.push(TAtomic::Object(TObject::Named(countable)));

            redundant = false;
        } else if let TAtomic::Iterable(iterable) = atomic {
            if iterable.key_type.is_array_key() || iterable.key_type.is_int() || iterable.key_type.is_any_string() {
                countable_types.push(TAtomic::Array(TArray::Keyed(TKeyedArray::new_with_parameters(
                    Box::new(iterable.get_key_type().clone()),
                    Box::new(iterable.get_value_type().clone()),
                ))));
            }

            let traversable_name = context.interner.intern("Traversable");
            let countable_name = context.interner.intern("Countable");

            let mut object = TNamedObject::new(traversable_name)
                .with_type_parameters(Some(vec![iterable.get_key_type().clone(), iterable.get_value_type().clone()]));

            object.add_intersection_type(TAtomic::Object(TObject::Named(TNamedObject::new(countable_name))));

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
            let old_var_type_string = existing_var_type.get_id(Some(context.interner));

            trigger_issue_for_impossible(context, &old_var_type_string, key, assertion, redundant, negated, span);
        }

        if countable_types.is_empty() {
            return get_never();
        }
    }

    existing_var_type.clone_with_types(countable_types)
}

#[inline]
fn reconcile_less_than(
    context: &mut ReconcilationContext<'_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    value: &i64,
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
    context: &mut ReconcilationContext<'_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    value: &i64,
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
    context: &mut ReconcilationContext<'_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    value: &i64,
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
    context: &mut ReconcilationContext<'_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    value: &i64,
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
    context: &mut ReconcilationContext<'_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    value: &i64,
    is_less_than: bool,
    or_equal: bool,
) -> TUnion {
    let old_var_type_string = existing_var_type.get_id(Some(context.interner));

    let existing_var_types = &existing_var_type.types;
    let mut existing_var_type = existing_var_type.clone();

    let mut redundant = true;

    for atomic in existing_var_types {
        if is_less_than
            && *value == 0
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
                existing_var_type.types.push(TAtomic::Scalar(TScalar::Integer(TInteger::To(if or_equal {
                    *value
                } else {
                    value.saturating_sub(1)
                }))));
            } else {
                existing_var_type.types.push(TAtomic::Scalar(TScalar::Integer(TInteger::From(if or_equal {
                    *value
                } else {
                    value.saturating_add(1)
                }))));
            }
        } else {
            let new_integer = match (is_less_than, or_equal) {
                (true, false) => integer.to_less_than(*value),
                (true, true) => integer.to_less_than_or_equal(*value),
                (false, false) => integer.to_greater_than(*value),
                (false, true) => integer.to_greater_than_or_equal(*value),
            };

            if let Some(new_integer) = new_integer {
                if new_integer != *integer {
                    redundant = false;
                }

                existing_var_type.types.push(TAtomic::Scalar(TScalar::Integer(new_integer)));
            } else {
                redundant = false;
            }
        }
    }

    if redundant || existing_var_type.types.is_empty() {
        if let Some(key) = key
            && let Some(span) = span
        {
            trigger_issue_for_impossible(context, &old_var_type_string, key, assertion, redundant, negated, span);
        }

        if existing_var_type.types.is_empty() {
            return get_never();
        }
    }

    existing_var_type
}

fn reconcile_array_access(
    context: &mut ReconcilationContext<'_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    allow_int_key: bool,
) -> TUnion {
    let mut new_var_type = existing_var_type.clone();

    if new_var_type.is_mixed() || new_var_type.has_template() {
        return new_var_type;
    }

    new_var_type.types.retain(|atomic| {
        (allow_int_key && atomic.is_array_accessible_with_int_or_string_key())
            || (!allow_int_key && atomic.is_array_accessible_with_string_key())
    });

    if new_var_type.types.is_empty() {
        // every type was removed, this is an impossible assertion
        if let Some(key) = key
            && let Some(span) = span
        {
            let old_var_type_string = existing_var_type.get_id(Some(context.interner));

            trigger_issue_for_impossible(context, &old_var_type_string, key, assertion, false, negated, span);
        }

        if new_var_type.types.is_empty() {
            return get_never();
        }
    }

    new_var_type
}

fn reconcile_in_array(
    context: &mut ReconcilationContext<'_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
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
        trigger_issue_for_impossible(
            context,
            &existing_var_type.get_id(Some(context.interner)),
            key,
            assertion,
            true,
            negated,
            span,
        );
    }

    get_mixed_any()
}

fn reconcile_has_array_key(
    context: &mut ReconcilationContext<'_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    key_name: &ArrayKey,
    negated: bool,
    possibly_undefined: bool,
    span: Option<&Span>,
) -> TUnion {
    let mut did_remove_type = possibly_undefined;
    let mut new_var_type = existing_var_type.clone();
    let mut acceptable_types = vec![];
    let existing_var_types = new_var_type.types.drain(..).collect::<Vec<_>>();

    for mut atomic in existing_var_types {
        match &mut atomic {
            TAtomic::Array(TArray::Keyed(TKeyedArray { known_items, parameters, .. })) => {
                if let Some(known_items) = known_items {
                    if let Some(known_item) = known_items.get_mut(key_name) {
                        if known_item.0 {
                            *known_item = (false, known_item.1.clone());
                            did_remove_type = true;
                        }
                    } else if let Some((_, value_param)) = parameters {
                        known_items.insert(key_name.clone(), (false, (**value_param).clone()));
                        did_remove_type = true;
                    } else {
                        did_remove_type = true;
                        continue;
                    }
                } else if let Some((key_param, value_param)) = parameters {
                    did_remove_type = true;

                    if union_comparator::can_expression_types_be_identical(
                        context.codebase,
                        context.interner,
                        &key_name.to_general_union(),
                        key_param.as_ref(),
                        false,
                    ) {
                        *known_items = Some(BTreeMap::from([(key_name.clone(), (false, (**value_param).clone()))]));
                    } else {
                        continue;
                    }
                } else {
                    did_remove_type = true;
                    continue;
                }

                acceptable_types.push(atomic);
            }
            TAtomic::Array(TArray::List(TList { known_elements, element_type, .. })) => {
                if let ArrayKey::Integer(i) = key_name {
                    if let Some(known_elements) = known_elements {
                        if let Some(known_element) = known_elements.get_mut(&(*i as usize)) {
                            if known_element.0 {
                                *known_element = (false, known_element.1.clone());
                                did_remove_type = true;
                            }
                        } else if !element_type.is_never() {
                            known_elements.insert(*i as usize, (false, (**element_type).clone()));
                            did_remove_type = true;
                        } else {
                            did_remove_type = true;
                            continue;
                        }
                    } else if !element_type.is_never() {
                        *known_elements = Some(BTreeMap::from([(*i as usize, (false, (**element_type).clone()))]));
                        did_remove_type = true;
                    }

                    acceptable_types.push(atomic);
                } else {
                    did_remove_type = true;
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
                        constraint: Box::new(reconcile_has_array_key(
                            context,
                            assertion,
                            constraint,
                            None,
                            key_name,
                            negated,
                            possibly_undefined,
                            None,
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

    if !did_remove_type || acceptable_types.is_empty() {
        // every type was removed, this is an impossible assertion
        if let Some(key) = key
            && let Some(span) = span
        {
            let old_var_type_string = existing_var_type.get_id(Some(context.interner));

            trigger_issue_for_impossible(
                context,
                &old_var_type_string,
                key,
                assertion,
                !did_remove_type,
                negated,
                span,
            );
        }

        if acceptable_types.is_empty() {
            return get_never();
        }
    }

    new_var_type.types = acceptable_types;

    new_var_type
}

fn reconcile_has_nonnull_entry_for_key(
    context: &mut ReconcilationContext<'_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    key_name: &ArrayKey,
    negated: bool,
    possibly_undefined: bool,
    span: Option<&Span>,
) -> TUnion {
    let mut did_remove_type = possibly_undefined;

    let mut new_var_type = existing_var_type.clone();

    let existing_var_types = new_var_type.types.drain(..).collect::<Vec<_>>();

    let mut acceptable_types = vec![];

    for mut atomic in existing_var_types {
        match &mut atomic {
            TAtomic::Array(TArray::Keyed(TKeyedArray { known_items, parameters, .. })) => {
                if let Some(known_items) = known_items {
                    if let Some(known_item) = known_items.get_mut(key_name) {
                        let nonnull = subtract_null(context, assertion, &known_item.1, None, negated, None);

                        if known_item.0 {
                            *known_item = (false, nonnull);
                            did_remove_type = true;
                        } else if known_item.1 != nonnull {
                            known_item.1 = nonnull;
                            did_remove_type = true;
                        }
                    } else if let Some((_, value_param)) = parameters {
                        let nonnull = subtract_null(context, assertion, value_param, None, negated, None);
                        known_items.insert(key_name.clone(), (false, nonnull));
                        did_remove_type = true;
                    } else {
                        did_remove_type = true;
                        continue;
                    }
                } else if let Some((key_param, value_param)) = parameters {
                    did_remove_type = true;

                    if union_comparator::can_expression_types_be_identical(
                        context.codebase,
                        context.interner,
                        &key_name.to_general_union(),
                        key_param,
                        false,
                    ) {
                        let nonnull = subtract_null(context, assertion, value_param, None, negated, None);
                        *known_items = Some(BTreeMap::from([(key_name.clone(), (false, nonnull))]));
                    } else {
                        continue;
                    }
                } else {
                    did_remove_type = true;
                    continue;
                }

                acceptable_types.push(atomic);
            }
            TAtomic::Array(TArray::List(TList { known_elements, element_type, .. })) => {
                if let ArrayKey::Integer(i) = key_name {
                    if let Some(known_elements) = known_elements {
                        if let Some(known_element) = known_elements.get_mut(&(*i as usize)) {
                            let nonnull = subtract_null(context, assertion, &known_element.1, None, negated, None);

                            if known_element.0 {
                                *known_element = (false, nonnull);
                                did_remove_type = true;
                            } else if known_element.1 != nonnull {
                                known_element.1 = nonnull;
                                did_remove_type = true;
                            }
                        } else if !element_type.is_never() {
                            let nonnull = subtract_null(context, assertion, element_type, None, negated, None);
                            known_elements.insert(*i as usize, (false, nonnull));
                            did_remove_type = true;
                        } else {
                            did_remove_type = true;
                            continue;
                        }
                    } else if !element_type.is_never() {
                        let nonnull = subtract_null(context, assertion, element_type, None, negated, None);
                        *known_elements = Some(BTreeMap::from([(*i as usize, (false, nonnull))]));
                        did_remove_type = true;
                    }

                    acceptable_types.push(atomic);
                } else {
                    did_remove_type = true;
                }
            }
            TAtomic::GenericParameter(TGenericParameter { constraint, .. }) => {
                if constraint.is_mixed() {
                    acceptable_types.push(atomic);
                } else {
                    let new_type = reconcile_has_nonnull_entry_for_key(
                        context,
                        assertion,
                        constraint,
                        None,
                        key_name,
                        negated,
                        possibly_undefined,
                        None,
                    );

                    let atomic = atomic.replace_template_constraint(new_type);

                    acceptable_types.push(atomic);
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
            TAtomic::Scalar(TScalar::String(s)) if !s.is_known_literal() => {
                if let ArrayKey::Integer(_) = key_name {
                    acceptable_types.push(atomic);
                }

                did_remove_type = true;
            }
            _ => {
                did_remove_type = true;
            }
        }
    }

    if !did_remove_type || acceptable_types.is_empty() {
        // every type was removed, this is an impossible assertion
        if let Some(key) = key
            && let Some(span) = span
        {
            let old_var_type_string = existing_var_type.get_id(Some(context.interner));

            trigger_issue_for_impossible(
                context,
                &old_var_type_string,
                key,
                assertion,
                !did_remove_type,
                negated,
                span,
            );
        }

        if acceptable_types.is_empty() {
            return get_never();
        }
    }

    new_var_type.types = acceptable_types;

    new_var_type
}

pub(crate) fn get_acceptable_type(
    context: &mut ReconcilationContext<'_>,
    acceptable_types: Vec<TAtomic>,
    did_remove_type: bool,
    key: Option<&String>,
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
        let old_var_type_string = existing_var_type.get_id(Some(context.interner));

        trigger_issue_for_impossible(context, &old_var_type_string, key, assertion, !did_remove_type, negated, span);
    }

    if acceptable_types.is_empty() {
        return get_never();
    }

    new_var_type.types = acceptable_types;
    new_var_type
}
