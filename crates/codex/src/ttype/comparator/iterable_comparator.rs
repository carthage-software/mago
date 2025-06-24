use mago_interner::ThreadedInterner;

use crate::metadata::CodebaseMetadata;
use crate::ttype::atomic::TAtomic;
use crate::ttype::comparator::ComparisonResult;
use crate::ttype::comparator::generic_comparator::update_failed_result_from_nested;
use crate::ttype::comparator::union_comparator;
use crate::ttype::get_iterable_parameters;

pub(crate) fn is_contained_by(
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    input_type_part: &TAtomic,
    container_type_part: &TAtomic,
    inside_assertion: bool,
    atomic_comparison_result: &mut ComparisonResult,
) -> bool {
    let TAtomic::Iterable(iterable) = container_type_part else {
        return false;
    };

    let Some(input_parameters) = get_iterable_parameters(input_type_part, codebase, interner) else {
        return false;
    };

    let mut all_types_contain = true;

    let mut nested_comparison_result = ComparisonResult::new();
    if !union_comparator::is_contained_by(
        codebase,
        interner,
        &input_parameters.0,
        iterable.get_key_type(),
        false,
        input_parameters.0.ignore_falsable_issues,
        inside_assertion,
        &mut nested_comparison_result,
    ) {
        all_types_contain = false;

        update_failed_result_from_nested(atomic_comparison_result, nested_comparison_result);
    }

    let mut nested_comparison_result = ComparisonResult::new();

    if !union_comparator::is_contained_by(
        codebase,
        interner,
        &input_parameters.1,
        iterable.get_value_type(),
        false,
        input_parameters.1.ignore_falsable_issues,
        inside_assertion,
        &mut nested_comparison_result,
    ) {
        all_types_contain = false;

        update_failed_result_from_nested(atomic_comparison_result, nested_comparison_result);
    }

    all_types_contain
}
