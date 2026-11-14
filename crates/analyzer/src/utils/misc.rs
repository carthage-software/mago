use mago_allocator::Arena;
use mago_codex::assertion::Assertion;
use std::rc::Rc;

use foldhash::HashMap;
use foldhash::HashSet;

use mago_algebra::AlgebraThresholds;
use mago_algebra::clause::Clause;
use mago_algebra::negate_formula;
use mago_collector::Collector;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::BinaryOperator;
use mago_syntax::cst::Expression;
use mago_syntax::cst::UnaryPrefix;
use mago_syntax::cst::UnaryPrefixOperator;

use mago_word::Word;

use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::formula::get_formula;

/// Checks for two types of logical issues between a set of existing assertions (`formula_1`)
/// and a new set of assertions (`formula_2`) from a conditional expression.
pub fn check_for_paradox<A>(
    collector: &mut Collector<'_, '_, A>,
    formula_1: &[Rc<Clause>],
    formula_2: &[Clause],
    span: Span,
    algebra_thresholds: &AlgebraThresholds,
    report_exact_repeat: bool,
) where
    A: Arena,
{
    if formula_2.is_empty() || (formula_1.is_empty() && formula_2.len() == 1) {
        return;
    }

    let formula_1_hashes: HashMap<u32, Span> = formula_1.iter().map(|c| (c.hash, c.condition_span)).collect();
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

    if formula_1.is_empty() {
        return;
    }

    if !formula_1.iter().any(|clause_1| {
        clause_1.reconcilable
            && !clause_1.wedge
            && !clause_1.possibilities.is_empty()
            && clause_1
                .possibilities
                .keys()
                .all(|key| formula_2.iter().any(|clause_2| clause_2.possibilities.contains_key(key)))
    }) {
        return;
    }

    let Some(negated_formula_2) = negate_formula(formula_2, algebra_thresholds) else {
        return;
    };

    let formula_1_clauses = formula_1.iter().map(|c| &**c).collect::<Vec<_>>();
    let mut paradox = None;
    let mut implied = None;
    for negated_clause_2 in &negated_formula_2 {
        if !negated_clause_2.reconcilable || negated_clause_2.wedge {
            return;
        }

        let mut is_implied = false;
        for clause_1 in &formula_1_clauses {
            if !clause_1.reconcilable || clause_1.wedge {
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
                is_implied = true;
                if negated_clause_2 != *clause_1 {
                    paradox.get_or_insert((*clause_1, negated_clause_2));
                    break;
                }

                if !clause_1.possibilities.values().any(|assertions| {
                    assertions
                        .values()
                        .any(|assertion| matches!(assertion, Assertion::InArray(_) | Assertion::NotInArray(_)))
                }) {
                    implied.get_or_insert((*clause_1, negated_clause_2));
                }
            }
        }

        if !is_implied {
            return;
        }
    }

    if let Some((original_clause, negated_clause)) = paradox.or(implied.filter(|_| report_exact_repeat)) {
        report_paradoxical_condition(collector, original_clause, negated_clause, span, algebra_thresholds);
    }
}

