use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeKind;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::metadata::property::PropertyMetadata;
use mago_php_version::PHPVersion;
use mago_php_version::feature::Feature;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::ClassLikeConstant;
use mago_syntax::ast::FunctionLikeParameter;
use mago_syntax::ast::FunctionLikeReturnTypeHint;
use mago_syntax::ast::Hint;
use mago_syntax::ast::Property;
use mago_syntax::ast::PropertyItem;

use crate::code::IssueCode;
use crate::context::Context;

/// Check if a constant is missing a type hint and whether it's safe to add one.
///
/// A constant should only be reported as missing a type hint if:
/// 1. It has no type hint
/// 2. Typed class constants are supported in the target PHP version
pub fn check_constant_type_hint<'arena>(
    context: &mut Context<'_, 'arena>,
    class_like_constant: &ClassLikeConstant<'arena>,
) {
    if !context.settings.check_missing_type_hints
        || !context.settings.version.is_supported(Feature::TypedClassLikeConstants)
    {
        return;
    }

    if class_like_constant.hint.is_some() {
        return;
    }

    let item = class_like_constant.first_item();

    let constant_name = item.name.value;

    context.collector.report_with_code(
        IssueCode::MissingConstantType,
        Issue::warning(format!("Class constant `{constant_name}` is missing a type hint."))
            .with_annotation(
                Annotation::primary(class_like_constant.span())
                    .with_message(format!("Class constant `{constant_name}` is defined here")),
            )
            .with_note("Adding a type hint to constants improves code readability and helps prevent type errors.")
            .with_help(format!("Consider specifying a type hint for `{constant_name}`.")),
    );
}

/// Check if a property is missing a type hint and whether it's safe to add one.
///
/// A property should only be reported as missing a type hint if:
/// 1. It has no type hint
/// 2. It is not prefixed with `$_` (ignored by convention)
/// 3. It would be safe to add a type hint (i.e., no parent class/trait has the same property without a type hint)
/// 4. Typed properties are supported in the target PHP version
pub fn check_property_type_hint<'arena>(
    context: &mut Context<'_, 'arena>,
    class_like_metadata: &ClassLikeMetadata,
    property: &Property<'arena>,
) {
    if !context.settings.check_missing_type_hints || !context.settings.version.is_supported(Feature::TypedProperties) {
        return;
    }

    let hint = match property {
        Property::Plain(plain) => plain.hint.as_ref(),
        Property::Hooked(hooked) => hooked.hint.as_ref(),
    };

    // If it already has a type hint, nothing to check
    if hint.is_some() {
        return;
    }

    let variables = match property {
        Property::Plain(plain) => plain
            .items
            .iter()
            .filter_map(
                |item| {
                    if let PropertyItem::Concrete(concrete) = item { Some(&concrete.variable) } else { None }
                },
            )
            .collect::<Vec<_>>(),
        Property::Hooked(hooked) => match &hooked.item {
            PropertyItem::Concrete(concrete) => vec![&concrete.variable],
            PropertyItem::Abstract(_) => vec![],
        },
    };

    for variable in variables {
        // Skip variables prefixed with `$_`
        if variable.name.starts_with("$_") {
            continue;
        }

        // Check if it's safe to add a type hint by verifying no parent class/trait has
        // the same property without a type hint
        if is_safe_to_add_property_type_hint(context, class_like_metadata, variable.name) {
            context.collector.report(
                Issue::warning(format!("Property `{}` is missing a type hint.", variable.name))
                    .with_code(IssueCode::MissingPropertyType.as_str())
                    .with_annotation(
                        Annotation::primary(property.span())
                            .with_message(format!("Property `{}` declared here without a type hint", variable.name)),
                    )
                    .with_note(
                        "Adding type hints to properties improves code readability and helps prevent type errors.",
                    )
                    .with_help(format!("Consider adding a type hint to property `{}`.", variable.name)),
            );
        }
    }
}

