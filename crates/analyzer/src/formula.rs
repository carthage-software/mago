use ahash::HashMap;
use indexmap::IndexMap;

use mago_algebra::assertion_set::AssertionSet;
use mago_algebra::clause::Clause;
use mago_algebra::disjoin_clauses;
use mago_algebra::negate_formula;
use mago_codex::assertion::Assertion;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::UnaryPrefix;
use mago_syntax::ast::*;

use crate::artifacts::AnalysisArtifacts;
use crate::assertion::scrape_assertions;
use crate::assertion::scrape_equality_assertions;
use crate::context::assertion::AssertionContext;
use crate::context::scope::var_has_root;
use crate::utils::misc::unwrap_expression;

const FORMULA_SIZE_THRESHOLD: usize = 1000;

/// Recursively traverses a conditional expression to generate a corresponding logical formula.
///
/// This function serves as the primary entry point for converting control flow conditions
/// (e.g., from `if`, `while`, or ternary expressions) into a set of logical clauses.
/// The resulting formula is represented in Disjunctive Normal Form (a vector of clauses
/// where each clause is a conjunction of assertions).
///
/// The function breaks down the expression by handling logical operators:
/// - **Binary `&&` and `||`**: Delegates to specialized handlers (`handle_binary_and_operation`,
///   `handle_binary_or_operation`) to correctly combine the formulas from the left and
///   right-hand sides.
/// - **Unary `!` (Not)**: Applies negation to the operand's formula. It includes an
///   optimization for De Morgan's laws, transforming `!(A || B)` into `!A && !B` and
///   `!(A && B)` into `!A || !B` before processing.
///
/// For any other expression (the base case of the recursion), it scrapes atomic
/// assertions and converts them into a set of clauses.
///
/// # Parameters
///
/// * `conditional_object_id`: The span of the overall conditional statement (e.g., the `if` keyword).
/// * `creating_object_id`: The span of the specific part of the expression currently being analyzed.
/// * `conditional`: The conditional expression to convert into a formula.
/// * `assertion_context`: The context required for generating assertions.
/// * `artifacts`: A mutable reference to the analysis artifacts.
///
/// # Returns
///
/// Returns `Some(Vec<Clause>)` representing the logical formula. Returns `None` if the
/// formula's complexity exceeds a predefined threshold (`FORMULA_SIZE_THRESHOLD`) at
/// any point during the recursive process.
pub fn get_formula(
    conditional_object_id: Span,
    creating_object_id: Span,
    conditional: &Expression,
    assertion_context: AssertionContext<'_, '_>,
    artifacts: &mut AnalysisArtifacts,
) -> Option<Vec<Clause>> {
    let expression = unwrap_expression(conditional);

    if let Expression::Binary(binary) = expression {
        if let BinaryOperator::And(_) = binary.operator {
            return handle_binary_and_operation(
                conditional_object_id,
                binary.lhs,
                binary.rhs,
                assertion_context,
                artifacts,
            );
        }

        if let BinaryOperator::Or(_) = binary.operator {
            return handle_binary_or_operation(
                conditional_object_id,
                binary.lhs,
                binary.rhs,
                assertion_context,
                artifacts,
            );
        }
    }

    if let Expression::UnaryPrefix(unary_prefix) = expression
        && unary_prefix.operator.is_not()
    {
        if let Expression::Construct(Construct::Isset(isset_construct)) = unary_prefix.operand
            && isset_construct.values.len() > 1
        {
            let scraped_assertions = scrape_assertions(unary_prefix.operand, artifacts, assertion_context);

            let mut clauses = Vec::new();

            for assertions in scraped_assertions {
                for (mut var, anded_types) in assertions {
                    if var.starts_with('=') {
                        var = var[1..].to_string();
                    }

                    for orred_types in anded_types {
                        let has_equality = orred_types.first().is_some_and(|t| t.has_equality());
                        let mapped_orred_types = orred_types
                            .into_iter()
                            .map(|orred_type| (orred_type.to_hash(), orred_type))
                            .collect::<IndexMap<_, _>>();

                        clauses.push(Clause::new(
                            {
                                let mut map = IndexMap::new();
                                map.insert(var.clone(), mapped_orred_types);
                                map
                            },
                            conditional_object_id,
                            creating_object_id,
                            Some(false),
                            Some(true),
                            Some(has_equality),
                        ));

                        if clauses.len() > FORMULA_SIZE_THRESHOLD {
                            return None;
                        }
                    }
                }
            }

            return negate_formula(clauses);
        }

        if let Expression::Binary(binary_expression) = unwrap_expression(unary_prefix.operand) {
            if let BinaryOperator::Or(_) = binary_expression.operator {
                return handle_binary_and_operation(
                    conditional_object_id,
                    &Expression::UnaryPrefix(UnaryPrefix {
                        operator: unary_prefix.operator.clone(),
                        operand: assertion_context.arena.alloc(binary_expression.lhs.clone()),
                    }),
                    &Expression::UnaryPrefix(UnaryPrefix {
                        operator: unary_prefix.operator.clone(),
                        operand: assertion_context.arena.alloc(binary_expression.rhs.clone()),
                    }),
                    assertion_context,
                    artifacts,
                );
            }

            if let BinaryOperator::And(_) = binary_expression.operator {
                return handle_binary_or_operation(
                    conditional_object_id,
                    &Expression::UnaryPrefix(UnaryPrefix {
                        operator: unary_prefix.operator.clone(),
                        operand: assertion_context.arena.alloc(binary_expression.lhs.clone()),
                    }),
                    &Expression::UnaryPrefix(UnaryPrefix {
                        operator: unary_prefix.operator.clone(),
                        operand: assertion_context.arena.alloc(binary_expression.rhs.clone()),
                    }),
                    assertion_context,
                    artifacts,
                );
            }
        }

        let unary_operand_span = unary_prefix.operand.span();
        let negated = negate_formula(get_formula(
            conditional_object_id,
            unary_operand_span,
            unary_prefix.operand,
            assertion_context,
            artifacts,
        )?)?;

        return if negated.len() > FORMULA_SIZE_THRESHOLD { None } else { Some(negated) };
    }

    get_formula_from_assertions(
        conditional_object_id,
        creating_object_id,
        expression,
        scrape_assertions(expression, artifacts, assertion_context),
    )
}

