use mago_php_version::feature::Feature;
use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::internal::context::Context;

#[inline]
pub fn check_parameter_list(
    function_like_parameter_list: &FunctionLikeParameterList,
    context: &mut Context<'_, '_, '_>,
) {
    let mut last_variadic = None;
    let mut parameters_seen: Vec<(&str, Span)> = vec![];
    for parameter in function_like_parameter_list.parameters.iter() {
        if parameter.is_promoted_property() && !context.version.is_supported(Feature::PromotedProperties) {
            context.report(
                Issue::error("Promoted properties are only available in PHP 8.0 and above.").with_annotation(
                    Annotation::primary(parameter.span()).with_message("Promoted property used here."),
                ),
            );
        }

        let name = parameter.variable.name;
        if let Some(prev_span) =
            parameters_seen.iter().find_map(|(n, s)| if parameter.variable.name.eq(*n) { Some(s) } else { None })
        {
            context.report(
                Issue::error(format!("Parameter `{name}` is already defined."))
                    .with_annotation(
                        Annotation::primary(parameter.variable.span())
                            .with_message("This parameter is redefined here."),
                    )
                    .with_annotation(
                        Annotation::secondary(*prev_span).with_message("The original parameter was defined here."),
                    )
                    .with_help("Ensure all parameter names are unique within the parameter list."),
            );
        } else if !parameter.is_promoted_property() {
            parameters_seen.push((parameter.variable.name, parameter.variable.span()));
        }

        let mut last_readonly = None;
        let mut last_read_visibility = None;
        let mut last_write_visibility = None;
        for modifier in parameter.modifiers.iter() {
            match &modifier {
                Modifier::Static(keyword) | Modifier::Final(keyword) | Modifier::Abstract(keyword) => {
                    context.report(
                        Issue::error(format!("Parameter `{}` cannot have the `{}` modifier.", name, keyword.value))
                            .with_annotation(
                                Annotation::primary(modifier.span())
                                    .with_message(format!("Invalid `{}` modifier used here.", keyword.value)),
                            )
                            .with_annotation(
                                Annotation::secondary(parameter.variable.span)
                                    .with_message(format!("Parameter `{name}` defined here.")),
                            )
                            .with_help("Remove the invalid modifier from the parameter."),
                    );
                }
                Modifier::Readonly(_) => {
                    if let Some(s) = last_readonly {
                        context.report(
                            Issue::error(format!("Parameter `{name}` cannot have multiple `readonly` modifiers."))
                                .with_annotation(
                                    Annotation::primary(modifier.span())
                                        .with_message("Duplicate `readonly` modifier used here."),
                                )
                                .with_annotation(
                                    Annotation::secondary(s).with_message("Previous `readonly` modifier used here."),
                                )
                                .with_help("Remove the duplicate `readonly` modifier."),
                        );
                    } else {
                        last_readonly = Some(modifier.span());
                    }
                }
                Modifier::Public(_) | Modifier::Protected(_) | Modifier::Private(_) => {
                    if let Some(s) = last_read_visibility {
                        context.report(
                            Issue::error(format!("Parameter `{name}` cannot have multiple visibility modifiers."))
                                .with_annotation(
                                    Annotation::primary(modifier.span())
                                        .with_message("Duplicate visibility modifier used here."),
                                )
                                .with_annotation(
                                    Annotation::secondary(s).with_message("Previous visibility modifier used here."),
                                )
                                .with_help("Remove the duplicate visibility modifier."),
                        );
                    } else {
                        last_read_visibility = Some(modifier.span());
                    }
                }
                Modifier::PrivateSet(_) | Modifier::ProtectedSet(_) | Modifier::PublicSet(_) => {
                    if let Some(s) = last_write_visibility {
                        context.report(
                            Issue::error(format!(
                                "Parameter `{name}` cannot have multiple write visibility modifiers."
                            ))
                            .with_annotation(
                                Annotation::primary(modifier.span())
                                    .with_message("Duplicate write visibility modifier used here."),
                            )
                            .with_annotation(
                                Annotation::secondary(s).with_message("Previous write visibility modifier used here."),
                            )
                            .with_help("Remove the duplicate write visibility modifier."),
                        );
                    } else {
                        last_write_visibility = Some(modifier.span());
                    }
                }
            }
        }

        if let Some((n, s)) = last_variadic {
            context.report(
                Issue::error(format!(
                    "Invalid parameter order: parameter `{name}` is defined after variadic parameter `{n}`.",
                ))
                .with_annotation(
                    Annotation::primary(parameter.variable.span())
                        .with_message(format!("Parameter `{name}` is defined here.")),
                )
                .with_annotation(
                    Annotation::secondary(s).with_message(format!("Variadic parameter `{n}` is defined here.")),
                )
                .with_help("Move all parameters following the variadic parameter to the end of the parameter list."),
            );
        }

        if let Some(ellipsis) = parameter.ellipsis {
            if let Some(default) = &parameter.default_value {
                context.report(
                    Issue::error(format!(
                        "Invalid parameter definition: variadic parameter `{name}` cannot have a default value."
                    ))
                    .with_annotation(
                        Annotation::primary(default.span())
                            .with_message(format!("Default value is defined for variadic parameter `{name}` here.")),
                    )
                    .with_annotation(
                        Annotation::secondary(ellipsis.join(parameter.variable.span))
                            .with_message(format!("Parameter `{name}` is variadic and marked with `...` here.")),
                    )
                    .with_help("Remove the default value from the variadic parameter."),
                );
            }

            last_variadic = Some((parameter.variable.name, parameter.span()));
            continue;
        }

        if let Some(hint) = &parameter.hint {
            if hint.is_bottom() {
                let hint_name = context.get_code_snippet(hint);

                context.report(
                    Issue::error(format!(
                        "Invalid parameter type: bottom type `{hint_name}` cannot be used as a parameter type."
                    ))
                    .with_annotation(
                        Annotation::primary(hint.span())
                            .with_message(format!("Bottom type `{hint_name}` is not allowed here.")),
                    )
                    .with_annotation(
                        Annotation::secondary(parameter.variable.span())
                            .with_message(format!("This parameter `{name}` is defined here.")),
                    )
                    .with_help("Use a valid parameter type to ensure compatibility with PHP's type system."),
                );
            } else if hint.is_union() && !context.version.is_supported(Feature::NativeUnionTypes) {
                context.report(
                    Issue::error(
                        "Union type hints (e.g. `int|float`) are only available in PHP 8.0 and above.",
                    )
                    .with_annotation(Annotation::primary(hint.span()).with_message("Union type hint used here."))
                    .with_note(
                        "Union type hints are only available in PHP 8.0 and above. Consider using a different approach.",
                    ),
                );
            }
        }
    }
}
