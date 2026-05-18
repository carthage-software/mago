use bumpalo::Bump;

use mago_names::ResolvedNames;
use mago_word::Word;

use mago_codex::metadata::CodebaseMetadata;

#[derive(Clone, Copy, Debug)]
pub struct AssertionContext<'ctx, 'arena> {
    pub resolved_names: &'ctx ResolvedNames<'arena>,
    pub arena: &'arena Bump,
    pub codebase: &'ctx CodebaseMetadata,
    pub this_class_name: Option<Word>,
    pub trust_existence_checks: bool,
}
