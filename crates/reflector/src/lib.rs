use mago_ast::Program;
use mago_interner::ThreadedInterner;
use mago_names::Names;
use mago_reflection::CodebaseReflection;
use mago_source::Source;
use mago_walker::*;

use crate::internal::context::Context;
use crate::internal::walker::ReflectionWalker;

mod internal;
mod populator;

/// Construct a codebase reflection from the given program.
///
/// # Arguments
///
/// - `interner`: The `ThreadedInterner` instance used for string interning.
/// - `source`: The `Source` instance containing the source code of the program.
/// - `program`: The `Program` instance to reflect.
/// - `names`: The `Names` instance containing the names of the program.
///
/// # Returns
///
/// A reflection result containing the reflection of the codebase and any issues found during reflection.
#[inline]
pub fn reflect(interner: &ThreadedInterner, source: &Source, program: &Program, names: &Names) -> CodebaseReflection {
    let mut walker = ReflectionWalker::new();

    let mut context = Context::new(interner, source, names);

    walker.walk_program(program, &mut context);

    walker.reflection
}

/// Merges two `ReflectionResult` instances.
///
/// This method combines the codebase reflections and issues from two `ReflectionResult` instances
/// into a single `ReflectionResult`. If duplicates are found during merging (such as functions,
/// classes, or constants with identical names), they are recorded as issues within the resulting
/// `ReflectionResult`.
///
/// # Parameters
///
/// - `left`: The first `ReflectionResult` to merge.
/// - `right`: The second `ReflectionResult` to merge.
///
/// # Returns
///
/// A new `ReflectionResult` containing the combined reflections and issues from both inputs.
/// If any conflicts are found (e.g., duplicate functions, classes, or constants), they are recorded
/// as issues in the returned result.
#[inline]
pub fn merge(
    interner: &ThreadedInterner,
    mut reflection: CodebaseReflection,
    other_reflection: CodebaseReflection,
) -> CodebaseReflection {
    for (_, function_like) in other_reflection.function_like_reflections.into_iter() {
        reflection.register_function_like(interner, function_like);
    }

    for (_, class_like) in other_reflection.class_like_reflections.into_iter() {
        reflection.register_class_like(interner, class_like);
    }

    for (_, constant) in other_reflection.constant_reflections.into_iter() {
        reflection.register_constant(interner, constant);
    }

    reflection.populated = false;
    reflection
}

/// Populates additional data into an existing `ReflectionResult`.
///
/// This method updates an existing `ReflectionResult` by adding any additional details
/// to the `reflection` field based on further analysis. It may also add new issues
/// encountered during this process.
///
/// # Parameters
///
/// - `interner`: The `ThreadedInterner` instance used for string interning.
/// - `result`: The mutable `ReflectionResult` to populate with additional data.
///
/// This function is useful for supplementing a `ReflectionResult` with more comprehensive
/// information after initial reflection or to populate unresolved details.
#[inline]
pub fn populate(interner: &ThreadedInterner, reflection: &mut CodebaseReflection) {
    populator::populate(interner, reflection);
}
