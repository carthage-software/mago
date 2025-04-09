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
    context: &mut Context<'_>,
) {
    let first_variable = property.first_variable();
    let first_variable_id = first_variable.name;
    let first_variable_name = context.interner.lookup(&first_variable_id);

    let modifiers = property.modifiers();
    let mut last_final: Option<Span> = None;
    let mut last_static: Option<Span> = None;
    let mut last_readonly: Option<Span> = None;
    let mut last_read_visibility: Option<Span> = None;
    let mut last_write_visibility: Option<Span> = None;

    for modifier in modifiers.iter() {
        match modifier {
            Modifier::Abstract(_) => {
                context.issues.push(
                    Issue::error(format!(
                        "Property `{}::{}` cannot be declared abstract",
                        class_like_name, first_variable_name
                    ))
                    .with_annotation(
                        Annotation::primary(modifier.span())
                            .with_message("`abstract` modifier cannot be used on properties"),
                    )
                    .with_annotation(
                        Annotation::secondary(first_variable.span())
                            .with_message(format!("Property `{}` declared here.", first_variable_name)),
                    )
                    .with_annotation(
                        Annotation::secondary(class_like_span)
                            .with_message(format!("{} `{}` defined here.", class_like_kind, class_like_fqcn)),
                    ),
                );
            }
            Modifier::Static(_) => {
                if let Some(last_readonly) = last_readonly {
                    context.issues.push(
                        Issue::error(format!(
                            "Readonly property `{}::{}` cannot be static.",
                            class_like_name, first_variable_name
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
                                .with_message(format!("Property `{}` declared here.", first_variable_name)),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` defined here.", class_like_kind, class_like_fqcn)),
                        ),
                    );
                }

                if let Some(last_static) = last_static {
                    context.issues.push(
                        Issue::error(format!(
                            "Property `{}::{}` has multiple `static` modifiers.",
                            class_like_name, first_variable_name
                        ))
                        .with_annotation(
                            Annotation::primary(modifier.span()).with_message("Duplicate `static` modifier."),
                        )
                        .with_annotation(Annotation::secondary(last_static).with_message("Previous `static` modifier."))
                        .with_annotation(
                            Annotation::secondary(first_variable.span())
                                .with_message(format!("Property `{}` declared here.", first_variable_name)),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` defined here.", class_like_kind, class_like_fqcn)),
                        ),
                    );
                }

                if let Some(last_visibility) = last_write_visibility {
                    context.issues.push(
                        Issue::error(format!(
                            "static property `{}::{}` cannot have a write visibility modifier.",
                            class_like_name, first_variable_name
                        ))
                        .with_annotation(
                            Annotation::primary(modifier.span()).with_message("Duplicate visibility modifier."),
                        )
                        .with_annotation(
                            Annotation::secondary(last_visibility).with_message("Previous visibility modifier."),
                        )
                        .with_annotation(
                            Annotation::secondary(first_variable.span())
                                .with_message(format!("Property `{}` declared here.", first_variable_name)),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` defined here.", class_like_kind, class_like_fqcn)),
                        ),
                    );
                }

                last_static = Some(modifier.span());
            }
            Modifier::Readonly(modifier) => {
                if !context.version.is_supported(Feature::ReadonlyProperties) {
                    context.issues.push(
                        Issue::error("Readonly properties are only available in PHP 8.1 and above.").with_annotation(
                            Annotation::primary(modifier.span()).with_message("Readonly modifier used here."),
                        ),
                    );

                    continue;
                }

                if let Some(last_static) = last_static {
                    context.issues.push(
                        Issue::error(format!(
                            "Static property `{}::{}` cannot be readonly.",
                            class_like_name, first_variable_name
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
                                .with_message(format!("Property `{}` declared here.", first_variable_name)),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` defined here.", class_like_kind, class_like_fqcn)),
                        ),
                    );
                }

                if let Some(last_readonly) = last_readonly {
                    context.issues.push(
                        Issue::error(format!(
                            "Property `{}::{}` has multiple `readonly` modifiers.",
                            class_like_name, first_variable_name
                        ))
                        .with_annotation(
                            Annotation::primary(modifier.span()).with_message("Duplicate `readonly` modifier."),
                        )
                        .with_annotation(
                            Annotation::secondary(last_readonly).with_message("Previous `readonly` modifier."),
                        )
                        .with_annotation(
                            Annotation::secondary(first_variable.span())
                                .with_message(format!("Property `{}` declared here.", first_variable_name)),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` defined here.", class_like_kind, class_like_fqcn)),
                        ),
                    );
                }

                last_readonly = Some(modifier.span());
            }
            Modifier::Final(_) => {
                if let Some(last_final) = last_final {
                    context.issues.push(
                        Issue::error("Property has multiple `final` modifiers.")
                            .with_annotation(
                                Annotation::primary(modifier.span()).with_message("Duplicate `final` modifier."),
                            )
                            .with_annotation(Annotation::primary(last_final).with_message("Previous `final` modifier."))
                            .with_annotation(
                                Annotation::secondary(first_variable.span())
                                    .with_message(format!("Property `{}` declared here.", first_variable_name)),
                            )
                            .with_annotation(
                                Annotation::secondary(class_like_span)
                                    .with_message(format!("{} `{}` defined here.", class_like_kind, class_like_fqcn)),
                            ),
                    );
                }

                last_final = Some(modifier.span());
            }
            Modifier::Private(_) | Modifier::Protected(_) | Modifier::Public(_) => {
                if let Some(last_visibility) = last_read_visibility {
                    context.issues.push(
                        Issue::error(format!(
                            "Property `{}::{}` has multiple visibility modifiers.",
                            class_like_name, first_variable_name
                        ))
                        .with_annotation(
                            Annotation::primary(modifier.span()).with_message("Duplicate visibility modifier."),
                        )
                        .with_annotation(
                            Annotation::primary(last_visibility).with_message("Previous visibility modifier."),
                        )
                        .with_annotation(
                            Annotation::secondary(first_variable.span())
                                .with_message(format!("Property `{}` declared here.", first_variable_name)),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` defined here.", class_like_kind, class_like_fqcn)),
                        ),
                    );
                }

                last_read_visibility = Some(modifier.span());
            }
            Modifier::PrivateSet(_) | Modifier::ProtectedSet(_) | Modifier::PublicSet(_) => {
                if let Some(last_visibility) = last_write_visibility {
                    context.issues.push(
                        Issue::error(format!(
                            "Property `{}::{}` has multiple write visibility modifiers.",
                            class_like_name, first_variable_name
                        ))
                        .with_annotation(
                            Annotation::primary(modifier.span()).with_message("Duplicate write visibility modifier."),
                        )
                        .with_annotation(
                            Annotation::primary(last_visibility).with_message("Previous write visibility modifier."),
                        )
                        .with_annotation(
                            Annotation::secondary(first_variable.span())
                                .with_message(format!("Property `{}` declared here.", first_variable_name)),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` defined here.", class_like_kind, class_like_fqcn)),
                        ),
                    );
                }

                if let Some(last_static) = last_static {
                    context.issues.push(
                        Issue::error(format!(
                            "Static property `{}::{}` cannot have a write visibility modifier.",
                            class_like_name, first_variable_name
                        ))
                        .with_annotation(
                            Annotation::primary(modifier.span()).with_message("Write visibility modifier."),
                        )
                        .with_annotation(
                            Annotation::primary(last_static).with_message("Property is marked as static here."),
                        )
                        .with_annotation(
                            Annotation::secondary(first_variable.span())
                                .with_message(format!("Property `{}` declared here.", first_variable_name)),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` defined here.", class_like_kind, class_like_fqcn)),
                        ),
                    );
                }

                last_write_visibility = Some(modifier.span());
            }
        }
    }

    if let Some(var) = property.var() {
        if !modifiers.is_empty() {
            let first = modifiers.first().unwrap();
            let last = modifiers.last().unwrap();

            context.issues.push(
                Issue::error(format!(
                    "Var property `{}::{}` cannot have modifiers.",
                    class_like_name, first_variable_name
                ))
                .with_annotation(
                    Annotation::primary(first.span().join(last.span())).with_message("Modifiers used here."),
                )
                .with_annotation(Annotation::primary(var.span()).with_message("Property is marked as `var` here."))
                .with_annotation(
                    Annotation::secondary(first_variable.span())
                        .with_message(format!("Property `{}` declared here.", first_variable_name)),
                )
                .with_annotation(
                    Annotation::secondary(class_like_span)
                        .with_message(format!("{} `{}` defined here.", class_like_kind, class_like_fqcn)),
                )
                .with_help("Remove either the `var` keyword, or the modifiers.".to_string()),
            );
        }
    }

    if let Some(hint) = property.hint() {
        if !context.version.is_supported(Feature::TypedProperties) {
            context.issues.push(
                Issue::error("Typed properties are only available in PHP 7.4 and above.")
                    .with_annotation(Annotation::primary(hint.span()).with_message("Type hint used here."))
                    .with_help("Remove the type hint to make the code compatible with PHP 7.3 and earlier versions, or upgrade to PHP 7.4 or later."),
            );
        }

        if !context.version.is_supported(Feature::NativeUnionTypes) && hint.is_union() {
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

        if hint.is_bottom() {
            let hint_name = context.get_code_snippet(hint);
            // cant be used on properties
            context.issues.push(
                Issue::error(format!(
                    "Property `{}::{}` cannot have type `{}`.",
                    class_like_name, first_variable_name, hint_name
                ))
                .with_annotation(
                    Annotation::primary(hint.span())
                        .with_message(format!("Type `{}` is not allowed on properties.", hint_name)),
                )
                .with_annotation(
                    Annotation::secondary(first_variable.span())
                        .with_message(format!("Property `{}` declared here.", first_variable_name)),
                )
                .with_annotation(
                    Annotation::secondary(class_like_span)
                        .with_message(format!("{} `{}` defined here.", class_like_kind, class_like_fqcn)),
                ),
            );
        }
    } else if let Some(readonly) = last_readonly {
        // readonly properties must have a type hint
        context.issues.push(
            Issue::error(format!(
                "Readonly property `{}::{}` must have a type hint.",
                class_like_name, first_variable_name
            ))
            .with_annotation(Annotation::primary(readonly).with_message("Property is marked as readonly here."))
            .with_annotation(
                Annotation::secondary(first_variable.span())
                    .with_message(format!("Property `{}` declared here.", first_variable_name)),
            )
            .with_annotation(
                Annotation::secondary(class_like_span)
                    .with_message(format!("{} `{}` defined here.", class_like_kind, class_like_fqcn)),
            ),
        );
    }

    match &property {
        Property::Plain(plain_property) => {
            if !context.version.is_supported(Feature::AsymmetricVisibility) {
                if let Some(write_visibility) = plain_property.modifiers.get_first_write_visibility() {
                    context.issues.push(
                        Issue::error("Asymmetric visibility is only available in PHP 8.4 and above.").with_annotation(
                            Annotation::primary(write_visibility.span())
                                .with_message("Asymmetric visibility used here."),
                        ),
                    );
                };
            }

            for item in plain_property.items.iter() {
                if let PropertyItem::Concrete(property_concrete_item) = &item {
                    let item_name_id = property_concrete_item.variable.name;
                    let item_name = context.interner.lookup(&item_name_id);

                    if !property_concrete_item.value.is_constant(context.version, false) {
                        context.issues.push(
                            Issue::error(format!(
                                "Property `{}::{}` value contains a non-constant expression.",
                                class_like_name, item_name
                            ))
                            .with_annotation(
                                Annotation::primary(property_concrete_item.value.span())
                                    .with_message("This is a non-constant expression."),
                            )
                            .with_annotation(
                                Annotation::secondary(property_concrete_item.variable.span())
                                    .with_message(format!("Property `{}` declared here.", item_name)),
                            )
                            .with_annotation(
                                Annotation::secondary(class_like_span)
                                    .with_message(format!("{} `{}` defined here.", class_like_kind, class_like_fqcn)),
                            ),
                        );
                    }

                    if let Some(readonly) = last_readonly {
                        context.issues.push(
                            Issue::error(format!(
                                "Readonly property `{}::{}` cannot have a default value.",
                                class_like_name, item_name
                            ))
                            .with_annotation(
                                Annotation::primary(property_concrete_item.value.span())
                                    .with_message("This is a default value."),
                            )
                            .with_annotation(Annotation::primary(readonly).with_message(format!(
                                "Property `{}::{}` is marked as readonly here.",
                                class_like_name, item_name
                            )))
                            .with_annotation(
                                Annotation::secondary(property_concrete_item.variable.span())
                                    .with_message(format!("Property `{}` is declared here.", item_name)),
                            )
                            .with_annotation(
                                Annotation::secondary(class_like_span)
                                    .with_message(format!("{} `{}` defined here.", class_like_kind, class_like_fqcn)),
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

                context.issues.push(issue);
            }

            let item_name_id = hooked_property.item.variable().name;
            let item_name = context.interner.lookup(&item_name_id);

            if let Some(readonly) = last_readonly {
                context.issues.push(
                    Issue::error(format!("Hooked property `{}::{}` cannot be readonly.", class_like_name, item_name))
                        .with_annotation(Annotation::primary(readonly).with_message(format!(
                            "Property `{}::{}` is marked as readonly here.",
                            class_like_name, item_name
                        )))
                        .with_annotation(
                            Annotation::secondary(hooked_property.hooks.span())
                                .with_message("Property hooks are defined here."),
                        )
                        .with_annotation(
                            Annotation::secondary(hooked_property.item.variable().span())
                                .with_message(format!("Property `{}` is declared here.", item_name)),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` defined here.", class_like_kind, class_like_fqcn)),
                        ),
                );
            }

            if let Some(r#static) = last_static {
                context.issues.push(
                    Issue::error(format!("Hooked property `{}::{}` cannot be static.", class_like_name, item_name))
                        .with_annotation(Annotation::primary(r#static).with_message(format!(
                            "Property `{}::{}` is marked as static here.",
                            class_like_name, item_name
                        )))
                        .with_annotation(
                            Annotation::secondary(hooked_property.hooks.span())
                                .with_message("Property hooks are defined here."),
                        )
                        .with_annotation(
                            Annotation::secondary(hooked_property.item.variable().span())
                                .with_message(format!("Property `{}` is declared here.", item_name)),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` defined here.", class_like_kind, class_like_fqcn)),
                        ),
                );
            }

            let mut hook_names: Vec<(std::string::String, Span)> = vec![];
            for hook in hooked_property.hooks.hooks.iter() {
                let name = context.interner.lookup(&hook.name.value);
                let lowered_name = name.to_ascii_lowercase();

                if !hook.modifiers.is_empty() {
                    let first = hook.modifiers.first().unwrap();
                    let last = hook.modifiers.last().unwrap();

                    context.issues.push(
                        Issue::error(format!(
                            "Hook `{}` for property `{}::{}` cannot have modifiers.",
                            name, class_like_name, item_name
                        ))
                        .with_annotation(
                            Annotation::primary(first.span().join(last.span())).with_message("Hook modifiers here."),
                        )
                        .with_annotation(
                            Annotation::secondary(hooked_property.item.variable().span())
                                .with_message(format!("Property `{}` is declared here.", item_name)),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` defined here.", class_like_kind, class_like_fqcn)),
                        ),
                    );
                }

                if !class_like_is_interface {
                    if let PropertyHookBody::Abstract(property_hook_abstract_body) = &hook.body {
                        context.issues.push(
                            Issue::error(format!("Non-abstract property hook `{}` must have a body.", name))
                                .with_annotation(
                                    Annotation::primary(property_hook_abstract_body.span())
                                        .with_message("Abstract hook body here."),
                                )
                                .with_annotation(
                                    Annotation::secondary(hook.name.span())
                                        .with_message(format!("Hook `{}` is declared here.", name)),
                                )
                                .with_annotation(
                                    Annotation::secondary(hooked_property.item.variable().span())
                                        .with_message(format!("Property `{}` is declared here.", item_name)),
                                )
                                .with_annotation(
                                    Annotation::secondary(class_like_span).with_message(format!(
                                        "{} `{}` defined here.",
                                        class_like_kind, class_like_fqcn
                                    )),
                                ),
                        );
                    }
                }

                if let Some(parameter_list) = &hook.parameters {
                    check_for_promoted_properties_outside_constructor(parameter_list, context);

                    match lowered_name.as_str() {
                        "set" => {
                            if parameter_list.parameters.len() != 1 {
                                context.issues.push(
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
                                            .with_message(format!("Hook `{}` is declared here.", name)),
                                    )
                                    .with_annotation(
                                        Annotation::secondary(hooked_property.item.variable().span())
                                            .with_message(format!("Property `{}` is declared here.", item_name)),
                                    )
                                    .with_annotation(
                                        Annotation::secondary(class_like_span).with_message(format!(
                                            "{} `{}` defined here.",
                                            class_like_kind, class_like_fqcn
                                        )),
                                    ),
                                );
                            } else {
                                let first_parameter = parameter_list.parameters.first().unwrap();
                                let first_parameter_name = context.interner.lookup(&first_parameter.variable.name);

                                if first_parameter.hint.is_none() {
                                    context.issues.push(
                                        Issue::error(format!(
                                            "Parameter `{}` of hook `{}::{}::{}` must contain a type hint.",
                                            first_parameter_name, class_like_name, item_name, name
                                        ))
                                        .with_annotation(
                                            Annotation::primary(first_parameter.variable.span()).with_message(format!(
                                                "Parameter `{}` declared here.",
                                                first_parameter_name
                                            )),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(hook.name.span())
                                                .with_message(format!("Hook `{}` is declared here.", name)),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(hooked_property.item.variable().span())
                                                .with_message(format!("Property `{}` is declared here.", item_name)),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(class_like_span).with_message(format!(
                                                "{} `{}` defined here.",
                                                class_like_kind, class_like_fqcn
                                            )),
                                        ),
                                    );
                                }

                                if let Some(ellipsis) = first_parameter.ellipsis {
                                    context.issues.push(
                                        Issue::error(format!(
                                            "Parameter `{}` of hook `{}::{}::{}` must not be variadic.",
                                            first_parameter_name, class_like_name, item_name, name
                                        ))
                                        .with_annotation(Annotation::primary(ellipsis.span()).with_message(format!(
                                            "Parameter `{}` is marked as variadic here.",
                                            first_parameter_name
                                        )))
                                        .with_annotation(
                                            Annotation::secondary(first_parameter.variable.span()).with_message(
                                                format!("Parameter `{}` declared here.", first_parameter_name),
                                            ),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(hook.name.span())
                                                .with_message(format!("Hook `{}` is declared here.", name)),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(hooked_property.item.variable().span())
                                                .with_message(format!("Property `{}` is declared here.", item_name)),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(class_like_span).with_message(format!(
                                                "{} `{}` defined here.",
                                                class_like_kind, class_like_fqcn
                                            )),
                                        ),
                                    );
                                }

                                if let Some(ampersand) = first_parameter.ampersand {
                                    context.issues.push(
                                        Issue::error(format!(
                                            "Parameter `{}` of hook `{}::{}::{}` must not be pass-by-reference.",
                                            first_parameter_name, class_like_name, item_name, name
                                        ))
                                        .with_annotation(Annotation::primary(ampersand.span()).with_message(format!(
                                            "Parameter `{}` is marked as pass-by-reference here.",
                                            first_parameter_name
                                        )))
                                        .with_annotation(
                                            Annotation::secondary(first_parameter.variable.span()).with_message(
                                                format!("Parameter `{}` declared here.", first_parameter_name),
                                            ),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(hook.name.span())
                                                .with_message(format!("Hook `{}` is declared here.", name)),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(hooked_property.item.variable().span())
                                                .with_message(format!("Property `{}` is declared here.", item_name)),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(class_like_span).with_message(format!(
                                                "{} `{}` defined here.",
                                                class_like_kind, class_like_fqcn
                                            )),
                                        ),
                                    );
                                }

                                if let Some(default_value) = &first_parameter.default_value {
                                    context.issues.push(
                                        Issue::error(format!(
                                            "Parameter `{}` of hook `{}::{}::{}` must not have a default value.",
                                            first_parameter_name, class_like_name, item_name, name
                                        ))
                                        .with_annotation(Annotation::primary(default_value.span()))
                                        .with_annotation(
                                            Annotation::secondary(first_parameter.variable.span()).with_message(
                                                format!("Parameter `{}` declared here.", first_parameter_name),
                                            ),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(hook.name.span())
                                                .with_message(format!("Hook `{}` is declared here.", name)),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(hooked_property.item.variable().span())
                                                .with_message(format!("Property `{}` is declared here.", item_name)),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(class_like_span).with_message(format!(
                                                "{} `{}` defined here.",
                                                class_like_kind, class_like_fqcn
                                            )),
                                        ),
                                    );
                                }
                            }
                        }
                        "get" => {
                            context.issues.push(
                                Issue::error(format!(
                                    "Hook `{}` of property `{}::{}` must not have a parameters list.",
                                    name, class_like_name, item_name
                                ))
                                .with_annotation(
                                    Annotation::primary(parameter_list.span())
                                        .with_message("Parameters are defined here."),
                                )
                                .with_annotation(
                                    Annotation::secondary(hook.name.span())
                                        .with_message(format!("Hook `{}` is declared here.", name)),
                                )
                                .with_annotation(
                                    Annotation::secondary(hooked_property.item.variable().span())
                                        .with_message(format!("Property `{}` is declared here.", item_name)),
                                )
                                .with_annotation(
                                    Annotation::secondary(class_like_span).with_message(format!(
                                        "{} `{}` defined here.",
                                        class_like_kind, class_like_fqcn
                                    )),
                                ),
                            );
                        }
                        _ => {}
                    }
                }

                if !lowered_name.as_str().eq("set") && !lowered_name.as_str().eq("get") {
                    context.issues.push(
                        Issue::error(format!(
                            "Hooked property `{}::{}` contains an unknwon hook `{}`, expected `set` or `get`.",
                            class_like_name, item_name, name
                        ))
                        .with_annotation(
                            Annotation::primary(hook.name.span())
                                .with_message(format!("Hook `{}` declared here.", name)),
                        )
                        .with_annotation(
                            Annotation::secondary(hooked_property.item.variable().span())
                                .with_message(format!("Property `{}` is declared here.", item_name)),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` defined here.", class_like_kind, class_like_fqcn)),
                        ),
                    );
                }

                if let Some((_, previous_span)) = hook_names.iter().find(|(previous, _)| previous.eq(&lowered_name)) {
                    context.issues.push(
                        Issue::error(format!(
                            "Hook `{}` has already been defined for property `{}::{}`.",
                            name, class_like_name, item_name
                        ))
                        .with_annotation(
                            Annotation::primary(hook.name.span()).with_message(format!("Duplicate hook `{}`.", name)),
                        )
                        .with_annotation(
                            Annotation::secondary(*previous_span)
                                .with_message(format!("Previous declaration of hook `{}`", previous_span)),
                        )
                        .with_annotation(
                            Annotation::secondary(hooked_property.item.variable().span())
                                .with_message(format!("Property `{}` is declared here.", item_name)),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` defined here.", class_like_kind, class_like_fqcn)),
                        ),
                    );
                } else {
                    hook_names.push((lowered_name, hook.name.span()));
                }
            }
        }
    };
}
