use mago_atom::Atom;

use crate::metadata::CodebaseMetadata;
use crate::ttype::TType;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::generic::TGenericParameter;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::comparator::ComparisonResult;
use crate::ttype::comparator::atomic_comparator;
use crate::ttype::comparator::iterable_comparator;
use crate::ttype::template::TemplateBound;
use crate::ttype::union::TUnion;
use crate::ttype::wrap_atomic;

use super::integer_comparator;

#[inline]
#[allow(clippy::too_many_arguments)]
pub fn is_contained_by(
    codebase: &CodebaseMetadata,
    input_type: &TUnion,
    container_type: &TUnion,
    ignore_null: bool,
    ignore_false: bool,
    inside_assertion: bool,
    union_comparison_result: &mut ComparisonResult,
) -> bool {
    if input_type == container_type {
        return true;
    }

    let container_has_template = container_type.has_template_or_static();

    let mut container_atomic_types = container_type.types.iter().collect::<Vec<_>>();
    container_atomic_types.reverse();

    let mut input_atomic_types = input_type.types.iter().collect::<Vec<_>>();
    input_atomic_types.reverse();

    'outer: while let Some(input_type_part) = input_atomic_types.pop() {
        match input_type_part {
            TAtomic::Null => {
                if ignore_null {
                    continue;
                }
            }
            TAtomic::Scalar(TScalar::Bool(bool)) if bool.is_false() => {
                if ignore_false {
                    continue;
                }
            }
            TAtomic::Variable(name) => {
                if container_type.is_single()
                    && let TAtomic::Variable(container_name) = container_type.get_single()
                    && container_name == name
                {
                    continue;
                }

                union_comparison_result
                    .type_variable_upper_bounds
                    .push((*name, TemplateBound::new(container_type.clone(), 0, None, None)));

                continue;
            }
            TAtomic::GenericParameter(TGenericParameter { intersection_types: None, constraint, .. }) => {
                if !container_has_template {
                    input_atomic_types.extend(constraint.types.iter().collect::<Vec<_>>());
                    continue;
                }
            }
            _ => (),
        }

        let mut type_match_found = false;
        let mut all_type_coerced = None;
        let mut all_type_coerced_from_nested_mixed = None;
        let mut all_type_coerced_from_as_mixed = None;
        let mut some_type_coerced = false;
        let mut some_type_coerced_from_nested_mixed = false;

        if let TAtomic::Scalar(TScalar::ArrayKey) = input_type_part {
            if container_type.has_int_and_string() {
                continue;
            }
            let mut has_int = false;
            let mut has_string = false;

            for container_atomic_type in &container_atomic_types {
                if let TAtomic::GenericParameter(TGenericParameter { constraint, .. }) = container_atomic_type {
                    if constraint.has_int_and_string() {
                        continue 'outer;
                    }

                    if constraint.has_int() {
                        has_int = true;
                    }

                    if constraint.has_string() {
                        has_string = true;
                    }
                }
            }

            if has_int && has_string {
                continue;
            }
        }

        if let TAtomic::Scalar(TScalar::Generic) = input_type_part
            && !container_type.has_scalar()
            && container_type.has_scalar_combination()
        {
            continue;
        }

        if let TAtomic::Iterable(_) = input_type_part
            && !container_type.has_iterable()
            && container_type.has_array()
            && container_type.has_traversable(codebase)
        {
            let mut matched_all = true;
            for container_atomic_type in &container_atomic_types {
                if !container_atomic_type.is_array() && !container_atomic_type.is_traversable(codebase) {
                    continue;
                }

                matched_all &= iterable_comparator::is_contained_by(
                    codebase,
                    input_type_part,
                    container_atomic_type,
                    inside_assertion,
                    union_comparison_result,
                );
            }

            if matched_all {
                continue;
            }
        }

        if let TAtomic::Scalar(TScalar::Integer(input_integer)) = input_type_part
            && container_type.has_int()
            && integer_comparator::is_contained_by_union(*input_integer, container_type)
        {
            continue;
        }

        for container_type_part in &container_atomic_types {
            if ignore_null && matches!(container_type_part, TAtomic::Null) && !matches!(input_type_part, TAtomic::Null)
            {
                continue;
            }

            if ignore_false
                && matches!(container_type_part, TAtomic::Scalar(TScalar::Bool(bool)) if bool.is_false())
                && !matches!(input_type_part, TAtomic::Scalar(TScalar::Bool(bool)) if bool.is_false())
            {
                continue;
            }

            if let TAtomic::Variable(name) = &container_type_part {
                union_comparison_result
                    .type_variable_lower_bounds
                    .push((*name, TemplateBound::new(input_type.clone(), 0, None, None)));

                type_match_found = true;

                continue;
            }

            let mut atomic_comparison_result = ComparisonResult::new();
            let is_atomic_contained_by = atomic_comparator::is_contained_by(
                codebase,
                input_type_part,
                container_type_part,
                inside_assertion,
                &mut atomic_comparison_result,
            );

            if (input_type_part.is_mixed() || matches!(input_type_part, TAtomic::Scalar(TScalar::ArrayKey)))
                && input_type.from_template_default
                && atomic_comparison_result.type_coerced_from_nested_mixed.unwrap_or(false)
            {
                atomic_comparison_result.type_coerced_from_as_mixed = Some(true);
            }

            if atomic_comparison_result.type_coerced_to_literal.is_some() {
                union_comparison_result.type_coerced_to_literal = atomic_comparison_result.type_coerced_to_literal;
            }

            if is_atomic_contained_by {
                if let Some(replacement_atomic_type) = atomic_comparison_result.replacement_atomic_type {
                    if let Some(replacement_union_type) = &mut union_comparison_result.replacement_union_type {
                        replacement_union_type.replace_type(input_type_part, replacement_atomic_type);
                    } else {
                        union_comparison_result.replacement_union_type = Some(wrap_atomic(replacement_atomic_type));
                    }
                }

                union_comparison_result
                    .type_variable_lower_bounds
                    .extend(atomic_comparison_result.type_variable_lower_bounds);

                union_comparison_result
                    .type_variable_upper_bounds
                    .extend(atomic_comparison_result.type_variable_upper_bounds);
            }

            if atomic_comparison_result.type_coerced.unwrap_or(false) {
                some_type_coerced = true;
            }

            if atomic_comparison_result.type_coerced_from_nested_mixed.unwrap_or(false) {
                some_type_coerced_from_nested_mixed = true;
            }

            if !atomic_comparison_result.type_coerced.unwrap_or(false) || !all_type_coerced.unwrap_or(true) {
                all_type_coerced = Some(false);
            } else {
                all_type_coerced = Some(true);
            }

            if !atomic_comparison_result.type_coerced_from_nested_mixed.unwrap_or(false)
                || !all_type_coerced_from_nested_mixed.unwrap_or(true)
            {
                all_type_coerced_from_nested_mixed = Some(false);
            } else {
                all_type_coerced_from_nested_mixed = Some(true);
            }

            if is_atomic_contained_by {
                type_match_found = true;
                all_type_coerced_from_nested_mixed = Some(false);
                all_type_coerced_from_as_mixed = Some(false);
                all_type_coerced = Some(false);
            }
        }

        if all_type_coerced.unwrap_or(false) {
            union_comparison_result.type_coerced = Some(true);
        }

        if all_type_coerced_from_nested_mixed.unwrap_or(false) {
            union_comparison_result.type_coerced_from_nested_mixed = Some(true);

            if input_type.from_template_default || all_type_coerced_from_as_mixed.unwrap_or(false) {
                union_comparison_result.type_coerced_from_as_mixed = Some(true);
            }
        }

        if !type_match_found {
            if some_type_coerced {
                union_comparison_result.type_coerced = Some(true);
            }

            if some_type_coerced_from_nested_mixed {
                union_comparison_result.type_coerced_from_nested_mixed = Some(true);

                if input_type.from_template_default || all_type_coerced_from_as_mixed.unwrap_or(false) {
                    union_comparison_result.type_coerced_from_as_mixed = Some(true);
                }
            }

            return false;
        }
    }

    true
}

