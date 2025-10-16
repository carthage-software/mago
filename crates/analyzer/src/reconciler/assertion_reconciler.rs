use std::collections::BTreeMap;

use mago_atom::Atom;
use mago_atom::atom;
use mago_codex::assertion::Assertion;

use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::generic::TGenericParameter;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::r#enum::TEnum;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::bool::TBool;
use mago_codex::ttype::atomic::scalar::class_like_string::TClassLikeString;
use mago_codex::ttype::atomic::scalar::float::TFloat;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_codex::ttype::atomic::scalar::string::TStringLiteral;
use mago_codex::ttype::combiner;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::atomic_comparator;
use mago_codex::ttype::comparator::atomic_comparator::is_contained_by;
use mago_codex::ttype::expander;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_mixed_maybe_from_loop;
use mago_codex::ttype::get_never;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::wrap_atomic;
use mago_span::Span;

use crate::context::Context;
use crate::reconciler::negated_assertion_reconciler;
use crate::reconciler::simple_assertion_reconciler;
use crate::reconciler::trigger_issue_for_impossible;

pub fn reconcile<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    assertion: &Assertion,
    existing_var_type: Option<&TUnion>,
    key: Option<&String>,
    inside_loop: bool,
    span: Option<&Span>,
    can_report_issues: bool,
    negated: bool,
) -> TUnion {
    let codebase = context.codebase;
    let is_negation = assertion.is_negation();

    let existing_var_type = if let Some(existing_var_type) = existing_var_type {
        existing_var_type
    } else {
        return get_missing_type(assertion, inside_loop);
    };

    let old_var_type_atom = existing_var_type.get_id();

    if is_negation {
        return negated_assertion_reconciler::reconcile(
            context,
            assertion,
            existing_var_type,
            key,
            old_var_type_atom,
            if can_report_issues { span } else { None },
            negated,
        );
    }

    if assertion.has_literal_value()
        && let Some(assertion_type) = assertion.get_type()
    {
        return handle_literal_equality(
            context,
            assertion,
            assertion_type,
            existing_var_type,
            key,
            old_var_type_atom,
            if can_report_issues { span } else { None },
            negated,
        );
    }

    let simple_asserted_type = simple_assertion_reconciler::reconcile(
        context,
        assertion,
        existing_var_type,
        key,
        if can_report_issues { span } else { None },
        negated,
        inside_loop,
    );

    if let Some(simple_asserted_type) = simple_asserted_type {
        return simple_asserted_type;
    }

    if let Some(assertion_type) = assertion.get_type() {
        let mut refined_type = refine_atomic_with_union(context, assertion_type, existing_var_type);

        if can_report_issues && let (Some(key), Some(span)) = (key, span) {
            if existing_var_type.types == refined_type.types {
                if !assertion.has_equality() && !assertion_type.is_mixed() {
                    trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, true, negated, span);
                }
            } else if refined_type.is_never() {
                trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, false, negated, span);
            }
        }

        expander::expand_union(
            codebase,
            &mut refined_type,
            &TypeExpansionOptions { expand_generic: true, ..Default::default() },
        );

        return refined_type;
    }

    get_mixed()
}

