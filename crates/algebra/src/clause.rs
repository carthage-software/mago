use std::collections::BTreeMap;
use std::hash::Hash;
use std::hash::Hasher;
use std::num::Wrapping;

use ahash::AHasher;
use indexmap::IndexMap;

use mago_codex::assertion::Assertion;
use mago_codex::ttype::TType;
use mago_interner::ThreadedInterner;
use mago_span::Span;

#[derive(Clone, Debug, Eq)]
pub struct Clause {
    pub condition_span: Span,
    pub span: Span,
    pub hash: usize,
    pub possibilities: BTreeMap<String, IndexMap<u64, Assertion>>,
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
        possibilities: BTreeMap<String, IndexMap<u64, Assertion>>,
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

        possibilities.remove(var_id);

        if possibilities.is_empty() {
            return None;
        }

        Some(Clause {
            hash: get_hash(&possibilities, self.span, self.wedge, self.reconcilable),
            possibilities,
            condition_span: self.condition_span,
            span: self.span,
            wedge: self.wedge,
            reconcilable: self.reconcilable,
            generated: self.generated,
        })
    }

    pub fn add_possibility(&self, var_id: String, new_possibility: IndexMap<u64, Assertion>) -> Clause {
        let mut possibilities = self.possibilities.clone();

        possibilities.insert(var_id, new_possibility);

        Clause {
            hash: get_hash(&possibilities, self.span, self.wedge, self.reconcilable),
            possibilities,
            condition_span: self.condition_span,
            span: self.span,
            wedge: self.wedge,
            reconcilable: self.reconcilable,
            generated: self.generated,
        }
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
        let mut impossibilities = BTreeMap::new();

        for (variable, possiblity) in &self.possibilities {
            let mut negations = vec![];

            for (_, assertion) in possiblity {
                match assertion {
                    Assertion::IsIdentical(atomic) | Assertion::IsNotIdentical(atomic) => {
                        if atomic.is_literal() {
                            negations.push(assertion.get_negation());
                        }
                    }
                    _ => {
                        negations.push(assertion.get_negation());
                    }
                }
            }

            if !negations.is_empty() {
                impossibilities.insert(variable.clone(), negations);
            }
        }

        impossibilities
    }

    pub fn to_string(&self, interner: &ThreadedInterner) -> String {
        let mut clause_strings = vec![];

        if self.possibilities.is_empty() {
            return "<empty>".to_string();
        }

        for (var_id, values) in self.possibilities.iter() {
            let mut var_id = var_id.clone();

            if var_id[0..1] == *"*" {
                var_id = "<expr>".to_string()
            }

            let mut clause_string_parts = vec![];

            for (_, value) in values {
                match value {
                    Assertion::Any => {
                        clause_string_parts.push(var_id.to_string() + " is any");
                    }
                    Assertion::Falsy => {
                        clause_string_parts.push("!".to_string() + &var_id);
                        continue;
                    }
                    Assertion::Truthy => {
                        clause_string_parts.push(var_id.clone());
                        continue;
                    }
                    Assertion::IsType(value) | Assertion::IsIdentical(value) => {
                        clause_string_parts.push(var_id.to_string() + " is " + value.get_id(Some(interner)).as_str());
                    }
                    Assertion::IsNotType(value) | Assertion::IsNotIdentical(value) => {
                        clause_string_parts
                            .push(var_id.to_string() + " is not " + value.get_id(Some(interner)).as_str());
                    }
                    _ => {
                        clause_string_parts.push(value.as_string(Some(interner)));
                    }
                }
            }

            if clause_string_parts.len() > 1 {
                let bracketed = "(".to_string() + &clause_string_parts.join(") || (") + ")";
                clause_strings.push(bracketed)
            } else {
                clause_strings.push(clause_string_parts[0].clone());
            }
        }

        let joined_clause = clause_strings.join(") || (");

        if clause_strings.len() > 1 { format!("({joined_clause})") } else { joined_clause }
    }
}

#[inline]
fn get_hash(
    possibilities: &BTreeMap<String, IndexMap<u64, Assertion>>,
    clause_span: Span,
    wedge: bool,
    reconcilable: bool,
) -> usize {
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

        hasher.finish() as usize
    }
}
