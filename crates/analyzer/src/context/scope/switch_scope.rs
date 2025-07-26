use std::collections::BTreeMap;
use std::rc::Rc;

use ahash::HashMap;

use mago_algebra::clause::Clause;
use mago_codex::ttype::union::TUnion;
use mago_span::Span;
use mago_syntax::ast::NodeKind;

#[derive(Debug, Clone, Default)]
pub struct SwitchScope {
    pub new_locals: Option<BTreeMap<String, Rc<TUnion>>>,
    pub redefined_vars: Option<HashMap<String, Rc<TUnion>>>,
    pub possibly_redefined_vars: Option<BTreeMap<String, TUnion>>,
    pub leftover_statements: Vec<(NodeKind, Span)>,
    pub leftover_case_equality_expr: Option<(NodeKind, Span)>,
    pub negated_clauses: Vec<Clause>,
    pub new_assigned_var_ids: HashMap<String, usize>,
}

impl SwitchScope {
    pub fn new() -> Self {
        Self::default()
    }
}