pub(crate) fn refine_atomic_with_union(
    context: &mut Context<'_, '_>,
    new_type: &TAtomic,
    existing_var_type: &TUnion,
) -> TUnion {
    if new_type.is_mixed() {
        return existing_var_type.clone();
    }

    if let TAtomic::Array(TArray::Keyed(TKeyedArray {
        known_items: Some(known_items),
        non_empty: new_type_non_empty,
        ..
    })) = new_type
    {
        let mut acceptable_atomic_types = vec![];

        for existing_var_type_part in existing_var_type.types.as_ref() {
            let TAtomic::Array(TArray::Keyed(existing_keyed_array)) = existing_var_type_part else {
                continue;
            };

            let Some(existing_known_items) = &existing_keyed_array.known_items else {
                continue;
            };

            if !known_items.keys().any(|k| existing_known_items.contains_key(k)) {
                continue;
            }

            let mut new_known_items = existing_known_items.clone();
            let mut has_non_optional = false;
            for (key, (new_is_optional, new_item_type)) in known_items {
                if let Some((is_optional, existing_item_type)) = new_known_items.get_mut(key) {
                    *is_optional = *new_is_optional;
                    *existing_item_type = new_item_type.clone();

                    if !*new_is_optional {
                        has_non_optional = true;
                    }
                }
            }

            acceptable_atomic_types.push(TAtomic::Array(TArray::Keyed(TKeyedArray {
                known_items: Some(new_known_items),
                parameters: existing_keyed_array.parameters.clone(),
                non_empty: has_non_optional || existing_keyed_array.non_empty || *new_type_non_empty,
            })));
        }

        if !acceptable_atomic_types.is_empty() {
            return TUnion::from_vec(acceptable_atomic_types);
        }
    }

    if let TAtomic::Array(TArray::List(TList {
        known_elements: Some(known_elements),
        non_empty: new_type_non_empty,
        ..
    })) = new_type
    {
        let mut acceptable_atomic_types = vec![];

        for existing_var_type_part in existing_var_type.types.as_ref() {
            let TAtomic::Array(TArray::List(existing_list)) = existing_var_type_part else {
                continue;
            };

            let Some(existing_known_elements) = &existing_list.known_elements else {
                continue;
            };

            if !known_elements.keys().any(|k| existing_known_elements.contains_key(k)) {
                continue;
            }

            let mut new_known_elements = existing_known_elements.clone();
            let mut has_non_optional = false;
            for (key, (new_is_optional, new_item_type)) in known_elements {
                if let Some((is_optional, existing_item_type)) = new_known_elements.get_mut(key) {
                    *is_optional = *new_is_optional;
                    *existing_item_type = new_item_type.clone();

                    if !*new_is_optional {
                        has_non_optional = true;
                    }
                }
            }

            acceptable_atomic_types.push(TAtomic::Array(TArray::List(TList {
                known_elements: Some(new_known_elements),
                element_type: existing_list.element_type.clone(),
                non_empty: has_non_optional || existing_list.non_empty || *new_type_non_empty,
                known_count: existing_list.known_count,
            })));
        }

        if !acceptable_atomic_types.is_empty() {
            return TUnion::from_vec(acceptable_atomic_types);
        }
    }

    let intersection_type = intersect_union_with_atomic(context, existing_var_type, new_type);
    if let Some(mut intersection_type) = intersection_type {
        for intersection_atomic_type in intersection_type.types.to_mut() {
            intersection_atomic_type.remove_placeholders();
        }

        return intersection_type;
    }

    get_never()
}

fn intersect_union_with_atomic(
    context: &mut Context<'_, '_>,
    existing_var_type: &TUnion,
    new_type: &TAtomic,
) -> Option<TUnion> {
    let mut acceptable_types = Vec::new();

    for existing_atomic in existing_var_type.types.as_ref() {
        let intersected_atomic_type = intersect_atomic_with_atomic(context, existing_atomic, new_type);
        if let Some(intersected_atomic_type) = intersected_atomic_type {
            acceptable_types.push(intersected_atomic_type);
        }
    }

    if !acceptable_types.is_empty() {
        if acceptable_types.len() > 1 {
            acceptable_types = combiner::combine(acceptable_types, context.codebase, false);
        }

        return Some(TUnion::from_vec(acceptable_types));
    }

    None
}

pub(crate) fn intersect_atomic_with_atomic(
    context: &mut Context<'_, '_>,
    first_type: &TAtomic,
    second_type: &TAtomic,
) -> Option<TAtomic> {
    let mut atomic_comparison_results = ComparisonResult::new();
    if atomic_comparator::is_contained_by(
        context.codebase,
        second_type,
        first_type,
        true,
        &mut atomic_comparison_results,
    ) {
        let second_type = if let Some(replacement) = atomic_comparison_results.replacement_atomic_type {
            replacement
        } else {
            second_type.clone()
        };

        return intersect_contained_atomic_with_another(
            context,
            first_type,
            &second_type,
            atomic_comparison_results.type_coerced.unwrap_or(false),
        );
    }

    atomic_comparison_results = ComparisonResult::new();
    if atomic_comparator::is_contained_by(
        context.codebase,
        first_type,
        second_type,
        false,
        &mut atomic_comparison_results,
    ) {
        let type_1_atomic = if let Some(replacement) = atomic_comparison_results.replacement_atomic_type {
            replacement
        } else {
            first_type.clone()
        };

        return intersect_contained_atomic_with_another(
            context,
            second_type,
            &type_1_atomic,
            atomic_comparison_results.type_coerced.unwrap_or(false),
        );
    }

    if let TAtomic::Variable { .. } = first_type {
        return Some(first_type.clone());
    }

    if let TAtomic::Variable { .. } = second_type {
        return Some(second_type.clone());
    }

    match (first_type, second_type) {
        (TAtomic::Object(TObject::Enum(first_enum)), TAtomic::Object(TObject::Enum(second_enum))) => {
            if context.codebase.is_instance_of(&first_enum.name, &second_enum.name)
                && first_enum.case == second_enum.case
            {
                return Some(first_type.clone());
            }

            return None;
        }
        (TAtomic::Object(TObject::Named(first_object)), TAtomic::Object(TObject::Named(second_object))) => {
            let first_object_name = first_object.get_name_ref();
            let second_object_name = second_object.get_name_ref();

            if (context.codebase.interface_exists(first_object_name)
                && context.codebase.is_inheritable(second_object_name))
                || (context.codebase.interface_exists(second_object_name)
                    && context.codebase.is_inheritable(first_object_name))
            {
                let mut first_type = first_type.clone();
                first_type.add_intersection_type(second_type.clone());

                return Some(first_type);
            }
        }
        (TAtomic::Array(TArray::Keyed(first_array)), TAtomic::Array(TArray::Keyed(second_array))) => {
            return intersect_keyed_arrays(context, first_array, second_array);
        }
        (TAtomic::Array(TArray::List(first_list)), TAtomic::Array(TArray::List(second_list))) => {
            return intersect_list_arrays(context, first_list, second_list);
        }
        (TAtomic::GenericParameter(TGenericParameter { constraint, .. }), TAtomic::Object(TObject::Named(_))) => {
            let new_as = intersect_union_with_atomic(context, constraint, second_type);

            if let Some(new_as) = new_as {
                let mut type_1_atomic = first_type.clone();

                if let TAtomic::GenericParameter(TGenericParameter { constraint, .. }) = &mut type_1_atomic {
                    *constraint = Box::new(new_as);
                }

                return Some(type_1_atomic);
            }
        }
        (TAtomic::Object(TObject::Named(_)), TAtomic::GenericParameter(TGenericParameter { constraint, .. })) => {
            let new_as = intersect_union_with_atomic(context, constraint, first_type);

            if let Some(new_as) = new_as {
                let mut type_2_atomic = second_type.clone();

                if let TAtomic::GenericParameter(TGenericParameter { constraint, .. }) = &mut type_2_atomic {
                    *constraint = Box::new(new_as);
                }

                return Some(type_2_atomic);
            }
        }
        _ => (),
    }

    None
}

