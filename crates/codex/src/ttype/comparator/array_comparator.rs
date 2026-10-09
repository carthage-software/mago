use std::borrow::Cow;
use std::collections::BTreeMap;

use crate::metadata::CodebaseMetadata;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::array::key::ArrayKey;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::int::TInteger;
use crate::ttype::comparator::ComparisonResult;
use crate::ttype::comparator::union_comparator;
use crate::ttype::get_never;
use crate::ttype::union::TUnion;
use crate::ttype::wrap_atomic;

fn has_required_known_entry(array: &TArray) -> bool {
    match array {
        TArray::List(list) => list
            .known_elements
            .as_deref()
            .is_some_and(|elements| elements.values().any(|(is_optional, _)| !*is_optional)),
        TArray::Keyed(keyed_array) => {
            keyed_array.known_items.as_deref().is_some_and(|items| items.values().any(|(is_optional, _)| !*is_optional))
        }
    }
}

fn key_and_value_types(array: &TArray) -> (Option<Cow<'_, TUnion>>, Cow<'_, TUnion>) {
    match array {
        TArray::List(list) => (
            Some(Cow::Owned(wrap_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::non_negative()))))),
            Cow::Borrowed(list.element_type.as_ref()),
        ),
        TArray::Keyed(keyed_array) => match &keyed_array.parameters {
            Some((key_type, value_type)) => {
                (Some(Cow::Borrowed(key_type.as_ref())), Cow::Borrowed(value_type.as_ref()))
            }
            None => (None, Cow::Owned(get_never())),
        },
    }
}

#[derive(Clone, Copy)]
enum KnownItems<'array> {
    Keyed(&'array BTreeMap<ArrayKey, (bool, TUnion)>),
    List(&'array BTreeMap<usize, (bool, TUnion)>),
}

impl<'array> KnownItems<'array> {
    fn get(self, key: &ArrayKey) -> Option<&'array (bool, TUnion)> {
        match self {
            Self::Keyed(items) => items.get(key),
            Self::List(elements) => {
                let ArrayKey::Integer(index) = *key else {
                    return None;
                };
                let offset = index as usize;
                if offset as i64 != index { None } else { elements.get(&offset) }
            }
        }
    }

    fn iter(self) -> impl Iterator<Item = (ArrayKey, &'array (bool, TUnion))> {
        let (items, elements) = match self {
            Self::Keyed(items) => (Some(items), None),
            Self::List(elements) => (None, Some(elements)),
        };
        // Casting large list offsets to i64 puts them before nonnegative keys.
        let negative = elements.into_iter().flat_map(|elements| {
            elements.range((std::ops::Bound::Excluded(i64::MAX as usize), std::ops::Bound::Unbounded))
        });
        let nonnegative = elements.into_iter().flat_map(|elements| elements.range(..=i64::MAX as usize));
        items
            .into_iter()
            .flat_map(|items| items.iter().map(|(key, value)| (*key, value)))
            .chain(negative.chain(nonnegative).map(|(index, value)| (ArrayKey::Integer(*index as i64), value)))
    }
}

