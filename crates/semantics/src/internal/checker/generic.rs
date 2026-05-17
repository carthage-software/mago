use mago_php_version::feature::Feature;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::GenericArgumentList;
use mago_syntax::ast::GenericParameter;
use mago_syntax::ast::GenericParameterList;
use mago_syntax::ast::GenericVariance;
use mago_syntax::ast::Hint;
use mago_syntax::ast::Turbofish;

use crate::internal::context::Context;

/// Maximum entries in any generic parameter or argument list.
const MAX_GENERIC_LIST_LEN: usize = 127;

#[inline]
pub fn check_generic_parameter_list(list: &GenericParameterList, context: &mut Context<'_, '_, '_>) {
    if !context.version.is_supported(Feature::BoundErasedGenericTypes) {
        context.report(
            Issue::error("Generic type parameters are only available in PHP 8.6 and above.")
                .with_annotation(Annotation::primary(list.span()).with_message("Generic parameter list used here."))
                .with_note("PHP recognises generic type parameters on classes, interfaces, traits, functions, methods, closures, and arrow functions starting in PHP 8.6.")
                .with_help("Upgrade to PHP 8.6 or above to use generic type parameters."),
        );
        return;
    }

    let nodes = list.parameters.as_slice();
    if nodes.len() > MAX_GENERIC_LIST_LEN {
        context.report(
            Issue::error(format!(
                "Generic parameter list has {} entries; the maximum is {MAX_GENERIC_LIST_LEN}.",
                nodes.len()
            ))
            .with_annotation(Annotation::primary(list.span()).with_message("Too many generic parameters."))
            .with_note("PHP caps every generic parameter list at 127 entries.")
            .with_help("Reduce the number of type parameters on this declaration to 127 or fewer."),
        );
    }

    let mut seen_default = false;
    for parameter in nodes {
        if parameter.default.is_some() {
            seen_default = true;
        } else if seen_default {
            context.report(
                Issue::error(format!(
                    "Required type parameter `{}` cannot follow an optional one.",
                    parameter.name.value
                ))
                .with_annotation(Annotation::primary(parameter.span()).with_message("Required parameter here."))
                .with_note("Type-parameter defaults follow the same rule as value-parameter defaults: once a default is given, every later entry must also have a default.")
                .with_help("Move parameters with defaults to the end of the list, or give this parameter a default."),
            );
        }

        check_top_level_self_reference(parameter, context);
    }
}

fn check_top_level_self_reference(parameter: &GenericParameter, context: &mut Context<'_, '_, '_>) {
    let name = parameter.name.value;

    if let Some(bound) = &parameter.bound
        && hint_is_bare_local(&bound.hint, name)
    {
        context.report(
            Issue::error(format!("Type parameter `{name}` cannot reference itself at the top level of its bound."))
                .with_annotation(Annotation::primary(bound.hint.span()).with_message("Top-level self-reference here."))
                .with_note("A bare recursive reference would have no fixed point and would loop the resolver forever.")
                .with_help("Nest the recursive reference inside another type's arguments, e.g. `T : Comparable<T>`."),
        );
    }

    if let Some(default) = &parameter.default
        && hint_is_bare_local(&default.hint, name)
    {
        context.report(
            Issue::error(format!("Type parameter `{name}` cannot reference itself at the top level of its default."))
                .with_annotation(Annotation::primary(default.hint.span()).with_message("Top-level self-reference here."))
                .with_note("Defaults must resolve in a single pass at instantiation time; a bare self-reference cannot terminate.")
                .with_help("Nest the recursive reference inside another type's arguments, or remove the default."),
        );
    }

    if let Some(variance) = &parameter.variance {
        let (label, span) = match variance {
            GenericVariance::Covariant(span) => ("Covariant", *span),
            GenericVariance::Contravariant(span) => ("Contravariant", *span),
        };

        if let Some(bound) = &parameter.bound
            && hint_mentions_local(&bound.hint, parameter.name.value)
        {
            context.report(
                Issue::error(format!("{label} type parameter `{}` cannot appear in its own bound.", parameter.name.value))
                    .with_annotation(Annotation::primary(span).with_message("Variance marker here."))
                    .with_annotation(
                        Annotation::secondary(bound.hint.span()).with_message("Self-reference inside its own bound."),
                    )
                    .with_note("Bounds are invariant positions: a non-invariant type parameter cannot appear inside its own bound expression.")
                    .with_help("Drop the variance marker, or remove the self-reference from the bound."),
            );
        }

        if let Some(default) = &parameter.default
            && hint_mentions_local(&default.hint, parameter.name.value)
        {
            context.report(
                Issue::error(format!(
                    "{label} type parameter `{}` cannot appear in its own default.",
                    parameter.name.value
                ))
                .with_annotation(Annotation::primary(span).with_message("Variance marker here."))
                .with_annotation(
                    Annotation::secondary(default.hint.span()).with_message("Self-reference inside its own default."),
                )
                .with_note("Defaults are invariant positions: a non-invariant type parameter cannot appear inside its own default expression.")
                .with_help("Drop the variance marker, or remove the self-reference from the default."),
            );
        }
    }
}

