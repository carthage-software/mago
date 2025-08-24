use std::borrow::Cow;

use crate::metadata::CodebaseMetadata;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::array::key::ArrayKey;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::int::TInteger;
use crate::ttype::comparator::ComparisonResult;
use crate::ttype::comparator::union_comparator;
use crate::ttype::get_never;
use crate::ttype::wrap_atomic;

pub(crate) fn is_contained_by(
    codebase: &CodebaseMetadata,
    input_type_part: &TAtomic,
    container_type_part: &TAtomic,
    inside_assertion: bool,
    atomic_comparison_result: &mut ComparisonResult,
) -> bool {
    let (TAtomic::Array(input_array), TAtomic::Array(container_array)) = (input_type_part, container_type_part) else {
        return false;
    };

    if input_array.is_empty() {
        return !container_array.is_non_empty();
    }

    if container_array.is_non_empty() && !input_array.is_non_empty() {
        return false;
    }

    let container_key_type;
    let container_value_type;
    match container_array {
        TArray::List(list) => {
            container_key_type =
                Some(Cow::Owned(wrap_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::non_negative())))));
            container_value_type = Cow::Borrowed(list.element_type.as_ref());
        }

        TArray::Keyed(keyed_array) => {
            if let Some((k, v)) = &keyed_array.parameters {
                container_key_type = Some(Cow::Borrowed(k.as_ref()));
                container_value_type = Cow::Borrowed(v.as_ref());
            } else {
                container_key_type = None;
                container_value_type = Cow::Owned(get_never());
            }
        }
    }

    let input_key_type;
    let input_value_type;
    match input_array {
        TArray::List(list) => {
            input_key_type = Some(Cow::Owned(wrap_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::non_negative())))));
            input_value_type = Cow::Borrowed(list.element_type.as_ref());
        }
        TArray::Keyed(keyed_array) => {
            if let Some((k, v)) = &keyed_array.parameters {
                if container_array.is_list() {
                    return false; // A keyed array cannot be contained by a list.
                }

                input_key_type = Some(Cow::Borrowed(k.as_ref()));
                input_value_type = Cow::Borrowed(v.as_ref());
            } else {
                input_key_type = None;
                input_value_type = Cow::Owned(get_never());
            }
        }
    }

    let input_known_items_cow = if let TArray::Keyed(input_keyed) = input_array {
        input_keyed.known_items.as_ref().map(Cow::Borrowed)
    } else if let TArray::List(input_list) = input_array {
        input_list.known_elements.as_ref().map(|elements| {
            let keyed_view = elements
                .iter()
                .map(|(index, value_tuple)| (ArrayKey::Integer(*index as i64), value_tuple.clone()))
                .collect();
            Cow::Owned(keyed_view)
        })
    } else {
        None
    };

    let container_known_items = if let TArray::Keyed(container_keyed) = container_array {
        container_keyed.known_items.as_ref().map(Cow::Borrowed)
    } else if let TArray::List(container_list) = container_array {
        container_list.known_elements.as_ref().map(|elements| {
            let keyed_view = elements
                .iter()
                .map(|(index, value_tuple)| (ArrayKey::Integer(*index as i64), value_tuple.clone()))
                .collect();
            Cow::Owned(keyed_view)
        })
    } else {
        None
    };

    if let Some(input_known_items) = input_known_items_cow {
        for (input_key, (input_is_optional, input_item_value_type)) in input_known_items.iter() {
            if let Some((container_is_optional, container_item_value_type)) =
                container_known_items.as_ref().and_then(|items| items.get(input_key))
            {
                if *input_is_optional && !*container_is_optional {
                    return false;
                }
                if !union_comparator::is_contained_by(
                    codebase,
                    input_item_value_type,
                    container_item_value_type,
                    false,
                    false,
                    inside_assertion,
                    atomic_comparison_result,
                ) {
                    return false;
                }
            } else if let (Some(ck_type), cv_type) = (&container_key_type, &container_value_type) {
                if !union_comparator::is_contained_by(
                    codebase,
                    &input_key.to_union(),
                    ck_type,
                    false,
                    false,
                    inside_assertion,
                    atomic_comparison_result,
                ) || !union_comparator::is_contained_by(
                    codebase,
                    input_item_value_type,
                    cv_type,
                    false,
                    false,
                    inside_assertion,
                    atomic_comparison_result,
                ) {
                    return false;
                }
            } else {
                return false;
            }
        }
    } else if container_known_items.is_some_and(|i| !i.iter().filter(|i| !i.1.0).count() >= 1) {
        return false;
    }

    if let (Some(input_key_type), Some(container_key_type)) = (input_key_type, container_key_type)
        && !union_comparator::is_contained_by(
            codebase,
            &input_key_type,
            &container_key_type,
            false,
            input_key_type.ignore_falsable_issues,
            inside_assertion,
            atomic_comparison_result,
        )
    {
        return false;
    }

    if !input_value_type.is_never()
        && !union_comparator::is_contained_by(
            codebase,
            &input_value_type,
            &container_value_type,
            false,
            input_value_type.ignore_falsable_issues,
            inside_assertion,
            atomic_comparison_result,
        )
    {
        return false;
    }

    true
}
