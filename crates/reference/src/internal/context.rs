use mago_interner::ThreadedInterner;
use mago_names::ResolvedNames;

use crate::Reference;
use crate::query::Query;

#[derive(Debug, Clone)]
pub struct Context<'a> {
    pub interner: &'a ThreadedInterner,
    pub query: &'a Query,
    pub resolved_names: &'a ResolvedNames,
    pub references: Vec<Reference>,
}

impl<'a> Context<'a> {
    pub fn new(interner: &'a ThreadedInterner, query: &'a Query, resolved_names: &'a ResolvedNames) -> Self {
        Self { interner, query, resolved_names, references: Vec::new() }
    }

    pub fn take_references(&mut self) -> Vec<Reference> {
        std::mem::take(&mut self.references)
    }
}
