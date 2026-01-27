use std::hash::Hash;

use ahash::HashSet;
use ahash::HashSetExt;
use indexmap::IndexMap;
use itertools::Itertools;

use mago_atom::Atom;
use mago_atom::AtomSet;
use mago_codex::assertion::Assertion;
use mago_span::Span;

use crate::assertion_set::AssertionSet;
use crate::clause::Clause;

pub mod assertion_set;
pub mod clause;

pub type SatisfyingAssignments = IndexMap<Atom, AssertionSet>;
pub type ActiveTruths = IndexMap<Atom, HashSet<usize>>;

/// Default maximum clauses during CNF saturation.
pub const DEFAULT_SATURATION_COMPLEXITY: u16 = 8_192;

/// Default maximum clauses per side in disjunction operations.
pub const DEFAULT_DISJUNCTION_COMPLEXITY: u16 = 4_096;

/// Default maximum cumulative complexity during negation.
pub const DEFAULT_NEGATION_COMPLEXITY: u16 = 4_096;

/// Default upper limit for consensus optimization.
pub const DEFAULT_CONSENSUS_LIMIT: u16 = 256;

/// Configuration thresholds for CNF formula operations.
///
/// These thresholds control how deeply the analyzer will explore complex logical formulas.
/// Higher values allow more precise analysis but may significantly increase analysis time.
/// Lower values improve speed but may reduce precision on complex conditional code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlgebraThresholds {
    /// Maximum number of clauses to process during saturation (default: 8192).
    ///
    /// Controls how many clauses the simplification algorithm will work with.
    /// If exceeded, saturation returns an empty result to avoid performance issues.
    pub saturation_complexity: u16,

    /// Maximum number of clauses per side in disjunction operations (default: 4096).
    ///
    /// Controls the complexity limit for OR operations between clause sets.
    /// If either side exceeds this, the disjunction returns an empty result.
    pub disjunction_complexity: u16,

    /// Maximum cumulative complexity during negation (default: 4096).
    ///
    /// Controls how complex the negation of a formula can become.
    /// If exceeded, negation returns None to avoid exponential blowup.
    pub negation_complexity: u16,

    /// Upper limit for consensus optimization (default: 256).
    ///
    /// Controls when the consensus rule is applied during saturation.
    /// Only applies when clause count is between 3 and this limit.
    pub consensus_limit: u16,
}

impl Default for AlgebraThresholds {
    fn default() -> Self {
        Self {
            saturation_complexity: DEFAULT_SATURATION_COMPLEXITY,
            disjunction_complexity: DEFAULT_DISJUNCTION_COMPLEXITY,
            negation_complexity: DEFAULT_NEGATION_COMPLEXITY,
            consensus_limit: DEFAULT_CONSENSUS_LIMIT,
        }
    }
}