fn intersect_list_arrays(context: &mut Context<'_, '_>, first_list: &TList, second_list: &TList) -> Option<TAtomic> {
    let element_type = intersect_union_with_union(context, &first_list.element_type, &second_list.element_type);

    match (first_list.known_elements.as_ref(), second_list.known_elements.as_ref()) {
        (Some(first_list_known_elements), Some(second_list_known_elements)) => {
            let mut second_list_known_elements = second_list_known_elements.clone();

            for (second_key, second_value) in second_list_known_elements.iter_mut() {
                if let Some(first_value) = first_list_known_elements.get(second_key) {
                    second_value.0 = second_value.0 && first_value.0;
                    second_value.1 = intersect_union_with_union(context, &first_value.1, &second_value.1)?
                } else if !first_list.element_type.is_never() {
                    second_value.1 = intersect_union_with_union(context, &first_list.element_type, &second_value.1)?
                } else {
                    // if the second list entry key is always defined, the intersection is impossible
                    if !second_value.0 {
                        return None;
                    }
                }
            }

            if let Some(element_type) = element_type {
                return Some(TAtomic::Array(TArray::List(TList {
                    known_elements: Some(second_list_known_elements),
                    element_type: Box::new(element_type),
                    non_empty: true,
                    known_count: None,
                })));
            }

            None
        }
        (None, Some(second_known_elements)) => {
            let mut second_known_elements = second_known_elements.clone();

            for (_, second_value) in second_known_elements.iter_mut() {
                second_value.1 = intersect_union_with_union(context, &second_value.1, &first_list.element_type)?
            }

            if let Some(element_type) = element_type {
                return Some(TAtomic::Array(TArray::List(TList {
                    known_elements: Some(second_known_elements),
                    element_type: Box::new(element_type),
                    non_empty: false,
                    known_count: None,
                })));
            }

            None
        }
        (Some(first_known_elements), None) => {
            let mut first_known_elements = first_known_elements.clone();

            for (_, first_value) in first_known_elements.iter_mut() {
                first_value.1 = intersect_union_with_union(context, &first_value.1, &second_list.element_type)?
            }

            if let Some(element_type) = element_type {
                return Some(TAtomic::Array(TArray::List(TList {
                    known_elements: Some(first_known_elements),
                    element_type: Box::new(element_type),
                    non_empty: false,
                    known_count: None,
                })));
            }

            None
        }
        _ => {
            if let Some(element_type) = element_type {
                return Some(TAtomic::Array(TArray::List(TList {
                    known_elements: None,
                    element_type: Box::new(element_type),
                    non_empty: true,
                    known_count: None,
                })));
            }

            None
        }
    }
}