/// Check if a parameter is missing a type hint.
///
/// A parameter should only be reported as missing a type hint if:
/// 1. It has no type hint
/// 2. It is not prefixed with `$_` (ignored by convention)
/// 3. The method is not overriding a parent method (where adding a type hint might cause issues)
/// 4. If it's a closure/arrow function parameter, the corresponding ignore setting is not enabled
/// 5. Typed parameters are supported in the target PHP version
pub fn check_parameter_type_hint<'arena>(
    context: &mut Context<'_, 'arena>,
    class_like_metadata: Option<&ClassLikeMetadata>,
    function_like_metadata: &FunctionLikeMetadata,
    parameter: &FunctionLikeParameter<'arena>,
) {
    if !context.settings.check_missing_type_hints || context.settings.version < PHPVersion::PHP70 {
        return;
    }

    // If it already has a type hint, nothing to check
    if parameter.hint.is_some() || parameter.variable.name.starts_with("$_") {
        return;
    }

    // Check if we should skip based on function kind
    if matches!(function_like_metadata.kind, FunctionLikeKind::Closure)
        && !context.settings.check_closure_missing_type_hints
    {
        return;
    }

    if matches!(function_like_metadata.kind, FunctionLikeKind::ArrowFunction)
        && !context.settings.check_arrow_function_missing_type_hints
    {
        return;
    }

    // If this is a method, check if it's safe to add a type hint
    if let Some(class_metadata) = class_like_metadata
        && !is_safe_to_add_parameter_type_hint(context, class_metadata, function_like_metadata)
    {
        return;
    }

    context.collector.report(
        Issue::warning(format!("Parameter `{}` is missing a type hint.", parameter.variable.name))
            .with_code(IssueCode::MissingParameterType.as_str())
            .with_annotation(
                Annotation::primary(parameter.span())
                    .with_message(format!("Parameter `{}` declared here without a type hint", parameter.variable.name)),
            )
            .with_note("Type hints improve code readability and help prevent type-related errors.")
            .with_help(format!("Consider adding a type hint to parameter `{}`.", parameter.variable.name)),
    );
}

/// Check if a function or method is missing a return type hint.
///
/// A function/method should only be reported as missing a return type hint if:
/// 1. It has no return type hint
/// 2. It's not a constructor or destructor
/// 3. If it's a method, it's not overriding a parent method
/// 4. If it's a closure/arrow function, the corresponding ignore setting is not enabled
/// 5. Return type hints are supported in the target PHP version
pub fn check_return_type_hint<'arena>(
    context: &mut Context<'_, 'arena>,
    class_like_metadata: Option<&ClassLikeMetadata>,
    function_like_metadata: &FunctionLikeMetadata,
    function_name: &str,
    return_type_hint: Option<&FunctionLikeReturnTypeHint<'arena>>,
    span: Span,
) {
    if !context.settings.check_missing_type_hints || context.settings.version < PHPVersion::PHP70 {
        return;
    }

    // If it already has a return type hint, nothing to check
    if return_type_hint.is_some() {
        return;
    }

    // Check if we should skip based on function kind
    if matches!(function_like_metadata.kind, FunctionLikeKind::Closure)
        && !context.settings.check_closure_missing_type_hints
    {
        return;
    }
    if matches!(function_like_metadata.kind, FunctionLikeKind::ArrowFunction)
        && !context.settings.check_arrow_function_missing_type_hints
    {
        return;
    }

    // Skip constructors and destructors
    if function_name == "__construct" || function_name == "__destruct" {
        return;
    }

    // If this is a method, check if it's safe to add a return type hint
    if let Some(class_metadata) = class_like_metadata
        && !is_safe_to_add_return_type_hint(context, class_metadata, function_like_metadata)
    {
        return;
    }

    context.collector.report(
        Issue::warning(format!("Function `{function_name}` is missing a return type hint."))
            .with_code(IssueCode::MissingReturnType.as_str())
            .with_annotation(
                Annotation::primary(span)
                    .with_message(format!("Function `{function_name}` declared here without a return type hint")),
            )
            .with_note("Return type hints improve code readability and help prevent type-related errors.")
            .with_help(format!("Consider adding a return type hint to function `{function_name}`.")),
    );
}