/// Creates a logical formula representing a disjunction of equality/identity comparisons.
///
/// This function generates clauses for a formula that is logically equivalent to the
/// expression `subject === conditions[0] || subject === conditions[1] || ...`.
///
/// It iterates through each provided condition, generates clauses for the assertion
/// `subject === condition`, and combines them into a single disjunctive formula. This is
/// often used to model the behavior of `match` arms.
///
/// # Parameters
///
/// * `subject`: The expression on the left-hand side of the equal comparisons.
/// * `conditions`: A vec of expressions to compare against the `subject`.
/// * `assertion_context`: The context required for generating assertions.
/// * `artifacts`: A mutable reference to the analysis artifacts.
/// * `is_identity`: A boolean indicating whether the equality is an identity check (e.g., `===`).
///
/// # Returns
///
/// Returns `Some(Vec<Clause>)` containing the resulting logical formula if successful.
///
/// Returns `None` if the formula's complexity exceeds a predefined threshold
/// (`FORMULA_SIZE_THRESHOLD`), to avoid performance degradation.
#[allow(dead_code)]
pub fn get_disjunctive_equality_formula(
    subject: &Expression,
    conditions: Vec<&Expression>,
    assertion_context: AssertionContext<'_, '_>,
    artifacts: &mut AnalysisArtifacts,
    is_identity: bool,
) -> Option<Vec<Clause>> {
    let subject = unwrap_expression(subject);
    let subject_span = subject.span();

    let mut clauses = vec![];
    for condition in conditions {
        let condition = unwrap_expression(condition);
        let condition_span = condition.span();
        let formula = if subject.is_true() && (!is_identity || condition.evaluates_to_boolean()) {
            get_formula(condition_span, condition_span, condition, assertion_context, artifacts)?
        } else {
            let assertions = scrape_equality_assertions(subject, is_identity, condition, artifacts, assertion_context);
            get_formula_from_assertions(condition_span, subject_span, subject, assertions)?
        };

        clauses = disjoin_clauses(clauses, formula, condition_span);
        if clauses.len() > FORMULA_SIZE_THRESHOLD {
            return None;
        }
    }

    Some(clauses)
}