fn intersect_keyed_arrays(
    context: &mut Context<'_, '_>,
    first_keyed_array: &TKeyedArray,
    second_keyed_array: &TKeyedArray,
) -> Option<TAtomic> {
    let parameters = match (&first_keyed_array.parameters, &second_keyed_array.parameters) {
        (Some(first_parameters), Some(second_parameters)) => {
            let key = intersect_union_with_union(context, &first_parameters.0, &second_parameters.0);
            let value = intersect_union_with_union(context, &first_parameters.1, &second_parameters.1);

            if let (Some(key), Some(value)) = (key, value) {
                Some((Box::new(key), Box::new(value)))
            } else {
                return None;
            }
        }
        _ => None,
    };

    match (&first_keyed_array.known_items, &second_keyed_array.known_items) {
        (Some(first_known_items), Some(second_known_items)) => {
            let mut intersected_items = BTreeMap::new();

            for (second_key, second_value) in second_known_items {
                if let Some(first_value) = first_known_items.get(second_key) {
                    intersected_items.insert(
                        *second_key,
                        (
                            second_value.0 && first_value.0,
                            intersect_union_with_union(context, &first_value.1, &second_value.1)?,
                        ),
                    );
                } else if let Some(first_parameters) = &first_keyed_array.parameters {
                    intersected_items.insert(
                        *second_key,
                        (second_value.0, intersect_union_with_union(context, &first_parameters.1, &second_value.1)?),
                    );
                } else if !second_value.0 {
                    return None;
                }
            }

            Some(TAtomic::Array(TArray::Keyed(TKeyedArray {
                known_items: Some(intersected_items),
                parameters,
                non_empty: true,
            })))
        }
        (None, Some(second_known_items)) => {
            let mut second_known_items = second_known_items.clone();

            for (_, second_value) in second_known_items.iter_mut() {
                if let Some(first_parameters) = &first_keyed_array.parameters {
                    second_value.1 = intersect_union_with_union(context, &second_value.1, &first_parameters.1)?;
                } else if second_keyed_array.parameters.is_none() && !second_value.0 {
                    return None;
                }
            }

            Some(TAtomic::Array(TArray::Keyed(TKeyedArray {
                known_items: Some(second_known_items),
                parameters,
                non_empty: true,
            })))
        }
        (Some(first_known_items), None) => {
            let mut first_known_items = first_known_items.clone();

            for (_, first_value) in first_known_items.iter_mut() {
                if let Some(second_params) = &second_keyed_array.parameters {
                    first_value.1 = intersect_union_with_union(context, &first_value.1, &second_params.1)?;
                } else if first_keyed_array.parameters.is_none() && !first_value.0 {
                    return None;
                }
            }

            Some(TAtomic::Array(TArray::Keyed(TKeyedArray {
                known_items: Some(first_known_items),
                parameters,
                non_empty: true,
            })))
        }
        _ => Some(TAtomic::Array(TArray::Keyed(TKeyedArray { known_items: None, parameters, non_empty: true }))),
    }
}

pub(crate) fn intersect_union_with_union(
    context: &mut Context<'_, '_>,
    type_1_param: &TUnion,
    type_2_param: &TUnion,
) -> Option<TUnion> {
    match (type_1_param.is_single(), type_2_param.is_single()) {
        (true, true) => {
            intersect_atomic_with_atomic(context, type_1_param.get_single(), type_2_param.get_single()).map(wrap_atomic)
        }
        (false, true) => intersect_union_with_atomic(context, type_1_param, type_2_param.get_single()),
        (true, false) => intersect_union_with_atomic(context, type_2_param, type_1_param.get_single()),
        (false, false) => {
            if type_1_param == type_2_param {
                Some(type_1_param.clone())
            } else {
                let new_types = type_2_param
                    .types
                    .iter()
                    .flat_map(|t| {
                        intersect_union_with_atomic(context, type_1_param, t).unwrap_or(get_never()).types.into_owned()
                    })
                    .collect::<Vec<_>>();

                let combined_union = TUnion::from_vec(combiner::combine(new_types, context.codebase, false));

                if combined_union.is_never() { None } else { Some(combined_union) }
            }
        }
    }
}

fn intersect_contained_atomic_with_another(
    context: &mut Context<'_, '_>,
    super_atomic: &TAtomic,
    sub_atomic: &TAtomic,
    generic_coercion: bool,
) -> Option<TAtomic> {
    if let TAtomic::Object(TObject::Enum(TEnum { case: Some(_), .. })) = sub_atomic {
        return Some(sub_atomic.clone());
    };

    let TAtomic::Object(TObject::Named(named_object)) = sub_atomic else {
        return Some(sub_atomic.clone());
    };

    if let TAtomic::Object(TObject::Named(super_named_object)) = super_atomic
        && super_named_object.get_name() == named_object.get_name()
    {
        let object_intersection = match named_object.get_type_parameters() {
            None if generic_coercion => super_named_object.get_type_parameters().map(|super_type_parameters| {
                TNamedObject::new(named_object.name).with_type_parameters(Some(super_type_parameters.to_vec()))
            }),
            _ => None,
        };

        if let Some(mut object_intersection) = object_intersection {
            let resulting_atomic = TAtomic::Object(TObject::Named(
                if let Some(intersection_types) = named_object.get_intersection_types() {
                    for intersection_type in intersection_types.iter().cloned() {
                        object_intersection.add_intersection_type(intersection_type);
                    }

                    object_intersection
                } else {
                    object_intersection
                },
            ));

            return Some(resulting_atomic);
        }
    }

    let mut first_type_atomic = super_atomic.clone();
    if let TAtomic::GenericParameter(TGenericParameter { constraint: first_type_constraint, .. }) =
        &mut first_type_atomic
        && first_type_constraint.has_object_type()
    {
        let first_type_as = intersect_union_with_atomic(context, first_type_constraint, sub_atomic);

        if let Some(first_type_as) = first_type_as {
            *first_type_constraint = Box::new(first_type_as);
        } else {
            return None;
        }

        return Some(first_type_atomic);
    }

    Some(sub_atomic.clone())
}

