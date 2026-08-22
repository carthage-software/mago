use std::rc::Rc;

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::ttype::combine_union_types;
use mago_codex::ttype::combiner::CombinerOptions;
use mago_codex::ttype::union::TUnion;
use mago_word::WordMap;

#[derive(Clone, Debug)]
pub struct CaseScope {
    pub break_vars: Option<WordMap<Rc<TUnion>>>,
}

impl CaseScope {
    pub fn new() -> Self {
        Self { break_vars: None }
    }

    pub fn record_break(&mut self, locals: &WordMap<Rc<TUnion>>, codebase: &CodebaseMetadata) {
        let mut break_vars = self.break_vars.take().unwrap_or_default();

        for (var_id, var_type) in locals {
            let resulting_type = match break_vars.get(var_id) {
                Some(break_var_type) => {
                    Rc::new(combine_union_types(var_type, break_var_type, codebase, CombinerOptions::default()))
                }
                None => Rc::clone(var_type),
            };

            break_vars.insert(*var_id, resulting_type);
        }

        self.break_vars = Some(break_vars);
    }
}

impl Default for CaseScope {
    fn default() -> Self {
        Self::new()
    }
}
