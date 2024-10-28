use fennec_interner::ThreadedInterner;
use fennec_reflection::CodebaseReflection;
use fennec_semantics::Semantics;
use fennec_walker::*;

use crate::internal::context::Context;
use crate::internal::walker::ReflectionWalker;

pub mod internal;

/// Construct a codebase reflection from the given semantics.
///
/// # Parameters
///
/// - `interner`: The interner to use for string interning.
/// - `semantics`: The semantics of the program.
///
/// # Returns
///
/// A codebase reflection containing all the reflections from the given semantics.
pub fn reflect_semantics<'i, 'ast>(interner: &'i ThreadedInterner, semantics: &'ast Semantics) -> CodebaseReflection {
    let mut walker = ReflectionWalker::new();

    let mut context = Context::new(&interner, &semantics);

    walker.walk_program(&semantics.program, &mut context);

    walker.reflections
}
