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
/// 2. `possibly_thrown_exceptions`: Merges the sets of exceptions that can be thrown.
///
/// # Arguments
///
/// * `context`: The current analysis context, used for reporting issues.
/// * `destination_context`: The context into which properties are being merged.
/// * `source_context`: The context from which properties are being inherited.
pub(crate) fn inherit_branch_context_properties<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    destination_context: &mut BlockContext<'ctx>,
    source_context: &BlockContext<'ctx>,
) {
    for (variable, constraint) in &source_context.by_reference_constraints {
        let Some(outer_constraint) = destination_context.by_reference_constraints.get(variable) else {
            destination_context.by_reference_constraints.insert(variable.clone(), constraint.clone());
            continue;
        };

        if outer_constraint == constraint {
            // If the constraints are identical, no action is needed.
            continue;
        }

        let Some(constraint_type) = constraint.constraint_type.as_ref() else {
            destination_context.by_reference_constraints.insert(variable.clone(), constraint.clone());
            continue;
        };

        let Some(outer_constraint_type) = outer_constraint.constraint_type.as_ref() else {
            destination_context.by_reference_constraints.insert(variable.clone(), constraint.clone());
            continue;
        };

        if !union_comparator::is_contained_by(
            context.codebase,
            constraint_type,
            outer_constraint_type,
            false,
            false,
            false,
            &mut ComparisonResult::default(),
        ) {
            let constraint_type_str = constraint_type.get_id();
            let outer_constraint_type_str = outer_constraint_type.get_id();

            context.collector.report_with_code(
                IssueCode::ConflictingReferenceConstraint,
                Issue::error(format!(
                    "Conflicting pass-by-reference constraints for variable `{variable}`.",
                ))
                .with_annotation(
                    Annotation::secondary(outer_constraint.constraint_span).with_message(format!(
                        "An existing constraint requires this variable to be of type `{}`...",
                        outer_constraint_type_str
                    )),
                )
                .with_annotation(
                    Annotation::primary(constraint.constraint_span).with_message(format!(
                        "...but this branch imposes a conflicting constraint of `{}`.",
                        constraint_type_str
                    )),
                )
                .with_note(
                    "A variable reference cannot have multiple, incompatible type constraints applied to it in different branches of execution."
                )
                .with_help(format!(
                    "Refactor the code to ensure that `{variable}` adheres to a single, compatible type constraint across all execution paths.",
                )),
            );
        } else {
            destination_context.by_reference_constraints.insert(variable.clone(), constraint.clone());
        }
    }

    if context.settings.check_throws {
        for (exception, spans) in &source_context.possibly_thrown_exceptions {
            destination_context.possibly_thrown_exceptions.entry(*exception).or_default().extend(spans);
        }
    }
}
