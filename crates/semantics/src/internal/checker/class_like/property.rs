use mago_php_version::feature::Feature;
use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::internal::checker::function_like::check_for_promoted_properties_outside_constructor;
use crate::internal::context::Context;

#[inline]
pub fn check_property(
    property: &Property,
    class_like_span: Span,
    class_like_kind: &str,
    class_like_name: &str,
    class_like_fqcn: &str,
    class_like_is_interface: bool,
    context: &mut Context<'_, '_, '_>,
) {
    let first_variable = property.first_variable();
    let first_variable_name = first_variable.name;

    let modifiers = property.modifiers();
    let mut last_final: Option<Span> = None;
    let mut last_static: Option<Span> = None;
    let mut last_readonly: Option<Span> = None;
    let mut last_read_visibility: Option<Span> = None;
    let mut last_write_visibility: Option<Span> = None;

    for modifier in modifiers.iter() {
        match modifier {
            Modifier::Abstract(_) => {
                context.report(
                    Issue::error(format!(
                        "Property `{class_like_name}::{first_variable_name}` cannot be declared abstract"
                    ))
                    .with_annotation(
                        Annotation::primary(modifier.span())
                            .with_message("`abstract` modifier cannot be used on properties"),
                    )
                    .with_annotation(
                        Annotation::secondary(first_variable.span())
                            .with_message(format!("Property `{first_variable_name}` declared here.")),
                    )
                    .with_annotation(
                        Annotation::secondary(class_like_span)
                            .with_message(format!("{class_like_kind} `{class_like_fqcn}` defined here.")),
                    ),
                );
            }
            Modifier::Static(_) => {
                if let Some(last_readonly) = last_readonly {
                    context.report(
                        Issue::error(format!(
                            "Readonly property `{class_like_name}::{first_variable_name}` cannot be static."
                        ))
                        .with_annotation(
                            Annotation::primary(modifier.span())
                                .with_message("`static` modifier cannot be used on readonly properties."),
                        )
                        .with_annotation(
                            Annotation::primary(last_readonly).with_message("Property is marked as readonly here."),
                        )
                        .with_annotation(
                            Annotation::secondary(first_variable.span())
                                .with_message(format!("Property `{first_variable_name}` declared here.")),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{class_like_kind} `{class_like_fqcn}` defined here.")),
                        ),
                    );
                }

                if let Some(last_static) = last_static {
                    context.report(
                        Issue::error(format!(
                            "Property `{class_like_name}::{first_variable_name}` has multiple `static` modifiers."
                        ))
                        .with_annotation(
                            Annotation::primary(modifier.span()).with_message("Duplicate `static` modifier."),
                        )
                        .with_annotation(Annotation::secondary(last_static).with_message("Previous `static` modifier."))
                        .with_annotation(
                            Annotation::secondary(first_variable.span())
                                .with_message(format!("Property `{first_variable_name}` declared here.")),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{class_like_kind} `{class_like_fqcn}` defined here.")),
                        ),
                    );
                }

                if let Some(last_visibility) = last_write_visibility
                    && !context.version.is_supported(Feature::AsymmetricVisibilityForStaticProperties)
                {
                    context.report(
                            Issue::error(format!(
                                "Asymmetric visibility for static property `{class_like_name}::{first_variable_name}` is not available in your current PHP version, this feature was introduced in PHP 8.5.",
                            ))
                            .with_annotation(
                                Annotation::primary(last_visibility).with_message("This write visibility modifier is used here"),
                            )
                            .with_annotation(
                                Annotation::secondary(modifier.span()).with_message("On this static property"),
                            )
                            .with_annotation(
                                Annotation::secondary(first_variable.span()).with_message(format!("Static property `{first_variable_name}`")),
                            )
                            .with_note(
                                "PHP 8.4 introduced asymmetric visibility for properties, and PHP 8.5 extended this support to static properties."
                            )
                            .with_help(
                                "To use this feature, please configure your project for PHP 8.5+. Alternatively, remove the specific write visibility from this static property or make the property non-static."
                            ),
                        );
                }

                last_static = Some(modifier.span());
            }
            Modifier::Readonly(modifier) => {
                if !context.version.is_supported(Feature::ReadonlyProperties) {
                    context.report(
                        Issue::error("Readonly properties are only available in PHP 8.1 and above.").with_annotation(
                            Annotation::primary(modifier.span()).with_message("Readonly modifier used here."),
                        ),
                    );

                    continue;
                }

                if let Some(last_static) = last_static {
                    context.report(
                        Issue::error(format!(
                            "Static property `{class_like_name}::{first_variable_name}` cannot be readonly."
                        ))
                        .with_annotation(
                            Annotation::primary(modifier.span())
                                .with_message("`readonly` modifier cannot be used on static properties."),
                        )
                        .with_annotation(
                            Annotation::primary(last_static).with_message("Property is marked as static here."),
                        )
                        .with_annotation(
                            Annotation::secondary(first_variable.span())
                                .with_message(format!("Property `{first_variable_name}` declared here.")),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{class_like_kind} `{class_like_fqcn}` defined here.")),
                        ),
                    );
                }

                if let Some(last_readonly) = last_readonly {
                    context.report(
                        Issue::error(format!(
                            "Property `{class_like_name}::{first_variable_name}` has multiple `readonly` modifiers."
                        ))
                        .with_annotation(
                            Annotation::primary(modifier.span()).with_message("Duplicate `readonly` modifier."),
                        )
                        .with_annotation(
                            Annotation::secondary(last_readonly).with_message("Previous `readonly` modifier."),
                        )
                        .with_annotation(
                            Annotation::secondary(first_variable.span())
                                .with_message(format!("Property `{first_variable_name}` declared here.")),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{class_like_kind} `{class_like_fqcn}` defined here.")),
                        ),
                    );
                }

                last_readonly = Some(modifier.span());
            }
            Modifier::Final(_) => {
                if let Some(last_final) = last_final {
                    context.report(
                        Issue::error("Property has multiple `final` modifiers.")
                            .with_annotation(
                                Annotation::primary(modifier.span()).with_message("Duplicate `final` modifier."),
                            )
                            .with_annotation(Annotation::primary(last_final).with_message("Previous `final` modifier."))
                            .with_annotation(
                                Annotation::secondary(first_variable.span())
                                    .with_message(format!("Property `{first_variable_name}` declared here.")),
                            )
                            .with_annotation(
                                Annotation::secondary(class_like_span)
                                    .with_message(format!("{class_like_kind} `{class_like_fqcn}` defined here.")),
                            ),
                    );
                }

                last_final = Some(modifier.span());
            }
            Modifier::Private(_) | Modifier::Protected(_) | Modifier::Public(_) => {
                if let Some(last_visibility) = last_read_visibility {
                    context.report(
                        Issue::error(format!(
                            "Property `{class_like_name}::{first_variable_name}` has multiple visibility modifiers."
                        ))
                        .with_annotation(
                            Annotation::primary(modifier.span()).with_message("Duplicate visibility modifier."),
                        )
                        .with_annotation(
                            Annotation::primary(last_visibility).with_message("Previous visibility modifier."),
                        )
                        .with_annotation(
                            Annotation::secondary(first_variable.span())
                                .with_message(format!("Property `{first_variable_name}` declared here.")),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{class_like_kind} `{class_like_fqcn}` defined here.")),
                        ),
                    );
                }

                last_read_visibility = Some(modifier.span());
            }
            Modifier::PrivateSet(_) | Modifier::ProtectedSet(_) | Modifier::PublicSet(_) => {
                if !context.version.is_supported(Feature::AsymmetricVisibility) {
                    context.report(
                        Issue::error("Asymmetric visibility is only available in PHP 8.4 and above.").with_annotation(
                            Annotation::primary(modifier.span())
                                .with_message("Asymmetric visibility modifier used here."),
                        ),
                    );

                    continue;
                }

                if let Some(last_visibility) = last_write_visibility {
                    context.report(
                        Issue::error(format!(
                            "Property `{class_like_name}::{first_variable_name}` has multiple write visibility modifiers."
                        ))
                        .with_annotation(
                            Annotation::primary(modifier.span()).with_message("Duplicate write visibility modifier."),
                        )
                        .with_annotation(
                            Annotation::primary(last_visibility).with_message("Previous write visibility modifier."),
                        )
                        .with_annotation(
                            Annotation::secondary(first_variable.span())
                                .with_message(format!("Property `{first_variable_name}` declared here.")),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{class_like_kind} `{class_like_fqcn}` defined here.")),
                        ),
                    );
                }

                if let Some(last_static) = last_static
                    && !context.version.is_supported(Feature::AsymmetricVisibilityForStaticProperties)
                {
                    context.report(
                            Issue::error(format!(
                                "Asymmetric visibility for static property `{class_like_name}::{first_variable_name}` is not available in your current PHP version, this feature was introduced in PHP 8.5.",
                            ))
                            .with_annotation(
                                Annotation::primary(modifier.span()).with_message("This write visibility modifier is used here"),
                            )
                            .with_annotation(
                                Annotation::secondary(last_static).with_message("On this static property"),
                            )
                            .with_annotation(
                                Annotation::secondary(first_variable.span()).with_message(format!("Static property `{first_variable_name}`")),
                            )
                            .with_note(
                                "PHP 8.4 introduced asymmetric visibility for properties, and PHP 8.5 extended this support to static properties."
                            )
                            .with_help(
                                "To use this feature, please configure your project for PHP 8.5+. Alternatively, remove the specific write visibility from this static property or make the property non-static."
                            ),
                        );
                }

                last_write_visibility = Some(modifier.span());
            }
        }
    }

    if let Some(var) = property.var()
        && !modifiers.is_empty()
    {
        let first = modifiers.first().unwrap();
        let last = modifiers.last().unwrap();

        context.report(
            Issue::error(format!("Var property `{class_like_name}::{first_variable_name}` cannot have modifiers."))
                .with_annotation(
                    Annotation::primary(first.span().join(last.span())).with_message("Modifiers used here."),
                )
                .with_annotation(Annotation::primary(var.span()).with_message("Property is marked as `var` here."))
                .with_annotation(
                    Annotation::secondary(first_variable.span())
                        .with_message(format!("Property `{first_variable_name}` declared here.")),
                )
                .with_annotation(
                    Annotation::secondary(class_like_span)
                        .with_message(format!("{class_like_kind} `{class_like_fqcn}` defined here.")),
                )
                .with_help("Remove either the `var` keyword, or the modifiers.".to_string()),
        );
    }

    if let Some(hint) = property.hint() {
        if !context.version.is_supported(Feature::TypedProperties) {
            context.report(
                Issue::error("Typed properties are only available in PHP 7.4 and above.")
                    .with_annotation(Annotation::primary(hint.span()).with_message("Type hint used here."))
                    .with_help("Remove the type hint to make the code compatible with PHP 7.3 and earlier versions, or upgrade to PHP 7.4 or later."),
            );
        }

        if !context.version.is_supported(Feature::NativeUnionTypes) && hint.is_union() {
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

        if hint.is_bottom() {
            let hint_name = context.get_code_snippet(hint);
            // cant be used on properties
            context.report(
                Issue::error(format!(
                    "Property `{class_like_name}::{first_variable_name}` cannot have type `{hint_name}`."
                ))
                .with_annotation(
                    Annotation::primary(hint.span())
                        .with_message(format!("Type `{hint_name}` is not allowed on properties.")),
                )
                .with_annotation(
                    Annotation::secondary(first_variable.span())
                        .with_message(format!("Property `{first_variable_name}` declared here.")),
                )
                .with_annotation(
                    Annotation::secondary(class_like_span)
                        .with_message(format!("{class_like_kind} `{class_like_fqcn}` defined here.")),
                ),
            );
        }
    } else if let Some(readonly) = last_readonly {
        // readonly properties must have a type hint
        context.report(
            Issue::error(format!(
                "Readonly property `{class_like_name}::{first_variable_name}` must have a type hint."
            ))
            .with_annotation(Annotation::primary(readonly).with_message("Property is marked as readonly here."))
            .with_annotation(
                Annotation::secondary(first_variable.span())
                    .with_message(format!("Property `{first_variable_name}` declared here.")),
            )
            .with_annotation(
                Annotation::secondary(class_like_span)
                    .with_message(format!("{class_like_kind} `{class_like_fqcn}` defined here.")),
            ),
        );
    }

    match &property {
        Property::Plain(plain_property) => {
            if !context.version.is_supported(Feature::AsymmetricVisibility)
                && let Some(write_visibility) = plain_property.modifiers.get_first_write_visibility()
            {
                context.report(
                    Issue::error("Asymmetric visibility is only available in PHP 8.4 and above.").with_annotation(
                        Annotation::primary(write_visibility.span()).with_message("Asymmetric visibility used here."),
                    ),
                );
            };

            for item in plain_property.items.iter() {
                if let PropertyItem::Concrete(property_concrete_item) = &item {
                    let item_name = property_concrete_item.variable.name;

                    if !property_concrete_item.value.is_constant(&context.version, false) {
                        context.report(
                            Issue::error(format!(
                                "Property `{class_like_name}::{item_name}` value contains a non-constant expression."
                            ))
                            .with_annotation(
                                Annotation::primary(property_concrete_item.value.span())
                                    .with_message("This is a non-constant expression."),
                            )
                            .with_annotation(
                                Annotation::secondary(property_concrete_item.variable.span())
                                    .with_message(format!("Property `{item_name}` declared here.")),
                            )
                            .with_annotation(
                                Annotation::secondary(class_like_span)
                                    .with_message(format!("{class_like_kind} `{class_like_fqcn}` defined here.")),
                            ),
                        );
                    }

                    if let Some(readonly) = last_readonly {
                        context.report(
                            Issue::error(format!(
                                "Readonly property `{class_like_name}::{item_name}` cannot have a default value."
                            ))
                            .with_annotation(
                                Annotation::primary(property_concrete_item.value.span())
                                    .with_message("This is a default value."),
                            )
                            .with_annotation(Annotation::primary(readonly).with_message(format!(
                                "Property `{class_like_name}::{item_name}` is marked as readonly here."
                            )))
                            .with_annotation(
                                Annotation::secondary(property_concrete_item.variable.span())
                                    .with_message(format!("Property `{item_name}` is declared here.")),
                            )
                            .with_annotation(
                                Annotation::secondary(class_like_span)
                                    .with_message(format!("{class_like_kind} `{class_like_fqcn}` defined here.")),
                            ),
                        );
                    }
                }
            }
        }
        Property::Hooked(hooked_property) => {
            if !context.version.is_supported(Feature::PropertyHooks) {
                let issue = Issue::error("Hooked properties are only available in PHP 8.4 and above.").with_annotation(
                    Annotation::primary(hooked_property.span()).with_message("Hooked property declaration used here."),
                );

                context.report(issue);
            }

            let item_name = hooked_property.item.variable().name;

            if let Some(readonly) = last_readonly {
                context.report(
                    Issue::error(format!("Hooked property `{class_like_name}::{item_name}` cannot be readonly."))
                        .with_annotation(Annotation::primary(readonly).with_message(format!(
                            "Property `{class_like_name}::{item_name}` is marked as readonly here."
                        )))
                        .with_annotation(
                            Annotation::secondary(hooked_property.hook_list.span())
                                .with_message("Property hooks are defined here."),
                        )
                        .with_annotation(
                            Annotation::secondary(hooked_property.item.variable().span())
                                .with_message(format!("Property `{item_name}` is declared here.")),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{class_like_kind} `{class_like_fqcn}` defined here.")),
                        ),
                );
            }

            if let Some(r#static) = last_static {
                context.report(
                    Issue::error(format!("Hooked property `{class_like_name}::{item_name}` cannot be static."))
                        .with_annotation(Annotation::primary(r#static).with_message(format!(
                            "Property `{class_like_name}::{item_name}` is marked as static here."
                        )))
                        .with_annotation(
                            Annotation::secondary(hooked_property.hook_list.span())
                                .with_message("Property hooks are defined here."),
                        )
                        .with_annotation(
                            Annotation::secondary(hooked_property.item.variable().span())
                                .with_message(format!("Property `{item_name}` is declared here.")),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{class_like_kind} `{class_like_fqcn}` defined here.")),
                        ),
                );
            }

            let mut hook_names: Vec<(std::string::String, Span)> = vec![];
            for hook in hooked_property.hook_list.hooks.iter() {
                let name = hook.name.value;
                let lowered_name = name.to_ascii_lowercase();

                if !hook.modifiers.is_empty() {
                    let first = hook.modifiers.first().unwrap();
                    let last = hook.modifiers.last().unwrap();

                    context.report(
                        Issue::error(format!(
                            "Hook `{name}` for property `{class_like_name}::{item_name}` cannot have modifiers."
                        ))
                        .with_annotation(
                            Annotation::primary(first.span().join(last.span())).with_message("Hook modifiers here."),
                        )
                        .with_annotation(
                            Annotation::secondary(hooked_property.item.variable().span())
                                .with_message(format!("Property `{item_name}` is declared here.")),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{class_like_kind} `{class_like_fqcn}` defined here.")),
                        ),
                    );
                }

                if !class_like_is_interface && let PropertyHookBody::Abstract(property_hook_abstract_body) = &hook.body
                {
                    context.report(
                        Issue::error(format!("Non-abstract property hook `{name}` must have a body."))
                            .with_annotation(
                                Annotation::primary(property_hook_abstract_body.span())
                                    .with_message("Abstract hook body here."),
                            )
                            .with_annotation(
                                Annotation::secondary(hook.name.span())
                                    .with_message(format!("Hook `{name}` is declared here.")),
                            )
                            .with_annotation(
                                Annotation::secondary(hooked_property.item.variable().span())
                                    .with_message(format!("Property `{item_name}` is declared here.")),
                            )
                            .with_annotation(
                                Annotation::secondary(class_like_span)
                                    .with_message(format!("{class_like_kind} `{class_like_fqcn}` defined here.")),
                            ),
                    );
                }

                if let Some(parameter_list) = &hook.parameters {
                    check_for_promoted_properties_outside_constructor(parameter_list, context);

                    match lowered_name.as_str() {
                        "set" => {
                            if parameter_list.parameters.len() != 1 {
                                context.report(
                                    Issue::error(format!(
                                        "Hook `{}` of property `{}::{}` must accept exactly one parameter, found {}.",
                                        name,
                                        class_like_name,
                                        item_name,
                                        parameter_list.parameters.len()
                                    ))
                                    .with_annotation(
                                        Annotation::primary(parameter_list.span())
                                            .with_message("Parameters are defined here."),
                                    )
                                    .with_annotation(
                                        Annotation::secondary(hook.name.span())
                                            .with_message(format!("Hook `{name}` is declared here.")),
                                    )
                                    .with_annotation(
                                        Annotation::secondary(hooked_property.item.variable().span())
                                            .with_message(format!("Property `{item_name}` is declared here.")),
                                    )
                                    .with_annotation(
                                        Annotation::secondary(class_like_span).with_message(format!(
                                            "{class_like_kind} `{class_like_fqcn}` defined here."
                                        )),
                                    ),
                                );
                            } else {
                                let first_parameter = parameter_list.parameters.first().unwrap();
                                let first_parameter_name = first_parameter.variable.name;

                                if first_parameter.hint.is_none() {
                                    context.report(
                                        Issue::error(format!(
                                            "Parameter `{first_parameter_name}` of hook `{class_like_name}::{item_name}::{name}` must contain a type hint."
                                        ))
                                        .with_annotation(
                                            Annotation::primary(first_parameter.variable.span()).with_message(format!(
                                                "Parameter `{first_parameter_name}` declared here."
                                            )),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(hook.name.span())
                                                .with_message(format!("Hook `{name}` is declared here.")),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(hooked_property.item.variable().span())
                                                .with_message(format!("Property `{item_name}` is declared here.")),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(class_like_span).with_message(format!(
                                                "{class_like_kind} `{class_like_fqcn}` defined here."
                                            )),
                                        ),
                                    );
                                }

                                if let Some(ellipsis) = first_parameter.ellipsis {
                                    context.report(
                                        Issue::error(format!(
                                            "Parameter `{first_parameter_name}` of hook `{class_like_name}::{item_name}::{name}` must not be variadic."
                                        ))
                                        .with_annotation(Annotation::primary(ellipsis.span()).with_message(format!(
                                            "Parameter `{first_parameter_name}` is marked as variadic here."
                                        )))
                                        .with_annotation(
                                            Annotation::secondary(first_parameter.variable.span()).with_message(
                                                format!("Parameter `{first_parameter_name}` declared here."),
                                            ),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(hook.name.span())
                                                .with_message(format!("Hook `{name}` is declared here.")),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(hooked_property.item.variable().span())
                                                .with_message(format!("Property `{item_name}` is declared here.")),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(class_like_span).with_message(format!(
                                                "{class_like_kind} `{class_like_fqcn}` defined here."
                                            )),
                                        ),
                                    );
                                }

                                if let Some(ampersand) = first_parameter.ampersand {
                                    context.report(
                                        Issue::error(format!(
                                            "Parameter `{first_parameter_name}` of hook `{class_like_name}::{item_name}::{name}` must not be pass-by-reference."
                                        ))
                                        .with_annotation(Annotation::primary(ampersand.span()).with_message(format!(
                                            "Parameter `{first_parameter_name}` is marked as pass-by-reference here."
                                        )))
                                        .with_annotation(
                                            Annotation::secondary(first_parameter.variable.span()).with_message(
                                                format!("Parameter `{first_parameter_name}` declared here."),
                                            ),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(hook.name.span())
                                                .with_message(format!("Hook `{name}` is declared here.")),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(hooked_property.item.variable().span())
                                                .with_message(format!("Property `{item_name}` is declared here.")),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(class_like_span).with_message(format!(
                                                "{class_like_kind} `{class_like_fqcn}` defined here."
                                            )),
                                        ),
                                    );
                                }

                                if let Some(default_value) = &first_parameter.default_value {
                                    context.report(
                                        Issue::error(format!(
                                            "Parameter `{first_parameter_name}` of hook `{class_like_name}::{item_name}::{name}` must not have a default value."
                                        ))
                                        .with_annotation(Annotation::primary(default_value.span()))
                                        .with_annotation(
                                            Annotation::secondary(first_parameter.variable.span()).with_message(
                                                format!("Parameter `{first_parameter_name}` declared here."),
                                            ),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(hook.name.span())
                                                .with_message(format!("Hook `{name}` is declared here.")),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(hooked_property.item.variable().span())
                                                .with_message(format!("Property `{item_name}` is declared here.")),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(class_like_span).with_message(format!(
                                                "{class_like_kind} `{class_like_fqcn}` defined here."
                                            )),
                                        ),
                                    );
                                }
                            }
                        }
                        "get" => {
                            context.report(
                                Issue::error(format!(
                                    "Hook `{name}` of property `{class_like_name}::{item_name}` must not have a parameters list."
                                ))
                                .with_annotation(
                                    Annotation::primary(parameter_list.span())
                                        .with_message("Parameters are defined here."),
                                )
                                .with_annotation(
                                    Annotation::secondary(hook.name.span())
                                        .with_message(format!("Hook `{name}` is declared here.")),
                                )
                                .with_annotation(
                                    Annotation::secondary(hooked_property.item.variable().span())
                                        .with_message(format!("Property `{item_name}` is declared here.")),
                                )
                                .with_annotation(
                                    Annotation::secondary(class_like_span).with_message(format!(
                                        "{class_like_kind} `{class_like_fqcn}` defined here."
                                    )),
                                ),
                            );
                        }
                        _ => {}
                    }
                }

                if !lowered_name.as_str().eq("set") && !lowered_name.as_str().eq("get") {
                    context.report(
                        Issue::error(format!(
                            "Hooked property `{class_like_name}::{item_name}` contains an unknown hook `{name}`, expected `set` or `get`."
                        ))
                        .with_annotation(
                            Annotation::primary(hook.name.span())
                                .with_message(format!("Hook `{name}` declared here.")),
                        )
                        .with_annotation(
                            Annotation::secondary(hooked_property.item.variable().span())
                                .with_message(format!("Property `{item_name}` is declared here.")),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{class_like_kind} `{class_like_fqcn}` defined here.")),
                        ),
                    );
                }

                if let Some((_, previous_span)) = hook_names.iter().find(|(previous, _)| previous.eq(&lowered_name)) {
                    context.report(
                        Issue::error(format!(
                            "Hook `{name}` has already been defined for property `{class_like_name}::{item_name}`."
                        ))
                        .with_annotation(
                            Annotation::primary(hook.name.span()).with_message(format!("Duplicate hook `{name}`.")),
                        )
                        .with_annotation(
                            Annotation::secondary(*previous_span)
                                .with_message(format!("Previous declaration of hook `{previous_span}`")),
                        )
                        .with_annotation(
                            Annotation::secondary(hooked_property.item.variable().span())
                                .with_message(format!("Property `{item_name}` is declared here.")),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{class_like_kind} `{class_like_fqcn}` defined here.")),
                        ),
                    );
                } else {
                    hook_names.push((lowered_name, hook.name.span()));
                }
            }
        }
    };
}
