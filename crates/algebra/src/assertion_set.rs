use std::hash::Hash;
use std::hash::Hasher;

use ahash::AHasher;
use ahash::HashSet;

use mago_codex::assertion::Assertion;

/// A type alias representing a disjunction (an "OR" clause) of items.
///
/// For example, `Disjunction<Assertion>` is equivalent to `(Assertion1 OR Assertion2 OR ...)`.
pub type Disjunction<T> = Vec<T>;

/// A type alias representing a conjunction (an "AND" clause) of items.
///
/// For example, `Conjunction<Clause>` is equivalent to `(Clause1 AND Clause2 AND ...)`.
pub type Conjunction<T> = Vec<T>;

/// Represents a logical formula in Conjunctive Normal Form (CNF).
///
/// Each inner `Vec<Assertion>` is a single "OR" clause (a disjunction),
/// and the outer `Vec` represents an "AND" of all these clauses (a conjunction).
///
/// For example, `vec![vec![A, B], vec![C]]` corresponds to the logical
/// formula `(A OR B) AND (C)`.
///
/// See: [Conjunctive Normal Form](https://en.wikipedia.org/wiki/Conjunctive_normal_form)
pub type AssertionSet = Conjunction<Disjunction<Assertion>>;

/// Applies an `OR` operation to a formula in Conjunctive Normal Form (CNF).
///
/// This function takes a single `Assertion` and adds it to every existing `OR`
/// clause within the formula. For example, applying `C` to `(A) AND (B)`
/// results in `(A OR C) AND (B OR C)`.
///
/// See: [Distributive property](https://en.wikipedia.org/wiki/Distributive_property)
pub fn add_or_assertion(possibilities: &mut AssertionSet, assertion: Assertion) {
    if possibilities.is_empty() {
        // If the formula was empty (representing `true`), the result
        // is a single clause with the new assertion.
        possibilities.push(vec![assertion]);
        return;
    }

    for clause in possibilities {
        clause.push(assertion.clone());
    }
}

/// Applies an `AND` operation to a formula in Conjunctive Normal Form (CNF).
///
/// This function takes a single `Assertion` and adds it as a new, separate `AND`
/// clause to the formula. For example, applying `C` to `(A OR B)`
/// results in `(A OR B) AND (C)`.
pub fn add_and_assertion(possibilities: &mut AssertionSet, assertion: Assertion) {
    // Add a new clause containing only the new assertion.
    possibilities.push(vec![assertion]);
}

/// Applies an `AND` operation with a new `OR` clause to a CNF formula.
///
/// This function adds a new clause, which is itself a disjunction of the
/// provided assertions. For example, applying `(C OR D)` to `(A OR B)`
/// results in `(A OR B) AND (C OR D)`.
pub fn add_and_clause(assertion_set: &mut AssertionSet, or_assertions: &[Assertion]) {
    if or_assertions.is_empty() {
        // An empty OR clause is equivalent to `false`. ANDing with `false`
        // makes the entire formula `false`, represented by a single empty clause.
        *assertion_set = vec![vec![]];
        return;
    }

    assertion_set.push(or_assertions.to_vec());
}