fn known_items_view(array: &TArray) -> Option<KnownItems<'_>> {
    match array {
        TArray::Keyed(keyed_array) => keyed_array.known_items.as_deref().map(KnownItems::Keyed),
        TArray::List(list) => list.known_elements.as_deref().map(KnownItems::List),
    }
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

    if container_array.is_non_empty() && !input_array.is_non_empty() && !has_required_known_entry(input_array) {
        return false;
    }

    if input_array.is_empty() {
        return !container_array.is_non_empty() && !has_required_known_entry(container_array);
    }

    if container_array.is_list()
        && matches!(
            input_array,
            TArray::Keyed(keyed_array) if keyed_array.parameters.is_some() || keyed_array.known_non_list
        )
    {
        return false;
    }

    let (container_key_type, container_value_type) = key_and_value_types(container_array);
    let (input_key_type, input_value_type) = key_and_value_types(input_array);

    let input_known_items = known_items_view(input_array);
    let container_known_items = known_items_view(container_array);

    if let Some(input_known_items) = &input_known_items {
        for (input_key, (input_is_optional, input_item_value_type)) in input_known_items.iter() {
            if let Some((container_is_optional, container_item_value_type)) =
                container_known_items.as_ref().and_then(|items| items.get(&input_key))
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
            let input_has_key = input_known_items.as_ref().is_some_and(|items| items.get(&container_key).is_some());

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
            input_key_type.ignore_falsable_issues(),
            inside_assertion,
            atomic_comparison_result,
        )
    {
        return false;
    }

    input_value_type.is_never()
        || union_comparator::is_contained_by(
            codebase,
            &input_value_type,
            &container_value_type,
            false,
            input_value_type.ignore_falsable_issues(),
            inside_assertion,
            atomic_comparison_result,
        )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::Arc;

    use mago_word::word;

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

    #[test]
    fn known_items_borrow_types_and_keep_cast_key_order() {
        use crate::ttype::atomic::array::list::TList;

        for indices in [vec![], vec![0], vec![0, 1, 42, i64::MAX as usize, usize::MAX / 2 + 1, usize::MAX]] {
            let elements = indices
                .into_iter()
                .map(|index| (index, (index % 2 == 0, get_literal_string(word(format!("item{index}"))))))
                .collect::<BTreeMap<_, _>>();
            let expected = elements
                .iter()
                .map(|(index, value)| (ArrayKey::Integer(*index as i64), value.clone()))
                .collect::<BTreeMap<_, _>>();
            let arrays = [
                TArray::List(TList::from_known_elements(elements)),
                TArray::Keyed(TKeyedArray::new().with_known_items(expected.clone())),
            ];
            for array in &arrays {
                let Some(view) = super::known_items_view(array) else {
                    panic!("Known items should exist");
                };
                assert!(view.iter().eq(expected.iter().map(|(key, value)| (*key, value))));
                for (key, value) in view.iter() {
                    let Some(found) = view.get(&key) else {
                        panic!("An iterated key should exist");
                    };
                    assert!(std::ptr::eq(value, found));
                    let source = match array {
                        TArray::List(list) => list
                            .known_elements
                            .as_ref()
                            .and_then(|items| key.get_integer().and_then(|index| items.get(&(index as usize)))),
                        TArray::Keyed(array) => array.known_items.as_deref().and_then(|items| items.get(&key)),
                    };
                    assert!(source.is_some_and(|source| std::ptr::eq(source, value)));
                }
                for key in [
                    ArrayKey::Integer(-1),
                    ArrayKey::Integer(-2),
                    ArrayKey::Integer(41),
                    ArrayKey::String(word("0")),
                    ArrayKey::ClassLikeConstant { class_like_name: word("C"), constant_name: word("K") },
                ] {
                    assert_eq!(view.get(&key), expected.get(&key));
                }
            }
        }
    }

    fn t_keyed(arr: TKeyedArray) -> TUnion {
        TUnion::from_atomic(TAtomic::Array(TArray::Keyed(arr)))
    }

    #[test]
    fn test_sealed_array_missing_required_key_in_unsealed_container() {
        let codebase = create_test_codebase("<?php");

        // array{'foo': 'bar'}
        let input = t_keyed(TKeyedArray::new().with_known_items(BTreeMap::from([(
            ArrayKey::String(word("foo")),
            (false, get_literal_string(word("bar"))),
        )])));

        // array{'required_field': string, ...<array-key, mixed>}
        let container = t_keyed(
            TKeyedArray::new()
                .with_known_items(BTreeMap::from([(ArrayKey::String(word("required_field")), (false, get_string()))]))
                .with_parameters(Arc::new(get_arraykey()), Arc::new(get_mixed())),
        );

        assert_is_contained_by(&codebase, &input, &container, false, &mut ComparisonResult::default());
    }

    #[test]
    fn test_sealed_subset_contained_in_superset_with_optional() {
        let codebase = create_test_codebase("<?php");
        // array{'a': string}
        let input = t_keyed(
            TKeyedArray::new().with_known_items(BTreeMap::from([(ArrayKey::String(word("a")), (false, get_string()))])),
        );
        // array{'a': string, 'b'?: int}
        let container = t_keyed(TKeyedArray::new().with_known_items(BTreeMap::from([
            (ArrayKey::String(word("a")), (false, get_string())),
            (ArrayKey::String(word("b")), (true, get_int())),
        ])));
        assert_is_contained_by(&codebase, &input, &container, true, &mut ComparisonResult::default());
    }

    #[test]
    fn test_sealed_superset_not_contained_in_subset() {
        let codebase = create_test_codebase("<?php");
        // array{'a': string, 'b'?: int}
        let input = t_keyed(TKeyedArray::new().with_known_items(BTreeMap::from([
            (ArrayKey::String(word("a")), (false, get_string())),
            (ArrayKey::String(word("b")), (true, get_int())),
        ])));
        // array{'a': string}
        let container = t_keyed(
            TKeyedArray::new().with_known_items(BTreeMap::from([(ArrayKey::String(word("a")), (false, get_string()))])),
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
            TKeyedArray::new().with_known_items(BTreeMap::from([(ArrayKey::String(word("a")), (true, get_string()))])),
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
            TKeyedArray::new().with_known_items(BTreeMap::from([(ArrayKey::String(word("a")), (false, get_string()))])),
        );
        assert_is_contained_by(&codebase, &input, &container, false, &mut ComparisonResult::default());
    }

    #[test]
    fn test_optional_property_does_not_satisfy_required() {
        let codebase = create_test_codebase("<?php");
        // array{'a'?: string}
        let input = t_keyed(
            TKeyedArray::new().with_known_items(BTreeMap::from([(ArrayKey::String(word("a")), (true, get_string()))])),
        );
        // array{'a': string}
        let container = t_keyed(
            TKeyedArray::new().with_known_items(BTreeMap::from([(ArrayKey::String(word("a")), (false, get_string()))])),
        );
        assert_is_contained_by(&codebase, &input, &container, false, &mut ComparisonResult::default());
    }

    #[test]
    fn test_unsealed_compatible_generics() {
        let codebase = create_test_codebase("<?php");
        // array<string, int>
        let input = t_keyed(TKeyedArray::new_with_parameters(Arc::new(get_string()), Arc::new(get_int())));
        // array<array-key, mixed>
        let container = t_keyed(TKeyedArray::new_with_parameters(Arc::new(get_arraykey()), Arc::new(get_mixed())));
        assert_is_contained_by(&codebase, &input, &container, true, &mut ComparisonResult::default());
    }

    #[test]
    fn test_unsealed_incompatible_value_generic() {
        let codebase = create_test_codebase("<?php");
        // array<string, int>
        let input = t_keyed(TKeyedArray::new_with_parameters(Arc::new(get_string()), Arc::new(get_int())));
        // array<array-key, string>
        let container = t_keyed(TKeyedArray::new_with_parameters(Arc::new(get_arraykey()), Arc::new(get_string())));
        assert_is_contained_by(&codebase, &input, &container, false, &mut ComparisonResult::default());
    }

    #[test]
    fn test_unsealed_incompatible_key_generic() {
        let codebase = create_test_codebase("<?php");
        // array<array-key, int>
        let input = t_keyed(TKeyedArray::new_with_parameters(Arc::new(get_arraykey()), Arc::new(get_int())));
        // array<string, int>
        let container = t_keyed(TKeyedArray::new_with_parameters(Arc::new(get_string()), Arc::new(get_int())));
        assert_is_contained_by(&codebase, &input, &container, false, &mut ComparisonResult::default());
    }

    #[test]
    fn test_sealed_contained_in_compatible_unsealed() {
        let codebase = create_test_codebase("<?php");
        // array{'a': string}
        let input = t_keyed(
            TKeyedArray::new().with_known_items(BTreeMap::from([(ArrayKey::String(word("a")), (false, get_string()))])),
        );
        // array<array-key, mixed>
        let container = t_keyed(TKeyedArray::new_with_parameters(Arc::new(get_arraykey()), Arc::new(get_mixed())));
        assert_is_contained_by(&codebase, &input, &container, true, &mut ComparisonResult::default());
    }

    #[test]
    fn test_sealed_not_contained_in_incompatible_unsealed() {
        let codebase = create_test_codebase("<?php");
        // array{'a': string}
        let input = t_keyed(
            TKeyedArray::new().with_known_items(BTreeMap::from([(ArrayKey::String(word("a")), (false, get_string()))])),
        );
        // array<array-key, int>
        let container = t_keyed(TKeyedArray::new_with_parameters(Arc::new(get_arraykey()), Arc::new(get_int())));
        assert_is_contained_by(&codebase, &input, &container, false, &mut ComparisonResult::default());
    }

    #[test]
    fn test_sealed_contained_in_compatible_mixed() {
        let codebase = create_test_codebase("<?php");
        // array{'a': string, 'b': int}
        let input = t_keyed(TKeyedArray::new().with_known_items(BTreeMap::from([
            (ArrayKey::String(word("a")), (false, get_string())),
            (ArrayKey::String(word("b")), (false, get_int())),
        ])));
        // array{'a': string, ...<array-key, int>}
        let container = t_keyed(
            TKeyedArray::new()
                .with_known_items(BTreeMap::from([(ArrayKey::String(word("a")), (false, get_string()))]))
                .with_parameters(Arc::new(get_arraykey()), Arc::new(get_int())),
        );
        assert_is_contained_by(&codebase, &input, &container, true, &mut ComparisonResult::default());
    }

    #[test]
    fn test_sealed_not_contained_in_incompatible_mixed() {
        let codebase = create_test_codebase("<?php");
        // array{'a': string, 'b': string}
        let input = t_keyed(TKeyedArray::new().with_known_items(BTreeMap::from([
            (ArrayKey::String(word("a")), (false, get_string())),
            (ArrayKey::String(word("b")), (false, get_string())),
        ])));
        // array{'a': string, ...<array-key, int>}
        let container = t_keyed(
            TKeyedArray::new()
                .with_known_items(BTreeMap::from([(ArrayKey::String(word("a")), (false, get_string()))]))
                .with_parameters(Arc::new(get_arraykey()), Arc::new(get_int())),
        );
        assert_is_contained_by(&codebase, &input, &container, false, &mut ComparisonResult::default());
    }

    #[test]
    fn test_unsealed_does_not_satisfy_required_sealed() {
        let codebase = create_test_codebase("<?php");
        // array<array-key, string>
        let input = t_keyed(TKeyedArray::new_with_parameters(Arc::new(get_arraykey()), Arc::new(get_string())));
        // array{'a': string}
        let container = t_keyed(
            TKeyedArray::new().with_known_items(BTreeMap::from([(ArrayKey::String(word("a")), (false, get_string()))])),
        );
        assert_is_contained_by(&codebase, &input, &container, false, &mut ComparisonResult::default());
    }

    #[test]
    fn test_unsealed_does_not_satisfy_incompatible_sealed() {
        let codebase = create_test_codebase("<?php");
        // array<array-key, string>
        let input = t_keyed(TKeyedArray::new_with_parameters(Arc::new(get_arraykey()), Arc::new(get_string())));
        // array{'a': int}
        let container = t_keyed(
            TKeyedArray::new().with_known_items(BTreeMap::from([(ArrayKey::String(word("a")), (false, get_int()))])),
        );
        assert_is_contained_by(&codebase, &input, &container, false, &mut ComparisonResult::default());
    }

    #[test]
    fn test_non_empty_array_is_subtype_of_array() {
        let codebase = create_test_codebase("<?php");
        // non-empty-array<array-key, mixed>
        let input = t_keyed(
            TKeyedArray::new_with_parameters(Arc::new(get_arraykey()), Arc::new(get_mixed())).with_non_empty(true),
        );
        // array<array-key, mixed>
        let container = t_keyed(
            TKeyedArray::new_with_parameters(Arc::new(get_arraykey()), Arc::new(get_mixed())).with_non_empty(false),
        );
        assert_is_contained_by(&codebase, &input, &container, true, &mut ComparisonResult::default());
    }

    #[test]
    fn test_array_is_not_subtype_of_non_empty_array() {
        let codebase = create_test_codebase("<?php");
        // array<array-key, mixed>
        let input = t_keyed(
            TKeyedArray::new_with_parameters(Arc::new(get_arraykey()), Arc::new(get_mixed())).with_non_empty(false),
        );
        // non-empty-array<array-key, mixed>
        let container = t_keyed(
            TKeyedArray::new_with_parameters(Arc::new(get_arraykey()), Arc::new(get_mixed())).with_non_empty(true),
        );
        assert_is_contained_by(&codebase, &input, &container, false, &mut ComparisonResult::default());
    }

    #[test]
    fn test_potentially_empty_array_not_contained_in_definitely_non_empty() {
        let codebase = create_test_codebase("<?php");
        // array<array-key, mixed>
        let input = t_keyed(
            TKeyedArray::new_with_parameters(Arc::new(get_arraykey()), Arc::new(get_mixed())).with_non_empty(false),
        );
        // array{'a': string}
        let container = t_keyed(
            TKeyedArray::new().with_known_items(BTreeMap::from([(ArrayKey::String(word("a")), (false, get_string()))])),
        );

        assert_is_contained_by(&codebase, &input, &container, false, &mut ComparisonResult::default());
    }

    #[test]
    fn test_unsealed_input_with_conflicting_optional_key_in_container() {
        let codebase = create_test_codebase("<?php");

        // input: array{'a': string, ...<array-key, string>}
        let input = t_keyed(
            TKeyedArray::new()
                .with_known_items(BTreeMap::from([(ArrayKey::String(word("a")), (false, get_string()))]))
                .with_parameters(Arc::new(get_arraykey()), Arc::new(get_string())),
        );

        // container: array{'a': string, 'b'?: int, ...<array-key, mixed>}
        let container = t_keyed(
            TKeyedArray::new()
                .with_known_items(BTreeMap::from([
                    (ArrayKey::String(word("a")), (false, get_string())),
                    (ArrayKey::String(word("b")), (true, get_int())),
                ]))
                .with_parameters(Arc::new(get_arraykey()), Arc::new(get_mixed())),
        );

        // This should be false, because input could have a key 'b' which would be a string,
        // but the container requires 'b' to be an int.
        assert_is_contained_by(&codebase, &input, &container, false, &mut ComparisonResult::default());
    }
}