/// Reduces a set of CNF clauses by exhaustively applying logical simplification rules.
///
/// This function takes a set of clauses and "saturates" it by applying inference and
/// simplification rules until a fixed point is reached. The goal is to produce a
/// logically equivalent but simpler set of clauses.
///
/// ## Logic
///
/// The function applies several logical rules in a single pass:
///
/// 1. **Resolution**: Identifies clauses of the form `(P ∨ A)` and `(P ∨ ¬A)` and resolves
///    them to a new clause `P`.
/// 2. **Unit Propagation**: If a unit clause `(A)` exists, it simplifies any other clause
///    containing `¬A`. For example, `(A)` and `(¬A ∨ B)` becomes `(B)`.
/// 3. **Absorption**: Removes redundant clauses. A clause is redundant if another clause
///    that is a strict subset of it exists. For example, `(A ∨ B)` is removed if `(A)` exists.
/// 4. **Consensus**: Removes redundant consensus clauses. If `(A ∨ X)` and `(¬A ∨ Y)` exist,
///    their consensus is `(X ∨ Y)`. If `(X ∨ Y)` also exists in the set, it is removed
///    as it is logically redundant.
///
/// # Time Complexity
///
/// O(N² × K) where N = number of clauses, K = average variables per clause.
/// In practice, N is typically small (10-50 clauses), making the quadratic loops fast.
/// The function uses index-based tracking to avoid expensive Clause cloning.
///
/// # Arguments
///
/// * `clauses` - A slice of clauses to be simplified.
///
/// # Returns
///
/// A new `Vec<Clause>` containing the simplified, owned clauses.
#[inline]
pub fn saturate_clauses<'a>(
    clauses: impl IntoIterator<Item = &'a Clause>,
    thresholds: &AlgebraThresholds,
) -> Vec<Clause> {
    fn saturate_clauses_inner(
        unique_clauses: Vec<&Clause>,
        saturation_complexity: usize,
        consensus_limit: usize,
    ) -> Vec<Clause> {
        let unique_clauses_len = unique_clauses.len();
        if unique_clauses_len == 0 || unique_clauses_len > saturation_complexity {
            // If the complexity is too high, or there are no clauses, return an empty set.
            return vec![];
        }

        let mut removed_indices: HashSet<usize> = HashSet::default();
        let mut added_clauses: Vec<Clause> = Vec::new();

        // Main simplification loop for resolution and unit propagation.
        'outer: for (clause_a_idx, clause_a) in unique_clauses.iter().enumerate() {
            if !clause_a.reconcilable || clause_a.wedge {
                continue;
            }

            let is_clause_a_simple = clause_a.possibilities.len() == 1
                && clause_a.possibilities.values().next().is_some_and(|p| p.len() == 1);

            if is_clause_a_simple {
                // This is unit propagation: (A) & (!A | B) => (B)
                // `clause_a` is the unit clause (A).
                let (clause_var, var_possibilities) = clause_a.possibilities.iter().next().unwrap();
                let only_type = var_possibilities.values().next().unwrap();
                let negated_clause_type = only_type.get_negation();
                let negated_hash = negated_clause_type.to_hash();

                // Simple O(N) scan - fast for typical small N
                for (clause_b_idx, clause_b) in unique_clauses.iter().enumerate() {
                    if clause_a_idx == clause_b_idx || removed_indices.contains(&clause_b_idx) {
                        continue;
                    }

                    if !clause_b.reconcilable || clause_b.wedge {
                        continue;
                    }

                    // Check if clause_b contains the negated literal
                    let Some(matching_clause_possibilities) = clause_b.possibilities.get(clause_var) else {
                        continue;
                    };

                    if !matching_clause_possibilities.contains_key(&negated_hash) {
                        continue;
                    }

                    let mut clause_var_possibilities = matching_clause_possibilities.clone();
                    clause_var_possibilities.retain(|k, _| k != &negated_hash);

                    removed_indices.insert(clause_b_idx);

                    if clause_var_possibilities.is_empty() {
                        if let Some(updated_clause) = clause_b.remove_possibilities(clause_var) {
                            added_clauses.push(updated_clause);
                        }
                    } else {
                        let updated_clause = clause_b.add_possibility(*clause_var, clause_var_possibilities);
                        added_clauses.push(updated_clause);
                    }
                }
            } else {
                // Resolution: check all other clauses with the same size
                let clause_a_size = clause_a.possibilities.len();

                'inner: for (clause_b_idx, clause_b) in unique_clauses.iter().enumerate() {
                    if clause_a_idx >= clause_b_idx || removed_indices.contains(&clause_b_idx) {
                        continue;
                    }

                    if !clause_b.reconcilable || clause_b.wedge {
                        continue;
                    }

                    // Quick size check before detailed comparison
                    if clause_b.possibilities.len() != clause_a_size {
                        continue;
                    }

                    let mut opposing_key = None;
                    let mut mismatch = false;
                    for (key, a_possibilities) in &clause_a.possibilities {
                        if let Some(b_possibilities) = clause_b.possibilities.get(key) {
                            if index_keys_match(a_possibilities, b_possibilities) {
                                continue;
                            }

                            if a_possibilities.len() == 1
                                && b_possibilities.len() == 1
                                && a_possibilities.values().next().is_some_and(|a| {
                                    b_possibilities.values().next().is_some_and(|b| a.is_negation_of(b))
                                })
                            {
                                if opposing_key.is_some() {
                                    mismatch = true;
                                    break;
                                }
                                opposing_key = Some(key);
                            } else {
                                mismatch = true;
                                break;
                            }
                        } else {
                            mismatch = true;
                            break;
                        }
                    }

                    if mismatch {
                        continue 'inner;
                    }

                    if let Some(key_to_remove) = opposing_key {
                        removed_indices.insert(clause_a_idx);
                        let maybe_new_clause = clause_a.remove_possibilities(key_to_remove);

                        if let Some(new_clause) = maybe_new_clause {
                            added_clauses.push(new_clause);
                        } else {
                            // If removing the possibility makes the clause empty, it's a success,
                            // but no new clause is added.
                            continue 'outer;
                        }
                    }
                }
            }
        }

        // Combine original clauses (minus removed ones) with newly added clauses.
        let mut seen_hashes: HashSet<u32> = HashSet::with_capacity(unique_clauses_len);
        let mut combined_clauses: Vec<Clause> = Vec::with_capacity(unique_clauses_len);

        for (idx, clause) in unique_clauses.iter().enumerate() {
            if !removed_indices.contains(&idx) && seen_hashes.insert(clause.hash) {
                combined_clauses.push((*clause).clone());
            }
        }

        for clause in added_clauses {
            if seen_hashes.insert(clause.hash) {
                combined_clauses.push(clause);
            }
        }

        // Absorption rule: remove redundant clauses. e.g., (A | B) is redundant if A exists.
        // A clause `a` is redundant if a smaller clause `b` exists that is a subset of `a`.
        let mut simplified_clauses: Vec<Clause> = Vec::with_capacity(combined_clauses.len());

        for clause_a in &combined_clauses {
            if clause_a.wedge {
                simplified_clauses.push(clause_a.clone());
                continue;
            }

            let mut is_redundant = false;

            // Check if any smaller clause is a subset of clause_a
            for clause_b in &combined_clauses {
                if std::ptr::eq(clause_a, clause_b) {
                    continue;
                }

                if !clause_b.reconcilable || clause_b.wedge {
                    continue;
                }

                // Only check if clause_b is strictly smaller
                if clause_b.possibilities.len() >= clause_a.possibilities.len() {
                    continue;
                }

                if clause_a.contains(clause_b) {
                    is_redundant = true;
                    break;
                }
            }

            if !is_redundant {
                simplified_clauses.push(clause_a.clone());
            }
        }

        // Consensus rule: remove redundant consensus clauses.
        // (A | X) & (!A | Y) implies (X | Y). If (X | Y) already exists, it is redundant.
        let simplified_clauses_len = simplified_clauses.len();
        if simplified_clauses_len > 2 && simplified_clauses_len < consensus_limit {
            let mut compared_clauses: HashSet<(u32, u32)> = HashSet::default();
            let mut removed_hashes: HashSet<u32> = HashSet::default();

            for (clause_a_idx, clause_a) in simplified_clauses.iter().enumerate() {
                for clause_b in simplified_clauses.iter().skip(clause_a_idx + 1) {
                    if compared_clauses.contains(&(clause_b.hash, clause_a.hash)) {
                        continue;
                    }

                    compared_clauses.insert((clause_a.hash, clause_b.hash));

                    // Find common keys between the two clauses
                    let common_keys: Vec<_> =
                        clause_a.possibilities.keys().filter(|k| clause_b.possibilities.contains_key(*k)).collect();

                    if common_keys.is_empty() {
                        continue;
                    }

                    let mut common_negated_keys: HashSet<&Atom> = HashSet::default();
                    for common_key in &common_keys {
                        let clause_a_possibilities = &clause_a.possibilities[*common_key];
                        let clause_b_possibilities = &clause_b.possibilities[*common_key];

                        if clause_a_possibilities.len() == 1
                            && clause_b_possibilities.len() == 1
                            && clause_a_possibilities.values().next().is_some_and(|a| {
                                clause_b_possibilities.values().next().is_some_and(|b| a.is_negation_of(b))
                            })
                        {
                            common_negated_keys.insert(*common_key);
                        }
                    }

                    if !common_negated_keys.is_empty() {
                        let mut new_possibilities: IndexMap<Atom, IndexMap<u64, Assertion>> = IndexMap::default();

                        for (var_id, possibilities) in &clause_a.possibilities {
                            if !common_negated_keys.contains(var_id) {
                                new_possibilities
                                    .entry(*var_id)
                                    .or_default()
                                    .extend(possibilities.iter().map(|(&k, v)| (k, v.clone())));
                            }
                        }

                        for (var_id, possibilities) in &clause_b.possibilities {
                            if !common_negated_keys.contains(var_id) {
                                new_possibilities
                                    .entry(*var_id)
                                    .or_default()
                                    .extend(possibilities.iter().map(|(&k, v)| (k, v.clone())));
                            }
                        }

                        let conflict_clause =
                            Clause::new(new_possibilities, clause_a.condition_span, clause_a.span, None, None, None);
                        removed_hashes.insert(conflict_clause.hash);
                    }
                }
            }

            simplified_clauses.retain(|f| !removed_hashes.contains(&f.hash));
        }

        simplified_clauses
    }

    let unique_clauses = clauses.into_iter().unique().collect::<Vec<_>>();

    saturate_clauses_inner(unique_clauses, thresholds.saturation_complexity.into(), thresholds.consensus_limit.into())
}

