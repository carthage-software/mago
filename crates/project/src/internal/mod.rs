use mago_interner::ThreadedInterner;
use mago_names::ResolvedNames;
use mago_php_version::PHPVersion;
use mago_reflection::CodebaseReflection;
use mago_reporting::IssueCollection;
use mago_source::Source;
use mago_syntax::ast::Program;
use mago_syntax::walker::MutWalker;
use mago_syntax::walker::Walker;

use crate::internal::context::Context;
use crate::internal::walker::building::ModuleBuildingWalker;
use crate::internal::walker::checking::ModuleCheckingWalker;
use crate::internal::walker::reflection::ModuleReflectionWalker;
use crate::module::ModuleBuildOptions;

mod checker;
mod consts;
mod context;
mod reflector;
mod walker;

pub mod populator;

#[inline(always)]
pub fn build(
    interner: &ThreadedInterner,
    version: PHPVersion,
    source: &Source,
    program: &Program,
    names: &ResolvedNames,
    options: ModuleBuildOptions,
) -> (Option<CodebaseReflection>, IssueCollection) {
    let mut context = Context::new(interner, &version, program, names, source);

    let reflection = match options {
        ModuleBuildOptions { reflect: true, validate: true } => {
            let mut context = Context::new(interner, &version, program, names, source);
            let mut walker = ModuleBuildingWalker::new();
            walker.walk_program(program, &mut context);

            Some(walker.reflection)
        }
        ModuleBuildOptions { reflect: true, validate: false } => {
            let mut context = Context::new(interner, &version, program, names, source);
            let mut walker = ModuleReflectionWalker::new();
            walker.walk_program(program, &mut context);

            Some(walker.reflection)
        }
        ModuleBuildOptions { reflect: false, validate: true } => {
            ModuleCheckingWalker.walk_program(program, &mut context);

            None
        }
        _ => None,
    };

    let issues = context.take_issue_collection();

    (reflection, issues)
}
