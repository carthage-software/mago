use ahash::HashSet;

use mago_codex::assertion::Assertion;
use mago_codex::get_class_like;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::r#enum::TEnum;
use mago_codex::ttype::combine_union_types;
use mago_codex::ttype::combiner;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::comparator::union_comparator::can_expression_types_be_identical;
use mago_codex::ttype::get_mixed_any;
use mago_codex::ttype::get_never;
use mago_codex::ttype::union::TUnion;
use mago_interner::StringIdentifier;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::expression::binary::is_always_identical_to;
use crate::issue::TypingIssueKind;
use crate::reconciler::ReconcilationContext;
use crate::reconciler::negated_assertion_reconciler;
use crate::utils::conditional::add_branch_dataflow;

impl Analyzable for Match {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let was_inside_conditional = block_context.inside_conditional;
        block_context.inside_conditional = true;
        self.expression.analyze(context, block_context, artifacts)?;
        block_context.inside_conditional = was_inside_conditional;

        add_branch_dataflow(self.expression.as_ref(), artifacts);

        let mut match_arms = Vec::new();
        let mut default_arm: Option<&MatchDefaultArm> = None;
        for arm in self.arms.iter() {
            match arm {
                MatchArm::Expression(match_expression_arm) => {
                    match_arms.push(match_expression_arm);
                }
                MatchArm::Default(match_default_arm) => {
                    if let Some(previous_default_arm) = default_arm {
                        context.buffer.report(
                            TypingIssueKind::DuplicateMatchDefaultArm,
                            Issue::error("Match expression cannot have multiple `default` arms.")
                                .with_annotation(Annotation::primary(match_default_arm.span()).with_message("This `default` arm is a duplicate."))
                                .with_annotation(Annotation::secondary(previous_default_arm.span()).with_message("The first `default` arm was defined here."))
                                .with_note("The `default` arm in a `match` expression serves as a fallback and is executed only if no other conditional arm matches the subject expression's value.")
                                .with_help("Remove the redundant `default` arm, or change its condition if a different case was intended."),
                        );
                    }

                    default_arm = Some(match_default_arm);
                }
            }
        }

