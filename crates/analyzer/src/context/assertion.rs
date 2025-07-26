use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use mago_names::ResolvedNames;
use mago_source::Source;

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::reference::ReferenceSource;
use mago_codex::ttype::resolution::TypeResolutionContext;

use crate::settings::Settings;

#[derive(Clone, Copy, Debug)]
pub struct AssertionContext<'a> {
    pub file_source: &'a Source,
    pub resolved_names: &'a ResolvedNames,
    pub interner: &'a ThreadedInterner,
    pub codebase: &'a CodebaseMetadata,
    pub this_class_name: Option<&'a StringIdentifier>,
    pub type_resolution_context: &'a TypeResolutionContext,
    pub settings: &'a Settings,
    pub reference_source: Option<ReferenceSource>,
    pub in_loop: bool,
}
