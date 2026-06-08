use mago_names::ResolvedNames;
use mago_word::Word;

use mago_codex::metadata::CodebaseMetadata;

#[derive(Debug)]
pub struct AssertionContext<'ctx, 'arena, A> {
    pub resolved_names: &'ctx ResolvedNames<'arena>,
    pub arena: &'arena A,
    pub codebase: &'ctx CodebaseMetadata,
    pub this_class_name: Option<Word>,
    pub trust_existence_checks: bool,
}

impl<A> Clone for AssertionContext<'_, '_, A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A> Copy for AssertionContext<'_, '_, A> {}
