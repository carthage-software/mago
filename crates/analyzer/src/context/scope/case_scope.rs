use std::rc::Rc;

use mago_atom::AtomMap;
use mago_codex::ttype::union::TUnion;

#[derive(Clone, Debug)]
pub struct CaseScope {
    pub break_vars: Option<AtomMap<Rc<TUnion>>>,
}

impl CaseScope {
    pub fn new() -> Self {
        Self { break_vars: None }
    }
}

impl Default for CaseScope {
    fn default() -> Self {
        Self::new()
    }
}