/// Check if a return type hint uses a bare `array` or `iterable` without a more specific
/// docblock annotation.
pub fn check_imprecise_return_type_hint<'arena>(
    context: &mut Context<'_, 'arena>,
    function_like_metadata: &FunctionLikeMetadata,
    function_name: &str,
    return_type_hint: Option<&FunctionLikeReturnTypeHint<'arena>>,
) {
    if !context.settings.check_missing_type_hints {
        return;
    }

    if matches!(function_like_metadata.kind, FunctionLikeKind::Closure)
        && !context.settings.check_closure_missing_type_hints
    {
        return;
    }

    if matches!(function_like_metadata.kind, FunctionLikeKind::ArrowFunction)
        && !context.settings.check_arrow_function_missing_type_hints
    {
        return;
    }

    let Some(return_type_hint) = return_type_hint else {
        return;
    };

    if function_like_metadata.return_type_metadata.as_ref().is_some_and(|m| m.from_docblock) {
        return;
    }

    for (type_name, span) in collect_imprecise_hints(&return_type_hint.hint) {
        report_imprecise_type(context, type_name, span, &format!("return type of `{function_name}`"));
    }
}

/// Check if a parameter type hint uses a bare `array` or `iterable` without a more specific
/// docblock annotation.
pub fn check_imprecise_parameter_type_hint<'arena>(
    context: &mut Context<'_, 'arena>,
    function_like_metadata: &FunctionLikeMetadata,
    parameter: &FunctionLikeParameter<'arena>,
    parameter_index: usize,
) {
    if !context.settings.check_missing_type_hints {
        return;
    }

    // Skip closures/arrow functions based on settings
    if matches!(function_like_metadata.kind, FunctionLikeKind::Closure)
        && !context.settings.check_closure_missing_type_hints
    {
        return;
    }
    if matches!(function_like_metadata.kind, FunctionLikeKind::ArrowFunction)
        && !context.settings.check_arrow_function_missing_type_hints
    {
        return;
    }

    let Some(hint) = &parameter.hint else {
        return;
    };

    // If the docblock provides a more specific type, skip
    if let Some(param_meta) = function_like_metadata.parameters.get(parameter_index)
        && param_meta.type_metadata.as_ref().is_some_and(|m| m.from_docblock)
    {
        return;
    }

    for (type_name, span) in collect_imprecise_hints(hint) {
        report_imprecise_type(context, type_name, span, &format!("parameter `{}`", parameter.variable.name));
    }
}

/// Check if a property type hint uses a bare `array` or `iterable` without a more specific
/// docblock annotation.
pub fn check_imprecise_property_type_hint<'arena>(
    context: &mut Context<'_, 'arena>,
    property: &Property<'arena>,
    property_metadata: Option<&PropertyMetadata>,
) {
    if !context.settings.check_missing_type_hints {
        return;
    }

    let hint = match property {
        Property::Plain(plain) => plain.hint.as_ref(),
        Property::Hooked(hooked) => hooked.hint.as_ref(),
    };

    let Some(hint) = hint else {
        return;
    };

    // If the docblock provides a more specific type, skip
    if let Some(prop_meta) = property_metadata
        && prop_meta.type_metadata.as_ref().is_some_and(|m| m.from_docblock)
    {
        return;
    }

    let variables = match property {
        Property::Plain(plain) => plain
            .items
            .iter()
            .filter_map(|item| if let PropertyItem::Concrete(c) = item { Some(c.variable.name) } else { None })
            .collect::<Vec<_>>(),
        Property::Hooked(hooked) => match &hooked.item {
            PropertyItem::Concrete(c) => vec![c.variable.name],
            PropertyItem::Abstract(_) => vec![],
        },
    };

    let imprecise = collect_imprecise_hints(hint);
    if imprecise.is_empty() {
        return;
    }

    for variable_name in variables {
        for &(type_name, span) in &imprecise {
            report_imprecise_type(context, type_name, span, &format!("property `{variable_name}`"));
        }
    }
}

/// Collect all bare `array` or `iterable` hints from a type hint, recursing into
/// unions, intersections, nullable, and parenthesized types.
fn collect_imprecise_hints(hint: &Hint<'_>) -> Vec<(&'static str, Span)> {
    let mut results = vec![];
    collect_imprecise_hints_inner(hint, &mut results);
    results
}

fn collect_imprecise_hints_inner(hint: &Hint<'_>, results: &mut Vec<(&'static str, Span)>) {
    match hint {
        Hint::Array(keyword) => {
            results.push(("array", keyword.span()));
        }
        Hint::Iterable(identifier) => {
            results.push(("iterable", identifier.span()));
        }
        Hint::Nullable(nullable) => {
            collect_imprecise_hints_inner(nullable.hint, results);
        }
        Hint::Union(union) => {
            collect_imprecise_hints_inner(union.left, results);
            collect_imprecise_hints_inner(union.right, results);
        }
        Hint::Intersection(intersection) => {
            collect_imprecise_hints_inner(intersection.left, results);
            collect_imprecise_hints_inner(intersection.right, results);
        }
        Hint::Parenthesized(parenthesized) => {
            collect_imprecise_hints_inner(parenthesized.hint, results);
        }
        _ => {}
    }
}

