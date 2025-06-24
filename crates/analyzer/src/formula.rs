use std::collections::BTreeMap;

use indexmap::IndexMap;

use mago_algebra::clause::Clause;
use mago_algebra::disjoin_clauses;
use mago_algebra::negate_formula;
use mago_codex::assertion::Assertion;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::artifacts::AnalysisArtifacts;
use crate::assertion::scrape_assertions;
use crate::context::assertion::AssertionContext;
use crate::context::scope::var_has_root;
use crate::utils::misc::unwrap_expression;

pub fn get_formula(
    conditional_object_id: Span,
    creating_object_id: Span,
    conditional: &Expression,
    assertion_context: AssertionContext<'_>,
    artifacts: &mut AnalysisArtifacts,
) -> Vec<Clause> {
    let expression = unwrap_expression(conditional);

    if let Expression::Binary(binary) = expression
        && let Some(clauses) = handle_binary_operation(
            conditional_object_id,
            &binary.operator,
            &binary.lhs,
            &binary.rhs,
            assertion_context,
            artifacts,
        )
    {
        return clauses;
    }

    if let Expression::UnaryPrefix(unary_prefix) = expression
        && let Some(clauses) = handle_unary_prefix(
            conditional_object_id,
            &unary_prefix.operator,
            &unary_prefix.operand,
            assertion_context,
            artifacts,
        )
    {
        return clauses;
    }

    let anded_assertions = scrape_assertions(expression, artifacts, assertion_context);

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
                        let mut map = BTreeMap::new();
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
        return clauses;
    }

    let mut conditional_ref = String::new();
    conditional_ref += "*";
    conditional_ref += conditional.span().start.offset.to_string().as_str();
    conditional_ref += "-";
    conditional_ref += conditional.span().end.offset.to_string().as_str();

    vec![Clause::new(
        {
            let mut map = BTreeMap::new();
            map.insert(conditional_ref, IndexMap::from([(Assertion::Truthy.to_hash(), Assertion::Truthy)]));
            map
        },
        conditional_object_id,
        creating_object_id,
        None,
        None,
        None,
    )]
}

#[inline]
fn handle_binary_operation(
    conditional_object_id: Span,
    bop: &BinaryOperator,
    left: &Expression,
    right: &Expression,
    assertion_context: AssertionContext<'_>,
    artifacts: &mut AnalysisArtifacts,
) -> Option<Vec<Clause>> {
    if let BinaryOperator::And(_) = bop {
        return Some(handle_binary_and_operation(conditional_object_id, left, right, assertion_context, artifacts));
    }

    if let BinaryOperator::Or(_) = bop {
        return Some(handle_binary_or_operation(conditional_object_id, left, right, assertion_context, artifacts));
    }

    // TODO: shortcuts for
    // if (($a || $b) === false) {}
    // if (($a || $b) !== false) {}
    // if (!$a === true) {}
    // if (!$a === false) {}
    // OR we just remove that pattern with a lint (because it's redundant)

    None
}

#[inline]
fn handle_binary_or_operation(
    conditional_object_id: Span,
    left: &Expression,
    right: &Expression,
    assertion_context: AssertionContext<'_>,
    artifacts: &mut AnalysisArtifacts,
) -> Vec<Clause> {
    let left_span = left.span();
    let left_clauses = get_formula(conditional_object_id, left_span, left, assertion_context, artifacts);

    let right_span = right.span();
    let right_clauses = get_formula(conditional_object_id, right_span, right, assertion_context, artifacts);

    disjoin_clauses(left_clauses, right_clauses, conditional_object_id)
}

#[inline]
fn handle_binary_and_operation(
    conditional_object_id: Span,
    left: &Expression,
    right: &Expression,
    assertion_context: AssertionContext<'_>,
    artifacts: &mut AnalysisArtifacts,
) -> Vec<Clause> {
    let left_span = left.span();
    let mut left_clauses = get_formula(conditional_object_id, left_span, left, assertion_context, artifacts);

    let right_span = right.span();
    let right_clauses = get_formula(conditional_object_id, right_span, right, assertion_context, artifacts);

    left_clauses.extend(right_clauses);

    left_clauses
}

#[inline]
fn handle_unary_prefix(
    conditional_object_id: Span,
    unary_operator: &UnaryPrefixOperator,
    unary_oprand: &Expression,
    assertion_context: AssertionContext<'_>,
    artifacts: &mut AnalysisArtifacts,
) -> Option<Vec<Clause>> {
    if let UnaryPrefixOperator::Not(_) = unary_operator {
        if let Expression::Binary(binary_expression) = unwrap_expression(unary_oprand) {
            if let BinaryOperator::Or(_) = binary_expression.operator {
                return Some(self::handle_binary_and_operation(
                    conditional_object_id,
                    &Expression::UnaryPrefix(mago_syntax::ast::UnaryPrefix {
                        operator: unary_operator.clone(),
                        operand: binary_expression.lhs.clone(),
                    }),
                    &Expression::UnaryPrefix(mago_syntax::ast::UnaryPrefix {
                        operator: unary_operator.clone(),
                        operand: binary_expression.rhs.clone(),
                    }),
                    assertion_context,
                    artifacts,
                ));
            }

            if let BinaryOperator::And(_) = binary_expression.operator {
                return Some(self::handle_binary_or_operation(
                    conditional_object_id,
                    &Expression::UnaryPrefix(mago_syntax::ast::UnaryPrefix {
                        operator: unary_operator.clone(),
                        operand: binary_expression.lhs.clone(),
                    }),
                    &Expression::UnaryPrefix(mago_syntax::ast::UnaryPrefix {
                        operator: unary_operator.clone(),
                        operand: binary_expression.rhs.clone(),
                    }),
                    assertion_context,
                    artifacts,
                ));
            }
        }

        let unary_oprand_span = unary_oprand.span();
        let original_clauses =
            self::get_formula(conditional_object_id, unary_oprand_span, unary_oprand, assertion_context, artifacts);

        return Some(negate_formula(original_clauses));
    }

    None
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
                        return Clause::new(BTreeMap::new(), cond_object_id, cond_object_id, Some(true), None, None);
                    }
                }
            }

            c
        })
        .collect::<Vec<Clause>>()
}
