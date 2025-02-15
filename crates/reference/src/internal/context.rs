use mago_interner::ThreadedInterner;
use mago_project::module::Module;

use crate::query::Query;
use crate::Reference;

#[derive(Debug, Clone)]
pub struct Context<'a> {
    pub interner: &'a ThreadedInterner,
    pub query: &'a Query,
    pub module: &'a Module,
    pub references: Vec<Reference>,
}

impl<'a> Context<'a> {
    pub fn new(interner: &'a ThreadedInterner, query: &'a Query, module: &'a Module) -> Self {
        Self { interner, query, module, references: Vec::new() }
    }

    pub fn take_references(&mut self) -> Vec<Reference> {
        std::mem::take(&mut self.references)
    }
}