/// Reports the operands of an `||` condition that the established clauses rule out: such an operand can never be
/// true, so it is dead code even though the whole condition is not a paradox. Operands that all test the same
/// variables (`$a === 'x' || $a === 'y'`) are alternatives of which some may be excluded already, so they are left
/// alone.
pub fn check_for_paradoxical_or_operands<'ctx, 'arena, A>(
    context: &mut Context<'ctx, 'arena, A>,
    block_context: &BlockContext<'ctx>,
    artifacts: &AnalysisArtifacts,
    condition: &Expression<'arena>,
) where
    A: Arena,
{
    if block_context.clauses.is_empty()
        || artifacts.get_expression_type(condition).is_some_and(|condition_type| condition_type.is_always_falsy())
    {
        return;
    }

    let mut operands = Vec::new();
    collect_or_operands(condition, &mut operands);
    if operands.len() < 2 {
        return;
    }

    let algebra_thresholds = context.settings.algebra_thresholds();
    let mut variable_sets = HashSet::default();
    let mut negated_operand_formulas = Vec::with_capacity(operands.len());
    for operand in operands {
        let Some(formula) = get_formula(
            operand.span(),
            operand.span(),
            operand,
            context.get_assertion_context_from_block(block_context),
            artifacts,
            &algebra_thresholds,
            context.settings.formula_size_threshold,
        ) else {
            return;
        };

        let mut variables: Vec<Word> = formula.iter().flat_map(|clause| clause.possibilities.keys().copied()).collect();
        variables.sort_unstable();
        variables.dedup();
        variable_sets.insert(variables);

        let Some(negated_formula) = negate_formula(&formula, &algebra_thresholds) else {
            return;
        };

        let is_known_false =
            artifacts.get_expression_type(operand).is_some_and(|operand_type| operand_type.is_always_falsy());
        negated_operand_formulas.push((operand.span(), negated_formula, is_known_false));
    }

    if variable_sets.len() == 1 {
        return;
    }

    let dead_operands: Vec<(Span, Clause, Clause)> = negated_operand_formulas
        .iter()
        .filter_map(|(span, negated_formula, is_known_false)| {
            find_implication(&block_context.clauses, negated_formula)
                .filter(|_| !is_known_false)
                .map(|(negated_clause, clause)| (*span, clause.clone(), negated_clause.clone()))
        })
        .collect();

    if dead_operands.len() == negated_operand_formulas.len() {
        return;
    }

    for (span, clause, negated_clause) in dead_operands {
        report_paradoxical_condition(&mut context.collector, &clause, &negated_clause, span, &algebra_thresholds);
    }
}

fn collect_or_operands<'ast, 'arena>(
    expression: &'ast Expression<'arena>,
    operands: &mut Vec<&'ast Expression<'arena>>,
) {
    let expression = unwrap_expression(expression);
    if let Expression::Binary(binary) = expression
        && matches!(binary.operator, BinaryOperator::Or(_) | BinaryOperator::LowOr(_))
    {
        collect_or_operands(binary.lhs, operands);
        collect_or_operands(binary.rhs, operands);
    } else {
        operands.push(expression);
    }
}

/// The first clause of the negated formula and the established clause implying it, when the established clauses
/// imply every clause of the negated formula.
fn find_implication<'clauses>(
    formula: &'clauses [Rc<Clause>],
    negated_formula: &'clauses [Clause],
) -> Option<(&'clauses Clause, &'clauses Clause)> {
    let mut first_implication = None;
    let mut implied_count = 0;
    let mut negated_count = 0;

    for negated_clause in negated_formula {
        if !negated_clause.reconcilable || negated_clause.wedge {
            continue;
        }

        negated_count += 1;
        let implying_clause = formula.iter().find(|clause| {
            clause.reconcilable
                && !clause.wedge
                && !clause.possibilities.is_empty()
                && clause.possibilities.iter().all(|(key, possibilities)| {
                    negated_clause.possibilities.get(key) == Some(possibilities)
                        && !possibilities
                            .values()
                            .any(|assertion| matches!(assertion, Assertion::InArray(_) | Assertion::NotInArray(_)))
                })
        });

        if let Some(implying_clause) = implying_clause {
            implied_count += 1;
            first_implication.get_or_insert((negated_clause, &**implying_clause));
        }
    }

    first_implication.filter(|_| negated_count > 0 && implied_count == negated_count)
}

fn report_redundant_condition<A>(
    collector: &mut Collector<'_, '_, A>,
    redundant_clause: &Clause,
    redundant_span: Span,
    original_span: Span,
) where
    A: Arena,
{
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

fn report_paradoxical_condition<A>(
    collector: &mut Collector<'_, '_, A>,
    original_clause: &Clause,
    negated_conflicting_clause: &Clause,
    paradox_span: Span,
    algebra_thresholds: &AlgebraThresholds,
) where
    A: Arena,
{
    let Some(conflicting_clause) = negate_formula(std::slice::from_ref(negated_conflicting_clause), algebra_thresholds)
    else {
        return;
    };

    let new_condition_str =
        conflicting_clause.iter().map(std::string::ToString::to_string).collect::<Vec<_>>().join(" && ");
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
        // `@` only suppresses diagnostics; it yields the operand's value unchanged.
        Expression::UnaryPrefix(UnaryPrefix { operator: UnaryPrefixOperator::ErrorControl(_), operand }) => {
            unwrap_expression(operand)
        }
        _ => expression,
    }
}
