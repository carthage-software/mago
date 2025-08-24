use std::rc::Rc;

use ahash::HashMap;

use mago_algebra::clause::Clause;
use mago_algebra::negate_formula;
use mago_collector::Collector;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::Span;
use mago_syntax::ast::Expression;

use crate::code::IssueCode;

/// Checks for two types of logical issues between a set of existing assertions (`formula_1`)
/// and a new set of assertions (`formula_2`) from a conditional expression.
pub fn check_for_paradox(
    collector: &mut Collector<'_, '_>,
    formula_1: &[Rc<Clause>],
    formula_2: &[Clause],
    span: Span,
) {
    let formula_1_hashes: HashMap<u32, Span> = HashMap::from_iter(formula_1.iter().map(|c| (c.hash, c.condition_span)));
    let mut formula_2_hashes: HashMap<u32, Span> = HashMap::default();

    for formula_2_clause in formula_2 {
        if !formula_2_clause.generated
            && !formula_2_clause.wedge
            && formula_2_clause.reconcilable
            && let Some(original_span) =
                formula_1_hashes.get(&formula_2_clause.hash).or_else(|| formula_2_hashes.get(&formula_2_clause.hash))
            && *original_span != span
        {
            report_redundant_condition(collector, formula_2_clause, span, *original_span);
        }

        formula_2_hashes.entry(formula_2_clause.hash).or_insert(formula_2_clause.condition_span);
    }

    let Some(negated_formula_2) = negate_formula(formula_2.to_vec()) else {
        return;
    };

    let formula_1_clauses = formula_1.iter().map(|c| &**c).collect::<Vec<_>>();
    for negated_clause_2 in &negated_formula_2 {
        if !negated_clause_2.reconcilable || negated_clause_2.wedge {
            continue;
        }

        for clause_1 in &formula_1_clauses {
            if negated_clause_2 == *clause_1 || !clause_1.reconcilable || clause_1.wedge {
                continue;
            }

            let is_subset = clause_1.possibilities.iter().all(|(key, clause_1_possibilities)| {
                if let Some(clause_2_possibilities) = negated_clause_2.possibilities.get(key) {
                    clause_1_possibilities == clause_2_possibilities
                } else {
                    false
                }
            });

            if is_subset && !clause_1.possibilities.is_empty() {
                report_paradoxical_condition(collector, clause_1, negated_clause_2, span);

                return;
            }
        }
    }
}

fn report_redundant_condition(
    collector: &mut Collector<'_, '_>,
    redundant_clause: &Clause,
    redundant_span: Span,
    original_span: Span,
) {
    let clause_string = redundant_clause.to_string();
    let (kind, title) = if clause_string == "isset" {
        (IssueCode::RedundantIssetCheck, "Redundant `isset` check")
    } else {
        (IssueCode::RedundantCondition, "Redundant condition")
    };

    collector.report_with_code(
        kind,
        Issue::warning(title)
            .with_annotation(
                Annotation::primary(redundant_span)
                    .with_message(format!("This condition (`{clause_string}`) is always true here")),
            )
            .with_annotation(
                Annotation::secondary(original_span)
                    .with_message("This was already established as true by a previous condition here"),
            )
            .with_note("The analyzer determined this condition is guaranteed to be true based on preceding logic, making this check unnecessary.")
            .with_help("Consider removing this redundant conditional check to simplify the code.")
    );
}

fn report_paradoxical_condition(
    collector: &mut Collector<'_, '_>,
    original_clause: &Clause,
    negated_conflicting_clause: &Clause,
    paradox_span: Span,
) {
    let Some(conflicting_clause) = negate_formula(vec![negated_conflicting_clause.clone()]) else {
        return;
    };

    let new_condition_str = conflicting_clause.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" && ");
    let established_fact_str = original_clause.to_string();

    collector.report_with_code(
        IssueCode::ParadoxicalCondition,
        Issue::error("Paradoxical condition")
            .with_annotation(
                Annotation::primary(paradox_span)
                    .with_message(format!("This condition (`{new_condition_str}`) can never be true here")),
            )
            .with_annotation(
                Annotation::secondary(original_clause.condition_span)
                    .with_message("Because of this preceding condition..."),
            )
            .with_note(format!(
                "...the analyzer knows that `{established_fact_str}` must be true for this code path to be taken."
            ))
            .with_note(format!(
                "Therefore, this new condition (`{new_condition_str}`) directly contradicts that established fact."
            ))
            .with_note("As a result, the code this condition guards is unreachable.")
            .with_help("Remove the unreachable code or refactor the conditional logic."),
    );
}

#[inline]
pub const fn unwrap_expression<'ast, 'arena>(expression: &'ast Expression<'arena>) -> &'ast Expression<'arena> {
    match expression {
        Expression::Parenthesized(parenthesized) => unwrap_expression(parenthesized.expression),
        _ => expression,
    }
}

#[inline]
pub fn unique_vec<T: PartialEq>(actions: impl IntoIterator<Item = T>) -> Vec<T> {
    let mut unique_list = Vec::new();
    for action in actions {
        if !unique_list.contains(&action) {
            unique_list.push(action);
        }
    }

    unique_list
}
