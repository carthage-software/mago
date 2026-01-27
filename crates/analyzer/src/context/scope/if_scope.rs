use std::rc::Rc;

use ahash::HashSet;
use indexmap::IndexMap;

use mago_algebra::assertion_set::AssertionSet;
use mago_algebra::clause::Clause;
use mago_atom::Atom;
use mago_atom::AtomMap;
use mago_atom::AtomSet;
use mago_codex::ttype::union::TUnion;

use crate::context::block::BlockContext;
use crate::context::scope::control_action::ControlActionSet;

#[derive(Clone, Debug, Default)]
pub struct IfScope<'ctx> {
    pub new_variables: Option<AtomMap<Rc<TUnion>>>,
    pub new_variables_possibly_in_scope: AtomSet,
    pub redefined_variables: Option<AtomMap<Rc<TUnion>>>,
    pub assigned_variable_ids: Option<AtomMap<u32>>,
    pub possibly_assigned_variable_ids: AtomSet,
    pub possibly_redefined_variables: AtomMap<Rc<TUnion>>,
    pub updated_variables: AtomSet,
    pub negated_types: IndexMap<Atom, AssertionSet>,
    pub conditionally_changed_variable_ids: AtomSet,
    pub negated_clauses: Vec<Clause>,
    pub reasonable_clauses: Vec<Rc<Clause>>,
    pub final_actions: ControlActionSet,
    pub if_actions: ControlActionSet,
    pub post_leaving_if_context: Option<BlockContext<'ctx>>,
    /// Properties definitely initialized in ALL branches (intersection).
    /// None = no branches processed yet. Some(set) = intersection across branches.
    pub definitely_initialized_properties: Option<AtomSet>,
    /// Methods definitely called in ALL branches (intersection).
    /// None = no branches processed yet. Some(set) = intersection across branches.
    pub definitely_called_methods: Option<HashSet<Atom>>,
}

impl IfScope<'_> {
    pub fn new() -> Self {
        Self {
            new_variables: None,
            new_variables_possibly_in_scope: AtomSet::default(),
            redefined_variables: None,
            assigned_variable_ids: None,
            possibly_assigned_variable_ids: AtomSet::default(),
            possibly_redefined_variables: AtomMap::default(),
            updated_variables: AtomSet::default(),
            negated_types: IndexMap::default(),
            conditionally_changed_variable_ids: AtomSet::default(),
            negated_clauses: Vec::default(),
            reasonable_clauses: Vec::default(),
            final_actions: ControlActionSet::new(),
            if_actions: ControlActionSet::new(),
            post_leaving_if_context: None,
            definitely_initialized_properties: None,
            definitely_called_methods: None,
        }
    }
}
