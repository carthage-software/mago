use mago_interner::ThreadedInterner;
use mago_names::ResolvedNames;
use mago_reflection::CodebaseReflection;
use mago_reflection::r#type::TypeReflection;
use mago_reflection::r#type::kind::TypeKind;
use mago_source::Source;
use mago_span::HasSpan;
use mago_syntax::ast::Expression;

use crate::resolver::TypeResolver;

mod internal;

pub mod constant;
pub mod resolver;

/// Infers the type of a given expression by initializing a simple type reflection
/// that includes the inferred type kind and its source location.
///
/// # Arguments
///
/// - `interner`: Manages string interning.
/// - `source`: The source of the expression.
/// - `names`: The names of the program.
/// - `expression`: The expression to analyze.
///
/// # Returns
///
/// Returns a `TypeReflection` with the inferred type kind and span of the expression.
pub fn infere<'ast>(
    interner: &ThreadedInterner,
    source: &'ast Source,
    names: &'ast ResolvedNames,
    expression: &'ast Expression,
) -> TypeReflection {
    let kind = infere_kind(interner, source, names, expression);

    TypeReflection { kind, inferred: true, span: expression.span() }
}

/// Infers the general type kind of an expression without using a codebase for context.
///
/// # Arguments
///
/// - `interner`: Manages string interning.
/// - `source`: The source of the expression.
/// - `names`: The names of the program.
/// - `expression`: The expression to analyze.
///
/// # Returns
///
/// Returns a `TypeKind` that represents the initial inferred type of the expression.
pub fn infere_kind<'ast>(
    interner: &ThreadedInterner,
    source: &'ast Source,
    names: &'ast ResolvedNames,
    expression: &'ast Expression,
) -> TypeKind {
    let resolver = TypeResolver::new(interner, source, names, None);

    resolver.resolve(expression)
}

/// Resolves the type kind of an expression with additional codebase context for more
/// precise type information.
///
/// # Arguments
///
/// - `interner`: Manages string interning.
/// - `source`: The source of the expression.
/// - `names`: The names of the program.
/// - `codebase`: The codebase reflection to use for context.
/// - `expression`: The expression to analyze.
///
/// # Returns
///
/// Returns a `TypeKind` that represents the resolved type of the expression,
/// taking into account any known codebase types.
pub fn resolve_kind<'ast>(
    interner: &ThreadedInterner,
    source: &'ast Source,
    names: &'ast ResolvedNames,
    codebase: &'ast CodebaseReflection,
    expression: &'ast Expression,
) -> TypeKind {
    let resolver = TypeResolver::new(interner, source, names, Some(codebase));

    resolver.resolve(expression)
}
