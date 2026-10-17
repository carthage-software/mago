use mago_allocator::Arena;
use mago_codex::ttype::TType;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_reporting::Annotation;
use mago_reporting::Issue;

use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;

/// Inherits specific properties from a source context (e.g., a branch) into a destination context.
///
/// This function is responsible for merging:
/// 1. `by_reference_constraints`: Checks for and reports conflicting constraints.
/// 2. `possibly_thrown_exceptions` and `possibly_thrown_generic_exceptions`: Merge the exceptions that can be thrown.
///
/// # Arguments
///
/// * `context`: The current analysis context, used for reporting issues.
/// * `destination_context`: The context into which properties are being merged.
/// * `source_context`: The context from which properties are being inherited.
pub(crate) fn inherit_branch_context_properties<'ctx, A>(
    context: &mut Context<'ctx, '_, A>,
    destination_context: &mut BlockContext<'ctx>,
    source_context: &BlockContext<'ctx>,
) where
    A: Arena,
{
    for (variable, constraint) in &source_context.by_reference_constraints {
        let Some(outer_constraint) = destination_context.by_reference_constraints.get(variable) else {
            destination_context.by_reference_constraints.insert(*variable, constraint.clone());
            continue;
        };

        if outer_constraint == constraint {
            // If the constraints are identical, no action is needed.
            continue;
        }

        let Some(constraint_type) = constraint.constraint_type.as_ref() else {
            destination_context.by_reference_constraints.insert(*variable, constraint.clone());
            continue;
        };

        let Some(outer_constraint_type) = outer_constraint.constraint_type.as_ref() else {
            destination_context.by_reference_constraints.insert(*variable, constraint.clone());
            continue;
        };

        if union_comparator::is_contained_by(
            context.codebase,
            constraint_type,
            outer_constraint_type,
            false,
            false,
            false,
            &mut ComparisonResult::default(),
        ) {
            destination_context.by_reference_constraints.insert(*variable, constraint.clone());
        } else {
            let constraint_type_str = constraint_type.get_id();
            let outer_constraint_type_str = outer_constraint_type.get_id();

            context.collector.report_with_code(
                IssueCode::ConflictingReferenceConstraint,
                Issue::error(format!(
                    "Conflicting pass-by-reference constraints for variable `{variable}`.",
                ))
                .with_annotation(
                    Annotation::secondary(outer_constraint.constraint_span).with_message(format!(
                        "An existing constraint requires this variable to be of type `{outer_constraint_type_str}`..."
                    )),
                )
                .with_annotation(
                    Annotation::primary(constraint.constraint_span).with_message(format!(
                        "...but this branch imposes a conflicting constraint of `{constraint_type_str}`."
                    )),
                )
                .with_note(
                    "A variable reference cannot have multiple, incompatible type constraints applied to it in different branches of execution."
                )
                .with_help(format!(
                    "Refactor the code to ensure that `{variable}` adheres to a single, compatible type constraint across all execution paths.",
                )),
            );
        }
    }

    inherit_thrown_exceptions(context, destination_context, source_context);
}

pub(crate) fn inherit_thrown_exceptions<'ctx, A>(
    context: &Context<'ctx, '_, A>,
    destination_context: &mut BlockContext<'ctx>,
    source_context: &BlockContext<'ctx>,
) where
    A: Arena,
{
    if context.settings.check_throws || context.settings.find_overly_wide_throw_types {
        for (exception, spans) in &source_context.possibly_thrown_exceptions {
            destination_context.possibly_thrown_exceptions.entry(*exception).or_default().extend(spans);
        }

        destination_context
            .possibly_thrown_generic_exceptions
            .extend(source_context.possibly_thrown_generic_exceptions.iter().cloned());
    }

    if context.settings.find_overly_wide_throw_types {
        for (exception, spans) in &source_context.possibly_thrown_exceptions_for_too_wide {
            destination_context.possibly_thrown_exceptions_for_too_wide.entry(*exception).or_default().extend(spans);
        }

        destination_context
            .possibly_thrown_generic_exceptions_for_too_wide
            .extend(source_context.possibly_thrown_generic_exceptions_for_too_wide.iter().cloned());
    }
}