/// Extracts simple assertions from a set of clauses and identifies which are "active".
///
/// This function iterates through a set of clauses to pull out two kinds of information:
///
/// 1.  A collection of all simple assertions made about variables.
/// 2.  A subset of those assertions that are "active" based on a specific conditional context.
///
/// It also populates a given `HashSet` with all variable IDs referenced in the clauses.
///
/// Note: This function does not find a full satisfying assignment (a "model") in the
/// SAT-solver sense. It only extracts explicitly stated assertions where a clause concerns
/// a single variable (e.g., `$foo is TypeA | TypeB`).
///
/// # Arguments
///
/// * `clauses` - A slice of `Clause` objects to be analyzed.
/// * `creating_conditional_id` - An optional `Span` representing a conditional context.
///   An assertion is "active" if its `condition_span` matches this ID.
/// * `conditionally_referenced_var_ids` - A mutable `HashSet` that will be populated with all
///   variable IDs found in non-generated clauses.
///
/// # Returns
///
/// A tuple containing two maps:
///
/// 1. `SatisfyingAssignments`: Maps a variable ID to its list of possible
///    truth assignments. Each inner `Vec<Assertion>` represents a disjunction (OR).
/// 2. `ActiveTruths`: Maps a variable ID to a set of indices. These
///    indices point to the "active" assertions in the first map's `Vec`.
///
/// # Panics
///
/// Panics if a reconcilable clause with exactly one possibility has an empty possibilities map.
/// This should never happen with properly constructed clauses.
#[inline]
pub fn find_satisfying_assignments(
    clauses: &[Clause],
    creating_conditional_id: Option<Span>,
    conditionally_referenced_var_ids: &mut AtomSet,
) -> (SatisfyingAssignments, ActiveTruths) {
    let mut truths: IndexMap<Atom, AssertionSet> = IndexMap::default();
    let mut active_truths: IndexMap<Atom, HashSet<usize>> = IndexMap::default();

    for clause in clauses {
        // Populate referenced variables from all non-generated clauses.
        if !clause.generated {
            for var_id in clause.possibilities.keys() {
                // We only care about actual variables, not temporary expression placeholders.
                if !var_id.as_str().starts_with('*') {
                    conditionally_referenced_var_ids.insert(*var_id);
                }
            }
        }

        // A clause represents a "truth" if it's reconcilable and asserts a single fact
        // about a single variable. This is an oversimplification for `possible_types.len() > 1` cases
        // which represent ORs, but it's consistent with the original logic.
        if !clause.reconcilable || clause.possibilities.len() != 1 {
            continue;
        }

        // Extract the single variable and its possible assertions.
        let (variable_id, possible_types) = clause.possibilities.iter().next().unwrap();
        if variable_id.as_str().starts_with('*') {
            continue;
        }

        let assertions = possible_types.values().cloned().collect::<Vec<_>>();
        let truth_entry = truths.entry(*variable_id).or_default();
        let new_truth_index = truth_entry.len();
        truth_entry.push(assertions);

        if let Some(creating_conditional_id) = creating_conditional_id
            && creating_conditional_id == clause.condition_span
        {
            active_truths.entry(*variable_id).or_default().insert(new_truth_index);
        }
    }

    (truths, active_truths)
}

