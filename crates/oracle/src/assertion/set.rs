use std::hash::BuildHasher;

use foldhash::HashSet;
use foldhash::fast::FixedState;

use mago_allocator::Arena;
use mago_allocator::CollectIn;
use mago_allocator::vec::Vec;

use crate::assertion::Assertion;

/// A type alias representing a disjunction (an "OR" clause) of items.
///
/// For example, `Disjunction<Assertion>` is equivalent to `(Assertion1 OR Assertion2 OR ...)`.
pub type Disjunction<'arena, A, T> = Vec<'arena, T, A>;

/// A type alias representing a conjunction (an "AND" clause) of items.
///
/// For example, `Conjunction<Clause>` is equivalent to `(Clause1 AND Clause2 AND ...)`.
pub type Conjunction<'arena, A, T> = Vec<'arena, T, A>;

/// Represents a logical formula in Conjunctive Normal Form (CNF).
///
/// Each inner `Vec<Assertion>` is a single "OR" clause (a disjunction),
/// and the outer `Vec` represents an "AND" of all these clauses (a conjunction).
///
/// For example, `vec![vec![A, B], vec![C]]` corresponds to the logical
/// formula `(A OR B) AND (C)`.
///
/// See: [Conjunctive Normal Form](https://en.wikipedia.org/wiki/Conjunctive_normal_form)
pub type AssertionSet<'arena, A> = Conjunction<'arena, A, Disjunction<'arena, A, Assertion<'arena>>>;

/// Applies an `OR` operation to a formula in Conjunctive Normal Form (CNF).
///
/// This function takes a single `Assertion` and adds it to every existing `OR`
/// clause within the formula. For example, applying `C` to `(A) AND (B)`
/// results in `(A OR C) AND (B OR C)`.
///
/// See: [Distributive property](https://en.wikipedia.org/wiki/Distributive_property)
#[inline]
pub fn add_or_assertion<'arena, A>(
    arena: &'arena A,
    possibilities: &mut AssertionSet<'arena, A>,
    assertion: Assertion<'arena>,
) where
    A: Arena,
{
    if possibilities.is_empty() {
        let mut clause = Vec::new_in(arena);
        clause.push(assertion);
        possibilities.push(clause);
        return;
    }

    for clause in possibilities {
        clause.push(assertion);
    }
}

/// Applies an `AND` operation to a formula in Conjunctive Normal Form (CNF).
///
/// This function takes a single `Assertion` and adds it as a new, separate `AND`
/// clause to the formula. For example, applying `C` to `(A OR B)`
/// results in `(A OR B) AND (C)`.
#[inline]
pub fn add_and_assertion<'arena, A>(
    arena: &'arena A,
    possibilities: &mut AssertionSet<'arena, A>,
    assertion: Assertion<'arena>,
) where
    A: Arena,
{
    let mut clause = Vec::new_in(arena);
    clause.push(assertion);
    possibilities.push(clause);
}

/// Applies an `AND` operation with a new `OR` clause to a CNF formula.
///
/// This function adds a new clause, which is itself a disjunction of the
/// provided assertions. For example, applying `(C OR D)` to `(A OR B)`
/// results in `(A OR B) AND (C OR D)`.
#[inline]
pub fn add_and_clause<'arena, A>(
    arena: &'arena A,
    assertion_set: &mut AssertionSet<'arena, A>,
    or_assertions: &[Assertion<'arena>],
) where
    A: Arena,
{
    if or_assertions.is_empty() {
        let mut false_cnf: AssertionSet<'arena, A> = Vec::new_in(arena);
        false_cnf.push(Vec::new_in(arena));
        *assertion_set = false_cnf;
        return;
    }

    assertion_set.push(or_assertions.iter().copied().collect_in(arena));
}

/// Negates a formula in Conjunctive Normal Form (CNF).
///
/// This function applies De Morgan's laws to the formula. The process involves:
/// 1. Converting the CNF formula `(A OR B) AND C` to its negated DNF form: `(NOT A AND NOT B) OR (NOT C)`.
/// 2. Converting the resulting DNF back to CNF using the distributive property.
#[inline]
#[must_use]
pub fn negate_assertion_set<'arena, A>(
    arena: &'arena A,
    assertion_set: AssertionSet<'arena, A>,
) -> AssertionSet<'arena, A>
where
    A: Arena,
{
    let dnf: AssertionSet<'arena, A> = assertion_set
        .into_iter()
        .map(|or_clause| {
            let negated: Disjunction<'arena, A, Assertion<'arena>> =
                or_clause.into_iter().map(|a| a.get_negation()).collect_in(arena);

            negated
        })
        .filter(|and_clause| !and_clause.is_empty())
        .collect_in(arena);

    if dnf.is_empty() {
        let mut false_cnf: AssertionSet<'arena, A> = Vec::new_in(arena);
        false_cnf.push(Vec::new_in(arena));
        return false_cnf;
    }

    let mut result_cnf: AssertionSet<'arena, A> = dnf[0]
        .iter()
        .map(|literal| {
            let mut clause = Vec::new_in(arena);
            clause.push(*literal);

            clause
        })
        .collect_in(arena);

    for and_clause in dnf.iter().skip(1) {
        let mut next_result_cnf: AssertionSet<'arena, A> = Vec::new_in(arena);
        for literal in and_clause {
            for cnf_clause in &result_cnf {
                let mut new_clause = cnf_clause.clone();
                new_clause.push(*literal);
                next_result_cnf.push(new_clause);
            }
        }

        result_cnf = next_result_cnf;
    }

    result_cnf
}

/// Combines two CNF formulas with a logical `AND`, ensuring no duplicate clauses.
///
/// This function merges two sets of clauses, using a `HashSet` to efficiently
/// filter out any clauses from the second set that are already present in the first.
#[inline]
pub fn and_assertion_sets<'arena, A>(
    arena: &'arena A,
    set_a: AssertionSet<'arena, A>,
    set_b: AssertionSet<'arena, A>,
) -> AssertionSet<'arena, A>
where
    A: Arena,
{
    if (set_a.len() == 1 && set_a[0].is_empty()) || (set_b.len() == 1 && set_b[0].is_empty()) {
        let mut false_cnf: AssertionSet<'arena, A> = Vec::new_in(arena);
        false_cnf.push(Vec::new_in(arena));
        return false_cnf;
    }

    let mut result: AssertionSet<'arena, A> = set_a;

    let mut existing_clause_hashes: HashSet<u64> = result.iter().map(|d| hash_disjunction(arena, d)).collect();

    for disjunction in set_b {
        let disjunction_hash = hash_disjunction(arena, &disjunction);
        if existing_clause_hashes.insert(disjunction_hash) {
            result.push(disjunction);
        }
    }

    result
}

/// Calculates a stable hash for a disjunctive clause (an `Or<Assertion>`).
fn hash_disjunction<'arena, A>(arena: &'arena A, disjunction: &Disjunction<'arena, A, Assertion<'arena>>) -> u64
where
    A: Arena,
{
    let mut assertion_hashes: Vec<'arena, u64, A> = disjunction.iter().map(Assertion::to_hash).collect_in(arena);
    assertion_hashes.sort_unstable();

    FixedState::default().hash_one(&assertion_hashes)
}
