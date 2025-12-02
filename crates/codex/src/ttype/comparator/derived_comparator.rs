use crate::metadata::CodebaseMetadata;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::derived::TDerived;
use crate::ttype::atomic::derived::index_access::TIndexAccess;
use crate::ttype::atomic::derived::key_of::TKeyOf;
use crate::ttype::atomic::derived::value_of::TValueOf;
use crate::ttype::comparator::ComparisonResult;
use crate::ttype::comparator::atomic_comparator;
use crate::ttype::comparator::union_comparator;

pub fn is_contained_by(
    codebase: &CodebaseMetadata,
    input_type_part: &TAtomic,
    container_type_part: &TAtomic,
    inside_assertion: bool,
    atomic_comparison_result: &mut ComparisonResult,
) -> bool {
    if let TAtomic::Derived(derived_container) = container_type_part {
        let TAtomic::Derived(derived_input) = input_type_part else {
            return false;
        };

        return match (derived_container, derived_input) {
            (TDerived::KeyOf(key_of_container), TDerived::KeyOf(key_of_input)) => union_comparator::is_contained_by(
                codebase,
                key_of_input.get_target_type(),
                key_of_container.get_target_type(),
                false,
                false,
                inside_assertion,
                atomic_comparison_result,
            ),
            (TDerived::ValueOf(value_of_container), TDerived::ValueOf(value_of_input)) => {
                union_comparator::is_contained_by(
                    codebase,
                    value_of_input.get_target_type(),
                    value_of_container.get_target_type(),
                    false,
                    false,
                    inside_assertion,
                    atomic_comparison_result,
                )
            }
            (TDerived::IndexAccess(index_access_container), TDerived::IndexAccess(index_access_input)) => {
                let container_indexed = TIndexAccess::get_indexed_access_result(
                    &index_access_container.get_target_type().types,
                    &index_access_container.get_index_type().types,
                    false,
                );

                let input_indexed = TIndexAccess::get_indexed_access_result(
                    &index_access_input.get_target_type().types,
                    &index_access_input.get_index_type().types,
                    false,
                );

                match (container_indexed, input_indexed) {
                    (Some(container_union), Some(input_union)) => {
                        for input_atomic in input_union.types.iter() {
                            let mut found = false;
                            for container_atomic in container_union.types.iter() {
                                if atomic_comparator::is_contained_by(
                                    codebase,
                                    input_atomic,
                                    container_atomic,
                                    inside_assertion,
                                    atomic_comparison_result,
                                ) {
                                    found = true;
                                    break;
                                }
                            }
                            if !found {
                                return false;
                            }
                        }
                        true
                    }
                    _ => false,
                }
            }
            (TDerived::PropertiesOf(properties_of_container), TDerived::PropertiesOf(properties_of_input)) => {
                union_comparator::is_contained_by(
                    codebase,
                    properties_of_input.get_target_type(),
                    properties_of_container.get_target_type(),
                    false,
                    false,
                    inside_assertion,
                    atomic_comparison_result,
                )
            }
            _ => false,
        };
    }

    let TAtomic::Derived(derived_input) = input_type_part else {
        return false;
    };

    let input_union = match derived_input {
        TDerived::KeyOf(key_of) => TKeyOf::get_key_of_targets(&key_of.get_target_type().types, codebase, false),
        TDerived::ValueOf(value_of) => {
            TValueOf::get_value_of_targets(&value_of.get_target_type().types, codebase, false)
        }
        TDerived::IndexAccess(index_access) => TIndexAccess::get_indexed_access_result(
            &index_access.get_target_type().types,
            &index_access.get_index_type().types,
            false,
        ),
        TDerived::PropertiesOf(_) | TDerived::IntMask(_) | TDerived::IntMaskOf(_) => {
            return false;
        }
    };

    let Some(input_union) = input_union else {
        return false;
    };

    for input_atomic in input_union.types.iter() {
        if !atomic_comparator::is_contained_by(
            codebase,
            input_atomic,
            container_type_part,
            inside_assertion,
            atomic_comparison_result,
        ) {
            return false;
        }
    }

    true
}
