use mago_interner::ThreadedInterner;
use mago_semantics::Semantics;

use crate::query::Query;
use crate::Reference;

#[derive(Debug, Clone)]
pub struct Context<'a> {
    pub interner: &'a ThreadedInterner,
    pub query: &'a Query,
    pub semantics: &'a Semantics,
    pub references: Vec<Reference>,
}

impl<'a> Context<'a> {
    pub fn new(interner: &'a ThreadedInterner, query: &'a Query, semantics: &'a Semantics) -> Self {
        Self { interner, query, semantics, references: Vec::new() }
    }

    pub fn take_references(&mut self) -> Vec<Reference> {
        std::mem::take(&mut self.references)
    }
}