fn get_missing_type(assertion: &Assertion, inside_loop: bool) -> TUnion {
    if matches!(assertion, Assertion::IsIsset | Assertion::IsEqualIsset) {
        return get_mixed_maybe_from_loop(inside_loop);
    }

    if let Assertion::IsIdentical(atomic) | Assertion::IsType(atomic) = assertion {
        let mut atomic = atomic.clone();
        atomic.remove_placeholders();
        return wrap_atomic(atomic.clone());
    }

    get_mixed()
}

fn handle_literal_equality(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    assertion_type: &TAtomic,
    existing_var_type: &TUnion,
    key: Option<&String>,
    old_var_type_atom: Atom,
    span: Option<&Span>,
    negated: bool,
) -> TUnion {
    match assertion_type {
        TAtomic::Scalar(TScalar::Integer(TInteger::Literal(i))) => handle_literal_equality_with_int(
            context,
            assertion,
            *i,
            existing_var_type,
            key,
            old_var_type_atom,
            span,
            negated,
        ),
        TAtomic::Scalar(TScalar::String(TString { literal: Some(TStringLiteral::Value(assertion_str)), .. })) => {
            handle_literal_equality_with_str(
                context,
                assertion,
                assertion_str.as_ref(),
                existing_var_type,
                key,
                old_var_type_atom,
                span,
                negated,
            )
        }
        TAtomic::Scalar(TScalar::Float(TFloat { value: Some(assertion_float) })) => handle_literal_equality_with_float(
            context,
            assertion,
            (*assertion_float).into(),
            existing_var_type,
            key,
            old_var_type_atom,
            span,
            negated,
        ),
        TAtomic::Scalar(TScalar::Bool(TBool { value: Some(assertion_bool) })) => handle_literal_equality_with_bool(
            context,
            assertion,
            *assertion_bool,
            existing_var_type,
            key,
            old_var_type_atom,
            span,
            negated,
        ),
        TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Literal { value })) => {
            handle_literal_equality_with_class_string(
                context,
                assertion,
                *value,
                existing_var_type,
                key,
                old_var_type_atom,
                span,
                negated,
            )
        }
        _ => {
            unreachable!("unexpected assertion type for literal equality: {:?}", assertion_type);
        }
    }
}

fn handle_literal_equality_with_int(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    assertion_integer: i64,
    existing_var_type: &TUnion,
    key: Option<&String>,
    old_var_type_atom: Atom,
    span: Option<&Span>,
    negated: bool,
) -> TUnion {
    let literal_asserted_type = TAtomic::Scalar(TScalar::Integer(TInteger::Literal(assertion_integer)));
    let is_loose_equality = matches!(assertion, Assertion::IsEqual(_));

    if existing_var_type.has_scalar()
        || existing_var_type.has_numeric()
        || existing_var_type.has_array_key()
        || existing_var_type.has_mixed()
    {
        return if is_loose_equality { existing_var_type.clone() } else { TUnion::from_atomic(literal_asserted_type) };
    }

    for existing_var_atomic_type in existing_var_type.types.as_ref() {
        match existing_var_atomic_type {
            TAtomic::Scalar(TScalar::Integer(TInteger::Literal(existing_int)))
                if *existing_int == assertion_integer =>
            {
                if existing_var_type.is_single()
                    && let Some(key) = &key
                    && let Some(span) = span
                {
                    trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, true, negated, span);
                }

                return TUnion::from_atomic(literal_asserted_type);
            }
            TAtomic::Scalar(TScalar::Integer(_)) => return TUnion::from_atomic(literal_asserted_type),
            TAtomic::Scalar(TScalar::Float(TFloat { value: Some(float_value) }))
                if is_loose_equality && (*float_value == assertion_integer as f64) =>
            {
                return TUnion::from_atomic(existing_var_atomic_type.clone());
            }
            TAtomic::Scalar(TScalar::Bool(TBool { value: Some(false) }))
                if is_loose_equality && assertion_integer == 0 =>
            {
                return TUnion::from_atomic(existing_var_atomic_type.clone());
            }
            TAtomic::Scalar(TScalar::Bool(TBool { value: Some(true) }))
                if is_loose_equality && assertion_integer == 1 =>
            {
                return TUnion::from_atomic(existing_var_atomic_type.clone());
            }
            TAtomic::Scalar(TScalar::String(TString {
                literal: Some(TStringLiteral::Value(string_value)), ..
            })) if is_loose_equality && string_value.as_str() == assertion_integer.to_string() => {
                return TUnion::from_atomic(existing_var_atomic_type.clone());
            }
            _ => {}
        }
    }

    if is_loose_equality {
        for existing_var_atomic_type in existing_var_type.types.as_ref() {
            match existing_var_atomic_type {
                TAtomic::Scalar(TScalar::Float(TFloat { value: None })) => {
                    return TUnion::from_atomic(existing_var_atomic_type.clone());
                }
                TAtomic::Scalar(TScalar::String(TString { literal: None, .. })) => {
                    return TUnion::from_atomic(existing_var_atomic_type.clone());
                }
                TAtomic::Scalar(TScalar::Bool(TBool { value: None })) => {
                    return TUnion::from_atomic(existing_var_atomic_type.clone());
                }
                _ => {}
            }
        }
    }

    if let Some(key) = &key
        && let Some(span) = span
    {
        trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, false, negated, span);
    }

    get_never()
}