fn report_imprecise_type(context: &mut Context<'_, '_>, type_name: &str, span: Span, location: &str) {
    // `iterable` can have any key type (not just array-key), since iterators support arbitrary keys.
    let equivalent = if type_name == "iterable" { "iterable<mixed, mixed>" } else { "array<array-key, mixed>" };

    context.collector.report_with_code(
        IssueCode::ImpreciseType,
        Issue::warning(format!("Type `{type_name}` in {location} is imprecise, equivalent to `{equivalent}`."))
            .with_annotation(Annotation::primary(span).with_message(format!("imprecise `{type_name}` type hint")))
            .with_note(format!("Bare `{type_name}` does not specify key or value types, making it difficult for the analyzer to verify correctness."))
            .with_help(format!(
                "Specify a more precise type in a docblock annotation (e.g., `{type_name}<string, int>`, `list<Foo>`), or use `{equivalent}` to be explicit."
            )),
    );
}

/// Check if it's safe to add a type hint to a property.
///
/// It's safe to add a property type hint if no parent class or trait declares the same property
/// without a type hint (because adding a type hint would create a compile error in PHP).
fn is_safe_to_add_property_type_hint(
    context: &Context,
    class_like_metadata: &ClassLikeMetadata,
    property_name: &str,
) -> bool {
    let property_atom = mago_atom::atom(property_name);

    // Check all parent classes
    for parent_name in &class_like_metadata.all_parent_classes {
        if let Some(parent_metadata) = context.codebase.get_class_like(parent_name) {
            // If parent has this property
            if parent_metadata.properties.contains_key(&property_atom) {
                // Check if parent property has a type hint
                // If parent has no type hint, we can't safely add one to the child
                // For now, we'll be conservative and not report if parent has the property
                // TODO: We need to check if parent property actually has a type hint or not
                // This requires metadata about whether properties have type hints
                return false;
            }
        }
    }

    // Check all used traits
    for trait_name in &class_like_metadata.used_traits {
        if let Some(trait_metadata) = context.codebase.get_class_like(trait_name)
            && trait_metadata.properties.contains_key(&property_atom)
        {
            // Same reasoning as parent classes
            return false;
        }
    }

    true
}

/// Check if it's safe to add a type hint to a parameter.
///
/// It's safe to add a parameter type hint if the method is not overriding a parent method
/// that has no type hints on the corresponding parameter.
fn is_safe_to_add_parameter_type_hint(
    context: &Context,
    class_like_metadata: &ClassLikeMetadata,
    function_like_metadata: &FunctionLikeMetadata,
) -> bool {
    // If it's not a method, it's always safe
    if !matches!(function_like_metadata.kind, FunctionLikeKind::Method) {
        return true;
    }

    // For methods, check if we have a name
    let Some(method_name) = function_like_metadata.name else {
        return true;
    };

    // Check if this method is overriding a parent method
    if context.codebase.method_is_overriding(&class_like_metadata.name, &method_name) {
        // If overriding, we need to be conservative and not report
        // because we'd need to check if all parameters in the parent have type hints
        return false;
    }

    true
}

/// Check if it's safe to add a return type hint.
///
/// It's safe to add a return type hint if the method is not overriding a parent method
/// that has no return type hint.
fn is_safe_to_add_return_type_hint(
    context: &Context,
    class_like_metadata: &ClassLikeMetadata,
    function_like_metadata: &FunctionLikeMetadata,
) -> bool {
    // If it's not a method, it's always safe
    if !matches!(function_like_metadata.kind, FunctionLikeKind::Method) {
        return true;
    }

    // For methods, check if we have a name
    let Some(method_name) = function_like_metadata.name else {
        return true;
    };

    // Check if this method is overriding a parent method
    if context.codebase.method_is_overriding(&class_like_metadata.name, &method_name) {
        // If overriding, we need to be conservative and not report
        return false;
    }

    true
}
