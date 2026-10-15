use mago_codex::metadata::CodebaseMetadata;
use mago_names::ResolvedNames;
use mago_syntax::cst::Call;
use mago_syntax::cst::Expression;
use mago_syntax::cst::FunctionCall;
use mago_syntax::cst::Identifier;

use crate::utils::misc::unwrap_expression;

/// Identifies supported built-in functions used for narrowing expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinNarrowingFunction {
    GetClass,
    GetType,
}

/// Resolves a function identifier to its fully qualified or unqualified name,
/// following PHP's resolution rules:
/// 1. If imported via a `use function` statement, use the imported name.
/// 2. If fully qualified (starts with `\`), use the name without the leading `\`.
/// 3. If a namespaced function exists in the codebase under the current namespace, use that.
/// 4. If an unqualified global function exists in the codebase, fallback to that.
/// 5. Otherwise, default to the namespaced name.
pub fn resolve_function_name<'arena>(
    identifier: &Identifier<'arena>,
    resolved_names: &ResolvedNames<'arena>,
    codebase: &CodebaseMetadata,
) -> &'arena [u8] {
    if resolved_names.is_imported(identifier) {
        return resolved_names.get(identifier);
    }

    let name = identifier.value();

    if let Some(stripped) = name.strip_prefix(b"\\") {
        return stripped;
    }

    let fqfn = resolved_names.get(identifier);
    if codebase.function_exists(fqfn) {
        return fqfn;
    }

    if !name.contains(&b'\\') && codebase.function_exists(name) {
        return name;
    }

    fqfn
}

/// Checks whether a function call refers to a genuine built-in function with the given name.
pub fn is_builtin_function_call<'arena>(
    call: &FunctionCall<'arena>,
    builtin_name: &[u8],
    resolved_names: &ResolvedNames<'arena>,
    codebase: &CodebaseMetadata,
) -> bool {
    let Expression::Identifier(identifier) = call.function else {
        return false;
    };

    let resolved = resolve_function_name(identifier, resolved_names, codebase);
    if !resolved.eq_ignore_ascii_case(builtin_name) {
        return false;
    }

    codebase.get_function(resolved).is_some_and(|f| f.flags.is_built_in())
}

/// Inspects an expression to determine if it is a single-argument call to the built-in
/// `get_class` or `gettype` function.
///
/// Returns the narrowing function kind and the single argument expression if so.
pub fn get_builtin_get_class_or_gettype_call<'arena>(
    expression: &'arena Expression<'arena>,
    resolved_names: &ResolvedNames<'arena>,
    codebase: &CodebaseMetadata,
) -> Option<(BuiltinNarrowingFunction, &'arena Expression<'arena>)> {
    let Expression::Call(Call::Function(function_call)) = unwrap_expression(expression) else {
        return None;
    };

    if function_call.argument_list.arguments.len() != 1 {
        return None;
    }

    let Expression::Identifier(identifier) = function_call.function else {
        return None;
    };

    let resolved = resolve_function_name(identifier, resolved_names, codebase);
    let kind = if resolved.eq_ignore_ascii_case(b"get_class") {
        BuiltinNarrowingFunction::GetClass
    } else if resolved.eq_ignore_ascii_case(b"gettype") {
        BuiltinNarrowingFunction::GetType
    } else {
        return None;
    };

    if !codebase.get_function(resolved).is_some_and(|f| f.flags.is_built_in()) {
        return None;
    }

    let argument = function_call.argument_list.arguments.first()?.value();
    Some((kind, argument))
}