fn handle_literal_equality_with_str(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    assertion_str_val: &str,
    existing_var_type: &TUnion,
    key: Option<&String>,
    old_var_type_atom: Atom,
    span: Option<&Span>,
    negated: bool,
) -> TUnion {
    let literal_asserted_type = TAtomic::Scalar(TScalar::literal_string(atom(assertion_str_val)));
    let is_loose_equality = matches!(assertion, Assertion::IsEqual(_));

    if existing_var_type.has_scalar() || existing_var_type.has_array_key() || existing_var_type.has_mixed() {
        return if is_loose_equality { existing_var_type.clone() } else { TUnion::from_atomic(literal_asserted_type) };
    }

    for existing_var_atomic_type in existing_var_type.types.as_ref() {
        match existing_var_atomic_type {
            TAtomic::Scalar(TScalar::String(TString { literal: None | Some(TStringLiteral::Unspecified), .. })) => {
                return TUnion::from_atomic(literal_asserted_type);
            }
            TAtomic::Scalar(TScalar::String(TString {
                literal: Some(TStringLiteral::Value(existing_str)), ..
            })) if existing_str.eq(assertion_str_val) => {
                if existing_var_type.is_single()
                    && let Some(key) = &key
                    && let Some(span) = span
                {
                    trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, true, negated, span);
                }

                return TUnion::from_atomic(literal_asserted_type);
            }
            TAtomic::Scalar(TScalar::Float(TFloat { value: Some(float_value) }))
                if is_loose_equality && float_value.to_string() == assertion_str_val =>
            {
                return TUnion::from_atomic(existing_var_atomic_type.clone());
            }
            TAtomic::Scalar(TScalar::Bool(TBool { value: Some(false) }))
                if is_loose_equality && assertion_str_val.is_empty() =>
            {
                return TUnion::from_atomic(existing_var_atomic_type.clone());
            }
            TAtomic::Scalar(TScalar::Bool(TBool { value: Some(true) }))
                if is_loose_equality && assertion_str_val == "1" =>
            {
                return TUnion::from_atomic(existing_var_atomic_type.clone());
            }
            TAtomic::Scalar(TScalar::Integer(TInteger::Literal(int_value)))
                if is_loose_equality && int_value.to_string() == assertion_str_val =>
            {
                return TUnion::from_atomic(existing_var_atomic_type.clone());
            }
            _ => {}
        }
    }

    if is_loose_equality {
        for existing_var_atomic_type in existing_var_type.types.as_ref() {
            match existing_var_atomic_type {
                TAtomic::Scalar(TScalar::Float(TFloat { value: None })) => {
                    return TUnion::from_atomic(existing_var_atomic_type.clone());
                }
                TAtomic::Scalar(TScalar::Integer(TInteger::Unspecified)) => {
                    return TUnion::from_atomic(existing_var_atomic_type.clone());
                }
                _ => {}
            }
        }
    }

    if let Some(key) = &key
        && let Some(span) = span
    {
        trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, false, negated, span);
    }

    get_never()
}

