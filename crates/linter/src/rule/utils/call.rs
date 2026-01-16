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
/// # Returns
///
/// Returns `Some(name)` with the **first matching name from the input slice** if a
/// potential resolution matches, or `None` if no match is found.
pub fn function_call_matches_any<'arena, 'name>(
    context: &LintContext<'_, 'arena>,
    call: &FunctionCall<'arena>,
    names: &[&'name str],
) -> Option<&'name str> {
    function_name_matches_any(context, call.function, names)
}

/// Checks if a `FunctionCall` could possibly refer to a specific function name.
///
/// This is a convenience wrapper around `function_call_matches_any` for checking
/// against a single function name.
#[inline]
pub fn function_call_matches<'arena>(
    context: &LintContext<'_, 'arena>,
    call: &FunctionCall<'arena>,
    name: &str,
) -> bool {
    function_call_matches_any(context, call, std::slice::from_ref(&name)).is_some()
}

/// The internal implementation that checks if a function name `Expression`
/// could resolve to one of the provided names.
fn function_name_matches_any<'arena, 'name>(
    context: &LintContext<'_, 'arena>,
    function: &Expression<'arena>,
    names: &[&'name str],
) -> Option<&'name str> {
    let Expression::Identifier(function_identifier) = function else {
        return None;
    };

    // Case 1: The name is explicitly imported with `use function`.
    // We check against its fully qualified name.
    if context.is_name_imported(function_identifier) {
        let fqn = context.lookup_name(function_identifier);

        return names.iter().find(|&name| fqn.eq_ignore_ascii_case(name)).copied();
    }

    // Case 2: Unqualified name. This matches calls in the global namespace
    // or provides a match for the global fallback.
    let unqualified_name = function_identifier.value();
    if let Some(matched) = names.iter().find(|&name| unqualified_name.eq_ignore_ascii_case(name)) {
        return Some(matched);
    }

    // Case 3: If we are in a namespace, check against the fully qualified
    // namespaced name (e.g., `App\foo`).
    if !context.scope.get_namespace().is_empty() {
        let fqn_in_namespace = context.lookup_name(function_identifier);

        return names.iter().find(|&name| fqn_in_namespace.eq_ignore_ascii_case(name)).copied();
    }

    None
}

/// Gets the method name from a method call.
pub fn get_method_name<'a>(method_call: &'a MethodCall<'a>) -> Option<&'a str> {
    match &method_call.method {
        ClassLikeMemberSelector::Identifier(identifier) => Some(identifier.value),
        _ => None,
    }
}

/// Case-insensitive method name check (PHP methods are case-insensitive).
pub fn method_name_equals(method_call: &MethodCall<'_>, name: &str) -> bool {
    get_method_name(method_call).is_some_and(|n| n.eq_ignore_ascii_case(name))
}

/// Case-insensitive check against multiple method names.
/// Returns the matched name from the list if found.
pub fn method_name_matches_any<'a>(method_call: &MethodCall<'_>, names: &[&'a str]) -> Option<&'a str> {
    let method_name = get_method_name(method_call)?;
    names.iter().find(|&n| method_name.eq_ignore_ascii_case(n)).copied()
}
