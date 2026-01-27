use std::rc::Rc;

use mago_atom::AtomMap;
use mago_atom::AtomSet;

use mago_codex::ttype::union::TUnion;
use mago_span::Span;

use crate::context::scope::control_action::ControlActionSet;

#[derive(Clone, Debug)]
pub struct LoopScope {
    pub span: Span,
    pub iteration_count: usize,
    pub parent_context_variables: AtomMap<Rc<TUnion>>,
    pub redefined_loop_variables: AtomMap<Rc<TUnion>>,
    pub possibly_redefined_loop_variables: AtomMap<Rc<TUnion>>,
    pub possibly_redefined_loop_parent_variables: AtomMap<Rc<TUnion>>,
    pub possibly_defined_loop_parent_variables: AtomMap<Rc<TUnion>>,
    pub variables_possibly_in_scope: AtomSet,
    pub final_actions: ControlActionSet,
    pub truthy_pre_conditions: bool,
    pub parent_loop: Option<Box<LoopScope>>,
}

impl LoopScope {
    pub fn new(span: Span, parent_context_vars: AtomMap<Rc<TUnion>>, parent_loop: Option<Box<LoopScope>>) -> Self {
        Self {
            span,
            parent_context_variables: parent_context_vars,
            iteration_count: 0,
            redefined_loop_variables: AtomMap::default(),
            possibly_redefined_loop_variables: AtomMap::default(),
            possibly_redefined_loop_parent_variables: AtomMap::default(),
            possibly_defined_loop_parent_variables: AtomMap::default(),
            final_actions: ControlActionSet::new(),
            variables_possibly_in_scope: AtomSet::default(),
            parent_loop,
            truthy_pre_conditions: true,
        }
    }

    pub fn with_parent_loop(self, parent_loop: Option<Box<LoopScope>>) -> Self {
        Self { parent_loop, ..self }
    }
}
