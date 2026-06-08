use mago_allocator::Arena;
use mago_syntax::ast::ClassLikeMemberSelector;
use mago_syntax::ast::Expression;
use mago_syntax::ast::FunctionCall;
use mago_syntax::ast::MethodCall;

use crate::context::LintContext;

/// Checks if a `FunctionCall` could possibly refer to one of the names in a given slice.
///
/// This is the primary utility for checking a function call against a list of known
/// function names (e.g., all `assert*` methods in `PHPUnit`). It correctly handles
/// namespace resolution by checking against the function's fully qualified name
/// (if imported via `use function`), its namespaced name, and its global fallback.
///
/// Accepts both `&[&str]` (for compile-time function lists) and `&[String]` (for
/// user-configured lists), via the `AsRef<str>` bound on the slice's element type.
///
/// # Returns
///
/// Returns `Some(name)` with the **first matching name from the input slice** if a
/// potential resolution matches, or `None` if no match is found.
pub fn function_call_matches_any<'arena, 'names, S, A>(
    context: &LintContext<'_, 'arena, A>,
    call: &FunctionCall<'arena>,
    names: &'names [S],
) -> Option<&'names str>
where
    S: AsRef<str>,
    A: Arena,
{
    function_name_matches_any(context, call.function, names)
}

/// Checks if a `FunctionCall` could possibly refer to a specific function name.
///
/// This is a convenience wrapper around `function_call_matches_any` for checking
/// against a single function name.
#[inline]
pub fn function_call_matches<'arena, A>(
    context: &LintContext<'_, 'arena, A>,
    call: &FunctionCall<'arena>,
    name: &str,
) -> bool
where
    A: Arena,
{
    function_call_matches_any(context, call, std::slice::from_ref(&name)).is_some()
}

/// The internal implementation that checks if a function name `Expression`
/// could resolve to one of the provided names.
fn function_name_matches_any<'arena, 'names, S, A>(
    context: &LintContext<'_, 'arena, A>,
    function: &Expression<'arena>,
    names: &'names [S],
) -> Option<&'names str>
where
    S: AsRef<str>,
    A: Arena,
{
    let Expression::Identifier(function_identifier) = function else {
        return None;
    };

    let find = |candidate: &[u8]| -> Option<&'names str> {
        names.iter().find(|n| candidate.eq_ignore_ascii_case(n.as_ref().as_bytes())).map(AsRef::as_ref)
    };

    // Case 1: The name is explicitly imported with `use function`.
    // We check against its fully qualified name.
    if context.is_name_imported(function_identifier) {
        return find(context.lookup_name(function_identifier));
    }

    // Case 2: Unqualified name. This matches calls in the global namespace
    // or provides a match for the global fallback.
    if let Some(matched) = find(function_identifier.value()) {
        return Some(matched);
    }

    // Case 3: If we are in a namespace, check against the fully qualified
    // namespaced name (e.g., `App\foo`).
    if !context.scope.get_namespace().is_empty() {
        return find(context.lookup_name(function_identifier));
    }

    None
}

/// Gets the method name from a method call.
pub fn get_method_name<'arena>(method_call: &MethodCall<'arena>) -> Option<&'arena [u8]> {
    match &method_call.method {
        ClassLikeMemberSelector::Identifier(identifier) => Some(identifier.value),
        _ => None,
    }
}

/// Case-insensitive method name check (PHP methods are case-insensitive).
pub fn method_name_equals(method_call: &MethodCall<'_>, name: &str) -> bool {
    get_method_name(method_call).is_some_and(|n| n.eq_ignore_ascii_case(name.as_bytes()))
}

/// Case-insensitive check against multiple method names.
/// Returns the matched name from the list if found.
pub fn method_name_matches_any<'name>(method_call: &MethodCall<'_>, names: &[&'name str]) -> Option<&'name str> {
    let method_name = get_method_name(method_call)?;
    names.iter().find(|&n| method_name.eq_ignore_ascii_case(n.as_bytes())).copied()
}
