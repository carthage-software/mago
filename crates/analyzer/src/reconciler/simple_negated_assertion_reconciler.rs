use mago_atom::atom;
use mago_codex::assertion::Assertion;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::mixed::truthiness::TMixedTruthiness;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::atomic::resource::TResource;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::float::TFloat;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_never;
use mago_codex::ttype::get_null;
use mago_codex::ttype::intersect_union_types;
use mago_codex::ttype::union::TUnion;
use mago_span::Span;

use crate::reconciler::Context;
use crate::reconciler::map_generic_constraint;
use crate::reconciler::simple_assertion_reconciler::get_acceptable_type;
use crate::reconciler::simple_assertion_reconciler::intersect_null;
use crate::reconciler::trigger_issue_for_impossible;

pub(crate) fn reconcile(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    possibly_undefined: bool,
    key: Option<&String>,
    span: Option<&Span>,
    negated: bool,
) -> Option<TUnion> {
    let assertion_type = assertion.get_type();

    if let Some(assertion_type) = assertion_type {
        // `mixed is not T` -> `mixed`, unless `T` is null,
        // in which case it becomes `nonnull`
        if existing_var_type.is_mixed() && !assertion_type.is_null() {
            return Some(existing_var_type.clone());
        }

        match assertion_type {
            TAtomic::Object(TObject::Any) => {
                return Some(subtract_object(
                    context,
                    assertion,
                    existing_var_type,
                    key,
                    negated,
                    span,
                    assertion.has_equality(),
                ));
            }
            TAtomic::Scalar(TScalar::Bool(bool)) if bool.is_general() => {
                return Some(subtract_bool(
                    context,
                    assertion,
                    existing_var_type,
                    key,
                    negated,
                    span,
                    assertion.has_equality(),
                ));
            }
            TAtomic::Scalar(TScalar::Float(TFloat { value: None })) => {
                return Some(subtract_float(
                    context,
                    assertion,
                    existing_var_type,
                    key,
                    negated,
                    span,
                    assertion.has_equality(),
                ));
            }
            TAtomic::Scalar(TScalar::Integer(integer_to_subtract)) => {
                return Some(subtract_int(
                    context,
                    assertion,
                    existing_var_type,
                    key,
                    negated,
                    span,
                    assertion.has_equality(),
                    integer_to_subtract,
                ));
            }
            TAtomic::Scalar(TScalar::String(string)) if string.is_boring() => {
                return Some(subtract_string(
                    context,
                    assertion,
                    existing_var_type,
                    key,
                    negated,
                    span,
                    assertion.has_equality(),
                ));
            }
            TAtomic::Scalar(TScalar::ArrayKey) => {
                return Some(subtract_arraykey(
                    context,
                    assertion,
                    existing_var_type,
                    key,
                    negated,
                    span,
                    assertion.has_equality(),
                ));
            }
            TAtomic::Array(TArray::List(_)) => {
                return Some(subtract_list_array(
                    context,
                    assertion,
                    existing_var_type,
                    key,
                    negated,
                    span,
                    assertion.has_equality(),
                ));
            }
            TAtomic::Array(TArray::Keyed(TKeyedArray { known_items: None, parameters: Some(parameters), .. })) => {
                if parameters.0.is_placeholder() && parameters.1.is_placeholder() {
                    return Some(subtract_keyed_array(
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
            TAtomic::Null => {
                return Some(subtract_null(context, assertion, existing_var_type, key, negated, span));
            }
            TAtomic::Resource(resource_to_subtract) => {
                return Some(subtract_resource(
                    context,
                    assertion,
                    existing_var_type,
                    key,
                    negated,
                    span,
                    resource_to_subtract,
                ));
            }
            TAtomic::Mixed(mixed) if mixed.is_non_null() => {
                return Some(intersect_null(context, assertion, existing_var_type, key, negated, span));
            }
            TAtomic::Scalar(TScalar::Bool(bool)) if bool.is_false() => {
                return Some(subtract_false(
                    context,
                    assertion,
                    existing_var_type,
                    key,
                    negated,
                    span,
                    assertion.has_equality(),
                ));
            }
            TAtomic::Scalar(TScalar::Bool(bool)) if bool.is_true() => {
                return Some(subtract_true(
                    context,
                    assertion,
                    existing_var_type,
                    key,
                    negated,
                    span,
                    assertion.has_equality(),
                ));
            }
            _ => (),
        }
    }

    match assertion {
        Assertion::Falsy | Assertion::Empty => {
            Some(reconcile_falsy_or_empty(context, assertion, existing_var_type, key, negated, span))
        }
        Assertion::IsNotIsset => Some(reconcile_not_isset(context, existing_var_type, possibly_undefined, key, span)),
        Assertion::ArrayKeyDoesNotExist => Some(get_never()),
        Assertion::DoesNotHaveArrayKey(key_name) => {
            Some(reconcile_no_array_key(context, assertion, existing_var_type, key, span, key_name, negated))
        }
        Assertion::DoesNotHaveNonnullEntryForKey(key_name) => {
            Some(reconcile_no_nonnull_entry_for_key(existing_var_type, key_name))
        }
        Assertion::NotInArray(typed_value) => {
            Some(reconcile_not_in_array(context, assertion, existing_var_type, key, negated, span, typed_value))
        }
        Assertion::EmptyCountable => {
            Some(reconcile_empty_countable(context, assertion, existing_var_type, key, negated, span))
        }
        Assertion::DoesNotHaveExactCount(count) => {
            Some(reconcile_not_exactly_countable(context, assertion, existing_var_type, key, negated, span, count))
        }
        Assertion::NotCountable(_) => {
            let mut atomics = vec![];
            for existing_atomic in existing_var_type.types.as_ref() {
                match existing_atomic {
                    TAtomic::Array(_) => {
                        continue;
                    }
                    TAtomic::Iterable(iterable) => {
                        let mut traversable = TNamedObject::new(atom("Traversable"))
                            .with_type_parameters(Some(vec![*iterable.key_type.clone(), *iterable.value_type.clone()]));

                        if let Some(intersections) = iterable.get_intersection_types() {
                            for intersection in intersections.iter().cloned() {
                                traversable.add_intersection_type(intersection);
                            }
                        }

                        atomics.push(TAtomic::Object(TObject::Named(traversable)));
                        continue;
                    }
                    _ => {
                        atomics.push(existing_atomic.clone());
                    }
                };
            }

            let new_var_type = existing_var_type.clone_with_types(atomics);
            if new_var_type.types.is_empty() {
                return Some(get_never());
            }

            Some(new_var_type)
        }
        _ => None,
    }
}

fn subtract_object(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
) -> TUnion {
    if existing_var_type.is_mixed() {
        return existing_var_type.clone();
    }

    let mut did_remove_type = false;

    let mut new_var_type = existing_var_type.clone();

    let existing_var_types = new_var_type.types.to_mut().drain(..).collect::<Vec<_>>();

    let mut acceptable_types = vec![];

    for atomic in existing_var_types {
        if let TAtomic::GenericParameter(generic_parameter) = &atomic {
            if !is_equality && !generic_parameter.constraint.is_mixed() {
                if let Some(atomic) = map_generic_constraint(generic_parameter, |constraint| {
                    subtract_object(context, assertion, constraint, None, false, None, is_equality)
                }) {
                    acceptable_types.push(atomic);
                }
            } else {
                acceptable_types.push(atomic);
            }

            did_remove_type = true;
        } else if atomic.is_object_type() {
            did_remove_type = true;

            if is_equality {
                acceptable_types.push(atomic);
            }
        } else {
            acceptable_types.push(atomic);
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
        new_var_type,
    )
}

fn subtract_list_array(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
) -> TUnion {
    if existing_var_type.is_mixed() {
        return existing_var_type.clone();
    }

    let mut did_remove_type = false;
    let mut new_var_type = existing_var_type.clone();
    let mut acceptable_types = vec![];

    for atomic in new_var_type.types.to_mut().drain(..) {
        if let TAtomic::GenericParameter(generic_parameter) = &atomic {
            if !is_equality && !generic_parameter.constraint.is_mixed() {
                if let Some(atomic) = map_generic_constraint(generic_parameter, |constraint| {
                    subtract_list_array(context, assertion, constraint, None, false, None, is_equality)
                }) {
                    acceptable_types.push(atomic);
                }
            } else {
                acceptable_types.push(atomic);
            }

            did_remove_type = true;
        } else if let TAtomic::Array(TArray::List(_)) = atomic {
            did_remove_type = true;

            if is_equality {
                acceptable_types.push(atomic);
            }
        } else if let TAtomic::Array(TArray::Keyed(_)) = atomic {
            did_remove_type = true;
            acceptable_types.push(atomic);
        } else {
            acceptable_types.push(atomic);
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
        new_var_type,
    )
}

fn subtract_keyed_array(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
) -> TUnion {
    if existing_var_type.is_mixed() {
        return existing_var_type.clone();
    }

    let mut did_remove_type = false;
    let mut new_var_type = existing_var_type.clone();
    let mut acceptable_types = vec![];

    for atomic in new_var_type.types.to_mut().drain(..) {
        if let TAtomic::GenericParameter(generic_parameter) = &atomic {
            if !is_equality && !generic_parameter.constraint.is_mixed() {
                if let Some(atomic) = map_generic_constraint(generic_parameter, |constraint| {
                    subtract_keyed_array(context, assertion, constraint, None, false, None, is_equality)
                }) {
                    acceptable_types.push(atomic);
                }
            } else {
                acceptable_types.push(atomic);
            }

            did_remove_type = true;
        } else if let TAtomic::Array(TArray::Keyed(_)) = atomic {
            did_remove_type = true;

            if is_equality {
                acceptable_types.push(atomic);
            }
        } else {
            acceptable_types.push(atomic);
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
        new_var_type,
    )
}

fn subtract_string(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
) -> TUnion {
    if existing_var_type.is_mixed() {
        return existing_var_type.clone();
    }

    let mut did_remove_type = false;
    let mut new_var_type = existing_var_type.clone();
    let mut acceptable_types = vec![];

    for atomic in new_var_type.types.to_mut().drain(..) {
        if let TAtomic::GenericParameter(generic_parameter) = &atomic {
            did_remove_type = true;

            if !is_equality && !generic_parameter.constraint.is_mixed() {
                if let Some(atomic) = map_generic_constraint(generic_parameter, |constraint| {
                    subtract_string(context, assertion, constraint, None, false, None, is_equality)
                }) {
                    acceptable_types.push(atomic);
                }
            } else {
                acceptable_types.push(atomic);
            }
        } else if let TAtomic::Scalar(TScalar::ArrayKey) = atomic {
            did_remove_type = true;

            if !is_equality {
                acceptable_types.push(TAtomic::Scalar(TScalar::int()));
            } else {
                acceptable_types.push(atomic);
            }
        } else if let TAtomic::Scalar(TScalar::Generic) = atomic {
            did_remove_type = true;

            if !is_equality {
                acceptable_types.push(TAtomic::Scalar(TScalar::int()));
                acceptable_types.push(TAtomic::Scalar(TScalar::float()));
                acceptable_types.push(TAtomic::Scalar(TScalar::bool()));
            } else {
                acceptable_types.push(atomic);
            }
        } else if atomic.is_any_string() {
            did_remove_type = true;

            if is_equality {
                acceptable_types.push(atomic);
            }
        } else if let TAtomic::Scalar(TScalar::Numeric) = atomic {
            did_remove_type = true;

            if !is_equality {
                acceptable_types.push(TAtomic::Scalar(TScalar::int()));
                acceptable_types.push(TAtomic::Scalar(TScalar::float()));
            } else {
                acceptable_types.push(atomic);
            }
        } else {
            acceptable_types.push(atomic);
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
        new_var_type,
    )
}

fn subtract_int(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
    integer_to_subtract: &TInteger,
) -> TUnion {
    if existing_var_type.is_mixed() {
        return existing_var_type.clone();
    }

    let mut did_remove_type = false;
    let mut new_var_type = existing_var_type.clone();
    let mut acceptable_types = vec![];

    for atomic in new_var_type.types.to_mut().drain(..) {
        if let TAtomic::GenericParameter(generic_parameter) = &atomic {
            did_remove_type = true;

            if !is_equality && !generic_parameter.constraint.is_mixed() {
                if let Some(atomic) = map_generic_constraint(generic_parameter, |constraint| {
                    subtract_int(context, assertion, constraint, None, false, None, is_equality, integer_to_subtract)
                }) {
                    acceptable_types.push(atomic);
                }
            } else {
                acceptable_types.push(atomic);
            }
        } else if let TAtomic::Scalar(TScalar::ArrayKey) = atomic {
            did_remove_type = true;

            if !is_equality {
                acceptable_types.push(TAtomic::Scalar(TScalar::string()));
            } else {
                acceptable_types.push(atomic);
            }
        } else if let TAtomic::Scalar(TScalar::Generic) = atomic {
            did_remove_type = true;

            if !is_equality {
                acceptable_types.push(TAtomic::Scalar(TScalar::string()));
                acceptable_types.push(TAtomic::Scalar(TScalar::float()));
                acceptable_types.push(TAtomic::Scalar(TScalar::bool()));
            } else {
                acceptable_types.push(atomic);
            }
        } else if let TAtomic::Scalar(TScalar::Integer(existing_integer)) = atomic {
            did_remove_type = true;

            if !is_equality {
                acceptable_types.extend(
                    existing_integer
                        .difference(*integer_to_subtract, false)
                        .into_iter()
                        .map(|i| TAtomic::Scalar(TScalar::Integer(i))),
                );
            } else {
                acceptable_types.push(atomic);
            }
        } else if atomic.is_int() {
            did_remove_type = true;

            if is_equality {
                acceptable_types.push(atomic);
            }
        } else {
            acceptable_types.push(atomic);
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
        new_var_type,
    )
}

fn subtract_float(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
) -> TUnion {
    if existing_var_type.is_mixed() {
        return existing_var_type.clone();
    }

    let mut did_remove_type = false;
    let mut new_var_type = existing_var_type.clone();
    let mut acceptable_types = vec![];

    for atomic in new_var_type.types.to_mut().drain(..) {
        if let TAtomic::GenericParameter(generic_parameter) = &atomic {
            if !is_equality && !generic_parameter.constraint.is_mixed() {
                if let Some(new_atomic) = map_generic_constraint(generic_parameter, |constraint| {
                    subtract_float(context, assertion, constraint, None, false, None, is_equality)
                }) {
                    acceptable_types.push(new_atomic);
                }
            } else {
                acceptable_types.push(atomic);
            }

            did_remove_type = true;
        } else if let TAtomic::Scalar(TScalar::Generic) = atomic {
            if !is_equality {
                acceptable_types.push(TAtomic::Scalar(TScalar::string()));
                acceptable_types.push(TAtomic::Scalar(TScalar::int()));
                acceptable_types.push(TAtomic::Scalar(TScalar::bool()));
            } else {
                acceptable_types.push(atomic);
            }

            did_remove_type = true;
        } else if let TAtomic::Scalar(TScalar::Float(_)) = atomic {
            did_remove_type = true;

            if is_equality {
                acceptable_types.push(atomic);
            }
        } else {
            acceptable_types.push(atomic);
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
        new_var_type,
    )
}

fn subtract_arraykey(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
) -> TUnion {
    if existing_var_type.is_mixed() {
        return existing_var_type.clone();
    }

    let old_var_type_atom = existing_var_type.get_id();
    let mut did_remove_type = false;
    let existing_var_types = existing_var_type.types.as_ref();
    let mut existing_var_type = existing_var_type.clone();

    for atomic in existing_var_types {
        if let TAtomic::GenericParameter(generic_parameter) = atomic {
            did_remove_type = true;

            if !is_equality
                && !generic_parameter.constraint.is_mixed()
                && let Some(atomic) = map_generic_constraint(generic_parameter, |constraint| {
                    subtract_arraykey(context, assertion, constraint, None, false, None, is_equality)
                })
            {
                existing_var_type.remove_type(&atomic);
                existing_var_type.types.to_mut().push(atomic);
            }
        } else if let TAtomic::Scalar(TScalar::Generic) = atomic {
            if !is_equality {
                existing_var_type.remove_type(atomic);
                existing_var_type.types.to_mut().push(TAtomic::Scalar(TScalar::float()));
                existing_var_type.types.to_mut().push(TAtomic::Scalar(TScalar::bool()));
            }

            did_remove_type = true;
        } else if atomic.is_int() || atomic.is_any_string() || matches!(atomic, TAtomic::Scalar(TScalar::ArrayKey)) {
            did_remove_type = true;

            if !is_equality {
                existing_var_type.remove_type(atomic);
            }
        }
    }

    if (existing_var_type.types.is_empty() || !did_remove_type)
        && let Some(key) = key
        && let Some(pos) = span
    {
        trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, !did_remove_type, negated, pos);
    }

    if existing_var_type.types.is_empty() {
        return get_never();
    }

    existing_var_type
}

fn subtract_bool(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
) -> TUnion {
    if existing_var_type.is_mixed() {
        return existing_var_type.clone();
    }

    let old_var_type_atom = existing_var_type.get_id();
    let existing_var_types = existing_var_type.types.as_ref();
    let mut did_remove_type = false;
    let mut existing_var_type = existing_var_type.clone();

    for atomic in existing_var_types {
        if let TAtomic::GenericParameter(generic_parameter) = atomic {
            if !is_equality && !generic_parameter.constraint.is_mixed() {
                if let Some(atomic) = map_generic_constraint(generic_parameter, |constraint| {
                    subtract_bool(context, assertion, constraint, None, false, None, is_equality)
                }) {
                    existing_var_type.remove_type(&atomic);
                    existing_var_type.types.to_mut().push(atomic);
                }
            } else {
                did_remove_type = true;
            }
        } else if let TAtomic::Scalar(TScalar::Generic) = atomic {
            if !is_equality {
                existing_var_type.remove_type(atomic);
                existing_var_type.types.to_mut().push(TAtomic::Scalar(TScalar::string()));
                existing_var_type.types.to_mut().push(TAtomic::Scalar(TScalar::int()));
                existing_var_type.types.to_mut().push(TAtomic::Scalar(TScalar::float()));
            }

            did_remove_type = true;
        } else if atomic.is_bool() {
            did_remove_type = true;

            if !is_equality {
                existing_var_type.remove_type(atomic);
            }
        }
    }

    if (existing_var_type.types.is_empty() || !did_remove_type)
        && let Some(key) = key
        && let Some(pos) = span
    {
        trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, !did_remove_type, negated, pos);
    }

    if existing_var_type.types.is_empty() {
        return get_never();
    }

    existing_var_type
}

pub(crate) fn subtract_null(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
) -> TUnion {
    let mut did_remove_type = false;
    let mut new_var_type = existing_var_type.clone();
    let mut acceptable_types = vec![];

    for atomic in new_var_type.types.to_mut().drain(..) {
        match atomic {
            TAtomic::GenericParameter(generic_parameter) => {
                did_remove_type = true;

                if let Some(atomic) = map_generic_constraint(&generic_parameter, |constraint| {
                    subtract_null(context, assertion, constraint, None, false, None)
                }) {
                    acceptable_types.push(atomic);
                }
            }
            TAtomic::Variable { .. } => {
                did_remove_type = true;
                acceptable_types.push(atomic);
            }
            TAtomic::Mixed(mixed) => {
                if mixed.is_non_null() {
                    acceptable_types.push(atomic);
                } else {
                    acceptable_types.push(TAtomic::Mixed(mixed.with_is_non_null(true)));
                    did_remove_type = true;
                }
            }
            TAtomic::Null => {
                did_remove_type = true;
            }
            _ => {
                acceptable_types.push(atomic);
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
        new_var_type,
    )
}

pub(crate) fn subtract_resource(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    resource_to_subtract: &TResource,
) -> TUnion {
    let mut did_remove_type = false;
    let mut new_var_type = existing_var_type.clone();
    let mut acceptable_types = vec![];

    for atomic in new_var_type.types.to_mut().drain(..) {
        match atomic {
            TAtomic::GenericParameter(generic_parameter) => {
                did_remove_type = true;

                if let Some(atomic) = map_generic_constraint(&generic_parameter, |constraint| {
                    subtract_resource(context, assertion, constraint, None, false, None, resource_to_subtract)
                }) {
                    acceptable_types.push(atomic);
                }
            }
            TAtomic::Resource(existing_resource) => match (existing_resource.closed, resource_to_subtract.closed) {
                (Some(true), Some(true)) | (Some(false), Some(false)) | (_, None) => {
                    did_remove_type = true;
                }
                (None, Some(true)) => {
                    acceptable_types.push(TAtomic::Resource(TResource { closed: Some(false) }));
                    did_remove_type = true;
                }
                (None, Some(false)) => {
                    acceptable_types.push(TAtomic::Resource(TResource { closed: Some(true) }));
                    did_remove_type = true;
                }
                _ => {
                    acceptable_types.push(atomic);
                }
            },
            _ => {
                acceptable_types.push(atomic);
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
        new_var_type,
    )
}

fn subtract_false(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
) -> TUnion {
    if existing_var_type.is_mixed() {
        return existing_var_type.clone();
    }

    let old_var_type_atom = existing_var_type.get_id();
    let existing_var_types = existing_var_type.types.as_ref();
    let mut did_remove_type = false;
    let mut existing_var_type = existing_var_type.clone();

    for atomic in existing_var_types {
        match atomic {
            TAtomic::GenericParameter(generic_parameter) => {
                if !is_equality
                    && let Some(atomic) = map_generic_constraint(generic_parameter, |constraint| {
                        subtract_false(context, assertion, constraint, None, false, None, is_equality)
                    })
                {
                    existing_var_type.remove_type(&atomic);
                    existing_var_type.types.to_mut().push(atomic);
                } else {
                    did_remove_type = true;
                }
            }
            TAtomic::Scalar(TScalar::Generic) => {
                existing_var_type.remove_type(atomic);
                existing_var_type.types.to_mut().push(TAtomic::Scalar(TScalar::r#true()));
                existing_var_type.types.to_mut().push(TAtomic::Scalar(TScalar::string()));
                existing_var_type.types.to_mut().push(TAtomic::Scalar(TScalar::int()));
                existing_var_type.types.to_mut().push(TAtomic::Scalar(TScalar::float()));
                did_remove_type = true;
            }
            TAtomic::Scalar(TScalar::Bool(bool)) if bool.is_general() => {
                existing_var_type.remove_type(atomic);
                existing_var_type.types.to_mut().push(TAtomic::Scalar(TScalar::r#true()));
                did_remove_type = true;
            }
            TAtomic::Scalar(TScalar::Bool(bool)) if bool.is_false() => {
                did_remove_type = true;
                existing_var_type.remove_type(atomic);
            }
            _ => {}
        }
    }

    if (existing_var_type.types.is_empty() || !did_remove_type)
        && let Some(key) = key
        && let Some(pos) = span
    {
        trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, !did_remove_type, negated, pos);
    }

    if existing_var_type.types.is_empty() {
        return get_never();
    }

    existing_var_type
}

fn subtract_true(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    is_equality: bool,
) -> TUnion {
    if existing_var_type.is_mixed() {
        return existing_var_type.clone();
    }

    let old_var_type_atom = existing_var_type.get_id();
    let existing_var_types = existing_var_type.types.as_ref();
    let mut did_remove_type = false;
    let mut existing_var_type = existing_var_type.clone();

    for atomic in existing_var_types {
        match atomic {
            TAtomic::GenericParameter(generic_parameter) => {
                if !is_equality
                    && let Some(atomic) = map_generic_constraint(generic_parameter, |constraint| {
                        subtract_true(context, assertion, constraint, None, false, None, is_equality)
                    })
                {
                    existing_var_type.remove_type(&atomic);
                    existing_var_type.types.to_mut().push(atomic);
                } else {
                    did_remove_type = true;
                }
            }
            TAtomic::Scalar(TScalar::Generic) => {
                did_remove_type = true;
                existing_var_type.remove_type(atomic);
                existing_var_type.types.to_mut().push(TAtomic::Scalar(TScalar::r#false()));
                existing_var_type.types.to_mut().push(TAtomic::Scalar(TScalar::string()));
                existing_var_type.types.to_mut().push(TAtomic::Scalar(TScalar::int()));
                existing_var_type.types.to_mut().push(TAtomic::Scalar(TScalar::float()));
            }
            TAtomic::Scalar(TScalar::Bool(bool)) if bool.is_general() => {
                existing_var_type.remove_type(atomic);
                existing_var_type.types.to_mut().push(TAtomic::Scalar(TScalar::r#false()));
                did_remove_type = true;
            }
            TAtomic::Scalar(TScalar::Bool(bool)) if bool.is_true() => {
                did_remove_type = true;

                existing_var_type.remove_type(atomic);
            }
            _ => (),
        }
    }

    if (existing_var_type.types.is_empty() || !did_remove_type)
        && let Some(key) = key
        && let Some(pos) = span
    {
        trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, !did_remove_type, negated, pos);
    }

    if existing_var_type.types.is_empty() {
        return get_never();
    }

    existing_var_type
}

fn reconcile_falsy_or_empty(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
) -> TUnion {
    let mut did_remove_type = existing_var_type.possibly_undefined_from_try;
    let mut new_var_type = existing_var_type.clone();
    let mut acceptable_types = vec![];

    let is_empty_assertion = matches!(assertion, Assertion::Empty);

    for atomic in new_var_type.types.to_mut().drain(..) {
        if atomic.is_truthy() && !new_var_type.possibly_undefined_from_try {
            did_remove_type = true;
        } else if !atomic.is_falsy() {
            did_remove_type = true;

            match atomic {
                TAtomic::GenericParameter(generic_parameter) => {
                    if let Some(atomic) = map_generic_constraint(&generic_parameter, |constraint| {
                        reconcile_falsy_or_empty(context, assertion, constraint, None, false, None)
                    }) {
                        acceptable_types.push(atomic);
                    }
                }
                TAtomic::Variable { .. } => {
                    acceptable_types.push(atomic);
                }
                TAtomic::Array(TArray::List(_)) => {
                    acceptable_types.push(TAtomic::Array(TArray::List(TList::new(Box::new(get_never())))));
                }
                TAtomic::Array(TArray::Keyed(_)) => {
                    acceptable_types.push(TAtomic::Array(TArray::Keyed(TKeyedArray::new())));
                }
                TAtomic::Mixed(mixed) => {
                    if is_empty_assertion {
                        acceptable_types.push(TAtomic::Mixed(mixed.as_empty()));
                    } else {
                        acceptable_types.push(TAtomic::Mixed(mixed.with_truthiness(TMixedTruthiness::Falsy)));
                    }
                }
                TAtomic::Scalar(TScalar::Bool(bool)) if bool.is_general() => {
                    acceptable_types.push(TAtomic::Scalar(TScalar::r#false()));
                }
                TAtomic::Scalar(TScalar::String(s)) => {
                    if !s.is_non_empty {
                        acceptable_types.push(TAtomic::Scalar(TScalar::literal_string(atom(""))));
                    }

                    if !is_empty_assertion {
                        acceptable_types.push(TAtomic::Scalar(TScalar::literal_string(atom("0"))));
                    }
                }
                TAtomic::Scalar(TScalar::Integer(i)) => {
                    if i.contains(TInteger::Literal(0)) {
                        acceptable_types.push(TAtomic::Scalar(TScalar::literal_int(0)));
                    }
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
        true,
        new_var_type,
    )
}

fn reconcile_not_isset(
    _context: &mut Context<'_, '_>,
    existing_var_type: &TUnion,
    possibly_undefined: bool,
    key: Option<&String>,
    span: Option<&Span>,
) -> TUnion {
    if possibly_undefined {
        return get_never();
    }

    if !existing_var_type.is_nullable()
        && let Some(key) = key
        && !key.contains('[')
        && (!existing_var_type.is_mixed() || existing_var_type.is_always_truthy())
    {
        if let Some(_pos) = span {
            // todo
        }

        return get_never();
    }

    get_null()
}

fn reconcile_empty_countable(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
) -> TUnion {
    let mut did_remove_type = existing_var_type.possibly_undefined_from_try;
    let mut new_var_type = existing_var_type.clone();
    let mut acceptable_types = vec![];

    for atomic in new_var_type.types.to_mut().drain(..) {
        if let TAtomic::Array(TArray::List(_)) = atomic {
            did_remove_type = true;

            if !atomic.is_truthy() {
                acceptable_types.push(TAtomic::Array(TArray::List(TList::new(Box::new(get_never())))));
            }
        } else if let TAtomic::Array(TArray::Keyed(_)) = atomic {
            did_remove_type = true;

            if !atomic.is_truthy() {
                acceptable_types.push(TAtomic::Array(TArray::Keyed(TKeyedArray::new())));
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
        true,
        new_var_type,
    )
}

fn reconcile_not_exactly_countable(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    count: &usize,
) -> TUnion {
    let mut did_remove_type = existing_var_type.possibly_undefined_from_try;
    let mut new_var_type = existing_var_type.clone();
    let mut acceptable_types = vec![];

    for atomic in new_var_type.types.to_mut().drain(..) {
        if let TAtomic::Array(TArray::List(TList { known_count, .. })) = atomic {
            if let Some(known_count) = &known_count {
                if known_count == count {
                    did_remove_type = true;
                    continue;
                }
            } else if !atomic.is_falsy() {
                did_remove_type = true;
            }
        } else if atomic.is_keyed_array() && !atomic.is_false() {
            did_remove_type = true;
        }

        acceptable_types.push(atomic);
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
        true,
        new_var_type,
    )
}

fn reconcile_not_in_array(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    negated: bool,
    span: Option<&Span>,
    typed_value: &TUnion,
) -> TUnion {
    let intersection = intersect_union_types(typed_value, existing_var_type, context.codebase);

    if intersection.is_some() {
        return existing_var_type.clone();
    }

    if let Some(key) = key
        && let Some(pos) = span
    {
        trigger_issue_for_impossible(context, existing_var_type.get_id(), key, assertion, true, negated, pos);
    }

    get_mixed()
}

fn reconcile_no_array_key(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    existing_var_type: &TUnion,
    key: Option<&String>,
    span: Option<&Span>,
    key_name: &ArrayKey,
    negated: bool,
) -> TUnion {
    let mut did_remove_type = existing_var_type.possibly_undefined_from_try;
    let mut new_var_type = existing_var_type.clone();
    let mut acceptable_types = vec![];

    for mut atomic in new_var_type.types.to_mut().drain(..) {
        match &mut atomic {
            TAtomic::Array(TArray::Keyed(TKeyedArray { known_items, parameters, .. })) => {
                if let Some(known_items) = known_items {
                    if let Some(known_item) = known_items.get(key_name) {
                        if known_item.0 {
                            known_items.remove(key_name);
                            did_remove_type = true;
                        }
                    } else if let Some((key_parameter, _)) = parameters
                        && union_comparator::can_expression_types_be_identical(
                            context.codebase,
                            &key_name.to_general_union(),
                            key_parameter,
                            false,
                            false,
                        )
                    {
                        did_remove_type = true;
                    }
                } else if let Some((key_parameter, _)) = parameters
                    && union_comparator::can_expression_types_be_identical(
                        context.codebase,
                        &key_name.to_general_union(),
                        key_parameter,
                        false,
                        false,
                    )
                {
                    did_remove_type = true;
                }

                acceptable_types.push(atomic);
            }
            TAtomic::Array(TArray::List(TList { known_elements, element_type, .. })) => {
                if let ArrayKey::Integer(i) = key_name {
                    if let Some(known_elements) = known_elements {
                        if let Some(known_element) = known_elements.get(&(*i as usize)) {
                            if known_element.0 {
                                known_elements.remove(&(*i as usize));
                                did_remove_type = true;
                            }
                        } else if !element_type.is_never() {
                            did_remove_type = true;
                        }
                    } else if !element_type.is_never() {
                        did_remove_type = true;
                    }
                }

                acceptable_types.push(atomic);
            }
            TAtomic::GenericParameter(generic_parameter) => {
                if generic_parameter.constraint.is_mixed() {
                    acceptable_types.push(atomic);
                } else if let Some(atomic) = map_generic_constraint(generic_parameter, |constraint| {
                    reconcile_no_array_key(context, assertion, constraint, None, None, key_name, negated)
                }) {
                    acceptable_types.push(atomic);
                }

                did_remove_type = true;
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
        new_var_type,
    )
}

fn reconcile_no_nonnull_entry_for_key(existing_var_type: &TUnion, key_name: &ArrayKey) -> TUnion {
    let mut existing_var_type = existing_var_type.clone();

    for atomic in existing_var_type.types.to_mut() {
        if let TAtomic::Array(TArray::Keyed(TKeyedArray { known_items, .. })) = atomic {
            let mut all_known_items_removed = false;
            if let Some(known_items_inner) = known_items {
                if let Some(known_item) = known_items_inner.remove(key_name) {
                    if !known_item.0 {
                        // impossible to not have this key
                        // todo emit issue
                    }

                    if known_items_inner.is_empty() {
                        all_known_items_removed = true;
                    }
                } else {
                    // todo emit issue
                }
            } else {
                // do nothing
            }

            if all_known_items_removed {
                *known_items = None;
            }
        }
    }

    existing_var_type
}
