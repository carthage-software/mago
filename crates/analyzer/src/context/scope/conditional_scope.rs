use ahash::HashMap;
use ahash::HashSet;

use mago_algebra::clause::Clause;

use crate::context::block::BlockContext;

#[derive(Debug, Clone)]
pub struct IfConditionalScope<'a> {
    pub if_body_context: BlockContext<'a>,
    pub post_if_context: BlockContext<'a>,
    pub conditionally_referenced_variable_ids: HashSet<String>,
    pub assigned_in_conditional_variable_ids: HashMap<String, usize>,
    pub entry_clauses: Vec<Clause>,
}