/// Performs a logical OR (disjunction) on two sets of CNF clauses.
///
/// This function takes two formulas in Conjunctive Normal Form (CNF), `F1` and `F2`,
/// and computes `F1 ∨ F2`. The result is a new formula also in CNF.
///
/// ## Logic
///
/// The implementation uses the distributive property of logic:
/// `(A ∧ B) ∨ (C ∧ D) = (A ∨ C) ∧ (A ∨ D) ∧ (B ∨ C) ∧ (B ∨ D)`
///
/// This is achieved by computing the **Cartesian product** of the clauses from each set.
/// Each clause from `left_clauses` is merged with each clause from `right_clauses` to
/// form the new set of clauses.
///
/// During this process, any newly formed clause that is a **tautology** (e.g., contains
/// `A ∨ ¬A`) is discarded, as it is always true and thus redundant in a CNF formula.
///
/// # Arguments
///
/// * `left_clauses` - A `Vec<Clause>` representing the first CNF formula.
/// * `right_clauses` - A `Vec<Clause>` representing the second CNF formula.
/// * `conditional_object_id` - The `Span` to assign to the newly created clauses.
///
/// # Returns
///
/// A `Vec<Clause>` representing the resulting CNF formula.
#[inline]
#[must_use]
pub fn disjoin_clauses(
    left_clauses: Vec<Clause>,
    right_clauses: Vec<Clause>,
    conditional_object_id: Span,
    thresholds: &AlgebraThresholds,
) -> Vec<Clause> {
    let left_clauses_len = left_clauses.len();
    let right_clauses_len = right_clauses.len();

    if left_clauses_len == 0 {
        // If there are no left clauses, return the right ones.
        return right_clauses;
    }

    if right_clauses_len == 0 {
        // If there are no right clauses, return the left ones.
        return left_clauses;
    }

    if left_clauses_len > usize::from(thresholds.disjunction_complexity)
        || right_clauses_len > usize::from(thresholds.disjunction_complexity)
    {
        // If either side is too complex, bail out early.
        return vec![];
    }

    let mut clauses = vec![];
    let mut has_wedge = false;

    // This is creating the cartesian product of two CNF formulas, which is correct for (F1 ∨ F2).
    // (A ∧ B) ∨ (C ∧ D) becomes (A ∨ C) ∧ (A ∨ D) ∧ (B ∨ C) ∧ (B ∨ D).
    for left_clause in left_clauses {
        for right_clause in &right_clauses {
            if left_clause.wedge && right_clause.wedge {
                has_wedge = true;
                continue;
            }

            if left_clause.wedge {
                clauses.push(right_clause.clone());
                continue;
            }

            if right_clause.wedge {
                clauses.push(left_clause.clone());
                continue;
            }

            let mut possibilities = left_clause.possibilities.clone();
            for (var, possible_types) in &right_clause.possibilities {
                possibilities.entry(*var).or_default().extend(possible_types.iter().map(|(&k, v)| (k, v.clone())));
            }

            // If a combined clause contains `A` and `!A`, it's a tautology (always true)
            // and can be removed from a CNF formula.
            let is_tautology = possibilities.values().any(|var_possibilities| {
                if var_possibilities.len() > 1 {
                    let vals = var_possibilities.values().collect::<Vec<_>>();
                    // This is a naive check; a better one would check all pairs.
                    for (i, v1) in vals.iter().enumerate() {
                        for v2 in &vals[i + 1..] {
                            if v1.is_negation_of(v2) {
                                return true;
                            }
                        }
                    }
                }

                false
            });

            if is_tautology {
                continue;
            }

            clauses.push(Clause::new(
                possibilities,
                conditional_object_id,
                conditional_object_id,
                Some(false),
                Some(left_clause.reconcilable && right_clause.reconcilable),
                Some(true),
            ));
        }
    }

    if has_wedge {
        // If there was `(A ∨ wedge) ∧ (B ∨ wedge)`, result is just `(A ∨ B)` which is handled, but what about just `wedge`?
        // Let's assume a wedge means "an impossible path", so combining with OR is a no-op on the other side.
        // The current logic of skipping seems reasonable.
    }

    clauses
}

