use mago_php_version::feature::Feature;
use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::internal::context::Context;

mod parameter;

pub use parameter::*;

use super::returns_generator;

#[inline]
pub fn check_function<'arena>(function: &Function<'arena>, context: &mut Context<'_, '_, 'arena>) {
    check_for_promoted_properties_outside_constructor(&function.parameter_list, context);
    let Some(return_hint) = &function.return_type_hint else {
        return;
    };

    let name = function.name.value;
    let fqfn = context.get_name(&function.name.span.start);

    match &return_hint.hint {
        Hint::Void(_) => {
            for r#return in mago_syntax::utils::find_returns_in_block(&function.body) {
                if let Some(val) = &r#return.value {
                    context.report(
                        Issue::error(format!("Function `{name}` with return type `void` must not return a value."))
                            .with_annotation(Annotation::primary(val.span()).with_message("Return value found here."))
                            .with_annotation(
                                Annotation::secondary(function.span())
                                    .with_message(format!("Function `{fqfn}` defined here.")),
                            )
                            .with_help("Remove the return type hint or the return value."),
                    );
                }
            }
        }
        Hint::Never(_) => {
            for r#return in mago_syntax::utils::find_returns_in_block(&function.body) {
                context.report(
                    Issue::error(format!("Function `{name}` with return type `never` must not return."))
                        .with_annotation(
                            Annotation::primary(r#return.span()).with_message("Return statement found here."),
                        )
                        .with_annotation(
                            Annotation::secondary(function.span())
                                .with_message(format!("Function `{fqfn}` defined here.")),
                        )
                        .with_help("Remove the return type hint or the return statement."),
                );
            }
        }
        _ if !returns_generator(context, &function.body, &return_hint.hint) => {
            for r#return in mago_syntax::utils::find_returns_in_block(&function.body) {
                if r#return.value.is_none() {
                    context.report(
                        Issue::error(format!("Function `{name}` with a return type must return a value."))
                            .with_annotation(
                                Annotation::primary(r#return.span()).with_message("Empty return statement found here."),
                            )
                            .with_annotation(
                                Annotation::secondary(function.span())
                                    .with_message(format!("Function `{fqfn}` defined here.")),
                            )
                            .with_note("Did you mean `return null;` instead of `return;`?")
                            .with_help("Add a return value to the statement."),
                    );
                }
            }
        }
        _ => {}
    }
}

