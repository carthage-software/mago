use std::collections::BTreeMap;
use std::rc::Rc;

use ahash::HashMap;
use ahash::HashSet;

use indexmap::IndexMap;
use mago_algebra::assertion_set::AssertionSet;
use mago_algebra::clause::Clause;
use mago_codex::ttype::union::TUnion;

use crate::context::block::BlockContext;
use crate::context::scope::control_action::ControlAction;

#[derive(Clone, Debug, Default)]
pub struct IfScope<'ctx> {
    pub new_variables: Option<BTreeMap<String, TUnion>>,
    pub new_variables_possibly_in_scope: HashSet<String>,
    pub redefined_variables: Option<HashMap<String, TUnion>>,
    pub assigned_variable_ids: Option<HashMap<String, u32>>,
    pub possibly_assigned_variable_ids: HashSet<String>,
    pub possibly_redefined_variables: HashMap<String, TUnion>,
    pub updated_variables: HashSet<String>,
    pub negated_types: IndexMap<String, AssertionSet>,
    pub conditionally_changed_variable_ids: HashSet<String>,
    pub negated_clauses: Vec<Clause>,
    pub reasonable_clauses: Vec<Rc<Clause>>,
    pub final_actions: HashSet<ControlAction>,
    pub if_actions: HashSet<ControlAction>,
    pub post_leaving_if_context: Option<BlockContext<'ctx>>,
}

impl<'ctx> IfScope<'ctx> {
    pub fn new() -> Self {
        Self {
            new_variables: None,
            new_variables_possibly_in_scope: HashSet::default(),
            redefined_variables: None,
            assigned_variable_ids: None,
            possibly_assigned_variable_ids: HashSet::default(),
            possibly_redefined_variables: HashMap::default(),
            updated_variables: HashSet::default(),
            negated_types: IndexMap::default(),
            conditionally_changed_variable_ids: HashSet::default(),
            negated_clauses: Vec::default(),
            reasonable_clauses: Vec::default(),
            final_actions: HashSet::default(),
            if_actions: HashSet::default(),
            post_leaving_if_context: None,
        }
    }
}