/// Computes the logical negation of a formula in Conjunctive Normal Form (CNF).
///
/// This function takes a formula `F` and computes `¬F`. The negation of a CNF formula
/// is transformed back into a simplified CNF representation.
///
/// ## Logic
///
/// The logical negation of a CNF formula typically involves applying De Morgan's laws,
/// which results in a Disjunctive Normal Form (DNF). This function then converts the
/// resulting DNF back into a simplified CNF.
///
/// The implementation achieves this by:
/// 1.  Grouping the "impossibilities" from the input clauses, which is equivalent to
///     negating each literal.
/// 2.  Applying `saturate_clauses` to simplify the resulting set of clauses into a
///     final, normalized CNF.
///
/// If the negation results in a contradiction (e.g., negating a tautology), the function
/// returns a `wedge` clause, which represents a logically false state.
///
/// # Arguments
///
/// * `clauses` - A `Vec<Clause>` representing the CNF formula to be negated.
///
/// # Returns
///
/// A `Some(Vec<Clause>)` representing the negated and simplified CNF formula,
/// or `None` if the negation is not possible due to complexity or other constraints.
#[inline]
#[must_use]
pub fn negate_formula(mut clauses: Vec<Clause>, thresholds: &AlgebraThresholds) -> Option<Vec<Clause>> {
    clauses.retain(|clause| clause.reconcilable);

    if clauses.is_empty() {
        return Some(vec![]);
    }

    let impossible_clauses = group_impossibilities(clauses, thresholds.negation_complexity.into())?;
    if impossible_clauses.is_empty() {
        return Some(vec![]);
    }

    let negated = saturate_clauses(impossible_clauses.iter().as_slice(), thresholds);

    Some(negated)
}

