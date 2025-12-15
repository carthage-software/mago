use mago_atom::AtomMap;
use mago_atom::AtomSet;

use mago_algebra::clause::Clause;

use crate::context::block::BlockContext;

#[derive(Debug, Clone)]
pub struct IfConditionalScope<'ctx> {
    pub if_body_context: BlockContext<'ctx>,
    pub post_if_context: BlockContext<'ctx>,
    pub conditionally_referenced_variable_ids: AtomSet,
    pub assigned_in_conditional_variable_ids: AtomMap<u32>,
    pub entry_clauses: Vec<Clause>,
}
