use std::rc::Rc;

use foldhash::HashSet;
use indexmap::IndexMap;

use mago_algebra::assertion_set::AssertionSet;
use mago_algebra::clause::Clause;
use mago_codex::ttype::union::TUnion;
use mago_word::Word;
use mago_word::WordMap;
use mago_word::WordSet;

use crate::context::block::BlockContext;
use crate::context::scope::control_action::ControlActionSet;

#[derive(Clone, Debug, Default)]
pub struct IfScope<'ctx> {
    pub new_variables: Option<WordMap<Rc<TUnion>>>,
    pub new_variables_possibly_in_scope: WordSet,
    pub redefined_variables: Option<WordMap<Rc<TUnion>>>,
    pub assigned_variable_ids: Option<WordMap<u32>>,
    pub possibly_assigned_variable_ids: WordSet,
    pub possibly_redefined_variables: WordMap<Rc<TUnion>>,
    pub updated_variables: WordSet,
    pub negated_types: IndexMap<Word, AssertionSet>,
    pub conditionally_changed_variable_ids: WordSet,
    pub negated_clauses: Vec<Clause>,
    pub reasonable_clauses: Vec<Rc<Clause>>,
    pub final_actions: ControlActionSet,
    pub if_actions: ControlActionSet,
    pub post_leaving_if_context: Option<BlockContext<'ctx>>,
    /// Properties definitely initialized in ALL branches (intersection).
    /// None = no branches processed yet. Some(set) = intersection across branches.
    pub definitely_initialized_properties: Option<WordSet>,
    /// Property access paths definitely known to be uninitialized in ALL continuing branches.
    pub definitely_uninitialized_property_ids: Option<WordSet>,
    /// Methods definitely called in ALL branches (intersection).
    /// None = no branches processed yet. Some(set) = intersection across branches.
    pub definitely_called_methods: Option<HashSet<Word>>,
}

impl IfScope<'_> {
    pub fn new() -> Self {
        Self {
            new_variables: None,
            new_variables_possibly_in_scope: WordSet::default(),
            redefined_variables: None,
            assigned_variable_ids: None,
            possibly_assigned_variable_ids: WordSet::default(),
            possibly_redefined_variables: WordMap::default(),
            updated_variables: WordSet::default(),
            negated_types: IndexMap::default(),
            conditionally_changed_variable_ids: WordSet::default(),
            negated_clauses: Vec::default(),
            reasonable_clauses: Vec::default(),
            final_actions: ControlActionSet::new(),
            if_actions: ControlActionSet::new(),
            post_leaving_if_context: None,
            definitely_initialized_properties: None,
            definitely_uninitialized_property_ids: None,
            definitely_called_methods: None,
        }
    }
}
