use std::collections::BTreeMap;
use std::hash::Hash;
use std::hash::Hasher;
use std::num::Wrapping;

use ahash::AHasher;
use indexmap::IndexMap;

use mago_atom::Atom;
use mago_atom::atom;
use mago_atom::concat_atom;
use mago_atom::empty_atom;
use mago_codex::assertion::Assertion;
use mago_codex::ttype::TType;
use mago_span::Span;

#[derive(Clone, Debug, Eq)]
pub struct Clause {
    pub condition_span: Span,
    pub span: Span,
    pub hash: u32,
    pub possibilities: IndexMap<String, IndexMap<u64, Assertion>>,
    pub wedge: bool,
    pub reconcilable: bool,
    pub generated: bool,
}

impl PartialEq for Clause {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Hash for Clause {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state)
    }
}

impl Clause {
    pub fn new(
        possibilities: IndexMap<String, IndexMap<u64, Assertion>>,
        condition_span: Span,
        span: Span,
        wedge: Option<bool>,
        reconcilable: Option<bool>,
        generated: Option<bool>,
    ) -> Clause {
        Clause {
            condition_span,
            span,
            wedge: wedge.unwrap_or(false),
            reconcilable: reconcilable.unwrap_or(true),
            generated: generated.unwrap_or(false),
            hash: get_hash(&possibilities, span, wedge.unwrap_or(false), reconcilable.unwrap_or(true)),
            possibilities,
        }
    }

    pub fn remove_possibilities(&self, var_id: &String) -> Option<Clause> {
        let mut possibilities = self.possibilities.clone();

        possibilities.shift_remove(var_id);

        if possibilities.is_empty() {
            return None;
        }

        Some(Clause::new(
            possibilities,
            self.condition_span,
            self.span,
            Some(self.wedge),
            Some(self.reconcilable),
            Some(self.generated),
        ))
    }

    pub fn add_possibility(&self, var_id: String, new_possibility: IndexMap<u64, Assertion>) -> Clause {
        let mut possibilities = self.possibilities.clone();

        possibilities.insert(var_id, new_possibility);

        Clause::new(
            possibilities,
            self.condition_span,
            self.span,
            Some(self.wedge),
            Some(self.reconcilable),
            Some(self.generated),
        )
    }

    pub fn contains(&self, other_clause: &Self) -> bool {
        if other_clause.possibilities.len() > self.possibilities.len() {
            return false;
        }

        other_clause.possibilities.iter().all(|(var, possible_types)| {
            self.possibilities
                .get(var)
                .map(|local_possibilities| possible_types.keys().all(|k| local_possibilities.contains_key(k)))
                .unwrap_or(false)
        })
    }

    pub fn get_impossibilities(&self) -> BTreeMap<String, Vec<Assertion>> {
        self.possibilities
            .iter()
            .filter_map(|(variable, possibility)| {
                let negations: Vec<Assertion> = possibility.values().map(Assertion::get_negation).collect();

                if !negations.is_empty() { Some((variable.clone(), negations)) } else { None }
            })
            .collect()
    }

    pub fn to_atom(&self) -> Atom {
        if self.possibilities.is_empty() {
            return atom("<empty>");
        }

        let mut final_result = empty_atom();
        let mut is_first_clause = true;

        for (var_id, values) in &self.possibilities {
            if !is_first_clause {
                final_result = concat_atom!(final_result, " && ");
            }

            is_first_clause = false;

            let var_name = if var_id.starts_with('*') { atom("<expr>") } else { atom(var_id) };

            let mut clause_result = empty_atom();
            let mut is_first_part_in_clause = true;
            for (_, value) in values {
                if !is_first_part_in_clause {
                    clause_result = concat_atom!(clause_result, " || ");
                }
                is_first_part_in_clause = false;

                let part_atom = match value {
                    Assertion::Any => concat_atom!(var_name, " is any"),
                    Assertion::Falsy => concat_atom!("!", var_name),
                    Assertion::Truthy => var_name,
                    Assertion::IsType(v) | Assertion::IsIdentical(v) => {
                        concat_atom!(var_name, " is ", v.get_id())
                    }
                    Assertion::IsNotType(v) | Assertion::IsNotIdentical(v) => {
                        concat_atom!(var_name, " is not ", v.get_id())
                    }
                    _ => value.to_atom(),
                };

                clause_result = concat_atom!(clause_result, part_atom);
            }

            if values.len() > 1 {
                clause_result = concat_atom!("(", clause_result, ")");
            }

            final_result = concat_atom!(final_result, clause_result);
        }

        final_result
    }
}

#[inline]
fn get_hash(
    possibilities: &IndexMap<String, IndexMap<u64, Assertion>>,
    clause_span: Span,
    wedge: bool,
    reconcilable: bool,
) -> u32 {
    if wedge || !reconcilable {
        (Wrapping(clause_span.start.offset)
            + Wrapping(clause_span.end.offset)
            + Wrapping(if wedge { 100000 } else { 0 }))
        .0
    } else {
        let mut hasher = AHasher::default();
        for possibility in possibilities {
            possibility.0.hash(&mut hasher);
            0.hash(&mut hasher);

            for i in possibility.1.keys() {
                i.hash(&mut hasher);
                1.hash(&mut hasher);
            }
        }

        hasher.finish() as u32
    }
}

impl std::fmt::Display for Clause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_atom())
    }
}
