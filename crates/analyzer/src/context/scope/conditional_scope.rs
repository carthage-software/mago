use ahash::HashMap;
use ahash::HashSet;

use mago_algebra::clause::Clause;

use crate::context::block::BlockContext;

#[derive(Debug, Clone)]
pub struct IfConditionalScope<'ctx> {
    pub if_body_context: BlockContext<'ctx>,
    pub post_if_context: BlockContext<'ctx>,
    pub conditionally_referenced_variable_ids: HashSet<String>,
    pub assigned_in_conditional_variable_ids: HashMap<String, u32>,
    pub entry_clauses: Vec<Clause>,
}
