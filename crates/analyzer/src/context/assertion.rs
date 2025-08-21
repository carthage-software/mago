use bumpalo::Bump;

use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use mago_names::ResolvedNames;

use mago_codex::metadata::CodebaseMetadata;

#[derive(Clone, Copy, Debug)]
pub struct AssertionContext<'ctx, 'arena> {
    pub resolved_names: &'ctx ResolvedNames<'arena>,
    pub arena: &'arena Bump,
    pub interner: &'ctx ThreadedInterner,
    pub codebase: &'ctx CodebaseMetadata,
    pub this_class_name: Option<&'ctx StringIdentifier>,
}
