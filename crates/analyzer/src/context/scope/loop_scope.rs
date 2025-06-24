use std::collections::BTreeMap;
use std::rc::Rc;

use ahash::HashMap;
use ahash::HashSet;

use mago_codex::ttype::union::TUnion;
use mago_span::Span;

use crate::context::scope::control_action::ControlAction;

#[derive(Clone, Debug)]
pub struct LoopScope {
    pub span: Span,
    pub iteration_count: usize,
    pub parent_context_variables: BTreeMap<String, Rc<TUnion>>,
    pub redefined_loop_variables: HashMap<String, TUnion>,
    pub possibly_redefined_loop_variables: HashMap<String, TUnion>,
    pub possibly_redefined_loop_parent_variables: HashMap<String, Rc<TUnion>>,
    pub possibly_defined_loop_parent_variables: HashMap<String, TUnion>,
    pub variables_possibly_in_scope: HashSet<String>,
    pub final_actions: HashSet<ControlAction>,
    pub parent_loop: Option<Box<LoopScope>>,
    pub truthy_pre_conditions: bool,
}

impl LoopScope {
    pub fn new(
        span: Span,
        parent_context_vars: BTreeMap<String, Rc<TUnion>>,
        parent_loop: Option<Box<LoopScope>>,
    ) -> Self {
        Self {
            span,
            parent_context_variables: parent_context_vars,
            iteration_count: 0,
            redefined_loop_variables: HashMap::default(),
            possibly_redefined_loop_variables: HashMap::default(),
            possibly_redefined_loop_parent_variables: HashMap::default(),
            possibly_defined_loop_parent_variables: HashMap::default(),
            final_actions: HashSet::default(),
            variables_possibly_in_scope: HashSet::default(),
            parent_loop,
            truthy_pre_conditions: true,
        }
    }

    pub fn with_parent_loop(self, parent_loop: Option<Box<LoopScope>>) -> Self {
        Self { parent_loop, ..self }
    }
}
