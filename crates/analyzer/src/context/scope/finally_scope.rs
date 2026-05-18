use std::rc::Rc;

use mago_codex::ttype::union::TUnion;
use mago_word::WordMap;

#[derive(Clone, Debug)]
pub struct FinallyScope {
    pub locals: WordMap<Rc<TUnion>>,
}

impl FinallyScope {
    pub fn new() -> Self {
        Self { locals: WordMap::default() }
    }
}

impl Default for FinallyScope {
    fn default() -> Self {
        Self::new()
    }
}
