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

    is_array_contained_by_array(codebase, input_array, container_array, inside_assertion, atomic_comparison_result)
}

pub(crate) fn is_array_contained_by_array(
    codebase: &CodebaseMetadata,
    input_array: &TArray,
    container_array: &TArray,
    inside_assertion: bool,
    atomic_comparison_result: &mut ComparisonResult,
) -> bool {
    if container_array.is_sealed() && !input_array.is_sealed() {
        return false;
    }

    if container_array.is_non_empty() && !input_array.is_non_empty() {
        return false;
    }

    if input_array.is_empty() {
        match container_array {
            TArray::List(list) => {
                if list.non_empty {
                    return false;
                }

                let Some(known_elements) = &list.known_elements else {
                    return true;
                };

                for (_, (is_optional, _)) in known_elements.iter() {
                    if !*is_optional {
                        return false;
                    }
                }
            }
            TArray::Keyed(keyed_array) => {
                if keyed_array.non_empty {
                    return false;
                }

                let Some(known_items) = &keyed_array.known_items else {
                    return true;
                };

                for (_, (is_optional, _)) in known_items.iter() {
                    if !*is_optional {
                        return false;
                    }
                }
            }
        }

        return true;
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

    if let Some(input_known_items) = &input_known_items_cow {
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
    }

    if let Some(container_known_items) = &container_known_items {
        for (container_key, (container_is_optional, container_item_value_type)) in container_known_items.iter() {
            let input_has_key = input_known_items_cow.as_ref().is_some_and(|items| items.contains_key(container_key));

            if !*container_is_optional {
                if !input_has_key {
                    if input_value_type.is_never() {
                        return false;
                    }

                    if !union_comparator::is_contained_by(
                        codebase,
                        &input_value_type,
                        container_item_value_type,
                        false,
                        false,
                        inside_assertion,
                        atomic_comparison_result,
                    ) {
                        return false;
                    }
                }
            } else if !input_has_key
                && !input_value_type.is_never()
                && !union_comparator::is_contained_by(
                    codebase,
                    &input_value_type,
                    container_item_value_type,
                    false,
                    false,
                    inside_assertion,
                    atomic_comparison_result,
                )
            {
                return false;
            }
        }
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use mago_atom::atom;

    use crate::ttype::atomic::TAtomic;
    use crate::ttype::atomic::array::TArray;
    use crate::ttype::atomic::array::key::ArrayKey;
    use crate::ttype::atomic::array::keyed::TKeyedArray;
    use crate::ttype::comparator::ComparisonResult;
    use crate::ttype::comparator::tests::assert_is_contained_by;
    use crate::ttype::comparator::tests::create_test_codebase;
    use crate::ttype::get_arraykey;
    use crate::ttype::get_int;
    use crate::ttype::get_literal_string;
    use crate::ttype::get_mixed;
    use crate::ttype::get_string;
    use crate::ttype::union::TUnion;

    fn t_keyed(arr: TKeyedArray) -> TUnion {
        TUnion::from_atomic(TAtomic::Array(TArray::Keyed(arr)))
    }

    #[test]
    fn test_sealed_array_missing_required_key_in_unsealed_container() {
        let codebase = create_test_codebase("<?php");

        // array{'foo': 'bar'}
        let input = t_keyed(TKeyedArray::new().with_known_items(BTreeMap::from([(
            ArrayKey::String(atom("foo")),
            (false, get_literal_string(atom("bar"))),
        )])));

        // array{'required_field': string, ...<array-key, mixed>}
        let container = t_keyed(
            TKeyedArray::new()
                .with_known_items(BTreeMap::from([(ArrayKey::String(atom("required_field")), (false, get_string()))]))
                .with_parameters(Box::new(get_arraykey()), Box::new(get_mixed())),
        );

        assert_is_contained_by(&codebase, &input, &container, false, &mut ComparisonResult::default());
    }

    #[test]
    fn test_sealed_subset_contained_in_superset_with_optional() {
        let codebase = create_test_codebase("<?php");
        // array{'a': string}
        let input = t_keyed(
            TKeyedArray::new().with_known_items(BTreeMap::from([(ArrayKey::String(atom("a")), (false, get_string()))])),
        );
        // array{'a': string, 'b'?: int}
        let container = t_keyed(TKeyedArray::new().with_known_items(BTreeMap::from([
            (ArrayKey::String(atom("a")), (false, get_string())),
            (ArrayKey::String(atom("b")), (true, get_int())),
        ])));
        assert_is_contained_by(&codebase, &input, &container, true, &mut ComparisonResult::default());
    }

    #[test]
    fn test_sealed_superset_not_contained_in_subset() {
        let codebase = create_test_codebase("<?php");
        // array{'a': string, 'b'?: int}
        let input = t_keyed(TKeyedArray::new().with_known_items(BTreeMap::from([
            (ArrayKey::String(atom("a")), (false, get_string())),
            (ArrayKey::String(atom("b")), (true, get_int())),
        ])));
        // array{'a': string}
        let container = t_keyed(
            TKeyedArray::new().with_known_items(BTreeMap::from([(ArrayKey::String(atom("a")), (false, get_string()))])),
        );
        assert_is_contained_by(&codebase, &input, &container, false, &mut ComparisonResult::default());
    }

    #[test]
    fn test_empty_sealed_array_contained_in_optional_shape() {
        let codebase = create_test_codebase("<?php");
        // array{}
        let input = t_keyed(TKeyedArray::new());
        // array{'a'?: string}
        let container = t_keyed(
            TKeyedArray::new().with_known_items(BTreeMap::from([(ArrayKey::String(atom("a")), (true, get_string()))])),
        );
        assert_is_contained_by(&codebase, &input, &container, true, &mut ComparisonResult::default());
    }

    #[test]
    fn test_empty_sealed_array_not_contained_in_required_shape() {
        let codebase = create_test_codebase("<?php");
        // array{}
        let input = t_keyed(TKeyedArray::new());
        // array{'a': string}
        let container = t_keyed(
            TKeyedArray::new().with_known_items(BTreeMap::from([(ArrayKey::String(atom("a")), (false, get_string()))])),
        );
        assert_is_contained_by(&codebase, &input, &container, false, &mut ComparisonResult::default());
    }

    #[test]
    fn test_optional_property_does_not_satisfy_required() {
        let codebase = create_test_codebase("<?php");
        // array{'a'?: string}
        let input = t_keyed(
            TKeyedArray::new().with_known_items(BTreeMap::from([(ArrayKey::String(atom("a")), (true, get_string()))])),
        );
        // array{'a': string}
        let container = t_keyed(
            TKeyedArray::new().with_known_items(BTreeMap::from([(ArrayKey::String(atom("a")), (false, get_string()))])),
        );
        assert_is_contained_by(&codebase, &input, &container, false, &mut ComparisonResult::default());
    }

    #[test]
    fn test_unsealed_compatible_generics() {
        let codebase = create_test_codebase("<?php");
        // array<string, int>
        let input = t_keyed(TKeyedArray::new_with_parameters(Box::new(get_string()), Box::new(get_int())));
        // array<array-key, mixed>
        let container = t_keyed(TKeyedArray::new_with_parameters(Box::new(get_arraykey()), Box::new(get_mixed())));
        assert_is_contained_by(&codebase, &input, &container, true, &mut ComparisonResult::default());
    }

    #[test]
    fn test_unsealed_incompatible_value_generic() {
        let codebase = create_test_codebase("<?php");
        // array<string, int>
        let input = t_keyed(TKeyedArray::new_with_parameters(Box::new(get_string()), Box::new(get_int())));
        // array<array-key, string>
        let container = t_keyed(TKeyedArray::new_with_parameters(Box::new(get_arraykey()), Box::new(get_string())));
        assert_is_contained_by(&codebase, &input, &container, false, &mut ComparisonResult::default());
    }

    #[test]
    fn test_unsealed_incompatible_key_generic() {
        let codebase = create_test_codebase("<?php");
        // array<array-key, int>
        let input = t_keyed(TKeyedArray::new_with_parameters(Box::new(get_arraykey()), Box::new(get_int())));
        // array<string, int>
        let container = t_keyed(TKeyedArray::new_with_parameters(Box::new(get_string()), Box::new(get_int())));
        assert_is_contained_by(&codebase, &input, &container, false, &mut ComparisonResult::default());
    }

    #[test]
    fn test_sealed_contained_in_compatible_unsealed() {
        let codebase = create_test_codebase("<?php");
        // array{'a': string}
        let input = t_keyed(
            TKeyedArray::new().with_known_items(BTreeMap::from([(ArrayKey::String(atom("a")), (false, get_string()))])),
        );
        // array<array-key, mixed>
        let container = t_keyed(TKeyedArray::new_with_parameters(Box::new(get_arraykey()), Box::new(get_mixed())));
        assert_is_contained_by(&codebase, &input, &container, true, &mut ComparisonResult::default());
    }

    #[test]
    fn test_sealed_not_contained_in_incompatible_unsealed() {
        let codebase = create_test_codebase("<?php");
        // array{'a': string}
        let input = t_keyed(
            TKeyedArray::new().with_known_items(BTreeMap::from([(ArrayKey::String(atom("a")), (false, get_string()))])),
        );
        // array<array-key, int>
        let container = t_keyed(TKeyedArray::new_with_parameters(Box::new(get_arraykey()), Box::new(get_int())));
        assert_is_contained_by(&codebase, &input, &container, false, &mut ComparisonResult::default());
    }

    #[test]
    fn test_sealed_contained_in_compatible_mixed() {
        let codebase = create_test_codebase("<?php");
        // array{'a': string, 'b': int}
        let input = t_keyed(TKeyedArray::new().with_known_items(BTreeMap::from([
            (ArrayKey::String(atom("a")), (false, get_string())),
            (ArrayKey::String(atom("b")), (false, get_int())),
        ])));
        // array{'a': string, ...<array-key, int>}
        let container = t_keyed(
            TKeyedArray::new()
                .with_known_items(BTreeMap::from([(ArrayKey::String(atom("a")), (false, get_string()))]))
                .with_parameters(Box::new(get_arraykey()), Box::new(get_int())),
        );
        assert_is_contained_by(&codebase, &input, &container, true, &mut ComparisonResult::default());
    }

    #[test]
    fn test_sealed_not_contained_in_incompatible_mixed() {
        let codebase = create_test_codebase("<?php");
        // array{'a': string, 'b': string}
        let input = t_keyed(TKeyedArray::new().with_known_items(BTreeMap::from([
            (ArrayKey::String(atom("a")), (false, get_string())),
            (ArrayKey::String(atom("b")), (false, get_string())),
        ])));
        // array{'a': string, ...<array-key, int>}
        let container = t_keyed(
            TKeyedArray::new()
                .with_known_items(BTreeMap::from([(ArrayKey::String(atom("a")), (false, get_string()))]))
                .with_parameters(Box::new(get_arraykey()), Box::new(get_int())),
        );
        assert_is_contained_by(&codebase, &input, &container, false, &mut ComparisonResult::default());
    }

    #[test]
    fn test_unsealed_does_not_satisfy_required_sealed() {
        let codebase = create_test_codebase("<?php");
        // array<array-key, string>
        let input = t_keyed(TKeyedArray::new_with_parameters(Box::new(get_arraykey()), Box::new(get_string())));
        // array{'a': string}
        let container = t_keyed(
            TKeyedArray::new().with_known_items(BTreeMap::from([(ArrayKey::String(atom("a")), (false, get_string()))])),
        );
        assert_is_contained_by(&codebase, &input, &container, false, &mut ComparisonResult::default());
    }

    #[test]
    fn test_unsealed_does_not_satisfy_incompatible_sealed() {
        let codebase = create_test_codebase("<?php");
        // array<array-key, string>
        let input = t_keyed(TKeyedArray::new_with_parameters(Box::new(get_arraykey()), Box::new(get_string())));
        // array{'a': int}
        let container = t_keyed(
            TKeyedArray::new().with_known_items(BTreeMap::from([(ArrayKey::String(atom("a")), (false, get_int()))])),
        );
        assert_is_contained_by(&codebase, &input, &container, false, &mut ComparisonResult::default());
    }

    #[test]
    fn test_non_empty_array_is_subtype_of_array() {
        let codebase = create_test_codebase("<?php");
        // non-empty-array<array-key, mixed>
        let input = t_keyed(
            TKeyedArray::new_with_parameters(Box::new(get_arraykey()), Box::new(get_mixed())).with_non_empty(true),
        );
        // array<array-key, mixed>
        let container = t_keyed(
            TKeyedArray::new_with_parameters(Box::new(get_arraykey()), Box::new(get_mixed())).with_non_empty(false),
        );
        assert_is_contained_by(&codebase, &input, &container, true, &mut ComparisonResult::default());
    }

    #[test]
    fn test_array_is_not_subtype_of_non_empty_array() {
        let codebase = create_test_codebase("<?php");
        // array<array-key, mixed>
        let input = t_keyed(
            TKeyedArray::new_with_parameters(Box::new(get_arraykey()), Box::new(get_mixed())).with_non_empty(false),
        );
        // non-empty-array<array-key, mixed>
        let container = t_keyed(
            TKeyedArray::new_with_parameters(Box::new(get_arraykey()), Box::new(get_mixed())).with_non_empty(true),
        );
        assert_is_contained_by(&codebase, &input, &container, false, &mut ComparisonResult::default());
    }

    #[test]
    fn test_potentially_empty_array_not_contained_in_definitely_non_empty() {
        let codebase = create_test_codebase("<?php");
        // array<array-key, mixed>
        let input = t_keyed(
            TKeyedArray::new_with_parameters(Box::new(get_arraykey()), Box::new(get_mixed())).with_non_empty(false),
        );
        // array{'a': string}
        let container = t_keyed(
            TKeyedArray::new().with_known_items(BTreeMap::from([(ArrayKey::String(atom("a")), (false, get_string()))])),
        );

        assert_is_contained_by(&codebase, &input, &container, false, &mut ComparisonResult::default());
    }

    #[test]
    fn test_unsealed_input_with_conflicting_optional_key_in_container() {
        let codebase = create_test_codebase("<?php");

        // input: array{'a': string, ...<array-key, string>}
        let input = t_keyed(
            TKeyedArray::new()
                .with_known_items(BTreeMap::from([(ArrayKey::String(atom("a")), (false, get_string()))]))
                .with_parameters(Box::new(get_arraykey()), Box::new(get_string())),
        );

        // container: array{'a': string, 'b'?: int, ...<array-key, mixed>}
        let container = t_keyed(
            TKeyedArray::new()
                .with_known_items(BTreeMap::from([
                    (ArrayKey::String(atom("a")), (false, get_string())),
                    (ArrayKey::String(atom("b")), (true, get_int())),
                ]))
                .with_parameters(Box::new(get_arraykey()), Box::new(get_mixed())),
        );

        // This should be false, because input could have a key 'b' which would be a string,
        // but the container requires 'b' to be an int.
        assert_is_contained_by(&codebase, &input, &container, false, &mut ComparisonResult::default());
    }
}