/// Negates a formula in Conjunctive Normal Form (CNF).
///
/// This function applies De Morgan's laws to the formula. The process involves:
/// 1. Converting the CNF formula `(A OR B) AND C` to its negated DNF form: `(NOT A AND NOT B) OR (NOT C)`.
/// 2. Converting the resulting DNF back to CNF using the distributive property.
pub fn negate_assertion_set(assertion_set: AssertionSet) -> AssertionSet {
    // 1. Apply De Morgan's laws to get the DNF representation.
    //    `(A OR B) AND C` becomes `(¬A AND ¬B) OR (¬C)`.
    let dnf: AssertionSet = assertion_set
        .into_iter()
        .map(|or_clause| or_clause.into_iter().map(|a| a.get_negation()).collect::<Vec<_>>())
        .filter(|and_clause| !and_clause.is_empty())
        .collect();

    if dnf.is_empty() {
        // The original formula was `true` (no clauses), so its negation is `false`.
        // A `false` CNF is represented by a single, empty OR clause.
        return vec![vec![]];
    }

    // 2. Convert the DNF back to CNF.
    //    Start with the first AND clause of the DNF, converted to CNF.
    //    e.g., `(¬A AND ¬B)` becomes `(¬A) AND (¬B)`.
    let mut result_cnf: AssertionSet = dnf[0].iter().map(|literal| vec![literal.clone()]).collect();

    // Iteratively combine the rest of the DNF clauses.
    for and_clause in dnf.iter().skip(1) {
        let mut next_result_cnf = AssertionSet::new();
        // This is the distributive step: (F1) OR (A AND B) => (F1 OR A) AND (F1 OR B)
        // where F1 is the current CNF.
        for literal in and_clause {
            for cnf_clause in &result_cnf {
                let mut new_clause = cnf_clause.clone();
                new_clause.push(literal.clone());
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
pub fn and_assertion_sets(set_a: AssertionSet, set_b: AssertionSet) -> AssertionSet {
    if (set_a.len() == 1 && set_a[0].is_empty()) || (set_b.len() == 1 && set_b[0].is_empty()) {
        // If either formula is `false`, the result is `false`.
        return vec![vec![]];
    }

    let mut result: AssertionSet = set_a;

    // Create a set of hashes from the first set for fast lookups.
    let mut existing_clause_hashes: HashSet<u64> = result.iter().map(hash_disjunction).collect();

    // Only add clauses from the second set if they are not already present.
    for disjunction in set_b {
        let disjunction_hash = hash_disjunction(&disjunction);
        if existing_clause_hashes.insert(disjunction_hash) {
            result.push(disjunction);
        }
    }

    result
}

/// Calculates a stable hash for a disjunctive clause (an `Or<Assertion>`).
fn hash_disjunction(disjunction: &Disjunction<Assertion>) -> u64 {
    let mut hasher = AHasher::default();
    let mut assertion_hashes: Vec<_> = disjunction.iter().map(mago_codex::assertion::Assertion::to_hash).collect();
    assertion_hashes.sort_unstable();
    assertion_hashes.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use mago_codex::ttype::atomic::TAtomic;
    use mago_codex::ttype::atomic::scalar::TScalar;

    use super::*;

    fn assert_sets_equal(a: AssertionSet, b: AssertionSet) {
        let mut sorted_a: Vec<_> = a
            .into_iter()
            .map(|mut v| {
                v.sort();
                v
            })
            .collect();
        sorted_a.sort();
        let mut sorted_b: Vec<_> = b
            .into_iter()
            .map(|mut v| {
                v.sort();
                v
            })
            .collect();
        sorted_b.sort();
        assert_eq!(sorted_a, sorted_b);
    }

    #[test]
    fn test_add_or_assertion() {
        // Start with (Truthy) AND (Falsy)
        let mut set = vec![vec![Assertion::Truthy], vec![Assertion::Falsy]];
        // OR with IsString => (Truthy OR IsString) AND (Falsy OR IsString)
        add_or_assertion(&mut set, Assertion::IsType(TAtomic::Scalar(TScalar::string())));
        let expected = vec![
            vec![Assertion::Truthy, Assertion::IsType(TAtomic::Scalar(TScalar::string()))],
            vec![Assertion::Falsy, Assertion::IsType(TAtomic::Scalar(TScalar::string()))],
        ];
        assert_sets_equal(expected, set);
    }

    #[test]
    fn test_add_or_assertion_to_empty() {
        let mut set = vec![];
        add_or_assertion(&mut set, Assertion::IsType(TAtomic::Scalar(TScalar::string())));
        let expected = vec![vec![Assertion::IsType(TAtomic::Scalar(TScalar::string()))]];
        assert_sets_equal(expected, set);
    }

    #[test]
    fn test_add_and_assertion() {
        let mut set = vec![vec![Assertion::Truthy, Assertion::Falsy]];
        add_and_assertion(&mut set, Assertion::IsType(TAtomic::Scalar(TScalar::string())));
        let expected = vec![
            vec![Assertion::Truthy, Assertion::Falsy],
            vec![Assertion::IsType(TAtomic::Scalar(TScalar::string()))],
        ];
        assert_sets_equal(expected, set);
    }

    #[test]
    fn test_add_and_clause() {
        let mut set = vec![vec![Assertion::Truthy]];
        let or_clause = vec![
            Assertion::IsType(TAtomic::Scalar(TScalar::string())),
            Assertion::IsType(TAtomic::Scalar(TScalar::int())),
        ];
        add_and_clause(&mut set, &or_clause);
        let expected = vec![
            vec![Assertion::Truthy],
            vec![
                Assertion::IsType(TAtomic::Scalar(TScalar::string())),
                Assertion::IsType(TAtomic::Scalar(TScalar::int())),
            ],
        ];
        assert_sets_equal(expected, set);
    }

    #[test]
    fn test_add_and_empty_clause() {
        let mut set = vec![vec![Assertion::Truthy]];
        add_and_clause(&mut set, &[]);
        let expected = vec![vec![]];
        assert_sets_equal(expected, set);
    }

    #[test]
    fn test_negate_simple_or_clause() {
        let initial_cnf = vec![vec![
            Assertion::IsType(TAtomic::Scalar(TScalar::string())),
            Assertion::IsType(TAtomic::Scalar(TScalar::int())),
        ]];
        let expected_cnf = vec![
            vec![Assertion::IsNotType(TAtomic::Scalar(TScalar::string()))],
            vec![Assertion::IsNotType(TAtomic::Scalar(TScalar::int()))],
        ];
        assert_sets_equal(expected_cnf, negate_assertion_set(initial_cnf));
    }

    #[test]
    fn test_negate_simple_and_clause() {
        let initial_cnf = vec![vec![Assertion::Truthy], vec![Assertion::Falsy]];
        let expected_cnf = vec![vec![Assertion::Falsy, Assertion::Truthy]];
        assert_sets_equal(expected_cnf, negate_assertion_set(initial_cnf));
    }
}