fn handle_literal_equality_with_class_string(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    assertion_class_string_val: Atom,
    existing_var_type: &TUnion,
    key: Option<&String>,
    old_var_type_atom: Atom,
    span: Option<&Span>,
    negated: bool,
) -> TUnion {
    let asserted_atomic =
        TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::literal(assertion_class_string_val)));

    if existing_var_type.has_scalar() || existing_var_type.has_array_key() || existing_var_type.has_mixed() {
        return TUnion::from_atomic(asserted_atomic);
    }

    for existing_var_atomic_type in existing_var_type.types.as_ref() {
        match existing_var_atomic_type {
            TAtomic::Scalar(TScalar::String(TString { literal: None | Some(TStringLiteral::Unspecified), .. })) => {
                return TUnion::from_atomic(asserted_atomic);
            }
            TAtomic::Scalar(TScalar::ClassLikeString(class_like_string)) => {
                let constraint = match class_like_string {
                    TClassLikeString::Any { .. } => {
                        return TUnion::from_atomic(asserted_atomic);
                    }
                    TClassLikeString::Literal { value } => {
                        if value == &assertion_class_string_val {
                            if existing_var_type.is_single()
                                && let Some(key) = &key
                                && let Some(span) = span
                            {
                                trigger_issue_for_impossible(
                                    context,
                                    old_var_type_atom,
                                    key,
                                    assertion,
                                    true,
                                    negated,
                                    span,
                                );
                            }

                            return TUnion::from_atomic(asserted_atomic);
                        }

                        continue;
                    }
                    TClassLikeString::Generic { constraint, .. } => constraint.as_ref(),
                    TClassLikeString::OfType { constraint, .. } => constraint.as_ref(),
                };

                if is_contained_by(
                    context.codebase,
                    &TClassLikeString::literal(assertion_class_string_val).get_object_type(context.codebase),
                    constraint,
                    true,
                    &mut ComparisonResult::new(),
                ) {
                    if existing_var_type.is_single()
                        && let Some(key) = &key
                        && let Some(span) = span
                    {
                        trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, true, negated, span);
                    }

                    return TUnion::from_atomic(asserted_atomic);
                }
            }
            TAtomic::Scalar(TScalar::String(TString {
                literal: Some(TStringLiteral::Value(existing_str)), ..
            })) if existing_str.eq(&assertion_class_string_val) => {
                if existing_var_type.is_single()
                    && let Some(key) = &key
                    && let Some(span) = span
                {
                    trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, true, negated, span);
                }

                return TUnion::from_atomic(asserted_atomic);
            }
            _ => {}
        }
    }

    if let Some(key) = &key
        && let Some(span) = span
    {
        trigger_issue_for_impossible(context, old_var_type_atom, key, assertion, false, negated, span);
    }

    get_never()
}

fn handle_literal_equality_with_float(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    assertion_float_val: f64,
    existing_var_type: &TUnion,
    key: Option<&String>,
    old_var_type_atom: Atom,
    span: Option<&Span>,
    negated: bool,
) -> TUnion {
    let literal_asserted_type = TAtomic::Scalar(TScalar::literal_float(assertion_float_val));
    let is_loose_equality = matches!(assertion, Assertion::IsEqual(_));

    if existing_var_type.has_scalar() || existing_var_type.has_numeric() || existing_var_type.has_mixed() {
        return if is_loose_equality { existing_var_type.clone() } else { TUnion::from_atomic(literal_asserted_type) };
    }

    for existing_var_atomic_type in existing_var_type.types.as_ref() {
        match existing_var_atomic_type {
            TAtomic::Scalar(TScalar::Float(TFloat { value: None, .. })) => {
                return TUnion::from_atomic(literal_asserted_type);
            }
            TAtomic::Scalar(TScalar::Float(TFloat { value: Some(existing_float), .. })) => {
                if (existing_float.0 - assertion_float_val).abs() < f64::EPSILON {
                    if existing_var_type.is_single()
                        && let Some(k_str) = &key
                        && let Some(s_ref) = span
                    {
                        trigger_issue_for_impossible(
                            context,
                            old_var_type_atom,
                            k_str,
                            assertion,
                            true,
                            negated,
                            s_ref,
                        );
                    }
                    return TUnion::from_atomic(literal_asserted_type);
                }
            }
            TAtomic::Scalar(TScalar::Integer(TInteger::Literal(existing_int))) if is_loose_equality => {
                if (*existing_int as f64 - assertion_float_val).abs() < f64::EPSILON {
                    return TUnion::from_atomic(existing_var_atomic_type.clone());
                }
            }
            TAtomic::Scalar(TScalar::String(TString {
                literal: Some(TStringLiteral::Value(string_value)), ..
            })) if is_loose_equality => {
                if string_value.parse::<f64>().is_ok_and(|f_val| (f_val - assertion_float_val).abs() < f64::EPSILON) {
                    return TUnion::from_atomic(existing_var_atomic_type.clone());
                }
            }
            TAtomic::Scalar(TScalar::Bool(TBool { value: Some(b_val), .. })) if is_loose_equality => {
                let bool_as_f64 = if *b_val { 1.0 } else { 0.0 };
                if (bool_as_f64 - assertion_float_val).abs() < f64::EPSILON {
                    return TUnion::from_atomic(existing_var_atomic_type.clone());
                }
            }
            TAtomic::Null if is_loose_equality => {
                if (0.0 - assertion_float_val).abs() < f64::EPSILON {
                    return TUnion::from_atomic(existing_var_atomic_type.clone());
                }
            }
            _ => {}
        }
    }

    if is_loose_equality {
        for existing_var_atomic_type in existing_var_type.types.as_ref() {
            match existing_var_atomic_type {
                TAtomic::Scalar(TScalar::Integer(TInteger::Unspecified))
                | TAtomic::Scalar(TScalar::String(TString { literal: None, .. }))
                | TAtomic::Scalar(TScalar::Bool(TBool { value: None, .. })) => {
                    return TUnion::from_atomic(existing_var_atomic_type.clone());
                }
                _ => {}
            }
        }
    }

    if let Some(k_str) = &key
        && let Some(s_ref) = span
    {
        trigger_issue_for_impossible(context, old_var_type_atom, k_str, assertion, false, negated, s_ref);
    }

    get_never()
}

