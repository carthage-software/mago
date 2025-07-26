use std::collections::BTreeMap;
use std::rc::Rc;

use mago_codex::ttype::union::TUnion;

#[derive(Clone, Debug)]
pub struct FinallyScope {
    pub locals: BTreeMap<String, Rc<TUnion>>,
}

impl FinallyScope {
    pub fn new() -> Self {
        Self { locals: BTreeMap::new() }
    }

    pub fn contains(&self, var_id: &str) -> bool {
        self.locals.contains_key(var_id)
    }
}

impl Default for FinallyScope {
    fn default() -> Self {
        Self::new()
    }
}