fn get_formula_from_assertions(
    conditional_object_id: Span,
    creating_object_id: Span,
    conditional: &Expression,
    anded_assertions: Vec<HashMap<String, AssertionSet>>,
) -> Option<Vec<Clause>> {
    let mut clauses = Vec::new();
    for assertions in anded_assertions {
        for (var_id, anded_types) in assertions {
            for orred_types in anded_types {
                let Some(first_type) = orred_types.first() else {
                    continue; // should not happen
                };

                let has_equality = first_type.has_equality();
                clauses.push(Clause::new(
                    {
                        let mut map = IndexMap::new();
                        map.insert(
                            var_id.clone(),
                            orred_types.into_iter().map(|a| (a.to_hash(), a)).collect::<IndexMap<_, _>>(),
                        );
                        map
                    },
                    conditional_object_id,
                    creating_object_id,
                    Some(false),
                    Some(true),
                    Some(has_equality),
                ))
            }
        }
    }

    if !clauses.is_empty() {
        return if clauses.len() > FORMULA_SIZE_THRESHOLD { None } else { Some(clauses) };
    }

    let conditional_span = conditional.span();
    let conditional_ref = format!("*{}-{}", conditional_span.start.offset, conditional_span.end.offset);

    Some(vec![Clause::new(
        {
            let mut map = IndexMap::new();
            map.insert(conditional_ref, IndexMap::from([(Assertion::Truthy.to_hash(), Assertion::Truthy)]));
            map
        },
        conditional_object_id,
        creating_object_id,
        None,
        None,
        None,
    )])
}

pub fn negate_or_synthesize(
    clauses: Vec<Clause>,
    conditional: &Expression,
    assertion_context: AssertionContext<'_, '_>,
    artifacts: &mut AnalysisArtifacts,
) -> Vec<Clause> {
    match negate_formula(clauses) {
        Some(negated_clauses) => negated_clauses,
        None => match get_formula(
            conditional.span(),
            conditional.span(),
            &Expression::UnaryPrefix(UnaryPrefix {
                operator: UnaryPrefixOperator::Not(conditional.span()),
                operand: assertion_context.arena.alloc(conditional.clone()),
            }),
            assertion_context,
            artifacts,
        ) {
            Some(synthesized_clauses) => synthesized_clauses,
            None => {
                // If we cannot negate the formula, we return an empty vector
                // This is a fallback, and it should not happen in normal cases
                vec![Clause::new(IndexMap::new(), conditional.span(), conditional.span(), Some(true), None, None)]
            }
        },
    }
}

#[inline]
fn handle_binary_or_operation(
    conditional_object_id: Span,
    left: &Expression,
    right: &Expression,
    assertion_context: AssertionContext<'_, '_>,
    artifacts: &mut AnalysisArtifacts,
) -> Option<Vec<Clause>> {
    let left_clauses = get_formula(conditional_object_id, left.span(), left, assertion_context, artifacts)?;
    let right_clauses = get_formula(conditional_object_id, right.span(), right, assertion_context, artifacts)?;
    let clauses = disjoin_clauses(left_clauses, right_clauses, conditional_object_id);

    if clauses.len() > FORMULA_SIZE_THRESHOLD { None } else { Some(clauses) }
}

#[inline]
fn handle_binary_and_operation(
    conditional_object_id: Span,
    left: &Expression,
    right: &Expression,
    assertion_context: AssertionContext<'_, '_>,
    artifacts: &mut AnalysisArtifacts,
) -> Option<Vec<Clause>> {
    let mut clauses = get_formula(conditional_object_id, left.span(), left, assertion_context, artifacts)?;
    clauses.extend(get_formula(conditional_object_id, right.span(), right, assertion_context, artifacts)?);

    if clauses.len() > FORMULA_SIZE_THRESHOLD { None } else { Some(clauses) }
}

pub fn remove_clauses_with_mixed_variables(
    clauses: Vec<Clause>,
    mut mixed_var_ids: Vec<&String>,
    cond_object_id: Span,
) -> Vec<Clause> {
    clauses
        .into_iter()
        .map(|c| {
            let keys = c.possibilities.keys().collect::<Vec<_>>();

            let mut new_mixed_var_ids = vec![];
            for i in &mixed_var_ids {
                if !keys.contains(i) {
                    new_mixed_var_ids.push(*i);
                }
            }

            mixed_var_ids = new_mixed_var_ids;
            for key in keys {
                for mixed_var_id in &mixed_var_ids {
                    if var_has_root(key, mixed_var_id) {
                        return Clause::new(IndexMap::new(), cond_object_id, cond_object_id, Some(true), None, None);
                    }
                }
            }

            c
        })
        .collect::<Vec<Clause>>()
}