pub(crate) fn can_be_contained_by(
    codebase: &CodebaseMetadata,
    input_type: &TUnion,
    container_type: &TUnion,
    ignore_null: bool,
    ignore_false: bool,
    matching_input_keys: &mut Vec<Atom>,
) -> bool {
    if container_type.is_mixed() {
        return true;
    }

    if input_type.is_never() {
        return true;
    }

    for container_type_part in container_type.types.as_ref() {
        if matches!(container_type_part, TAtomic::Null) && ignore_null {
            continue;
        }

        if matches!(container_type_part, TAtomic::Scalar(TScalar::Bool(bool)) if bool.is_false()) && ignore_false {
            continue;
        }

        for input_type_part in input_type.types.as_ref() {
            let mut atomic_comparison_result = ComparisonResult::new();

            let is_atomic_contained_by = atomic_comparator::is_contained_by(
                codebase,
                input_type_part,
                container_type_part,
                false,
                &mut atomic_comparison_result,
            );

            if is_atomic_contained_by || atomic_comparison_result.type_coerced_from_nested_mixed.unwrap_or(false) {
                matching_input_keys.push(input_type_part.get_id());
            }
        }
    }

    !matching_input_keys.is_empty()
}

pub fn can_expression_types_be_identical(
    codebase: &CodebaseMetadata,
    type1: &TUnion,
    type2: &TUnion,
    inside_assertion: bool,
    allow_type_coercion: bool,
) -> bool {
    // If either type is mixed, they can be identical
    if type1.has_mixed() || type1.has_mixed_template() || type2.has_mixed() || type2.has_mixed_template() {
        return true;
    }

    if (type1.is_nullable() && type2.has_nullish()) || (type2.is_nullable() && type1.has_nullish()) {
        return true;
    }

    for type1_part in type1.types.as_ref() {
        for type2_part in type2.types.as_ref() {
            if atomic_comparator::can_be_identical(
                codebase,
                type1_part,
                type2_part,
                inside_assertion,
                allow_type_coercion,
            ) {
                return true;
            }
        }
    }

    false
}
