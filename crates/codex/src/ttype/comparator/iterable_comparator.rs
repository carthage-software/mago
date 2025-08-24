use crate::metadata::CodebaseMetadata;
use crate::ttype::atomic::TAtomic;
use crate::ttype::comparator::ComparisonResult;
use crate::ttype::comparator::generic_comparator::update_failed_result_from_nested;
use crate::ttype::comparator::union_comparator;
use crate::ttype::get_iterable_parameters;

pub fn is_contained_by(
    codebase: &CodebaseMetadata,
    input_type_part: &TAtomic,
    container_type_part: &TAtomic,
    inside_assertion: bool,
    atomic_comparison_result: &mut ComparisonResult,
) -> bool {
    let Some((container_k, container_v)) = get_iterable_parameters(container_type_part, codebase) else {
        return false;
    };

    let Some((input_k, input_v)) = get_iterable_parameters(input_type_part, codebase) else {
        return false;
    };

    let mut all_types_contain = true;

    let mut nested_comparison_result = ComparisonResult::new();
    if !union_comparator::is_contained_by(
        codebase,
        &input_k,
        &container_k,
        false,
        input_k.ignore_falsable_issues,
        inside_assertion,
        &mut nested_comparison_result,
    ) {
        all_types_contain = false;

        update_failed_result_from_nested(atomic_comparison_result, nested_comparison_result);
    }

    let mut nested_comparison_result = ComparisonResult::new();

    if !union_comparator::is_contained_by(
        codebase,
        &input_v,
        &container_v,
        false,
        input_v.ignore_falsable_issues,
        inside_assertion,
        &mut nested_comparison_result,
    ) {
        all_types_contain = false;

        update_failed_result_from_nested(atomic_comparison_result, nested_comparison_result);
    }

    all_types_contain
}
