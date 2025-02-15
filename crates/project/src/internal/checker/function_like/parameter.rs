use mago_ast::*;
use mago_php_version::feature::Feature;
use mago_reporting::*;
use mago_span::*;

use crate::internal::context::Context;

#[inline]
pub fn check_parameter_list(function_like_parameter_list: &FunctionLikeParameterList, context: &mut Context<'_>) {
    let mut last_variadic = None;
    let mut parameters_seen = vec![];
    for parameter in function_like_parameter_list.parameters.iter() {
        if parameter.is_promoted_property() && !context.version.is_supported(Feature::PromotedProperties) {
            context.issues.push(
                Issue::error("Promoted properties are only available in PHP 8.0 and above.").with_annotation(
                    Annotation::primary(parameter.span()).with_message("Promoted property used here."),
                ),
            );
        }

        let name = context.interner.lookup(&parameter.variable.name);
        if let Some(prev_span) =
            parameters_seen.iter().find_map(|(n, s)| if parameter.variable.name.eq(n) { Some(s) } else { None })
        {
            context.issues.push(
                Issue::error(format!("Parameter `{}` is already defined.", name))
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
                    context.issues.push(
                        Issue::error(format!(
                            "Parameter `{}` cannot have the `{}` modifier.",
                            name,
                            context.interner.lookup(&keyword.value)
                        ))
                        .with_annotation(Annotation::primary(modifier.span()).with_message(format!(
                            "Invalid `{}` modifier used here.",
                            context.interner.lookup(&keyword.value)
                        )))
                        .with_annotation(
                            Annotation::secondary(parameter.variable.span)
                                .with_message(format!("Parameter `{}` defined here.", name)),
                        )
                        .with_help("Remove the invalid modifier from the parameter."),
                    );
                }
                Modifier::Readonly(_) => {
                    if let Some(s) = last_readonly {
                        context.issues.push(
                            Issue::error(format!("Parameter `{}` cannot have multiple `readonly` modifiers.", name))
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
                        context.issues.push(
                            Issue::error(format!("Parameter `{}` cannot have multiple visibility modifiers.", name))
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
                        context.issues.push(
                            Issue::error(format!(
                                "Parameter `{}` cannot have multiple write visibility modifiers.",
                                name
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
            context.issues.push(
                Issue::error(format!(
                    "Invalid parameter order: parameter `{}` is defined after variadic parameter `{}`.",
                    name,
                    context.interner.lookup(&n)
                ))
                .with_annotation(
                    Annotation::primary(parameter.variable.span())
                        .with_message(format!("Parameter `{}` is defined here.", name)),
                )
                .with_annotation(
                    Annotation::secondary(s)
                        .with_message(format!("Variadic parameter `{}` is defined here.", context.interner.lookup(&n))),
                )
                .with_help("Move all parameters following the variadic parameter to the end of the parameter list."),
            );
        }

        if let Some(ellipsis) = parameter.ellipsis {
            if let Some(default) = &parameter.default_value {
                context.issues.push(
                    Issue::error(format!(
                        "Invalid parameter definition: variadic parameter `{}` cannot have a default value.",
                        name
                    ))
                    .with_annotation(
                        Annotation::primary(default.span())
                            .with_message(format!("Default value is defined for variadic parameter `{}` here.", name)),
                    )
                    .with_annotation(
                        Annotation::secondary(ellipsis.join(parameter.variable.span))
                            .with_message(format!("Parameter `{}` is variadic and marked with `...` here.", name)),
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

                context.issues.push(
                    Issue::error(format!(
                        "Invalid parameter type: bottom type `{}` cannot be used as a parameter type.",
                        hint_name
                    ))
                    .with_annotation(
                        Annotation::primary(hint.span())
                            .with_message(format!("Bottom type `{}` is not allowed here.", hint_name)),
                    )
                    .with_annotation(
                        Annotation::secondary(parameter.variable.span())
                            .with_message(format!("This parameter `{}` is defined here.", name)),
                    )
                    .with_help("Use a valid parameter type to ensure compatibility with PHP's type system."),
                );
            } else if hint.is_union() && !context.version.is_supported(Feature::NativeUnionTypes) {
                context.issues.push(
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