fn handle_literal_equality_with_bool(
    context: &mut Context<'_, '_>,
    assertion: &Assertion,
    assertion_bool_val: bool,
    existing_var_type: &TUnion,
    key: Option<&String>,
    old_var_type_atom: Atom,
    span: Option<&Span>,
    negated: bool,
) -> TUnion {
    let literal_asserted_type = TAtomic::Scalar(TScalar::Bool(TBool { value: Some(assertion_bool_val) }));
    let is_loose_equality = matches!(assertion, Assertion::IsEqual(_));

    if existing_var_type.has_scalar() || existing_var_type.has_mixed() {
        return if is_loose_equality { existing_var_type.clone() } else { TUnion::from_atomic(literal_asserted_type) };
    }

    for existing_var_atomic_type in existing_var_type.types.as_ref() {
        match existing_var_atomic_type {
            TAtomic::Scalar(TScalar::Bool(TBool { value: None, .. })) => {
                return TUnion::from_atomic(literal_asserted_type);
            }
            TAtomic::Scalar(TScalar::Bool(TBool { value: Some(existing_bool_val), .. })) => {
                if *existing_bool_val == assertion_bool_val {
                    if existing_var_type.is_single()
                        && let Some(k_str) = &key
                        && let Some(s_ref) = span
                    {
                        trigger_issue_for_impossible(
                            context,
                            old_var_type_atom,
                            k_str,
                            assertion,
                            true,
                            negated,
                            s_ref,
                        );
                    }

                    return TUnion::from_atomic(literal_asserted_type);
                }
            }
            TAtomic::Scalar(TScalar::Integer(TInteger::Literal(existing_int))) if is_loose_equality => {
                let int_as_bool = *existing_int != 0;

                if int_as_bool == assertion_bool_val {
                    return TUnion::from_atomic(existing_var_atomic_type.clone());
                }
            }
            TAtomic::Scalar(TScalar::Float(TFloat { value: Some(existing_float), .. })) if is_loose_equality => {
                let float_as_bool = (existing_float.0).abs() > f64::EPSILON;
                if float_as_bool == assertion_bool_val {
                    return TUnion::from_atomic(existing_var_atomic_type.clone());
                }
            }
            TAtomic::Scalar(TScalar::String(TString {
                literal: Some(TStringLiteral::Value(string_value)), ..
            })) if is_loose_equality => {
                let string_as_bool = !string_value.is_empty() && string_value != "0";

                if string_as_bool == assertion_bool_val {
                    return TUnion::from_atomic(existing_var_atomic_type.clone());
                }
            }
            TAtomic::Null if is_loose_equality => {
                if !assertion_bool_val {
                    return TUnion::from_atomic(existing_var_atomic_type.clone());
                }
            }
            _ => {}
        }
    }

    if is_loose_equality {
        for existing_var_atomic_type in existing_var_type.types.as_ref() {
            match existing_var_atomic_type {
                TAtomic::Scalar(TScalar::Integer(TInteger::Unspecified))
                | TAtomic::Scalar(TScalar::Float(TFloat { value: None, .. }))
                | TAtomic::Scalar(TScalar::String(TString { literal: None, .. })) => {
                    return TUnion::from_atomic(existing_var_atomic_type.clone());
                }
                _ => {}
            }
        }
    }

    if let Some(k_str) = &key
        && let Some(s_ref) = span
    {
        trigger_issue_for_impossible(context, old_var_type_atom, k_str, assertion, false, negated, s_ref);
    }

    get_never()
}