        if match_arms.is_empty() {
            if let Some(default_arm) = default_arm {
                context.buffer.report(
                    TypingIssueKind::MatchExpressionOnlyDefaultArm,
                    Issue::error("Match expression contains only a `default` arm.")
                    .with_annotation(
                        Annotation::primary(self.r#match.span())
                            .with_message("This `match` expression has no conditional arms."),
                    )
                    .with_annotation(
                        Annotation::secondary(default_arm.span())
                            .with_message("Only this `default` arm is present."),
                    )
                    .with_note(
                        "Since there are no conditional arms, the `default` arm will always be executed. The `match` structure might be unnecessary."
                    )
                    .with_help(
                        "Consider refactoring to directly use the `default` arm's expression if no other conditions are intended, or add conditional arms if specific cases should be handled differently."
                    ),
                );

                default_arm.expression.analyze(context, block_context, artifacts)?;
                if let Some(expression_type) = artifacts.get_rc_expression_type(&default_arm.expression).cloned() {
                    artifacts.set_rc_expression_type(self, expression_type);
                }

                return Ok(());
            } else {
                context.buffer.report(
                    TypingIssueKind::EmptyMatchExpression,
                    Issue::error("Match expression must have at least one arm.")
                    .with_annotation(
                        Annotation::primary(self.span()).with_message("This `match` expression is empty."),
                    )
                    .with_note("A `match` expression without any arms (neither conditional nor `default`) is syntactically invalid and will cause a fatal error.")
                    .with_help("Add at least one conditional arm (e.g., `value => result,`) or a `default` arm (e.g., `default => result,`) to the `match` expression."),
                );

                artifacts.set_expression_type(self, get_never());

                return Ok(());
            }
        }

        let subject_type = match artifacts.get_expression_type(&self.expression).cloned() {
            Some(subject_type) => subject_type,
            None => {
                context.buffer.report(
                    TypingIssueKind::UnknownMatchSubjectType,
                    Issue::error("Match subject expression type cannot be determined.")
                        .with_annotation(Annotation::primary(self.expression.span()).with_message("This match subject expression has an unknown type."))
                        .with_annotation(Annotation::secondary(self.r#match.span()).with_message("The corresponding match expression is defined here."))
                        .with_note("The type of the match subject expression must be known to determine which arm will be executed at runtime.")
                        .with_help("Ensure that the match subject expression is well-formed and has a resolvable type."),
                );

                get_mixed_any()
            }
        };

        if subject_type.is_never() {
            context.buffer.report(
                TypingIssueKind::MatchSubjectTypeIsNever,
                Issue::error("Match subject expression type is `never`.")
                    .with_annotation(Annotation::primary(self.expression.span()).with_message("This match subject expression has type `never`."))
                    .with_annotation(Annotation::secondary(self.r#match.span()).with_message("The corresponding match expression is defined here."))
                    .with_note("A `never` type indicates that the expression can never be executed, making the match expression effectively unreachable.")
                    .with_help("Consider revising the logic to ensure that the match subject expression can have a valid type."),
            );

            artifacts.set_expression_type(self, get_never());

            return Ok(());
        }

        let mut remaining_subject_type = subject_type.clone();

        let mut possible_results = vec![];
        let mut definite_match: Option<&MatchExpressionArm> = None;
        for match_expression_arm in match_arms.iter() {
            let mut arm_is_unreachable = false;
            if remaining_subject_type.is_never() {
                arm_is_unreachable = true;

                context.buffer.report(
                    TypingIssueKind::UnreachableMatchArm,
                    Issue::warning("This match arm is unreachable because all possible values of the subject expression have been handled by preceding arms.")
                        .with_annotation(Annotation::primary(match_expression_arm.span()).with_message("This arm will never be reached"))
                        .with_annotation(Annotation::secondary(self.expression.span()).with_message(format!("Subject expression of type `{}`", subject_type.get_id(Some(context.interner)))))
                        .with_note("Based on the type of the subject expression and the conditions of the preceding arms, there are no remaining values that could match this arm's conditions.")
                        .with_help("Remove this unreachable arm or review the subject's type and the conditions of preceding arms to ensure all intended cases are reachable."),
                );
            }

            let mut arm_block_context = block_context.clone();
            arm_block_context.inside_conditional = true;

            let mut definitly_matches = false;
            let mut unreachable_conditions = vec![];
            for condition in match_expression_arm.conditions.iter() {
                condition.analyze(context, &mut arm_block_context, artifacts)?;

                let Some(condition_type) = artifacts.get_expression_type(condition) else {
                    context.buffer.report(
                        TypingIssueKind::UnknownMatchConditionType,
                        Issue::error("Cannot infer the type of this match condition.")
                            .with_annotation(
                                Annotation::primary(condition.span())
                                    .with_message("This match condition has an unknown type."),
                            )
                            .with_annotation(
                                Annotation::secondary(match_expression_arm.expression.span())
                                    .with_message("This is the corresponding arm expression."),
                            )
                            .with_note("The type must be known to evaluate which arm can match.")
                            .with_help("Make sure the condition is valid and its type can be resolved."),
                    );

                    continue;
                };

                if arm_is_unreachable {
                    continue;
                }

                if !can_expression_types_be_identical(
                    context.codebase,
                    context.interner,
                    &subject_type,
                    condition_type,
                    true,
                ) {
                    unreachable_conditions.push((condition_type.get_id(Some(context.interner)), condition.span()));
                } else if definite_match.is_none()
                    && is_always_identical_to(condition_type, &subject_type, context.codebase, context.interner)
                {
                    let condition_type_str = condition_type.get_id(Some(context.interner));
                    let subject_type_str = subject_type.get_id(Some(context.interner));

                    context.buffer.report(
                        TypingIssueKind::MatchArmAlwaysTrue,
                        Issue::error(format!(
                            "This condition (type `{condition_type_str}`) always strictly matches the subject (type `{subject_type_str}`)."
                        ))
                        .with_annotation(Annotation::primary(condition.span()).with_message(format!("Always matches: `{condition_type_str}`")))
                        .with_annotation(Annotation::secondary(self.expression.span()).with_message(format!("Subject: `{subject_type_str}`")))
                        .with_note("This arm will always match if reached, making following conditional arms (and possibly `default`) unreachable.")
                        .with_help("Recheck the match order and logic. If intentional, consider making this the final arm."),
                    );

                    definitly_matches = true;
                } else {
                    remaining_subject_type =
                        subtract_union_types(context, artifacts, remaining_subject_type, condition_type.clone());
                }
            }

            if arm_is_unreachable {
                continue;
            }

            match_expression_arm.expression.analyze(context, &mut arm_block_context, artifacts)?;

            let expression_type = match artifacts.get_expression_type(&match_expression_arm.expression) {
                Some(expression_type) => expression_type.clone(),
                None => get_never(),
            };

            if let Some(matched_arm) = definite_match {
                context.buffer.report(
                    TypingIssueKind::UnreachableMatchArm,
                    Issue::warning("This match arm will never be executed.")
                        .with_annotation(
                            Annotation::primary(match_expression_arm.span()).with_message("Unreachable arm."),
                        )
                        .with_annotation(
                            Annotation::secondary(matched_arm.span())
                                .with_message("This earlier arm always matches if reached."),
                        )
                        .with_annotation(
                            Annotation::secondary(self.expression.span())
                                .with_message("Subject expression being matched."),
                        )
                        .with_note("A prior arm always strictly matches the subject, making this arm unreachable.")
                        .with_help("Reorder or remove the unreachable arm to clean up your code."),
                );

                continue;
            }

            if !unreachable_conditions.is_empty() {
                let subject_type_str = subject_type.get_id(Some(context.interner));

                if unreachable_conditions.len() == match_expression_arm.conditions.len() {
                    context.buffer.report(
                        TypingIssueKind::UnreachableMatchArm,
                        Issue::warning("This match arm will never be executed.")
                            .with_annotation(
                                Annotation::primary(match_expression_arm.expression.span())
                                    .with_message("Arm expression is unreachable."),
                            )
                            .with_annotations(unreachable_conditions.into_iter().map(
                                |(condition_type_str, condition)| {
                                    Annotation::secondary(condition).with_message(format!(
                                        "Condition type `{condition_type_str}` is never strictly equal to the subject type `{subject_type_str}`"
                                    ))
                                },
                            ))
                            .with_annotation(
                                Annotation::secondary(self.expression.span())
                                    .with_message(format!("Subject expression has type `{subject_type_str}`")),
                            )
                            .with_note(
                                if match_expression_arm.conditions.len() == 1 {
                                    "This arm's condition is never strictly equal to the subject type, making the arm unreachable."
                                } else {
                                    "All conditions are unreachable, making this arm unreachable."
                                }
                            )
                            .with_help(
                                if match_expression_arm.conditions.len() == 1 {
                                    "Consider removing this unreachable arm, or adjust the condition to ensure it can match."
                                } else {
                                    "Consider removing this unreachable arm, or adjust the conditions to ensure at least one can match."
                                }
                            ),
                    );

                    continue;
                } else {
                    for (condition_type_str, condition) in unreachable_conditions {
                        context.buffer.report(
                            TypingIssueKind::UnreachableMatchArmCondition,
                            Issue::error(format!(
                                "This condition type `{condition_type_str}` can never strictly equal the subject type `{subject_type_str}`."
                            ))
                            .with_annotation(Annotation::primary(condition).with_message(format!("This condition has type `{condition_type_str}`")))
                            .with_annotation(Annotation::secondary(self.expression.span()).with_message(format!("Subject has type `{subject_type_str}`")))
                            .with_note("Match uses strict comparison (`===`). The condition’s type is incompatible, so this arm can never be selected.")
                            .with_help("Consider removing this unreachable condition, or adjust the types to ensure compatibility."),
                        );
                    }
                }
            }

            if definitly_matches {
                definite_match = Some(match_expression_arm);
                possible_results = vec![expression_type];
            } else {
                possible_results.push(expression_type);
            }
        }

        if let Some(default_arm) = default_arm {
            let mut default_arm_block_context = block_context.clone();
            default_arm_block_context.inside_conditional = true;

            default_arm.expression.analyze(context, &mut default_arm_block_context, artifacts)?;

            if let Some(definite_match) = definite_match {
                context.buffer.report(
                    TypingIssueKind::UnreachableMatchDefaultArm,
                    Issue::warning("This default arm will never be executed.")
                        .with_annotation(
                            Annotation::primary(default_arm.span()).with_message("Unreachable default arm."),
                        )
                        .with_annotation(
                            Annotation::secondary(definite_match.span())
                                .with_message("This earlier arm always matches if reached."),
                        )
                        .with_annotation(
                            Annotation::secondary(self.expression.span())
                                .with_message("Subject expression being matched."),
                        )
                        .with_note(
                            "A prior arm always strictly matches the subject, making this default arm unreachable.",
                        )
                        .with_help("Reorder or remove the unreachable default arm to clean up your code."),
                );
            } else {
                if remaining_subject_type.is_never() {
                    context.buffer.report(
                        TypingIssueKind::UnreachableMatchDefaultArm,
                        Issue::warning(
                            "The `default` arm of this match expression is unreachable."
                        )
                        .with_annotation(
                            Annotation::primary(default_arm.span()).with_message("This `default` arm will never be executed")
                        )
                        .with_annotation(
                            Annotation::secondary(self.expression.span()).with_message(format!("Subject expression of type `{}`", subject_type.get_id(Some(context.interner))))
                        )
                        .with_note(
                            "All possible values of the subject expression are covered by the preceding conditional arms, leaving no cases for the `default` arm to handle."
                        )
                        .with_help(
                            "Remove this unreachable `default` arm, or review the conditions of the preceding arms if some cases were intended to fall through to default."
                        ),
                    );
                }

                if let Some(default_arm_type) = artifacts.get_expression_type(&default_arm.expression).cloned() {
                    if possible_results.is_empty() && !match_arms.is_empty() {
                        context.buffer.report(
                            TypingIssueKind::MatchDefaultArmAlwaysExecuted,
                            Issue::warning("This default arm will always execute.")
                                .with_annotation(
                                    Annotation::primary(default_arm.span()).with_message("This default arm is always executed."),
                                )
                                .with_annotations(
                                    match_arms.iter().map(|arm| {
                                        Annotation::secondary(arm.span()).with_message("This arm will never match.")
                                    })
                                )
                                .with_annotation(
                                    Annotation::secondary(self.expression.span())
                                        .with_message("Subject expression being matched."),
                                )
                                .with_note("This default arm is the only one that will match, making it always executed.")
                                .with_help("Consider removing the `match` expression and using the default arm's expression directly."),
                        );
                    }

                    possible_results.push(default_arm_type);
                } else {
                    possible_results.push(get_never());
                }
            }
        } else if possible_results.is_empty() || (!remaining_subject_type.is_never() && definite_match.is_none()) {
            context.buffer.report(
                TypingIssueKind::MatchNotExhaustive,
                Issue::error(format!(
                    "Non-exhaustive `match` expression: subject of type `{}` is not fully handled.",
                    subject_type.get_id(Some(context.interner))
                ))
                .with_annotation(
                    Annotation::primary(self.expression.span())
                        .with_message(format!(
                            "Unhandled portion of subject: `{}`",
                            remaining_subject_type.get_id(Some(context.interner))
                        ))
                )
                .with_annotation(
                    Annotation::secondary(self.r#match.span()).with_message("This `match` expression does not cover all cases and lacks a `default` arm.")
                )
                .with_note(
                    "If the subject expression evaluates to one of the unhandled types at runtime, PHP will throw an `UnhandledMatchError`."
                )
                .with_help(format!(
                    "Add conditional arms to cover type(s) `{}` or include a `default` arm to handle all other possibilities.",
                    remaining_subject_type.get_id(Some(context.interner))
                )),
            );
        }

        let resulting_type = if possible_results.is_empty() {
            get_never()
        } else if possible_results.len() == 1 {
            possible_results[0].clone()
        } else {
            let mut union = possible_results[0].clone();
            for possible_result in possible_results.iter().skip(1) {
                union = combine_union_types(&union, possible_result, context.codebase, context.interner, false);
            }

            union
        };

        artifacts.set_expression_type(self, resulting_type);

        Ok(())
    }
}

/// Subtracts the types in `type_to_remove` from `existing_type`.
///
/// This function iterates through each atomic type in `existing_type`. For each of these,
/// it iteratively applies the logic of "is not `atomic_from_remove_set`" for every
/// atomic type in `type_to_remove`. This effectively refines each part of `existing_type`
/// to exclude any possibilities covered by `type_to_remove`.
///
/// This is primarily useful for determining remaining possible types for a match subject
/// after some conditional arms have been considered.
///
/// # Arguments
///
/// * `context` - The reconciliation context, providing access to codebase and interner.
/// * `existing_type` - The initial `TUnion` type (the minuend).
/// * `type_to_remove` - The `TUnion` type whose components should be subtracted from `existing_type`.
///
/// # Returns
///
/// A new `TUnion` representing `existing_type - type_to_remove`.
pub fn subtract_union_types(
    context: &mut Context<'_>,
    artifacts: &mut AnalysisArtifacts,
    existing_type: TUnion,
    type_to_remove: TUnion,
) -> TUnion {
    if type_to_remove.is_never() || existing_type.is_never() {
        return existing_type;
    }

    if existing_type == type_to_remove {
        return get_never();
    }

    if !can_expression_types_be_identical(context.codebase, context.interner, &existing_type, &type_to_remove, true) {
        return existing_type;
    }

    if !(existing_type.has_literal_value() && type_to_remove.has_literal_value())
        && union_comparator::is_contained_by(
            context.codebase,
            context.interner,
            &existing_type,
            &type_to_remove,
            false,
            false,
            true,
            &mut ComparisonResult::new(),
        )
    {
        return existing_type;
    }

    let mut final_refined_union = subtract_handled_enum_cases(context, existing_type, &type_to_remove);
    let mut reconcilation_context =
        ReconcilationContext::new(context.interner, context.codebase, &mut context.buffer, artifacts);

    for atomic in type_to_remove.types {
        if let TAtomic::Object(TObject::Enum(_)) = atomic {
            continue; // Enums are handled separately
        }

        let assertion = Assertion::IsNotType(atomic);
        let key = final_refined_union.get_id(Some(reconcilation_context.interner));
        final_refined_union = negated_assertion_reconciler::reconcile(
            &mut reconcilation_context,
            &assertion,
            &final_refined_union,
            false,
            None,
            key,
            None,
            true,
        );

        if final_refined_union.is_never() {
            break;
        }
    }

    final_refined_union
}

fn subtract_handled_enum_cases(
    context: &mut Context<'_>,
    remaining_subject_type: TUnion,
    condition_type: &TUnion,
) -> TUnion {
    let handled_cases = condition_type
        .get_enum_cases()
        .into_iter()
        .filter_map(|(enum_name, case_name)| case_name.map(|name| (enum_name, name)))
        .collect::<HashSet<_>>();

    if handled_cases.is_empty() {
        return remaining_subject_type;
    }

    let mut final_atomic_types: Vec<TAtomic> = Vec::new();
    let mut subject_possible_cases: HashSet<(StringIdentifier, StringIdentifier)> = HashSet::default();

    for atomic in remaining_subject_type.types {
        if let TAtomic::Object(TObject::Enum(enum_object)) = atomic {
            if let Some(case_name) = enum_object.case {
                subject_possible_cases.insert((enum_object.name, case_name));
            } else if let Some(enum_metadata) = get_class_like(context.codebase, context.interner, &enum_object.name) {
                for (case_name, _) in enum_metadata.get_enum_cases() {
                    subject_possible_cases.insert((enum_object.name, *case_name));
                }
            }
        } else {
            final_atomic_types.push(atomic);
        }
    }

    for handled_case in &handled_cases {
        subject_possible_cases.remove(handled_case);
    }

    for (enum_name, case_name) in &subject_possible_cases {
        final_atomic_types.push(TAtomic::Object(TObject::Enum(TEnum { name: *enum_name, case: Some(*case_name) })));
    }

    TUnion::new(combiner::combine(final_atomic_types, context.codebase, context.interner, false))
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::issue::TypingIssueKind;
    use crate::test_analysis;

    test_analysis! {
        name = match_expression_empty,
        code = indoc! {r#"
            <?php

            $subject = 1;
            $result = match ($subject) {
                // No arms
            };
        "#},
        issues = [
            TypingIssueKind::EmptyMatchExpression,
            TypingIssueKind::ImpossibleAssignment,
        ]
    }

    test_analysis! {
        name = match_expression_only_default,
        code = indoc! {r#"
            <?php

            /** @param int $_res */
            function expect_int(int $_res): void {}

            $subject = "a";
            $result = match ($subject) {
                default => 42,
            };

            expect_int($result);
        "#},
        issues = [TypingIssueKind::MatchExpressionOnlyDefaultArm]
    }

    test_analysis! {
        name = match_expression_duplicate_default,
        code = indoc! {r#"
            <?php

            $subject = "test";
            $result = match ($subject) {
                default => 1,
                "a" => 2,
                default => 3, // Duplicate
            };
        "#},
        issues = [
            TypingIssueKind::DuplicateMatchDefaultArm, // Duplicate default arm
            TypingIssueKind::UnreachableMatchArm, // `"a"` is unreachable
            TypingIssueKind::MatchDefaultArmAlwaysExecuted, // For the default arm
        ]
    }

    test_analysis! {
        name = match_expression_unknown_subject_type,
        code = indoc! {r#"
            <?php

            // undefined_function() will result in mixed or unknown type for $subject
            $result = match (undefined_function()) {
                1 => "one",
                default => "other",
            };
        "#},
        issues = [
            TypingIssueKind::NonExistentFunction, // For undefined_function()
            TypingIssueKind::UnknownMatchSubjectType
        ]
    }

    test_analysis! {
        name = match_expression_unknown_condition_type,
        code = indoc! {r#"
            <?php

            $subject = 1;
            $result = match ($subject) {
                undefined_function() => "one", // Condition type is unknown
                default => "other",
            };
        "#},
        issues = [
            TypingIssueKind::NonExistentFunction, // For undefined_function()
            TypingIssueKind::UnknownMatchConditionType
        ]
    }

    test_analysis! {
        name = match_expression_unknown_arm_expression_type,
        code = indoc! {r#"
            <?php

            $subject = 1;
            $result = match ($subject) {
                1 => undefined_function(), // Arm expression type is unknown
                default => "other",
            };
        "#},
        issues = [
            TypingIssueKind::NonExistentFunction, // For undefined_function()
            TypingIssueKind::MatchArmAlwaysTrue, // This arm always matches the subject type
            TypingIssueKind::UnreachableMatchDefaultArm, // Default arm is unreachable
            TypingIssueKind::ImpossibleAssignment // If the result is unknown
        ]
    }

    test_analysis! {
        name = matching_enum_cases,
        code = indoc! {r#"
            <?php

            enum Color {
                case Red;
                case Green;
                case Blue;
            }

            enum ExtendedColor {
                case Red;
                case Green;
                case Blue;
                case Yellow;
            }

            enum TextColor {
                case White;
                case Black;
            }

            function get_hex_color(Color|ExtendedColor|TextColor $color): string {
                return match ($color) {
                    Color::Red, ExtendedColor::Red => '#FF0000',
                    Color::Green, ExtendedColor::Green => '#00FF00',
                    Color::Blue, ExtendedColor::Blue => '#0000FF',
                    ExtendedColor::Yellow => '#FFFF00',
                    TextColor::White => '#FFFFFF',
                    TextColor::Black => '#000000',
                };
            }
        "#},
    }

    test_analysis! {
        name = matching_enum_cases_with_default,
        code = indoc! {r#"
            <?php

            enum Color {
                case Red;
                case Green;
                case Blue;
            }

            enum ExtendedColor {
                case Red;
                case Green;
                case Blue;
                case Yellow;
            }

            enum TextColor {
                case White;
                case Black;
            }

            function get_hex_color(Color|ExtendedColor|TextColor $color): string {
                return match ($color) {
                    Color::Red, ExtendedColor::Red => '#FF0000',
                    Color::Green, ExtendedColor::Green => '#00FF00',
                    Color::Blue, ExtendedColor::Blue => '#0000FF',
                    ExtendedColor::Yellow => '#FFFF00',
                    TextColor::White => '#FFFFFF',
                    TextColor::Black => '#000000',
                    default => '#000000', // matches nothing..
                };
            }
        "#},
        issues = [
            TypingIssueKind::UnreachableMatchDefaultArm, // Default arm is unreachable since all cases are handled
        ]
    }

    test_analysis! {
        name = matching_enum_cases_missing,
        code = indoc! {r#"
            <?php

            enum Color {
                case Red;
                case Green;
                case Blue;
            }

            enum ExtendedColor {
                case Red;
                case Green;
                case Blue;
                case Yellow;
            }

            enum TextColor {
                case White;
                case Black;
            }

            function get_hex_color(Color|ExtendedColor|TextColor $color): string {
                return match ($color) {
                    Color::Red, ExtendedColor::Red => '#FF0000',
                    Color::Green, ExtendedColor::Green => '#00FF00',
                    Color::Blue => '#0000FF',
                    ExtendedColor::Yellow => '#FFFF00',
                    TextColor::White => '#FFFFFF',
                    TextColor::Black => '#000000',
                };
            }
        "#},
        issues = [
            TypingIssueKind::MatchNotExhaustive, // Missing cases for ExtendedColor::Blue
        ]
    }

    test_analysis! {
        name = matching_enum_cases_missing_with_default,
        code = indoc! {r#"
            <?php

            enum Color {
                case Red;
                case Green;
                case Blue;
            }

            enum ExtendedColor {
                case Red;
                case Green;
                case Blue;
                case Yellow;
            }

            enum TextColor {
                case White;
                case Black;
            }

            function get_hex_color(Color|ExtendedColor|TextColor $color): string {
                return match ($color) {
                    Color::Red, ExtendedColor::Red => '#FF0000',
                    Color::Green, ExtendedColor::Green => '#00FF00',
                    Color::Blue, ExtendedColor::Blue => '#0000FF',
                    ExtendedColor::Yellow => '#FFFF00',
                    TextColor::White => '#FFFFFF',
                    default => '#000000', // Default case handling `TextColor::Black`
                };
            }
        "#},
    }

    test_analysis! {
        name = match_expression_unreachable_condition_type_mismatch,
        code = indoc! {r#"
            <?php

            $subject = 1; // int
            $result = match ($subject) {
                "1" => "one_string", // string !== int
                1 => "one_int",
                default => "other",
            };
        "#},

        issues = [
            TypingIssueKind::UnreachableMatchArm, // `"1"` is unreachable
            TypingIssueKind::MatchArmAlwaysTrue, // `1` is always true for the subject type
            TypingIssueKind::UnreachableMatchDefaultArm // Default arm is unreachable
        ]
    }

    test_analysis! {
        name = match_expression_always_true_condition,
        code = indoc! {r#"
            <?php

            /** @param string $_res */
            function expect_string(string $_res): void {}

            $subject = "specific_value";
            $result = match ($subject) {
                "specific_value" => "matched", // This will always match if subject is "specific_value"
                "another" => "never_here",
                default => "default_never_here",
            };

            expect_string($result);
        "#},
        issues = [
            TypingIssueKind::MatchArmAlwaysTrue, // For the first arm
            TypingIssueKind::UnreachableMatchArm, // For the second arm
            TypingIssueKind::UnreachableMatchDefaultArm // For the default arm
        ]
    }

    test_analysis! {
        name = match_expression_unreachable_due_to_previous_always_true,
        code = indoc! {r#"
            <?php

            $subject = 10;
            $result = match ($subject) {
                10 => "first",      // This arm always matches if $subject is 10
                10 => "second",     // This arm is unreachable
                default => "default",
            };
        "#},
        issues = [
            TypingIssueKind::MatchArmAlwaysTrue, // For the first `10 => ...`
            TypingIssueKind::UnreachableMatchArm, // For the second `10 => ...`
            TypingIssueKind::UnreachableMatchDefaultArm, // For the default arm
        ]
    }

    test_analysis! {
        name = match_expression_unreachable_default_due_to_always_true,
        code = indoc! {r#"
            <?php
            $subject = true;
            $result = match ($subject) {
                true => "always_true_match", // This arm always matches if $subject is true
                default => "unreachable_default",
            };
        "#},
        issues = [
            TypingIssueKind::MatchArmAlwaysTrue,
            TypingIssueKind::UnreachableMatchDefaultArm
        ]
    }

    test_analysis! {
        name = match_expression_simple_int,
        code = indoc! {r#"
            <?php

            function expect_string(string $_): void {}

            $value = 1;
            $result = match ($value) {
                1 => "one",
                2 => "two",
                default => "other",
            };

            expect_string($result);
        "#},
        issues = [
            TypingIssueKind::MatchArmAlwaysTrue, // For the first arm
            TypingIssueKind::UnreachableMatchArm, // For the second arm
            TypingIssueKind::UnreachableMatchDefaultArm // For the default arm
        ]
    }

    test_analysis! {
        name = match_expression_simple_string,
        code = indoc! {r#"
            <?php

            function expect_int(int $_): void {}

            $value = "a";
            $result = match ($value) {
                "a" => 10,
                "b" => 20,
                default => 0,
            };

            expect_int($result); // Result type int
        "#},
        issues = [
            TypingIssueKind::MatchArmAlwaysTrue, // For the first arm
            TypingIssueKind::UnreachableMatchArm, // For the second arm
            TypingIssueKind::UnreachableMatchDefaultArm // For the default arm
        ]
    }

    test_analysis! {
        name = match_expression_multiple_conditions_one_arm,
        code = indoc! {r#"
            <?php

            function expect_string(string $_): void {}

            $value = 2;
            $result = match ($value) {
                1, 2, 3 => "small_number",
                4, 5 => "medium_number",
                default => "large_number",
            };

            expect_string($result);
        "#},
        issues = [
            TypingIssueKind::MatchArmAlwaysTrue, // For the first arm 2nd condition
            TypingIssueKind::UnreachableMatchArmCondition, // For the first arm other 1st condition
            TypingIssueKind::UnreachableMatchArmCondition, // For the first arm other 3rd condition
            TypingIssueKind::UnreachableMatchArm, // For the second arm
            TypingIssueKind::UnreachableMatchDefaultArm // For the default arm

        ]
    }

    test_analysis! {
        name = match_expression_only_default_matches,
        code = indoc! {r#"
            <?php
            $value = 100;
            $result = match ($value) {
                1 => true,
                2 => true,
                default => false,
            };
        "#},
        issues = [
            TypingIssueKind::UnreachableMatchArm, // For the first arm
            TypingIssueKind::UnreachableMatchArm, // For the second arm
            TypingIssueKind::MatchDefaultArmAlwaysExecuted, // For the default arm
        ]
    }

    test_analysis! {
        name = match_expression_null_condition,
        code = indoc! {r#"
            <?php

            $value = null;
            $result = match ($value) {
                null => "is_null",
                default => "not_null",
            };
        "#},
        issues = [
            TypingIssueKind::MatchArmAlwaysTrue, // For the first arm
            TypingIssueKind::UnreachableMatchDefaultArm, // For the second arm
        ]
    }

    test_analysis! {
        name = match_expression_bool_condition,
        code = indoc! {r#"
            <?php
            /** @param 0 $_ */
            function expect_int(int $_): void {}

            $value = false;
            $result = match ($value) {
                true => 1,
                false => 0,
            };

            expect_int($result);
        "#},
        issues = [
            TypingIssueKind::UnreachableMatchArm, // For the first arm
            TypingIssueKind::MatchArmAlwaysTrue, // For the second arm
        ]
    }

    test_analysis! {
        name = match_expression_subject_is_variable,
        code = indoc! {r#"
            <?php

            /** @return "start"|"stop"|"unknown" */
            function get_command(): string { return "start"; }

            function expect_string(string $_): void {}

            $command = get_command();
            $message = match ($command) {
                "start" => "Processing started.",
                "stop" => "Processing stopped.",
                default => "Unknown command.",
            };

            expect_string($message);
        "#}
    }

    test_analysis! {
        name = match_expression_result_type_union,
        code = indoc! {r#"
            <?php

            function get_int(): int { return 1; }
            function take_int_or_string(int|string $_): void {}

            $result = match (get_int()) {
                1 => 100,           // int
                2 => "two hundred", // string
                default => 0,       // int
            };

            take_int_or_string($result); // Result type is int|string
        "#}
    }

    test_analysis! {
        name = match_expression_unhandled_subject_no_default,
        code = indoc! {r#"
            <?php

            $value = 3; // This value is not handled
            $result = match ($value) {
                1 => "one",
                2 => "two",
            };
        "#},
        issues = [
            TypingIssueKind::MatchNotExhaustive, // No default arm to handle unhandled subjects
            TypingIssueKind::UnreachableMatchArm, // For the first arm
            TypingIssueKind::UnreachableMatchArm, // For the second arm
            TypingIssueKind::ImpossibleAssignment, // Result type is never
        ]
    }

    test_analysis! {
        name = match_expression_condition_is_expression,
        code = indoc! {r#"
            <?php

            /** @param 20|0 $_ */
            function expect_int(int $_): void {}
            function get_target(): int { return 2; }

            $value = 2;
            $result = match ($value) {
                1 + 0 => 10,
                get_target() => 20, // Type of get_target() should be used
                default => 0,
            };

            expect_int($result);
        "#},
        issues = [
            TypingIssueKind::UnreachableMatchArm,
        ]
    }

    test_analysis! {
        name = match_expression_partially_redundant_condition,
        code = indoc! {r#"
            <?php

            $status = 2;
            $message = match ($status) {
                1, 2 => "Processed",
                2 => "Also Processed, but unreachable", // This specific condition '2' is covered by '1, 2'
                default => "Error"
            };
        "#},
        issues = [
            TypingIssueKind::UnreachableMatchArmCondition, // For the `1` in the first arm
            TypingIssueKind::MatchArmAlwaysTrue, // For the `2` in the first arm
            TypingIssueKind::UnreachableMatchArm, // For the second arm
            TypingIssueKind::UnreachableMatchDefaultArm, // For the default arm
        ]
    }

    test_analysis! {
        name = match_expression_exhaustive_no_default,
        code = indoc! {r#"
            <?php

            function get_bool(): bool { return true; }
            function expect_bool(bool $_b): void {}

            $subject = get_bool(); // Type is bool (true|false)
            $result = match ($subject) {
                true => false,
                false => true,
            };
            expect_bool($result);
        "#}
    }

    test_analysis! {
        name = match_expression_unreachable_later_conditional_arm,
        code = indoc! {r#"
            <?php

            /** @return 1|2 */
            function get_one_or_two(): int { return 1; }

            /** @param "one"|"two" $_ */
            function take_one_or_two_string(string $_): void {}

            $subject = get_one_or_two();
            $result = match ($subject) {
                1 => "one",
                2 => "two",
                3 => "three", // Unreachable, as subject can only be 1 or 2
            };

            take_one_or_two_string($result); // Result type is "one"|"two"
        "#},
        issues = [
            TypingIssueKind::UnreachableMatchArm,
        ]
    }

    test_analysis! {
        name = match_expression_unreachable_conditional_due_to_exhaustion,
        code = indoc! {r#"
            <?php

            /** @return 1|2 $val */
            function get_one_or_two_for_match(): int { return 1; }

            $val = get_one_or_two_for_match();
            $result = match ($val) {
                1 => 'A',
                2 => 'B',
                3 => 'C', // Unreachable because $val can only be 1 or 2
            };
        "#},
        issues = [
            TypingIssueKind::UnreachableMatchArm, // For the '3 => C' arm
        ]
    }

    test_analysis! {
        name = match_expression_useless_default_arm,
        code = indoc! {r#"
            <?php

            function get_bool(): bool { return true; }

            $subject = get_bool(); // Type is bool
            $result = match ($subject) {
                true => "is_true",
                false => "is_false",
                default => "unreachable_default", // Useless default
            };
        "#},
        issues = [TypingIssueKind::UnreachableMatchDefaultArm]
    }

    test_analysis! {
        name = match_expression_not_exhaustive_union_subject_missing_default,
        code = indoc! {r#"
            <?php

            /** @return int|string */
            function get_int_or_string(): int|string { return "hello"; }

            $subject = get_int_or_string();
            $result = match ($subject) {
                1 => "one",
                "hello" => "greeting",
                // Missing case for other ints or other strings, and no default
            };
        "#},
        issues = [
            TypingIssueKind::MatchNotExhaustive
        ]
    }

    test_analysis! {
        name = match_expression_unreachable_specific_condition_in_arm,
        code = indoc! {r#"
            <?php

            /** @return 1|2 */
            function get_one_or_two_again(): int { return 1; }

            $subject = get_one_or_two_again(); // Subject is int(1)|int(2)
            $result = match ($subject) {
                1 => "is one",
                2, 3 => "is two or three", // Condition '3' is unreachable as subject is never int(3)
                default => "other",
            };
        "#},
        issues = [
            TypingIssueKind::UnreachableMatchArmCondition, // For condition '3'
            TypingIssueKind::UnreachableMatchDefaultArm // Default is unreachable
        ]
    }

    test_analysis! {
        name = match_expression_all_conditional_arms_unreachable_no_default,
        code = indoc! {r#"
            <?php

            $subject = "actual_string";
            $result = match ($subject) {
                1 => "one",        // Unreachable (string vs int)
                true => "trueish", // Unreachable (string vs bool)
            };
        "#},
        issues = [
            TypingIssueKind::UnreachableMatchArm, // For 1 => "one"
            TypingIssueKind::UnreachableMatchArm, // For true => "trueish"
            TypingIssueKind::MatchNotExhaustive,  // Because no arms match and no default
            TypingIssueKind::ImpossibleAssignment // Because $result gets type `never`
        ]
    }

    test_analysis! {
        name = match_expression_uses_variables_from_condition,
        code = indoc! {r#"
            <?php

            function get_bool(): bool { return true; }
            function expect_false(false $_): void {}

            $subject = get_bool(); // Type is bool
            $result = match ($subject) {
                ($truthy = true) => !$truthy,
                ($falsy = false) => false,
            };

            expect_false($result); // Result type is false
        "#}
    }
}
