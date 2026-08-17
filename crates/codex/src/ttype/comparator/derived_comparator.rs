use crate::metadata::CodebaseMetadata;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::derived::TDerived;
use crate::ttype::atomic::derived::index_access::TIndexAccess;
use crate::ttype::atomic::derived::key_of::TKeyOf;
use crate::ttype::atomic::derived::new::TNew;
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
            if let TDerived::TemplateType(template_type) = derived_container
                && let Some(resolved_container) = template_type.resolve(codebase)
            {
                return resolved_container.types.iter().any(|resolved_atomic| {
                    atomic_comparator::is_contained_by(
                        codebase,
                        input_type_part,
                        resolved_atomic,
                        inside_assertion,
                        atomic_comparison_result,
                    )
                });
            }

            return false;
        };

        return match (derived_container, derived_input) {
            (TDerived::KeyOf(_), TDerived::KeyOf(_))
            | (TDerived::ValueOf(_), TDerived::ValueOf(_))
            | (TDerived::PropertiesOf(_), TDerived::PropertiesOf(_)) => {
                match (derived_input.get_target_type(), derived_container.get_target_type()) {
                    (Some(input_target), Some(container_target)) => union_comparator::is_contained_by(
                        codebase,
                        input_target,
                        container_target,
                        false,
                        false,
                        inside_assertion,
                        atomic_comparison_result,
                    ),
                    _ => false,
                }
            }
            (TDerived::IndexAccess(index_access_container), TDerived::IndexAccess(index_access_input)) => {
                let container_indexed = TIndexAccess::get_indexed_access_result(
                    &index_access_container.get_target_type().types,
                    &index_access_container.get_index_type().types,
                    codebase,
                    false,
                );

                let input_indexed = TIndexAccess::get_indexed_access_result(
                    &index_access_input.get_target_type().types,
                    &index_access_input.get_index_type().types,
                    codebase,
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
            codebase,
            false,
        ),
        TDerived::New(new_type) => TNew::get_new_targets(&new_type.get_target_type().types, codebase),
        TDerived::TemplateType(template_type) => template_type.resolve(codebase),
        TDerived::PropertiesOf(_) | TDerived::IntMask(_) | TDerived::IntMaskOf(_) | TDerived::Intersection(_) => {
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
