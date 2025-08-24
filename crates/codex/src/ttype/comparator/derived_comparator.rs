use crate::metadata::CodebaseMetadata;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::derived::TDerived;
use crate::ttype::atomic::derived::key_of::TKeyOf;
use crate::ttype::atomic::derived::value_of::TValueOf;
use crate::ttype::comparator::ComparisonResult;
use crate::ttype::comparator::atomic_comparator;

pub fn is_contained_by(
    codebase: &CodebaseMetadata,
    input_type_part: &TAtomic,
    container_type_part: &TAtomic,
    inside_assertion: bool,
    atomic_comparison_result: &mut ComparisonResult,
) -> bool {
    if let TAtomic::Derived(derived_container) = container_type_part {
        let TAtomic::Derived(dervied_input) = input_type_part else {
            return false;
        };

        return match (derived_container, dervied_input) {
            (TDerived::KeyOf(key_of_container), TDerived::KeyOf(key_of_input)) => atomic_comparator::is_contained_by(
                codebase,
                key_of_input.get_target_type(),
                key_of_container.get_target_type(),
                inside_assertion,
                atomic_comparison_result,
            ),
            (TDerived::ValueOf(value_of_container), TDerived::ValueOf(value_of_input)) => {
                atomic_comparator::is_contained_by(
                    codebase,
                    value_of_input.get_target_type(),
                    value_of_container.get_target_type(),
                    inside_assertion,
                    atomic_comparison_result,
                )
            }
            (TDerived::PropertiesOf(properties_of_container), TDerived::PropertiesOf(properties_of_input)) => {
                atomic_comparator::is_contained_by(
                    codebase,
                    properties_of_input.get_target_type(),
                    properties_of_container.get_target_type(),
                    inside_assertion,
                    atomic_comparison_result,
                )
            }
            _ => false,
        };
    }

    let TAtomic::Derived(dervied_input) = input_type_part else {
        return false;
    };

    let input_union = match dervied_input {
        TDerived::KeyOf(key_of_input) => {
            TKeyOf::get_key_of_targets(std::slice::from_ref(key_of_input.get_target_type()), codebase, false)
        }
        TDerived::ValueOf(tvalue_of) => {
            TValueOf::get_value_of_targets(std::slice::from_ref(tvalue_of.get_target_type()), codebase, false)
        }
        TDerived::PropertiesOf(_) => {
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