#[inline]
pub fn check_arrow_function(arrow_function: &ArrowFunction, context: &mut Context<'_, '_, '_>) {
    if !context.version.is_supported(Feature::ArrowFunctions) {
        let issue = Issue::error("The `fn` keyword for arrow functions is only available in PHP 7.4 and later.")
            .with_annotation(
                Annotation::primary(arrow_function.span()).with_message("Arrow function uses `fn` keyword."),
            );

        context.report(issue);
    }

    check_for_promoted_properties_outside_constructor(&arrow_function.parameter_list, context);

    if let Some(return_hint) = &arrow_function.return_type_hint {
        // while technically valid, it is not possible to return `void` from an arrow function
        // because the return value is always inferred from the body, even if the body does
        // not return a value, in the case it throws or exits the process.
        //
        // see: https://3v4l.org/VgoiO
        match &return_hint.hint {
            Hint::Void(_) => {
                context.report(
                    Issue::error("Arrow function cannot have a return type of `void`.")
                        .with_annotation(
                            Annotation::primary(return_hint.hint.span())
                                .with_message("Return type `void` is not valid for an arrow function."),
                        )
                        .with_annotation(
                            Annotation::secondary(arrow_function.r#fn.span)
                                .with_message("Arrow function defined here."),
                        )
                        .with_help("Remove the `void` return type hint, or replace it with a valid type."),
                );
            }
            Hint::Never(_) if !context.version.is_supported(Feature::NeverReturnTypeInArrowFunction) => {
                context.report(
                    Issue::error("The `never` return type in arrow functions is only available in PHP 8.2 and later.")
                        .with_annotation(
                            Annotation::primary(return_hint.hint.span())
                                .with_message("Return type `never` is not valid for an arrow function."),
                        )
                        .with_annotation(
                            Annotation::secondary(arrow_function.r#fn.span)
                                .with_message("Arrow function defined here."),
                        ),
                );
            }
            _ => {}
        }
    }
}

#[inline]
pub fn check_closure<'arena>(closure: &Closure<'arena>, context: &mut Context<'_, '_, 'arena>) {
    check_for_promoted_properties_outside_constructor(&closure.parameter_list, context);

    if !context.version.is_supported(Feature::TrailingCommaInClosureUseList)
        && let Some(trailing_comma) = &closure.use_clause.as_ref().and_then(|u| u.variables.get_trailing_token())
    {
        context.report(
                Issue::error("Trailing comma in closure use list is only available in PHP 8.0 and later.")
                .with_annotation(
                    Annotation::primary(trailing_comma.span).with_message("Trailing comma found here."),
                )
                .with_help(
                    "Remove the trailing comma to make the code compatible with PHP 7.4 and earlier versions, or upgrade to PHP 8.0 or later.",
                )
            );
    }

    let hint = if let Some(return_hint) = &closure.return_type_hint {
        &return_hint.hint
    } else {
        return;
    };

    let returns = mago_syntax::utils::find_returns_in_block(&closure.body);

    match &hint {
        Hint::Void(_) => {
            for r#return in returns {
                if let Some(val) = &r#return.value {
                    context.report(
                        Issue::error("Closure with a return type of `void` must not return a value.")
                            .with_annotation(
                                Annotation::primary(val.span())
                                    .with_message("This value is not allowed with a `void` return type."),
                            )
                            .with_annotation(
                                Annotation::secondary(closure.span()).with_message("Closure defined here."),
                            )
                            .with_help(
                                "Remove the return value, or change the return type hint to an appropriate type.",
                            ),
                    );
                }
            }
        }
        Hint::Never(_) => {
            for r#return in returns {
                context.report(
                    Issue::error("Closure with a return type of `never` must not include a return statement.")
                        .with_annotation(
                            Annotation::primary(r#return.span())
                                .with_message("Return statement is not allowed with a `never` return type."),
                        )
                        .with_annotation(Annotation::secondary(closure.span()).with_message("Closure defined here."))
                        .with_help("Remove the return statement, or change the return type hint to a compatible type."),
                );
            }
        }
        _ if !returns_generator(context, &closure.body, hint) => {
            for r#return in returns {
                if r#return.value.is_none() {
                    context.report(
                        Issue::error("Closure with a return type must return a value.")
                            .with_annotation(Annotation::primary(r#return.span()).with_message("Missing return value."))
                            .with_annotation(
                                Annotation::secondary(closure.span()).with_message("Closure defined here."),
                            )
                            .with_note("Did you mean `return null;` instead of `return;`?")
                            .with_help("Add a return value that matches the expected return type."),
                    );
                }
            }
        }
        _ => {}
    }
}

#[inline]
pub fn check_return_type_hint(
    function_like_return_type_hint: &FunctionLikeReturnTypeHint,
    context: &mut Context<'_, '_, '_>,
) {
    match &function_like_return_type_hint.hint {
        Hint::Union(union_hint) if !context.version.is_supported(Feature::NativeUnionTypes) => {
            context.report(
                Issue::error(
                    "Union type hints (e.g. `int|float`) are only available in PHP 8.0 and above."
                )
                .with_annotation(Annotation::primary(union_hint.span()).with_message("Union type hint used here."))
                .with_note(
                    "Union type hints are only available in PHP 8.0 and above. Consider using a different approach.",
                )
                .with_help("Remove the union type hint to make the code compatible with PHP 7.4 and earlier versions, or upgrade to PHP 8.0 or later."),
                        );
        }
        Hint::Static(r#static) if !context.version.is_supported(Feature::StaticReturnTypeHint) => {
            context.report(
                Issue::error("Static return type hints are only available in PHP 8.0 and above.").with_annotation(
                    Annotation::primary(r#static.span()).with_message("Static return type hint used here."),
                )
                .with_help("Remove the static return type hint to make the code compatible with PHP 7.4 and earlier versions, or upgrade to PHP 8.0 or later."),
            );
        }
        _ => {}
    }
}

#[inline]
pub fn check_for_promoted_properties_outside_constructor(
    parameter_list: &FunctionLikeParameterList,
    context: &mut Context<'_, '_, '_>,
) {
    for parameter in parameter_list.parameters.iter() {
        if parameter.is_promoted_property() {
            context.report(
                Issue::error("Promoted properties are not allowed outside of constructors.")
                    .with_annotation(
                        Annotation::primary(parameter.span()).with_message("Promoted property found here."),
                    )
                    .with_help("Move this promoted property to the constructor, or remove the promotion."),
            );
        }
    }
}
