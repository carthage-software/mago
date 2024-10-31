use fennec_interner::ThreadedInterner;
use fennec_reflection::CodebaseReflection;
use fennec_semantics::Semantics;
use fennec_walker::*;

use crate::internal::context::Context;
use crate::internal::walker::ReflectionWalker;

mod internal;
mod populator;

/// Construct a codebase reflection from the given semantics.
///
/// # Parameters
///
/// - `interner`: The interner to use for string interning.
/// - `semantics`: The semantics of the program.
///
/// # Returns
///
/// A reflection result containing the reflection of the codebase and any issues found during reflection.
#[inline]
pub fn reflect<'i, 'ast>(interner: &'i ThreadedInterner, semantics: &'ast Semantics) -> CodebaseReflection {
    let mut walker = ReflectionWalker::new();

    let mut context = Context::new(&interner, &semantics);

    walker.walk_program(&semantics.program, &mut context);

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
pub fn merge(mut reflection: CodebaseReflection, other_reflection: CodebaseReflection) -> CodebaseReflection {
    for (_, function_like) in other_reflection.function_like_reflections.into_iter() {
        reflection.register_function_like(function_like);
    }

    for (_, class_like) in other_reflection.class_like_reflections.into_iter() {
        reflection.register_class_like(class_like);
    }

    for (_, constant) in other_reflection.constant_reflections.into_iter() {
        reflection.register_constant(constant);
    }

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
pub fn populate<'i, 'r>(interner: &'i ThreadedInterner, reflection: &'r mut CodebaseReflection) {
    populator::populate(interner, reflection);
}
