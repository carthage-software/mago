//! Helpers for rendering symbol names in user-facing diagnostics.

use mago_atom::Atom;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;

use crate::context::Context;

/// Returns the case-preserved name of a class-like for user-facing diagnostics,
/// falling back to the input if no metadata is available.
#[inline]
pub(crate) fn display_class_like_name(context: &Context<'_, '_>, name: Atom) -> Atom {
    context.codebase.get_class_like(&name).map(|m| m.original_name).unwrap_or(name)
}

/// Returns the case-preserved method name on the given class-like for
/// user-facing diagnostics. Falls back to the input if metadata is missing.
#[inline]
pub(crate) fn display_method_name(context: &Context<'_, '_>, class_name: Atom, method_name: Atom) -> Atom {
    context.codebase.get_method(&class_name, &method_name).and_then(|m| m.original_name).unwrap_or(method_name)
}

/// Returns the case-preserved name of a global function for user-facing
/// diagnostics. Falls back to the input if metadata is missing.
#[inline]
pub(crate) fn display_function_name(context: &Context<'_, '_>, name: Atom) -> Atom {
    context.codebase.get_function(&name).and_then(|m| m.original_name).unwrap_or(name)
}

/// Produces a user-facing display string for a `FunctionLikeIdentifier`.
#[must_use]
pub(crate) fn display_function_like_identifier(
    context: &Context<'_, '_>,
    identifier: &FunctionLikeIdentifier,
) -> String {
    match identifier {
        FunctionLikeIdentifier::Function(name) => display_function_name(context, *name).to_string(),
        FunctionLikeIdentifier::Method(class_name, method_name) => {
            let class_display = display_class_like_name(context, *class_name);
            let method_display = display_method_name(context, *class_name, *method_name);
            format!("{class_display}::{method_display}")
        }
        FunctionLikeIdentifier::Closure(_, _) => identifier.as_string(),
    }
}
