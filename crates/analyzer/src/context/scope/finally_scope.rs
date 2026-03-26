use std::rc::Rc;

use mago_atom::AtomMap;
use mago_codex::ttype::union::TUnion;

#[derive(Clone, Debug)]
pub struct FinallyScope {
    pub locals: AtomMap<Rc<TUnion>>,
}

impl FinallyScope {
    pub fn new() -> Self {
        Self { locals: AtomMap::default() }
    }
}

impl Default for FinallyScope {
    fn default() -> Self {
        Self::new()
    }
}
