use bumpalo::Bump;

use mago_atom::Atom;
use mago_names::ResolvedNames;

use mago_codex::metadata::CodebaseMetadata;

#[derive(Clone, Copy, Debug)]
pub struct AssertionContext<'ctx, 'arena> {
    pub resolved_names: &'ctx ResolvedNames<'arena>,
    pub arena: &'arena Bump,
    pub codebase: &'ctx CodebaseMetadata,
    pub this_class_name: Option<Atom>,
}