fn hint_is_bare_local(hint: &Hint, name: &str) -> bool {
    match hint {
        Hint::Identifier(identifier) => identifier.value().eq_ignore_ascii_case(name),
        _ => false,
    }
}

fn hint_mentions_local(hint: &Hint, name: &str) -> bool {
    match hint {
        Hint::Identifier(id) => id.value().eq_ignore_ascii_case(name),
        Hint::Parenthesized(p) => hint_mentions_local(p.hint, name),
        Hint::Nullable(n) => hint_mentions_local(n.hint, name),
        Hint::Union(u) => hint_mentions_local(u.left, name) || hint_mentions_local(u.right, name),
        Hint::Intersection(i) => hint_mentions_local(i.left, name) || hint_mentions_local(i.right, name),
        Hint::Generic(g) => {
            hint_mentions_local(g.base, name)
                || g.arguments.arguments.iter().any(|argument| hint_mentions_local(argument, name))
        }
        _ => false,
    }
}

#[inline]
pub fn check_generic_argument_list(list: &GenericArgumentList, context: &mut Context<'_, '_, '_>) {
    if !context.version.is_supported(Feature::BoundErasedGenericTypes) {
        context.report(
            Issue::error("Generic type arguments are only available in PHP 8.6 and above.")
                .with_annotation(Annotation::primary(list.span()).with_message("Generic argument list used here."))
                .with_note("PHP recognises type arguments on named types (e.g. `Box<int>`, `Map<string, User>`) starting in PHP 8.6.")
                .with_help("Upgrade to PHP 8.6 or above to use generic type arguments."),
        );
        return;
    }

    if list.arguments.as_slice().len() > MAX_GENERIC_LIST_LEN {
        context.report(
            Issue::error(format!(
                "Generic argument list has {} entries; the maximum is {MAX_GENERIC_LIST_LEN}.",
                list.arguments.as_slice().len()
            ))
            .with_annotation(Annotation::primary(list.span()).with_message("Too many type arguments."))
            .with_note("PHP caps every generic argument list at 127 entries.")
            .with_help("Reduce the number of type arguments on this use site to 127 or fewer."),
        );
    }
}

#[inline]
pub fn check_turbofish(turbofish: &Turbofish, context: &mut Context<'_, '_, '_>) {
    if !context.version.is_supported(Feature::BoundErasedGenericTypes) {
        context.report(
            Issue::error("Turbofish (`::<...>`) is only available in PHP 8.6 and above.")
                .with_annotation(Annotation::primary(turbofish.span()).with_message("Turbofish used here."))
                .with_note("PHP recognises the turbofish call-site type-argument syntax starting in PHP 8.6.")
                .with_help("Upgrade to PHP 8.6 or above to use turbofish call-site type arguments."),
        );
        return;
    }

    if turbofish.arguments.as_slice().len() > MAX_GENERIC_LIST_LEN {
        context.report(
            Issue::error(format!(
                "Turbofish argument list has {} entries; the maximum is {MAX_GENERIC_LIST_LEN}.",
                turbofish.arguments.as_slice().len()
            ))
            .with_annotation(Annotation::primary(turbofish.span()).with_message("Too many type arguments."))
            .with_note("PHP caps every generic argument list at 127 entries.")
            .with_help("Reduce the number of type arguments at this call site to 127 or fewer."),
        );
    }
}

#[inline]
pub fn check_generic_hint_base(hint: &Hint, args_span: mago_span::Span, context: &mut Context<'_, '_, '_>) {
    match hint {
        Hint::Array(_) => {
            context.report(
                Issue::error("`array<...>` is not supported; PHP does not accept type arguments on `array`.")
                    .with_annotation(Annotation::primary(args_span).with_message("Type arguments are not allowed here."))
                    .with_note("PHP's `array` is both a hash map and a vector at runtime, with no parametric shape that maps cleanly onto `<K, V>`, so the engine does not recognise type arguments on it.")
                    .with_help("Use a parametric collection class instead, or describe the array shape with a docblock."),
            );
        }
        Hint::Iterable(_) => {
            context.report(
                Issue::error("`iterable<...>` is not supported; PHP does not accept type arguments on `iterable`.")
                    .with_annotation(Annotation::primary(args_span).with_message("Type arguments are not allowed here."))
                    .with_note("`iterable` is the union `array | Traversable`, and the two branches have different key-type constraints, so the engine does not recognise type arguments on it.")
                    .with_help("Use a more specific traversable type instead, or describe the iterable shape with a docblock."),
            );
        }
        _ => {}
    }
}