#[inline]
fn group_impossibilities(mut clauses: Vec<Clause>, max_complexity: usize) -> Option<Vec<Clause>> {
    let mut seed_clauses = Vec::new();
    let mut complexity = 1usize;

    let Some(clause) = clauses.pop() else {
        return Some(seed_clauses);
    };

    if !clause.wedge {
        let impossibilities = clause.get_impossibilities();

        for (var, impossible_types) in &impossibilities {
            for impossible_type in impossible_types {
                let mut seed_clause_possibilities = IndexMap::new();
                seed_clause_possibilities
                    .insert(*var, IndexMap::from([(impossible_type.to_hash(), impossible_type.clone())]));

                let seed_clause =
                    Clause::new(seed_clause_possibilities, clause.condition_span, clause.span, None, None, None);

                seed_clauses.push(seed_clause);
            }
        }
    }

    if clauses.is_empty() || seed_clauses.is_empty() {
        return Some(seed_clauses);
    }

    let mut complexity_upper_bound = seed_clauses.len();
    for clause in &clauses {
        let mut possibilities_count = 0;
        let impossibilities = clause.get_impossibilities();
        for impossible_types in impossibilities.values() {
            possibilities_count += impossible_types.len();
        }

        complexity_upper_bound = complexity_upper_bound.saturating_mul(possibilities_count);

        if complexity_upper_bound > max_complexity {
            // If the complexity is too high, bail out early
            return None;
        }
    }

    while let Some(clause) = clauses.pop() {
        let mut new_clauses = Vec::with_capacity(seed_clauses.len() * 4);
        let clause_impossibilities = clause.get_impossibilities();

        for grouped_clause in &seed_clauses {
            for (var, impossible_types) in &clause_impossibilities {
                'next: for impossible_type in impossible_types {
                    complexity += 1;
                    if complexity > max_complexity {
                        // Early bailout
                        return None;
                    }

                    if let Some(new_insert_value) = grouped_clause.possibilities.get(var) {
                        for (_, a) in new_insert_value {
                            if a.is_negation_of(impossible_type) {
                                break 'next;
                            }
                        }
                    }

                    let mut new_clause_possibilities = grouped_clause.possibilities.clone();

                    new_clause_possibilities
                        .entry(*var)
                        .or_insert_with(IndexMap::new)
                        .insert(impossible_type.to_hash(), impossible_type.clone());

                    new_clauses.push(Clause::new(
                        new_clause_possibilities,
                        grouped_clause.condition_span,
                        clause.span,
                        Some(false),
                        Some(true),
                        Some(true),
                    ));
                }
            }
        }

        seed_clauses = new_clauses;
    }

    seed_clauses.reverse();

    Some(seed_clauses)
}

#[inline]
fn index_keys_match<T: Eq + Ord + Hash, U, V>(map1: &IndexMap<T, U>, map2: &IndexMap<T, V>) -> bool {
    map1.len() == map2.len() && map1.keys().all(|k| map2.contains_key(k))
}
