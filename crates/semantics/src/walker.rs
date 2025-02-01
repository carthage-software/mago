#![allow(clippy::too_many_arguments)]

use mago_ast::ast::*;
use mago_ast::*;
use mago_interner::StringIdentifier;
use mago_php_version::feature::Feature;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_span::Span;
use mago_walker::Walker;

use crate::consts::ANONYMOUS_CLASS_NAME;
use crate::consts::CALL_MAGIC_METHOD;
use crate::consts::CALL_STATIC_MAGIC_METHOD;
use crate::consts::CLONE_MAGIC_METHOD;
use crate::consts::CONSTRUCTOR_MAGIC_METHOD;
use crate::consts::DEBUG_INFO_MAGIC_METHOD;
use crate::consts::DECLARE_DIRECTIVES;
use crate::consts::DESTRUCTOR_MAGIC_METHOD;
use crate::consts::ENCODING_DECLARE_DIRECTIVE;
use crate::consts::GET_MAGIC_METHOD;
use crate::consts::INVOKE_MAGIC_METHOD;
use crate::consts::ISSET_MAGIC_METHOD;
use crate::consts::MAGIC_METHODS;
use crate::consts::RESERVED_KEYWORDS;
use crate::consts::SERIALIZE_MAGIC_METHOD;
use crate::consts::SET_MAGIC_METHOD;
use crate::consts::SET_STATE_MAGIC_METHOD;
use crate::consts::SLEEP_MAGIC_METHOD;
use crate::consts::SOFT_RESERVED_KEYWORDS_MINUS_SYMBOL_ALLOWED;
use crate::consts::STRICT_TYPES_DECLARE_DIRECTIVE;
use crate::consts::TICKS_DECLARE_DIRECTIVE;
use crate::consts::TO_STRING_MAGIC_METHOD;
use crate::consts::UNSERIALIZE_MAGIC_METHOD;
use crate::consts::UNSET_MAGIC_METHOD;
use crate::consts::WAKEUP_MAGIC_METHOD;
use crate::context::Context;

#[derive(Clone, Debug)]
pub struct SemanticsWalker;

impl SemanticsWalker {
    #[inline]
    fn process_extends(
        &self,
        extends: &Extends,
        class_like_span: Span,
        class_like_kind: &str,
        class_like_name: &str,
        class_like_fqcn: &str,
        extension_limit: bool,
        context: &mut Context<'_>,
    ) {
        if extension_limit && extends.types.len() > 1 {
            context.report(
                Issue::error(format!(
                    "{} `{}` can only extend one other type, found {}.",
                    class_like_kind,
                    class_like_name,
                    extends.types.len()
                ))
                .with_annotation(Annotation::primary(extends.span()).with_message("Multiple extensions found here."))
                .with_annotation(
                    Annotation::secondary(class_like_span)
                        .with_message(format!("{} `{}` declared here.", class_like_kind, class_like_fqcn)),
                )
                .with_help("Remove the extra extensions to ensure only one type is extended."),
            );
        }

        for extended_type in extends.types.iter() {
            let extended_fqcn = context.lookup_name(&extended_type.span().start);

            if extended_fqcn.eq_ignore_ascii_case(class_like_fqcn) {
                context.report(
                    Issue::error(format!("{} `{}` cannot extend itself.", class_like_kind, class_like_name))
                        .with_annotation(
                            Annotation::primary(extended_type.span()).with_message(format!(
                                "{} `{}` extends itself here.",
                                class_like_kind, class_like_name
                            )),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` declared here.", class_like_kind, class_like_fqcn)),
                        )
                        .with_help("Remove the self-referencing extension."),
                );
            }
        }

        for extended_type in extends.types.iter() {
            let extended_name = context.interner.lookup(&extended_type.value());

            if RESERVED_KEYWORDS.iter().any(|keyword| keyword.eq_ignore_ascii_case(extended_name))
                || SOFT_RESERVED_KEYWORDS_MINUS_SYMBOL_ALLOWED
                    .iter()
                    .any(|keyword| keyword.eq_ignore_ascii_case(extended_name))
            {
                context.report(
                    Issue::error(format!(
                        "{} `{}` cannot extend reserved keyword `{}`.",
                        class_like_kind, class_like_name, extended_name
                    ))
                    .with_annotation(
                        Annotation::primary(extended_type.span()).with_message("Extension uses a reserved keyword."),
                    )
                    .with_annotation(
                        Annotation::secondary(class_like_span)
                            .with_message(format!("{} `{}` declared here.", class_like_kind, class_like_name)),
                    )
                    .with_help(format!(
                        "Change the extended type to a valid identifier. `{}` is a reserved keyword.",
                        extended_name
                    )),
                );
            }
        }
    }

    #[inline]
    fn process_implements(
        &self,
        implements: &Implements,
        class_like_span: Span,
        class_like_kind: &str,
        class_like_name: &str,
        class_like_fqcn: &str,
        check_for_self_implement: bool,
        context: &mut Context<'_>,
    ) {
        if check_for_self_implement {
            for implemented_type in implements.types.iter() {
                let implemented_fqcn = context.lookup_name(&implemented_type.span().start);

                if implemented_fqcn.eq_ignore_ascii_case(class_like_fqcn) {
                    context.report(
                        Issue::error(format!("{} `{}` cannot implement itself.", class_like_kind, class_like_name))
                            .with_annotation(Annotation::primary(implemented_type.span()).with_message(format!(
                                "{} `{}` implements itself here.",
                                class_like_kind, class_like_name
                            )))
                            .with_annotation(
                                Annotation::secondary(class_like_span)
                                    .with_message(format!("{} `{}` declared here.", class_like_kind, class_like_fqcn)),
                            )
                            .with_help("Remove the self-referencing implementation."),
                    );
                }
            }
        }

        for implemented_type in implements.types.iter() {
            let implemented_name = context.interner.lookup(&implemented_type.value());

            if RESERVED_KEYWORDS.iter().any(|keyword| keyword.eq_ignore_ascii_case(implemented_name))
                || SOFT_RESERVED_KEYWORDS_MINUS_SYMBOL_ALLOWED
                    .iter()
                    .any(|keyword| keyword.eq_ignore_ascii_case(implemented_name))
            {
                context.report(
                    Issue::error(format!(
                        "{} `{}` cannot implement reserved keyword `{}`.",
                        class_like_kind, class_like_name, implemented_name
                    ))
                    .with_annotation(
                        Annotation::primary(implemented_type.span()).with_message("This is a reserved keyword."),
                    )
                    .with_annotation(
                        Annotation::secondary(class_like_span)
                            .with_message(format!("{} `{}` declared here.", class_like_kind, class_like_name)),
                    )
                    .with_help(format!(
                        "Replace `{}` with a valid identifier. Reserved keywords cannot be used as implemented types.",
                        implemented_name
                    )),
                );
            }
        }
    }

    #[inline]
    fn process_property(
        &self,
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
                    context.report(
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
                        context.report(
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
                        context.report(
                            Issue::error(format!(
                                "Property `{}::{}` has multiple `static` modifiers.",
                                class_like_name, first_variable_name
                            ))
                            .with_annotation(
                                Annotation::primary(modifier.span()).with_message("Duplicate `static` modifier."),
                            )
                            .with_annotation(
                                Annotation::secondary(last_static).with_message("Previous `static` modifier."),
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

                    if let Some(last_visibility) = last_write_visibility {
                        context.report(
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
                Modifier::Readonly(_) => {
                    if let Some(last_static) = last_static {
                        context.report(
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
                        context.report(
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
                        context.report(
                            Issue::error("Property has multiple `final` modifiers.")
                                .with_annotation(
                                    Annotation::primary(modifier.span()).with_message("Duplicate `final` modifier."),
                                )
                                .with_annotation(
                                    Annotation::primary(last_final).with_message("Previous `final` modifier."),
                                )
                                .with_annotation(
                                    Annotation::secondary(first_variable.span())
                                        .with_message(format!("Property `{}` declared here.", first_variable_name)),
                                )
                                .with_annotation(
                                    Annotation::secondary(class_like_span).with_message(format!(
                                        "{} `{}` defined here.",
                                        class_like_kind, class_like_fqcn
                                    )),
                                ),
                        );
                    }

                    last_final = Some(modifier.span());
                }
                Modifier::Private(_) | Modifier::Protected(_) | Modifier::Public(_) => {
                    if let Some(last_visibility) = last_read_visibility {
                        context.report(
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
                        context.report(
                            Issue::error(format!(
                                "Property `{}::{}` has multiple write visibility modifiers.",
                                class_like_name, first_variable_name
                            ))
                            .with_annotation(
                                Annotation::primary(modifier.span())
                                    .with_message("Duplicate write visibility modifier."),
                            )
                            .with_annotation(
                                Annotation::primary(last_visibility)
                                    .with_message("Previous write visibility modifier."),
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
                        context.report(
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

                context.report(
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
            if hint.is_bottom() {
                let hint_name = context.lookup_hint(hint);
                // cant be used on properties
                context.report(
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
            context.report(
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
                for item in plain_property.items.iter() {
                    if let PropertyItem::Concrete(property_concrete_item) = &item {
                        let item_name_id = property_concrete_item.variable.name;
                        let item_name = context.interner.lookup(&item_name_id);

                        if !property_concrete_item.value.is_constant(context.version, false) {
                            context.report(
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
                                    Annotation::secondary(class_like_span).with_message(format!(
                                        "{} `{}` defined here.",
                                        class_like_kind, class_like_fqcn
                                    )),
                                ),
                            );
                        }

                        if let Some(readonly) = last_readonly {
                            context.report(
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
                                    Annotation::secondary(class_like_span).with_message(format!(
                                        "{} `{}` defined here.",
                                        class_like_kind, class_like_fqcn
                                    )),
                                ),
                            );
                        }
                    }
                }
            }
            Property::Hooked(hooked_property) => {
                let item_name_id = hooked_property.item.variable().name;
                let item_name = context.interner.lookup(&item_name_id);

                if let Some(readonly) = last_readonly {
                    context.report(
                        Issue::error(format!(
                            "Hooked property `{}::{}` cannot be readonly.",
                            class_like_name, item_name
                        ))
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
                    context.report(
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

                        context.report(
                            Issue::error(format!(
                                "Hook `{}` for property `{}::{}` cannot have modifiers.",
                                name, class_like_name, item_name
                            ))
                            .with_annotation(
                                Annotation::primary(first.span().join(last.span()))
                                    .with_message("Hook modifiers here."),
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
                            context.report(
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
                                    .with_annotation(Annotation::secondary(class_like_span).with_message(format!(
                                        "{} `{}` defined here.",
                                        class_like_kind, class_like_fqcn
                                    ))),
                            );
                        }
                    }

                    match lowered_name.as_str() {
                        "set" => {
                            if let Some(parameters) = &hook.parameters {
                                if parameters.parameters.len() != 1 {
                                    context.report(
                                        Issue::error(format!(
                                            "Hook `{}` of property `{}::{}` must accept exactly one parameter, found {}.",
                                            name, class_like_name, item_name, parameters.parameters.len()
                                        ))
                                        .with_annotation(Annotation::primary(parameters.span()).with_message("Parameters are defined here."))
                                        .with_annotation(
                                            Annotation::secondary(hook.name.span()).with_message(
                                                format!("Hook `{}` is declared here.", name),
                                            ),
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
                                    let first_parameter = parameters.parameters.first().unwrap();
                                    let first_parameter_name = context.interner.lookup(&first_parameter.variable.name);

                                    if first_parameter.hint.is_none() {
                                        context.report(
                                            Issue::error(format!(
                                                "Parameter `{}` of hook `{}::{}::{}` must contain a type hint.",
                                                first_parameter_name, class_like_name, item_name, name
                                            ))
                                            .with_annotation(
                                                Annotation::primary(first_parameter.variable.span()).with_message(
                                                    format!("Parameter `{}` declared here.", first_parameter_name),
                                                ),
                                            )
                                            .with_annotation(
                                                Annotation::secondary(hook.name.span())
                                                    .with_message(format!("Hook `{}` is declared here.", name)),
                                            )
                                            .with_annotation(
                                                Annotation::secondary(hooked_property.item.variable().span())
                                                    .with_message(format!(
                                                        "Property `{}` is declared here.",
                                                        item_name
                                                    )),
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
                                        context.report(
                                            Issue::error(format!(
                                                "Parameter `{}` of hook `{}::{}::{}` must not be variadic.",
                                                first_parameter_name, class_like_name, item_name, name
                                            ))
                                            .with_annotation(Annotation::primary(ellipsis.span()).with_message(
                                                format!(
                                                    "Parameter `{}` is marked as variadic here.",
                                                    first_parameter_name
                                                ),
                                            ))
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
                                                    .with_message(format!(
                                                        "Property `{}` is declared here.",
                                                        item_name
                                                    )),
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
                                        context.report(
                                            Issue::error(format!(
                                                "Parameter `{}` of hook `{}::{}::{}` must not be pass-by-reference.",
                                                first_parameter_name, class_like_name, item_name, name
                                            ))
                                            .with_annotation(Annotation::primary(ampersand.span()).with_message(
                                                format!(
                                                    "Parameter `{}` is marked as pass-by-reference here.",
                                                    first_parameter_name
                                                ),
                                            ))
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
                                                    .with_message(format!(
                                                        "Property `{}` is declared here.",
                                                        item_name
                                                    )),
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
                                        context.report(
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
                                                    .with_message(format!(
                                                        "Property `{}` is declared here.",
                                                        item_name
                                                    )),
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
                        }
                        "get" => {
                            if let Some(parameters) = &hook.parameters {
                                context.report(
                                    Issue::error(format!(
                                        "Hook `{}` of property `{}::{}` must not have a parameters list.",
                                        name, class_like_name, item_name
                                    ))
                                    .with_annotation(
                                        Annotation::primary(parameters.span())
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
                        }
                        _ => {
                            context.report(
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
                                    Annotation::secondary(class_like_span).with_message(format!(
                                        "{} `{}` defined here.",
                                        class_like_kind, class_like_fqcn
                                    )),
                                ),
                            );
                        }
                    }

                    if let Some((_, previous_span)) = hook_names.iter().find(|(previous, _)| previous.eq(&lowered_name))
                    {
                        context.report(
                            Issue::error(format!(
                                "Hook `{}` has already been defined for property `{}::{}`.",
                                name, class_like_name, item_name
                            ))
                            .with_annotation(
                                Annotation::primary(hook.name.span())
                                    .with_message(format!("Duplicate hook `{}`.", name)),
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

    #[inline]
    fn process_method(
        &self,
        method: &Method,
        method_name: &str,
        class_like_span: Span,
        class_like_name: &str,
        class_like_fqcn: &str,
        class_like_kind: &str,
        class_like_is_interface: bool,
        context: &mut Context<'_>,
    ) {
        let mut last_static: Option<Span> = None;
        let mut last_final: Option<Span> = None;
        let mut last_abstract: Option<Span> = None;
        let mut last_visibility: Option<Span> = None;
        let mut is_public = true;
        for modifier in method.modifiers.iter() {
            match modifier {
                Modifier::Static(_) => {
                    if let Some(last_static) = last_static {
                        context.report(
                            Issue::error(format!(
                                "duplicate `static` modifier on method `{}::{}`",
                                class_like_name, method_name
                            ))
                            .with_annotation(
                                Annotation::primary(modifier.span()).with_message("duplicate `static` modifier"),
                            )
                            .with_annotation(
                                Annotation::primary(last_static).with_message("previous `static` modifier"),
                            )
                            .with_annotation(
                                Annotation::secondary(method.span()).with_message(format!(
                                    "method `{}::{}` defined here.",
                                    class_like_name, method_name,
                                )),
                            )
                            .with_annotation(
                                Annotation::secondary(class_like_span).with_message(format!(
                                    "{} `{}` is defined here.",
                                    class_like_kind, class_like_fqcn
                                )),
                            ),
                        );
                    }

                    last_static = Some(modifier.span());
                }
                Modifier::Final(_) => {
                    if let Some(abstract_modifier) = last_abstract {
                        context.report(
                            Issue::error(format!(
                                "method `{}::{}` cannot be both `final` and `abstract`",
                                class_like_name, method_name
                            ))
                            .with_annotation(Annotation::primary(modifier.span()).with_message("`final` modifier"))
                            .with_annotation(Annotation::primary(abstract_modifier).with_message("`abstract` modifier"))
                            .with_annotation(
                                Annotation::secondary(method.span()).with_message(format!(
                                    "method `{}::{}` defined here.",
                                    class_like_name, method_name,
                                )),
                            )
                            .with_annotation(
                                Annotation::secondary(class_like_span).with_message(format!(
                                    "{} `{}` is defined here.",
                                    class_like_kind, class_like_fqcn
                                )),
                            ),
                        );
                    }

                    if let Some(last_final) = last_final {
                        context.report(
                            Issue::error(format!(
                                "duplicate `final` modifier on method `{}::{}`",
                                class_like_name, method_name
                            ))
                            .with_annotation(
                                Annotation::primary(modifier.span()).with_message("duplicate `final` modifier"),
                            )
                            .with_annotation(Annotation::primary(last_final).with_message("previous `final` modifier"))
                            .with_annotation(
                                Annotation::secondary(method.span()).with_message(format!(
                                    "method `{}::{}` defined here.",
                                    class_like_name, method_name,
                                )),
                            )
                            .with_annotation(
                                Annotation::secondary(class_like_span).with_message(format!(
                                    "{} `{}` is defined here.",
                                    class_like_kind, class_like_fqcn
                                )),
                            ),
                        );
                    }

                    last_final = Some(modifier.span());
                }
                Modifier::Abstract(_) => {
                    if let Some(final_modifier) = last_final {
                        context.report(
                            Issue::error(format!(
                                "method `{}::{}` cannot be both `final` and `abstract`",
                                class_like_name, method_name
                            ))
                            .with_annotation(Annotation::primary(modifier.span()).with_message("`abstract` modifier"))
                            .with_annotation(Annotation::primary(final_modifier).with_message("`final` modifier"))
                            .with_annotation(
                                Annotation::secondary(method.span()).with_message(format!(
                                    "method `{}::{}` defined here.",
                                    class_like_name, method_name,
                                )),
                            )
                            .with_annotation(
                                Annotation::secondary(class_like_span).with_message(format!(
                                    "{} `{}` is defined here.",
                                    class_like_kind, class_like_fqcn
                                )),
                            ),
                        );
                    }

                    if let Some(last_abstract) = last_abstract {
                        context.report(
                            Issue::error(format!(
                                "duplicate `abstract` modifier on method `{}::{}`",
                                class_like_name, method_name
                            ))
                            .with_annotation(
                                Annotation::primary(modifier.span()).with_message("duplicate `abstract` modifier"),
                            )
                            .with_annotation(
                                Annotation::primary(last_abstract).with_message("previous `abstract` modifier"),
                            )
                            .with_annotation(
                                Annotation::secondary(method.span()).with_message(format!(
                                    "method `{}::{}` defined here.",
                                    class_like_name, method_name,
                                )),
                            )
                            .with_annotation(
                                Annotation::secondary(class_like_span).with_message(format!(
                                    "{} `{}` is defined here.",
                                    class_like_kind, class_like_fqcn
                                )),
                            ),
                        );
                    }

                    last_abstract = Some(modifier.span());
                }
                Modifier::Readonly(_) => {
                    context.report(
                        Issue::error("`readonly` modifier is not allowed on methods".to_string())
                            .with_annotation(Annotation::primary(modifier.span()).with_message("`readonly` modifier"))
                            .with_annotation(
                                Annotation::secondary(method.span()).with_message(format!(
                                    "method `{}::{}` defined here.",
                                    class_like_name, method_name,
                                )),
                            )
                            .with_annotation(
                                Annotation::secondary(class_like_span).with_message(format!(
                                    "{} `{}` is defined here.",
                                    class_like_kind, class_like_fqcn
                                )),
                            ),
                    );
                }
                Modifier::Private(_) | Modifier::Protected(_) | Modifier::Public(_) => {
                    if let Some(last_visibility) = last_visibility {
                        context.report(
                            Issue::error(format!(
                                "duplicate visibility modifier on method `{}::{}`",
                                class_like_name, method_name
                            ))
                            .with_annotation(
                                Annotation::primary(modifier.span()).with_message("duplicate visibility modifier"),
                            )
                            .with_annotation(
                                Annotation::primary(last_visibility).with_message("previous visibility modifier"),
                            )
                            .with_annotation(
                                Annotation::secondary(method.span()).with_message(format!(
                                    "method `{}::{}` defined here.",
                                    class_like_name, method_name,
                                )),
                            )
                            .with_annotation(
                                Annotation::secondary(class_like_span).with_message(format!(
                                    "{} `{}` is defined here.",
                                    class_like_kind, class_like_fqcn
                                )),
                            ),
                        );
                    } else {
                        if !matches!(modifier, Modifier::Public(_)) {
                            is_public = false;
                        }

                        last_visibility = Some(modifier.span());
                    }
                }
                Modifier::PrivateSet(k) | Modifier::ProtectedSet(k) | Modifier::PublicSet(k) => {
                    let modifier_name = context.interner.lookup(&k.value);

                    context.report(
                        Issue::error(format!("`{}` modifier is not allowed on methods", modifier_name))
                            .with_annotation(
                                Annotation::primary(modifier.span())
                                    .with_message(format!("`{}` modifier", modifier_name)),
                            )
                            .with_annotation(
                                Annotation::secondary(method.span()).with_message(format!(
                                    "method `{}::{}` defined here.",
                                    class_like_name, method_name,
                                )),
                            )
                            .with_annotation(
                                Annotation::secondary(class_like_span).with_message(format!(
                                    "{} `{}` is defined here.",
                                    class_like_kind, class_like_fqcn
                                )),
                            ),
                    );
                }
            }
        }

        for (magic_method, parameter_count, must_be_public, must_be_static, can_have_return_type) in
            MAGIC_METHOD_SEMANTICS
        {
            if method_name.eq_ignore_ascii_case(magic_method) {
                if let Some(count) = parameter_count {
                    let mut found_count = 0;
                    let mut found_variadic = false;
                    for param in method.parameter_list.parameters.iter() {
                        found_count += 1;

                        if param.ellipsis.is_some() {
                            found_variadic = true;
                        }
                    }

                    if found_variadic || found_count.ne(count) {
                        let message = if found_variadic {
                            format!(
                                "magic method `{}::{}` must have exactly {} parameters, found more than {} due to variadic parameter",
                                class_like_name,
                                method_name,
                                count,
                                found_count
                            )
                        } else {
                            format!(
                                "magic method `{}::{}` must have exactly {} parameters, found {}",
                                class_like_name, method_name, count, found_count
                            )
                        };

                        context.report(
                            Issue::error(message)
                                .with_annotation(Annotation::primary(method.parameter_list.span()))
                                .with_annotation(Annotation::secondary(method.span()).with_message(format!(
                                    "method `{}::{}` defined here.",
                                    class_like_name, method_name,
                                )))
                                .with_annotation(Annotation::secondary(class_like_span).with_message(format!(
                                    "{} `{}` is defined here.",
                                    class_like_kind, class_like_fqcn
                                ))),
                        );
                    }
                }

                if *must_be_public && !is_public {
                    context.report(
                        Issue::error(format!("magic method `{}::{}` must be public", class_like_name, method_name))
                            .with_annotation(
                                Annotation::primary(last_visibility.unwrap())
                                    .with_message("non-public visibility modifier"),
                            )
                            .with_annotation(
                                Annotation::secondary(method.span()).with_message(format!(
                                    "method `{}::{}` defined here.",
                                    class_like_name, method_name,
                                )),
                            )
                            .with_annotation(
                                Annotation::secondary(class_like_span).with_message(format!(
                                    "{} `{}` is defined here.",
                                    class_like_kind, class_like_fqcn
                                )),
                            ),
                    );
                }

                match last_static.as_ref() {
                    Some(span) if !*must_be_static => {
                        context.report(
                            Issue::error(format!(
                                "magic method `{}::{}` cannot be static",
                                class_like_name, method_name
                            ))
                            .with_annotation(Annotation::primary(*span).with_message("`static` modifier"))
                            .with_annotation(
                                Annotation::secondary(method.span()).with_message(format!(
                                    "method `{}::{}` defined here.",
                                    class_like_name, method_name,
                                )),
                            )
                            .with_annotation(
                                Annotation::secondary(class_like_span).with_message(format!(
                                    "{} `{}` is defined here.",
                                    class_like_kind, class_like_fqcn
                                )),
                            ),
                        );
                    }
                    None if *must_be_static => {
                        context.report(
                            Issue::error(format!("magic method `{}::{}` must be static", class_like_name, method_name))
                                .with_annotation(Annotation::primary(method.name.span()))
                                .with_annotation(
                                    Annotation::secondary(class_like_span)
                                        .with_message(format!("{} `{}`", class_like_kind, class_like_fqcn)),
                                )
                                .with_annotation(Annotation::secondary(method.span()).with_message(format!(
                                    "method `{}::{}` defined here.",
                                    class_like_name, method_name,
                                ))),
                        );
                    }
                    _ => {}
                }

                if !*can_have_return_type {
                    if let Some(hint) = &method.return_type_hint {
                        context.report(
                            Issue::error(format!(
                                "magic method `{}::{}` cannot have a return type hint",
                                class_like_name, method_name
                            ))
                            .with_annotation(Annotation::primary(hint.span()))
                            .with_annotation(
                                Annotation::secondary(method.span()).with_message(format!(
                                    "method `{}::{}` defined here.",
                                    class_like_name, method_name,
                                )),
                            )
                            .with_annotation(
                                Annotation::secondary(class_like_span).with_message(format!(
                                    "{} `{}` is defined here.",
                                    class_like_kind, class_like_fqcn
                                )),
                            ),
                        );
                    }
                }
            }
        }

        match &method.body {
            MethodBody::Abstract(method_abstract_body) => {
                if !class_like_is_interface && !method.modifiers.contains_abstract() {
                    context.report(
                        Issue::error(format!(
                            "non-abstract method `{}::{}` must have a concrete body",
                            class_like_name, method_name,
                        ))
                        .with_annotation(Annotation::primary(method_abstract_body.span()))
                        .with_annotations([
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` is defined here.", class_like_kind, class_like_fqcn)),
                            Annotation::secondary(method.span())
                                .with_message(format!("method `{}::{}` defined here.", class_like_name, method_name)),
                        ]),
                    );
                }
            }
            MethodBody::Concrete(body) => {
                if let Some(abstract_modifier) = method.modifiers.get_abstract() {
                    context.report(
                        Issue::error(format!(
                            "method `{}::{}` is abstract and cannot have a concrete body",
                            class_like_name, method_name,
                        ))
                        .with_annotation(Annotation::primary(body.span()))
                        .with_annotations([
                            Annotation::primary(abstract_modifier.span()),
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` is defined here.", class_like_kind, class_like_fqcn)),
                            Annotation::secondary(method.span())
                                .with_message(format!("method `{}::{}` defined here.", class_like_name, method_name)),
                        ]),
                    );
                } else if class_like_is_interface {
                    context.report(
                        Issue::error(format!(
                            "interface method `{}::{}` is implicitly abstract and cannot have a concrete body",
                            class_like_name, method_name,
                        ))
                        .with_annotation(Annotation::primary(body.span()))
                        .with_annotations([
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` is defined here.", class_like_kind, class_like_fqcn)),
                            Annotation::secondary(method.span())
                                .with_message(format!("method `{}::{}` defined here.", class_like_name, method_name)),
                        ]),
                    );
                }

                let hint = if let Some(return_hint) = &method.return_type_hint {
                    &return_hint.hint
                } else {
                    return;
                };

                let returns = mago_ast_utils::find_returns_in_block(body);

                match &hint {
                    Hint::Void(_) => {
                        for r#return in returns {
                            if let Some(val) = &r#return.value {
                                context.report(
                                    Issue::error(format!(
                                        "method `{}::{}` with return type of `void` must not return a value",
                                        class_like_name, method_name,
                                    ))
                                    .with_annotation(Annotation::primary(val.span()))
                                    .with_annotations([
                                        Annotation::secondary(class_like_span).with_message(format!(
                                            "{} `{}` is defined here.",
                                            class_like_kind, class_like_fqcn
                                        )),
                                        Annotation::secondary(method.span()).with_message(format!(
                                            "method `{}::{}` defined here.",
                                            class_like_name, method_name,
                                        )),
                                    ])
                                    .with_help("remove the return type hint, or remove the return value"),
                                );
                            }
                        }
                    }
                    Hint::Never(_) => {
                        for r#return in returns {
                            context.report(
                                Issue::error(format!(
                                    "function `{}::{}` with return type of `never` must not return",
                                    class_like_name, method_name,
                                ))
                                .with_annotation(Annotation::primary(r#return.span()))
                                .with_annotations([
                                    Annotation::secondary(class_like_span).with_message(format!(
                                        "{} `{}` is defined here.",
                                        class_like_kind, class_like_fqcn
                                    )),
                                    Annotation::secondary(method.span()).with_message(format!(
                                        "method `{}::{}` defined here.",
                                        class_like_name, method_name,
                                    )),
                                ])
                                .with_help("remove the return type hint, or remove the return statement"),
                            );
                        }
                    }
                    _ if !returns_generator(context, body, hint) => {
                        for r#return in returns {
                            if r#return.value.is_none() {
                                context.report(
                                    Issue::error(format!(
                                        "method `{}::{}` with return type must return a value",
                                        class_like_name, method_name,
                                    ))
                                    .with_annotation(Annotation::primary(r#return.span()))
                                    .with_annotations([
                                        Annotation::secondary(class_like_span).with_message(format!(
                                            "{} `{}` is defined here.",
                                            class_like_kind, class_like_fqcn
                                        )),
                                        Annotation::secondary(method.span()).with_message(format!(
                                            "method `{}::{}` defined here.",
                                            class_like_name, method_name,
                                        )),
                                    ])
                                    .with_note("did you mean `return null;` instead of `return;`?")
                                    .with_help("add a return value to the statement"),
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        };
    }

    #[inline]
    fn process_members(
        &self,
        members: &Sequence<ClassLikeMember>,
        class_like_span: Span,
        class_like_kind: &str,
        class_like_name: &str,
        class_like_fqcn: &str,
        context: &mut Context<'_>,
    ) {
        let mut method_names: Vec<(Span, StringIdentifier)> = vec![];
        let mut constant_names: Vec<(bool, std::string::String, Span)> = vec![];
        let mut property_names: Vec<(bool, StringIdentifier, Span)> = vec![];

        for member in members.iter() {
            match &member {
                ClassLikeMember::Property(property) => match &property {
                    Property::Plain(plain_property) => {
                        for item in plain_property.items.iter() {
                            let item_name_id = item.variable().name;
                            let item_name = context.interner.lookup(&item_name_id);

                            if let Some((is_promoted, _, span)) =
                                property_names.iter().find(|(_, name, _)| item_name_id.eq(name))
                            {
                                let message = if *is_promoted {
                                    format!(
                                        "property `{}::{}` has already been defined as a promoted property",
                                        class_like_name, item_name
                                    )
                                } else {
                                    format!("property `{}::{}` has already been defined", class_like_name, item_name)
                                };

                                context.report(
                                    Issue::error(message)
                                        .with_annotation(Annotation::primary(item.variable().span()))
                                        .with_annotations([
                                            Annotation::secondary(*span).with_message(format!(
                                                "property `{}::{}` previously defined here.",
                                                class_like_name, item_name
                                            )),
                                            Annotation::secondary(class_like_span.span()).with_message(format!(
                                                "{} `{}` defined here.",
                                                class_like_kind, class_like_fqcn
                                            )),
                                        ])
                                        .with_help("remove the duplicate property"),
                                );
                            } else {
                                property_names.push((false, item_name_id, item.variable().span()));
                            }
                        }
                    }
                    Property::Hooked(hooked_property) => {
                        let item_variable = hooked_property.item.variable();
                        let item_name_id = item_variable.name;
                        let item_name = context.interner.lookup(&item_name_id);

                        if let Some((is_promoted, _, span)) =
                            property_names.iter().find(|(_, name, _)| item_name_id.eq(name))
                        {
                            let message = if *is_promoted {
                                format!(
                                    "property `{}::{}` has already been defined as a promoted property",
                                    class_like_name, item_name
                                )
                            } else {
                                format!("property `{}::{}` has already been defined", class_like_name, item_name)
                            };

                            context.report(
                                Issue::error(message)
                                    .with_annotation(Annotation::primary(item_variable.span()))
                                    .with_annotations([
                                        Annotation::secondary(*span).with_message(format!(
                                            "property `{}::{}` previously defined here.",
                                            class_like_name, item_name
                                        )),
                                        Annotation::secondary(class_like_span.span()).with_message(format!(
                                            "{} `{}` defined here.",
                                            class_like_kind, class_like_fqcn
                                        )),
                                    ])
                                    .with_help("remove the duplicate property"),
                            );
                        } else {
                            property_names.push((false, item_name_id, item_variable.span()));
                        }
                    }
                },
                ClassLikeMember::Method(method) => {
                    let method_name_id = method.name.value;
                    let method_name = context.interner.lookup(&method_name_id);
                    let method_name_lowered_id = context.interner.intern(method_name.to_ascii_lowercase());

                    if let Some((previous, _)) =
                        method_names.iter().find(|(_, previous_name)| method_name_lowered_id.eq(previous_name))
                    {
                        context.report(
                            Issue::error(format!(
                                "{} method `{}::{}` has already been defined",
                                class_like_kind, class_like_name, method_name
                            ))
                            .with_annotation(Annotation::primary(method.name.span()))
                            .with_annotations([
                                Annotation::secondary(*previous).with_message("previous definition"),
                                Annotation::secondary(class_like_span.span())
                                    .with_message(format!("{} `{}` defined here.", class_like_kind, class_like_fqcn)),
                            ]),
                        );
                    } else {
                        method_names.push((method.name.span(), method_name_lowered_id));
                    }

                    if method_name.eq_ignore_ascii_case(CONSTRUCTOR_MAGIC_METHOD) {
                        for parameter in method.parameter_list.parameters.iter() {
                            if parameter.is_promoted_property() {
                                let item_name_id = parameter.variable.name;
                                let item_name = context.interner.lookup(&item_name_id);

                                if let Some((is_promoted, _, span)) =
                                    property_names.iter().find(|(_, name, _)| item_name_id.eq(name))
                                {
                                    let message = if !*is_promoted {
                                        format!(
                                            "promoted property `{}::{}` has already been defined as a property",
                                            class_like_name, item_name
                                        )
                                    } else {
                                        format!(
                                            "promoted property `{}::{}` has already been defined",
                                            class_like_name, item_name
                                        )
                                    };

                                    context.report(
                                        Issue::error(message)
                                            .with_annotation(Annotation::primary(parameter.variable.span()))
                                            .with_annotations([
                                                Annotation::secondary(*span).with_message(format!(
                                                    "property `{}::{}` previously defined here.",
                                                    class_like_name, item_name
                                                )),
                                                Annotation::secondary(class_like_span.span()).with_message(format!(
                                                    "{} `{}` defined here.",
                                                    class_like_kind, class_like_fqcn
                                                )),
                                            ])
                                            .with_help("remove the duplicate property"),
                                    );
                                } else {
                                    property_names.push((true, item_name_id, parameter.variable.span()));
                                }
                            }
                        }
                    }
                }
                ClassLikeMember::Constant(class_like_constant) => {
                    for item in class_like_constant.items.iter() {
                        let item_name = context.interner.lookup(&item.name.value);

                        if let Some((is_constant, name, span)) = constant_names.iter().find(|t| t.1.eq(&item_name)) {
                            if *is_constant {
                                context.report(
                                    Issue::error(format!(
                                        "{} constant `{}::{}` has already been defined",
                                        class_like_kind, class_like_name, name,
                                    ))
                                    .with_annotation(Annotation::primary(item.name.span()))
                                    .with_annotations([
                                        Annotation::secondary(*span).with_message(format!(
                                            "Constant `{}::{}` previously defined here.",
                                            class_like_name, name
                                        )),
                                        Annotation::secondary(class_like_span.span()).with_message(format!(
                                            "{} `{}` defined here.",
                                            class_like_kind, class_like_fqcn
                                        )),
                                    ]),
                                );
                            } else {
                                context.report(
                                    Issue::error(format!(
                                        "{} case `{}::{}` and constant `{}::{}` cannot have the same name",
                                        class_like_kind, class_like_name, name, class_like_name, name
                                    ))
                                    .with_annotation(Annotation::primary(item.name.span()))
                                    .with_annotations([
                                        Annotation::secondary(*span).with_message(format!(
                                            "case `{}::{}` defined here.",
                                            class_like_name, name
                                        )),
                                        Annotation::secondary(class_like_span.span()).with_message(format!(
                                            "{} `{}` defined here.",
                                            class_like_kind, class_like_fqcn
                                        )),
                                    ]),
                                );
                            }
                        } else {
                            constant_names.push((true, item_name.to_string(), item.name.span()));
                        }
                    }
                }
                ClassLikeMember::EnumCase(enum_case) => {
                    let case_name = context.interner.lookup(&enum_case.item.name().value);

                    if let Some((is_constant, name, span)) = constant_names.iter().find(|t| t.1.eq(&case_name)) {
                        if *is_constant {
                            context.report(
                                Issue::error(format!(
                                    "{} case `{}::{}` and constant `{}::{}` cannot have the same name",
                                    class_like_kind, class_like_name, name, class_like_name, name
                                ))
                                .with_annotation(Annotation::primary(enum_case.item.name().span()))
                                .with_annotations([
                                    Annotation::secondary(*span).with_message(format!(
                                        "Constant `{}::{}` defined here.",
                                        class_like_name, name
                                    )),
                                    Annotation::secondary(class_like_span.span()).with_message(format!(
                                        "{} `{}` defined here.",
                                        class_like_kind, class_like_fqcn
                                    )),
                                ]),
                            );
                        } else {
                            context.report(
                                Issue::error(format!(
                                    "{} case `{}::{}` has already been defined",
                                    class_like_kind, class_like_name, name,
                                ))
                                .with_annotation(Annotation::primary(enum_case.item.name().span()))
                                .with_annotations([
                                    Annotation::secondary(*span).with_message(format!(
                                        "case `{}::{}` previously defined here.",
                                        class_like_name, name
                                    )),
                                    Annotation::secondary(class_like_span.span()).with_message(format!(
                                        "{} `{}` defined here.",
                                        class_like_kind, class_like_fqcn
                                    )),
                                ]),
                            );
                        }

                        continue;
                    } else {
                        constant_names.push((false, case_name.to_string(), enum_case.item.name().span()));
                    }
                }
                _ => {}
            }
        }
    }

    #[inline]
    fn process_class_like_constant(
        &self,
        class_like_constant: &ClassLikeConstant,
        class_like_span: Span,
        class_like_kind: &str,
        class_like_name: &str,
        class_like_fqcn: &str,
        context: &mut Context,
    ) {
        let first_item = class_like_constant.first_item();
        let first_item_name = context.interner.lookup(&first_item.name.value);

        let mut last_final: Option<Span> = None;
        let mut last_visibility: Option<Span> = None;
        for modifier in class_like_constant.modifiers.iter() {
            match modifier {
                Modifier::Readonly(k)
                | Modifier::Static(k)
                | Modifier::Abstract(k)
                | Modifier::PrivateSet(k)
                | Modifier::ProtectedSet(k)
                | Modifier::PublicSet(k) => {
                    context.report(
                        Issue::error(format!(
                            "`{}` modifier is not allowed on constants",
                            context.interner.lookup(&k.value),
                        ))
                        .with_annotation(Annotation::primary(modifier.span()))
                        .with_annotations([
                            Annotation::secondary(first_item.span()).with_message(format!(
                                "{} constant `{}::{}` is declared here.",
                                class_like_kind, class_like_name, first_item_name
                            )),
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` is declared here.", class_like_kind, class_like_fqcn)),
                        ]),
                    );
                }
                Modifier::Final(_) => {
                    if let Some(last_final) = last_final {
                        context.report(
                            Issue::error("duplicate `final` modifier on constant")
                                .with_annotation(Annotation::primary(modifier.span()))
                                .with_annotations([
                                    Annotation::secondary(last_final).with_message("previous `final` modifier"),
                                    Annotation::secondary(first_item.span()).with_message(format!(
                                        "{} constant `{}::{}` is declared here.",
                                        class_like_kind, class_like_name, first_item_name
                                    )),
                                    Annotation::secondary(class_like_span).with_message(format!(
                                        "{} `{}` is declared here.",
                                        class_like_kind, class_like_fqcn
                                    )),
                                ]),
                        );
                    }

                    last_final = Some(modifier.span());
                }
                Modifier::Private(_) | Modifier::Protected(_) | Modifier::Public(_) => {
                    if let Some(last_visibility) = last_visibility {
                        context.report(
                            Issue::error("duplicate visibility modifier on constant")
                                .with_annotation(Annotation::primary(modifier.span()))
                                .with_annotations([
                                    Annotation::secondary(last_visibility).with_message("previous visibility modifier"),
                                    Annotation::secondary(first_item.span()).with_message(format!(
                                        "{} constant `{}::{}` is declared here.",
                                        class_like_kind, class_like_name, first_item_name
                                    )),
                                    Annotation::secondary(class_like_span).with_message(format!(
                                        "{} `{}` is declared here.",
                                        class_like_kind, class_like_fqcn
                                    )),
                                ]),
                        );
                    }

                    last_visibility = Some(modifier.span());
                }
            }
        }

        for item in class_like_constant.items.iter() {
            let item_name = context.interner.lookup(&item.name.value);

            if !item.value.is_constant(context.version, false) {
                context.report(
                    Issue::error(format!(
                        "Constant `{}::{}` value contains a non-constant expression.",
                        class_like_name, item_name
                    ))
                    .with_annotation(Annotation::primary(item.value.span()))
                    .with_annotations([
                        Annotation::secondary(item.name.span()).with_message(format!(
                            "{} constant `{}::{}` is declared here.",
                            class_like_kind, class_like_name, item_name
                        )),
                        Annotation::secondary(class_like_span)
                            .with_message(format!("{} `{}` is declared here.", class_like_kind, class_like_fqcn)),
                    ]),
                );
            }
        }
    }

    #[inline]
    fn process_promoted_properties_outside_constructor(
        &self,
        parameter_list: &FunctionLikeParameterList,
        context: &mut Context<'_>,
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
}

impl Walker<Context<'_>> for SemanticsWalker {
    fn walk_in_statement(&self, statement: &Statement, context: &mut Context<'_>) {
        context.push_ancestor(statement.span());
    }

    fn walk_in_expression(&self, expression: &Expression, context: &mut Context<'_>) {
        context.push_ancestor(expression.span());
    }

    fn walk_out_statement(&self, _statement: &Statement, context: &mut Context<'_>) {
        context.pop_ancestor();
    }

    fn walk_out_expression(&self, _expression: &Expression, context: &mut Context<'_>) {
        context.pop_ancestor();
    }

    fn walk_in_program(&self, program: &Program, context: &mut Context<'_>) {
        let mut index = 0;
        let mut before = vec![];

        for statement in program.statements.iter() {
            if !matches!(
                statement,
                Statement::Declare(_)
                    | Statement::OpeningTag(_)
                    | Statement::Inline(Inline { kind: InlineKind::Shebang, .. })
            ) {
                index += 1;

                before.push(statement.span());

                continue;
            }

            if index == 0 {
                continue;
            }

            if let Statement::Declare(declare) = statement {
                for item in declare.items.iter() {
                    let name = context.interner.lookup(&item.name.value);

                    if name.eq_ignore_ascii_case(STRICT_TYPES_DECLARE_DIRECTIVE) {
                        context.report(
                            Issue::error("Strict type declaration must be the first statement in the file.")
                                .with_annotation(
                                    Annotation::primary(declare.span())
                                        .with_message("Strict type declaration found here."),
                                )
                                .with_annotations(before.iter().map(|span| {
                                    Annotation::secondary(*span)
                                        .with_message("This statement appears before the strict type declaration.")
                                }))
                                .with_help("Move all statements before the strict type declaration to after it."),
                        );
                    }
                }
            }
        }

        let mut index = 0;
        let mut before = vec![];

        for statement in program.statements.iter() {
            if !matches!(
                statement,
                Statement::Declare(_)
                    | Statement::Namespace(_)
                    | Statement::OpeningTag(_)
                    | Statement::Inline(Inline { kind: InlineKind::Shebang, .. })
            ) {
                index += 1;

                before.push(statement.span());

                continue;
            }

            if index == 0 {
                continue;
            }

            if let Statement::Namespace(namespace) = statement {
                context.report(
                    Issue::error("Namespace must be the first statement in the file.")
                        .with_annotation(
                            Annotation::primary(namespace.span()).with_message("Namespace statement found here."),
                        )
                        .with_annotations(before.iter().map(|span| {
                            Annotation::secondary(*span)
                                .with_message("This statement appears before the namespace declaration.")
                        }))
                        .with_help("Move all statements before the namespace declaration to after it."),
                );
            }
        }

        let program = context.program();
        let namespaces = program.filter_map(|node| if let Node::Namespace(ns) = node { Some(*ns) } else { None });

        let mut last_unbraced = None;
        let mut last_braced = None;

        for namespace in namespaces {
            let mut namespace_span = namespace.namespace.span();
            if let Some(name) = &namespace.name {
                namespace_span = namespace_span.join(name.span());
            }

            match &namespace.body {
                NamespaceBody::Implicit(body) => {
                    if namespace.name.is_none() {
                        context.report(
                            Issue::error("Unbraced namespace must be named.")
                                .with_annotation(
                                    Annotation::primary(namespace.span().join(body.terminator.span()))
                                        .with_message("Unnamed unbraced namespace."),
                                )
                                .with_annotation(
                                    Annotation::secondary(body.span()).with_message("Namespace body without a name."),
                                )
                                .with_help("Add a name to the unbraced namespace."),
                        );
                    }

                    last_unbraced = Some((namespace_span, body.span()));
                    if let Some((last_namespace_span, last_body_span)) = last_braced {
                        context.report(
                            Issue::error(
                                "Cannot mix unbraced namespace declarations with braced namespace declarations.",
                            )
                            .with_annotation(
                                Annotation::primary(namespace_span)
                                    .with_message("This is an unbraced namespace declaration."),
                            )
                            .with_annotations([
                                Annotation::primary(last_namespace_span)
                                    .with_message("Previous braced namespace declaration."),
                                Annotation::secondary(last_body_span).with_message("Braced namespace body."),
                                Annotation::secondary(body.span()).with_message("Unbraced namespace body."),
                            ])
                            .with_help(
                                "Use consistent namespace declaration styles: either all braced or all unbraced.",
                            ),
                        );
                    }
                }
                NamespaceBody::BraceDelimited(body) => {
                    last_braced = Some((namespace_span, body.span()));

                    if let Some((last_namespace_span, last_body_span)) = last_unbraced {
                        context.report(
                            Issue::error(
                                "Cannot mix braced namespace declarations with unbraced namespace declarations.",
                            )
                            .with_annotation(
                                Annotation::primary(namespace_span)
                                    .with_message("This is a braced namespace declaration."),
                            )
                            .with_annotations([
                                Annotation::primary(last_namespace_span)
                                    .with_message("Previous unbraced namespace declaration."),
                                Annotation::secondary(last_body_span).with_message("Unbraced namespace body."),
                                Annotation::secondary(body.span()).with_message("Braced namespace body."),
                            ])
                            .with_help(
                                "Use consistent namespace declaration styles: either all braced or all unbraced.",
                            ),
                        );
                    }
                }
            }
        }
    }

    fn walk_in_short_opening_tag(&self, short_opening_tag: &ShortOpeningTag, context: &mut Context<'_>) {
        context.report(
            Issue::error("Short opening tag `<?` is no longer supported.")
                .with_annotation(
                    Annotation::primary(short_opening_tag.span()).with_message("Short opening tag used here."),
                )
                .with_note("Short opening tags have been removed in modern PHP versions.")
                .with_help("Replace the short opening tag with the full opening tag `<?php`."),
        );
    }

    fn walk_in_declare(&self, declare: &Declare, context: &mut Context<'_>) {
        for item in declare.items.iter() {
            let name = context.interner.lookup(&item.name.value);

            match name.to_ascii_lowercase().as_str() {
                STRICT_TYPES_DECLARE_DIRECTIVE => {
                    let value = match &item.value {
                        Expression::Literal(Literal::Integer(LiteralInteger { value, .. })) => *value,
                        _ => None,
                    };

                    if !matches!(value, Some(0) | Some(1)) {
                        context.report(
                            Issue::error("The `strict_types` directive must be set to either `0` or `1`.")
                                .with_annotation(
                                    Annotation::primary(item.value.span())
                                        .with_message("Invalid value assigned to the directive."),
                                )
                                .with_note("The `strict_types` directive controls strict type enforcement and only accepts `0` (disabled) or `1` (enabled).")
                                .with_help("Set the directive value to either `0` or `1`."),
                        );
                    }

                    if context.get_ancestors_len() > 2 {
                        // get the span of the parent, and label it.
                        let parent = context.get_ancestor(context.get_ancestors_len() - 2);

                        context.report(
                            Issue::error("The `strict_types` directive must be declared at the top level.")
                                .with_annotation(
                                    Annotation::primary(declare.span()).with_message("Directive declared here."),
                                )
                                .with_annotation(
                                    Annotation::secondary(parent)
                                        .with_message("This statement should follow the `strict_types` directive."),
                                )
                                .with_help("Move the `strict_types` declaration to the top level of the file."),
                        );
                    }
                }
                TICKS_DECLARE_DIRECTIVE => {
                    if !matches!(item.value, Expression::Literal(Literal::Integer(_))) {
                        context.report(
                            Issue::error("The `ticks` directive must be set to a literal integer.")
                                .with_annotation(
                                    Annotation::primary(item.value.span())
                                        .with_message("Invalid value assigned to the directive."),
                                )
                                .with_note("The `ticks` directive requires a literal integer value to specify the tick interval.")
                                .with_help("Provide a literal integer value for the `ticks` directive."),
                        );
                    }
                }
                ENCODING_DECLARE_DIRECTIVE => {
                    if !matches!(item.value, Expression::Literal(Literal::String(_))) {
                        context.report(
                            Issue::error("The `encoding` declare directive must be set to a literal integer")
                                .with_annotation(
                                    Annotation::primary(item.value.span())
                                        .with_message("Invalid value assigned to the directive."),
                                )
                                .with_note("The `encoding` directive requires a literal string value to specify the character encoding.")
                                .with_help("Provide a literal string value for the `encoding` directive."),
                        );
                    }
                }
                _ => {
                    context.report(
                        Issue::error(format!(
                            "`{}` is not a supported `declare` directive. Supported directives are: `{}`.",
                            name,
                            DECLARE_DIRECTIVES.join("`, `")
                        ))
                        .with_annotation(
                            Annotation::primary(item.name.span()).with_message("Unsupported directive used here."),
                        )
                        .with_note("Only specific directives are allowed in `declare` statements.")
                        .with_help(format!(
                            "Use one of the supported directives: `{}`.",
                            DECLARE_DIRECTIVES.join("`, `")
                        )),
                    );
                }
            }
        }
    }

    fn walk_in_namespace(&self, namespace: &Namespace, context: &mut Context<'_>) {
        if context.get_ancestors_len() > 2 {
            // get the span of the parent, and label it.
            let parent = context.get_ancestor(context.get_ancestors_len() - 2);

            context.report(
                Issue::error("Namespace declaration must be at the top level.")
                    .with_annotation(
                        Annotation::primary(namespace.span())
                            .with_message("Namespace declared here."),
                    )
                    .with_annotation(
                        Annotation::secondary(parent)
                            .with_message("This statement should come after the namespace declaration."),
                    )
                    .with_note("Namespace declarations define the scope of the code and should always appear at the top level.")
                    .with_help("Move the namespace declaration to the top level of the file."),
            );
        }
    }

    fn walk_in_hint(&self, hint: &Hint, context: &mut Context<'_>) {
        match hint {
            Hint::Parenthesized(parenthesized_hint) => {
                if !parenthesized_hint.hint.is_parenthesizable() {
                    let val = context.lookup_hint(&parenthesized_hint.hint);

                    context.report(
                        Issue::error(format!("Type `{}` cannot be parenthesized.", val))
                            .with_annotation(
                                Annotation::primary(parenthesized_hint.hint.span())
                                    .with_message("Invalid parenthesized type."),
                            )
                            .with_annotation(
                                Annotation::secondary(parenthesized_hint.span())
                                    .with_message("Parenthesized type defined here."),
                            )
                            .with_note("Only union or intersection types can be enclosed in parentheses.")
                            .with_help("Remove the parentheses around the type."),
                    );
                }
            }
            Hint::Nullable(nullable_hint) => {
                if !context.version.is_supported(Feature::NullableTypeHint) {
                    context.report(
                        Issue::error("The `?` nullable type hint is only available in PHP 7.1 and above.")
                            .with_annotation(
                                Annotation::primary(hint.span()).with_message("`?` nullable type hint used here."),
                            )
                            .with_help("Upgrade to PHP 7.1 or above to use the `?` nullable type hint."),
                    );
                }

                if nullable_hint.hint.is_standalone() || nullable_hint.hint.is_complex() {
                    let val = context.lookup_hint(&nullable_hint.hint);

                    context.report(
                        Issue::error(format!("Type `{}` cannot be nullable.", val))
                            .with_annotation(
                                Annotation::primary(nullable_hint.hint.span()).with_message("Invalid nullable type."),
                            )
                            .with_annotation(
                                Annotation::secondary(nullable_hint.span()).with_message("Nullable type defined here."),
                            )
                            .with_help("Replace the type or remove the nullable modifier."),
                    );
                }
            }
            Hint::Union(union_hint) => {
                if !union_hint.left.is_unionable() {
                    let val = context.lookup_hint(&union_hint.left);

                    context.report(
                        Issue::error(format!("Type `{}` cannot be part of a union.", val))
                            .with_annotation(
                                Annotation::primary(union_hint.left.span()).with_message("Invalid union type."),
                            )
                            .with_annotation(
                                Annotation::secondary(union_hint.pipe).with_message("Union operator `|` used here."),
                            )
                            .with_note("Intersection and standalone types cannot be part of a union.")
                            .with_help("Replace the type or remove it from the union."),
                    );
                }

                if !union_hint.right.is_unionable() {
                    let val = context.lookup_hint(&union_hint.right);

                    context.report(
                        Issue::error(format!("Type `{}` cannot be part of a union.", val))
                            .with_annotation(
                                Annotation::primary(union_hint.right.span()).with_message("Invalid union type."),
                            )
                            .with_annotation(
                                Annotation::secondary(union_hint.pipe).with_message("Union operator `|` used here."),
                            )
                            .with_note("Intersection and standalone types cannot be part of a union.")
                            .with_help("Replace the type or remove it from the union."),
                    );
                }
            }
            Hint::Intersection(intersection_hint) => {
                if !context.version.is_supported(Feature::PureIntersectionTypes) {
                    context.report(
                        Issue::error("Intersection types are only available in PHP 8.1 and above.")
                            .with_annotation(
                                Annotation::primary(intersection_hint.span()).with_message("Intersection type used here."),
                            )
                            .with_note(
                                "Intersection types allow combining multiple types into a single type, but are only available in PHP 8.2 and above.",
                            )
                            .with_help("Upgrade to PHP 8.2 or above to use intersection types."),
                    );
                }

                if !intersection_hint.left.is_intersectable() {
                    let val = context.lookup_hint(&intersection_hint.left);

                    context.report(
                        Issue::error(format!("Type `{}` cannot be part of an intersection.", val))
                            .with_annotation(
                                Annotation::primary(intersection_hint.left.span())
                                    .with_message("Invalid intersection type."),
                            )
                            .with_annotation(
                                Annotation::secondary(intersection_hint.ampersand)
                                    .with_message("Intersection operator `&` used here."),
                            )
                            .with_note("Union and standalone types cannot be part of an intersection.")
                            .with_help("Replace the type or remove it from the intersection."),
                    );
                }

                if !intersection_hint.right.is_intersectable() {
                    let val = context.lookup_hint(&intersection_hint.right);

                    context.report(
                        Issue::error(format!("Type `{}` cannot be part of an intersection.", val))
                            .with_annotation(
                                Annotation::primary(intersection_hint.right.span())
                                    .with_message("Invalid intersection type."),
                            )
                            .with_annotation(
                                Annotation::secondary(intersection_hint.ampersand)
                                    .with_message("Intersection operator `&` used here."),
                            )
                            .with_note("Union and standalone types cannot be part of an intersection.")
                            .with_help("Replace the type or remove it from the intersection."),
                    );
                }
            }
            Hint::True(hint) if !context.version.is_supported(Feature::TrueTypeHint) => {
                context.report(
                    Issue::error("The `true` type hint is only available in PHP 8.2 and above.")
                        .with_annotation(Annotation::primary(hint.span()).with_message("`true` type hint used here."))
                        .with_help("Upgrade to PHP 8.2 or above to use the `true` type hint."),
                );
            }
            Hint::False(hint) if !context.version.is_supported(Feature::FalseTypeHint) => {
                context.report(
                    Issue::error("The `false` type hint is only available in PHP 8.2 and above.")
                        .with_annotation(Annotation::primary(hint.span()).with_message("`false` type hint used here."))
                        .with_help("Upgrade to PHP 8.2 or above to use the `false` type hint."),
                );
            }
            Hint::Null(hint) if !context.version.is_supported(Feature::NullTypeHint) => {
                context.report(
                    Issue::error("The `null` type hint is only available in PHP 8.2 and above.")
                        .with_annotation(Annotation::primary(hint.span()).with_message("`null` type hint used here."))
                        .with_help("Upgrade to PHP 8.2 or above to use the `null` type hint."),
                );
            }
            Hint::Iterable(hint) if !context.version.is_supported(Feature::IterableTypeHint) => {
                context.report(
                    Issue::error("The `iterable` type hint is only available in PHP 7.1 and above.")
                        .with_annotation(
                            Annotation::primary(hint.span()).with_message("`iterable` type hint used here."),
                        )
                        .with_help("Upgrade to PHP 7.1 or above to use the `iterable` type hint."),
                );
            }
            Hint::Void(hint) if !context.version.is_supported(Feature::VoidTypeHint) => {
                context.report(
                    Issue::error("The `void` type hint is only available in PHP 7.1 and above.")
                        .with_annotation(Annotation::primary(hint.span()).with_message("`void` type hint used here."))
                        .with_help("Upgrade to PHP 7.1 or above to use the `void` type hint."),
                );
            }
            Hint::Mixed(hint) if !context.version.is_supported(Feature::MixedTypeHint) => {
                context.report(
                    Issue::error("The `mixed` type hint is only available in PHP 8.0 and above.")
                        .with_annotation(Annotation::primary(hint.span()).with_message("`mixed` type hint used here."))
                        .with_help("Upgrade to PHP 8.0 or above to use the `mixed` type hint."),
                );
            }
            Hint::Never(hint) if !context.version.is_supported(Feature::NeverTypeHint) => {
                context.report(
                    Issue::error("The `never` type hint is only available in PHP 8.1 and above.")
                        .with_annotation(Annotation::primary(hint.span()).with_message("`never` type hint used here."))
                        .with_help("Upgrade to PHP 8.1 or above to use the `never` type hint."),
                );
            }
            _ => {}
        }

        if context.version.is_supported(Feature::DisjunctiveNormalForm) {
            return;
        }

        let is_dnf = match hint {
            Hint::Intersection(inter) if inter.left.is_union() || inter.right.is_union() => true,
            Hint::Union(union) if union.left.is_intersection() || union.right.is_intersection() => true,
            _ => false,
        };

        if !is_dnf {
            return;
        }

        context.report(
            Issue::error("Disjunctive Normal Form (DNF) types are only available in PHP 8.2 and above.")
                .with_annotation(Annotation::primary(hint.span()).with_message("DNF type used here.")),
        );
    }

    fn walk_in_try(&self, r#try: &Try, context: &mut Context<'_>) {
        if r#try.catch_clauses.is_empty() && r#try.finally_clause.is_none() {
            context.report(
                Issue::error("Cannot use `try` without a `catch` or `finally` clause.")
                    .with_annotation(
                        Annotation::primary(r#try.span()).with_message("`try` statement without `catch` or `finally`."),
                    )
                    .with_note("Each `try` block must have at least one corresponding `catch` or `finally` clause.")
                    .with_help("Add either a `catch` or `finally` clause to the `try` block.")
                    .with_link("https://www.php.net/manual/en/language.exceptions.php"),
            );
        }
    }

    fn walk_in_property_hook(&self, property_hook: &PropertyHook, context: &mut Context<'_>) {
        if let Some(parameter_list) = &property_hook.parameters {
            self.process_promoted_properties_outside_constructor(parameter_list, context);
        }
    }

    fn walk_in_method(&self, method: &Method, context: &mut Context<'_>) {
        let name = context.interner.lookup(&method.name.value);
        if name != "__construct" {
            self.process_promoted_properties_outside_constructor(&method.parameter_list, context);

            return;
        }

        if let Some(abstract_modifier) = method.modifiers.get_abstract() {
            for parameter in method.parameter_list.parameters.iter() {
                if parameter.is_promoted_property() {
                    context.report(
                        Issue::error("Promoted properties are not allowed in abstract constructors.")
                            .with_annotation(
                                Annotation::primary(parameter.span()).with_message("Promoted property used here."),
                            )
                            .with_annotation(
                                Annotation::secondary(abstract_modifier.span())
                                    .with_message("This constructor is abstract."),
                            )
                            .with_help(
                                "Remove the promoted property from the constructor or make the constructor concrete.",
                            ),
                    );
                }
            }
        }
    }

    fn walk_in_class(&self, class: &Class, context: &mut Context<'_>) {
        let class_name = context.interner.lookup(&class.name.value);
        let class_fqcn = context.lookup_name(&class.name.span.start);

        if RESERVED_KEYWORDS.iter().any(|keyword| keyword.eq_ignore_ascii_case(class_name))
            || SOFT_RESERVED_KEYWORDS_MINUS_SYMBOL_ALLOWED
                .iter()
                .any(|keyword| keyword.eq_ignore_ascii_case(class_name))
        {
            context.report(
                Issue::error(format!("Class `{}` name cannot be a reserved keyword.", class_name))
                    .with_annotation(
                        Annotation::primary(class.name.span())
                            .with_message(format!("Class name `{}` conflicts with a reserved keyword.", class_name)),
                    )
                    .with_annotation(
                        Annotation::secondary(class.span())
                            .with_message(format!("Class `{}` declared here.", class_fqcn)),
                    )
                    .with_help("Rename the class to avoid using reserved keywords."),
            );
        }

        let mut last_final = None;
        let mut last_abstract = None;
        let mut last_readonly = None;

        for modifier in class.modifiers.iter() {
            match &modifier {
                Modifier::Static(_) => {
                    context.report(
                        Issue::error(format!("Class `{}` cannot have the `static` modifier.", class_name))
                            .with_annotation(
                                Annotation::primary(modifier.span()).with_message("`static` modifier applied here."),
                            )
                            .with_annotation(
                                Annotation::secondary(class.span())
                                    .with_message(format!("Class `{}` declared here.", class_fqcn)),
                            )
                            .with_help("Remove the `static` modifier."),
                    );
                }
                Modifier::Public(keyword)
                | Modifier::Protected(keyword)
                | Modifier::Private(keyword)
                | Modifier::PublicSet(keyword)
                | Modifier::ProtectedSet(keyword)
                | Modifier::PrivateSet(keyword) => {
                    let visibility_name = context.interner.lookup(&keyword.value);

                    context.report(
                        Issue::error(format!(
                            "Class `{}` cannot have the `{}` visibility modifier.",
                            class_name, visibility_name
                        ))
                        .with_annotation(
                            Annotation::primary(keyword.span())
                                .with_message(format!("`{}` modifier applied here.", visibility_name)),
                        )
                        .with_annotation(
                            Annotation::secondary(class.span())
                                .with_message(format!("Class `{}` declared here.", class_fqcn)),
                        )
                        .with_help(format!("Remove the `{}` modifier.", visibility_name)),
                    );
                }
                Modifier::Final(keyword) => {
                    if let Some(span) = last_abstract {
                        context.report(
                            Issue::error(format!("Abstract class `{}` cannot have the `final` modifier.", class_name))
                                .with_annotation(
                                    Annotation::primary(keyword.span()).with_message("`final` modifier applied here."),
                                )
                                .with_annotations([
                                    Annotation::secondary(span)
                                        .with_message("Previous `abstract` modifier applied here."),
                                    Annotation::secondary(class.span())
                                        .with_message(format!("Class `{}` declared here.", class_fqcn)),
                                ])
                                .with_help("Remove the `final` modifier from the abstract class."),
                        );
                    }

                    if let Some(span) = last_final {
                        context.report(
                            Issue::error(format!("Class `{}` cannot have multiple `final` modifiers.", class_name))
                                .with_annotation(
                                    Annotation::primary(keyword.span())
                                        .with_message("Duplicate `final` modifier applied here."),
                                )
                                .with_annotations([
                                    Annotation::secondary(span).with_message("Previous `final` modifier applied here."),
                                    Annotation::secondary(class.span())
                                        .with_message(format!("Class `{}` declared here.", class_fqcn)),
                                ])
                                .with_help("Remove the duplicate `final` modifier."),
                        );
                    }

                    last_final = Some(keyword.span);
                }
                Modifier::Abstract(keyword) => {
                    if let Some(span) = last_final {
                        context.report(
                            Issue::error(format!("Final class `{}` cannot have the `abstract` modifier.", class_name))
                                .with_annotation(
                                    Annotation::primary(keyword.span())
                                        .with_message("`abstract` modifier applied here."),
                                )
                                .with_annotations([
                                    Annotation::secondary(span).with_message("Previous `final` modifier applied here."),
                                    Annotation::secondary(class.span())
                                        .with_message(format!("Class `{}` declared here.", class_fqcn)),
                                ])
                                .with_help("Remove the `abstract` modifier from the final class."),
                        );
                    }

                    if let Some(span) = last_abstract {
                        context.report(
                            Issue::error(format!("Class `{}` cannot have multiple `abstract` modifiers.", class_name))
                                .with_annotation(
                                    Annotation::primary(keyword.span())
                                        .with_message("Duplicate `abstract` modifier applied here."),
                                )
                                .with_annotations([
                                    Annotation::secondary(span)
                                        .with_message("Previous `abstract` modifier applied here."),
                                    Annotation::secondary(class.span())
                                        .with_message(format!("Class `{}` declared here.", class_fqcn)),
                                ])
                                .with_help("Remove the duplicate `abstract` modifier."),
                        );
                    }

                    last_abstract = Some(keyword.span);
                }
                Modifier::Readonly(keyword) => {
                    if let Some(span) = last_readonly {
                        context.report(
                            Issue::error(format!("Class `{}` cannot have multiple `readonly` modifiers.", class_name))
                                .with_annotation(
                                    Annotation::primary(keyword.span())
                                        .with_message("Duplicate `readonly` modifier applied here."),
                                )
                                .with_annotations([
                                    Annotation::secondary(span)
                                        .with_message("Previous `readonly` modifier applied here."),
                                    Annotation::secondary(class.span())
                                        .with_message(format!("Class `{}` declared here.", class_fqcn)),
                                ])
                                .with_help("Remove the duplicate `readonly` modifier."),
                        );
                    }

                    last_readonly = Some(keyword.span);
                }
            }
        }

        if !context.version.is_supported(Feature::ReadonlyClasses) {
            if let Some(modifier) = last_readonly {
                let issue = Issue::error("Readonly classes are only available in PHP 8.2 and above.")
                    .with_annotation(Annotation::primary(modifier.span()).with_message("Readonly modifier used here."));

                context.report(issue);
            }
        }

        if let Some(extends) = &class.extends {
            self.process_extends(extends, class.span(), "class", class_name, class_fqcn, true, context);
        }

        if let Some(implements) = &class.implements {
            self.process_implements(implements, class.span(), "class", class_name, class_fqcn, true, context);
        }

        self.process_members(&class.members, class.span(), "class", class_name, class_fqcn, context);

        for memeber in class.members.iter() {
            match &memeber {
                ClassLikeMember::EnumCase(case) => {
                    context.report(
                        Issue::error(format!("Class `{}` cannot contain enum cases.", class_name))
                            .with_annotation(Annotation::primary(case.span()).with_message("Enum case found in class."))
                            .with_annotation(
                                Annotation::secondary(class.span())
                                    .with_message(format!("Class `{}` declared here.", class_fqcn)),
                            )
                            .with_help("Remove the enum cases from the class definition."),
                    );
                }
                ClassLikeMember::Method(method) => {
                    let method_name = context.interner.lookup(&method.name.value);

                    if !class.modifiers.contains_abstract() && method.modifiers.contains_abstract() {
                        context.report(
                            Issue::error(format!(
                                "Class `{}` contains an abstract method `{}`, so the class must be declared abstract.",
                                class_name, method_name
                            ))
                            .with_annotation(
                                Annotation::primary(class.name.span())
                                    .with_message("Class is missing the `abstract` modifier."),
                            )
                            .with_annotation(Annotation::secondary(method.span()).with_message(format!(
                                "Abstract method `{}::{}` declared here.",
                                class_name, method_name
                            )))
                            .with_help("Add the `abstract` modifier to the class."),
                        );
                    }

                    self.process_method(
                        method,
                        method_name,
                        class.span(),
                        class_name,
                        class_fqcn,
                        "class",
                        false,
                        context,
                    );
                }
                ClassLikeMember::Property(property) => {
                    self.process_property(property, class.span(), "class", class_name, class_fqcn, false, context);
                }
                ClassLikeMember::Constant(constant) => {
                    self.process_class_like_constant(constant, class.span(), "class", class_name, class_fqcn, context);
                }
                _ => {}
            }
        }
    }

    fn walk_in_interface(&self, interface: &Interface, context: &mut Context<'_>) {
        let interface_name = context.interner.lookup(&interface.name.value);
        let interface_fqcn = context.lookup_name(&interface.name.span.start);

        if RESERVED_KEYWORDS.iter().any(|keyword| keyword.eq_ignore_ascii_case(interface_name))
            || SOFT_RESERVED_KEYWORDS_MINUS_SYMBOL_ALLOWED
                .iter()
                .any(|keyword| keyword.eq_ignore_ascii_case(interface_name))
        {
            context.report(
                Issue::error(format!("Interface `{}` name cannot be a reserved keyword.", interface_name))
                    .with_annotation(
                        Annotation::primary(interface.name.span())
                            .with_message(format!("Interface `{}` declared here.", interface_name)),
                    )
                    .with_annotation(
                        Annotation::secondary(interface.span())
                            .with_message(format!("Interface `{}` defined here.", interface_fqcn)),
                    )
                    .with_help("Rename the interface to avoid using a reserved keyword."),
            );
        }

        if let Some(extends) = &interface.extends {
            self.process_extends(
                extends,
                interface.span(),
                "interface",
                interface_name,
                interface_fqcn,
                false,
                context,
            );
        }

        self.process_members(
            &interface.members,
            interface.span(),
            "interface",
            interface_name,
            interface_fqcn,
            context,
        );

        for memeber in interface.members.iter() {
            match &memeber {
                ClassLikeMember::TraitUse(trait_use) => {
                    context.report(
                        Issue::error(format!("Interface `{}` cannot use traits.", interface_name))
                            .with_annotation(Annotation::primary(trait_use.span()).with_message("Trait use statement."))
                            .with_annotation(
                                Annotation::secondary(interface.span())
                                    .with_message(format!("Interface `{}` declared here.", interface_fqcn)),
                            )
                            .with_help("Remove the trait use statement."),
                    );
                }
                ClassLikeMember::EnumCase(case) => {
                    context.report(
                        Issue::error(format!("Interface `{}` cannot contain enum cases.", interface_name))
                            .with_annotation(
                                Annotation::primary(case.span())
                                    .with_message("Enum case declared here."),
                            )
                            .with_annotation(
                                Annotation::secondary(interface.span())
                                    .with_message(format!("Interface `{}` declared here.", interface_fqcn)),
                            )
                            .with_note("Consider moving the enum case to an enum or class if it represents state or constants."),
                    );
                }
                ClassLikeMember::Method(method) => {
                    let method_name_id = method.name.value;
                    let method_name = context.interner.lookup(&method_name_id);

                    let mut visibilities = vec![];
                    for modifier in method.modifiers.iter() {
                        if matches!(modifier, Modifier::Private(_) | Modifier::Protected(_)) {
                            visibilities.push(modifier);
                        }
                    }

                    for visibility in visibilities {
                        let visibility_name = visibility.as_str(context.interner);

                        context.report(
                            Issue::error(format!(
                                "Interface method `{}::{}` cannot have `{}` modifier.",
                                interface_name, method_name, visibility_name
                            ))
                            .with_annotation(
                                Annotation::primary(visibility.span())
                                    .with_message(format!("`{}` modifier applied here.", visibility_name)),
                            )
                            .with_annotation(
                                Annotation::secondary(interface.span())
                                    .with_message(format!("Interface `{}` declared here.", interface_fqcn)),
                            )
                            .with_help(format!(
                                "Remove the `{}` modifier from the method definition as methods in interfaces must always be public.",
                                visibility_name
                            ))
                            .with_note("Interface methods are always public and cannot have non-public visibility modifiers."),
                        );
                    }

                    if let MethodBody::Concrete(body) = &method.body {
                        context.report(
                            Issue::error(format!(
                                "Interface method `{}::{}` cannot have a body.",
                                interface_name, method_name
                            ))
                            .with_annotations([
                                Annotation::primary(body.span()).with_message("Method body declared here."),
                                Annotation::primary(method.name.span()).with_message("Method name defined here."),
                                Annotation::secondary(interface.span())
                                    .with_message(format!("Interface `{}` declared here.", interface_fqcn)),
                            ])
                            .with_help("Replace the method body with a `;` to indicate it is abstract.")
                            .with_note("Methods in interfaces cannot have implementations and must be abstract."),
                        );
                    }

                    if let Some(abstract_modifier) = method.modifiers.get_abstract() {
                        context.report(
                            Issue::error(format!(
                                "Interface method `{}::{}` must not be abstract.",
                                interface_name, method_name
                            ))
                            .with_annotation(
                                Annotation::primary(abstract_modifier.span())
                                    .with_message("Abstract modifier applied here."),
                            )
                            .with_annotations([
                                Annotation::secondary(interface.span())
                                    .with_message(format!("Interface `{}` declared here.", interface_fqcn)),
                                Annotation::secondary(method.span())
                                    .with_message(format!("Method `{}::{}` declared here.", interface_name, method_name)),
                            ])
                            .with_help("Remove the `abstract` modifier as all interface methods are implicitly abstract.")
                            .with_note(
                                "Adding the `abstract` modifier to an interface method is redundant because all interface methods are implicitly abstract.",
                            ),
                        );
                    }

                    self.process_method(
                        method,
                        method_name,
                        interface.span(),
                        interface_name,
                        interface_fqcn,
                        "interface",
                        true,
                        context,
                    );
                }
                ClassLikeMember::Property(property) => {
                    match &property {
                        Property::Plain(plain_property) => {
                            context.report(
                                Issue::error(format!(
                                    "Interface `{}` cannot have non-hooked properties.",
                                    interface_name
                                ))
                                .with_annotation(
                                    Annotation::primary(plain_property.span())
                                        .with_message("Non-hooked property declared here."),
                                )
                                .with_annotation(
                                    Annotation::secondary(interface.span())
                                        .with_message(format!("Interface `{}` declared here.", interface_fqcn)),
                                )
                                .with_note("Interfaces are intended to define behavior and cannot include concrete property declarations.")
                                .with_help("Remove the non-hooked property from the interface or convert it into a hooked property.")
                            );
                        }
                        Property::Hooked(hooked_property) => {
                            let property_name_id = hooked_property.item.variable().name;
                            let property_name = context.interner.lookup(&property_name_id);

                            let mut found_public = false;
                            let mut non_public_read_visibilities = vec![];
                            let mut write_visibilities = vec![];
                            for modifier in hooked_property.modifiers.iter() {
                                if matches!(modifier, Modifier::Public(_)) {
                                    found_public = true;
                                }

                                if matches!(modifier, Modifier::Private(_) | Modifier::Protected(_)) {
                                    non_public_read_visibilities.push(modifier);
                                }

                                if matches!(modifier, Modifier::PrivateSet(_)) {
                                    write_visibilities.push(modifier);
                                }
                            }

                            for visibility in write_visibilities {
                                let visibility_name = visibility.as_str(context.interner);

                                context.report(
                                    Issue::error(format!(
                                        "Interface virtual property `{}::{}` must not specify asymmetric visibility.",
                                        interface_name, property_name,
                                    ))
                                    .with_annotation(
                                        Annotation::primary(visibility.span())
                                            .with_message(format!("Asymmetric visibility modifier `{}` applied here.", visibility_name)),
                                    )
                                    .with_annotation(
                                        Annotation::secondary(interface.span())
                                            .with_message(format!("Interface `{}` defined here.", interface_fqcn)),
                                    )
                                    .with_help(format!(
                                        "Remove the `{}` modifier from the property to make it compatible with interface constraints.",
                                        visibility_name
                                    )),
                                );
                            }

                            for visibility in non_public_read_visibilities {
                                let visibility_name = visibility.as_str(context.interner);

                                context.report(
                                    Issue::error(format!(
                                        "Interface virtual property `{}::{}` cannot have `{}` modifier.",
                                        interface_name, property_name, visibility_name,
                                    ))
                                    .with_annotation(Annotation::primary(visibility.span()).with_message(format!(
                                        "Visibility modifier `{}` applied here.",
                                        visibility_name
                                    )))
                                    .with_annotation(
                                        Annotation::secondary(interface.span())
                                            .with_message(format!("Interface `{}` defined here.", interface_fqcn)),
                                    )
                                    .with_help(format!(
                                        "Remove the `{}` modifier from the property to meet interface requirements.",
                                        visibility_name
                                    )),
                                );
                            }

                            if !found_public {
                                context.report(
                                    Issue::error(format!(
                                        "Interface virtual property `{}::{}` must be declared public.",
                                        interface_name, property_name
                                    ))
                                    .with_annotation(
                                        Annotation::primary(hooked_property.span())
                                            .with_message("Property defined here."),
                                    )
                                    .with_annotation(
                                        Annotation::secondary(interface.span())
                                            .with_message(format!("Interface `{}` defined here.", interface_fqcn)),
                                    )
                                    .with_help("Add the `public` visibility modifier to the property."),
                                );
                            }

                            if let Some(abstract_modifier) = hooked_property.modifiers.get_abstract() {
                                context.report(
                                        Issue::error(format!(
                                            "Interface virtual property `{}::{}` cannot be abstract.",
                                            interface_name, property_name
                                        ))
                                        .with_annotation(
                                            Annotation::primary(abstract_modifier.span())
                                                .with_message("Abstract modifier applied here."),
                                        )
                                        .with_annotations([
                                            Annotation::secondary(hooked_property.span())
                                                .with_message("Property defined here."),
                                            Annotation::secondary(interface.span())
                                                .with_message(format!("Interface `{}` defined here.", interface_fqcn)),
                                        ])
                                        .with_note(
                                            "All interface virtual properties are implicitly abstract and cannot be explicitly declared as abstract.",
                                        ),
                                    );
                            }

                            if let PropertyItem::Concrete(item) = &hooked_property.item {
                                context.report(
                                    Issue::error(format!(
                                        "Interface virtual property `{}::{}` cannot have a default value.",
                                        interface_name, property_name
                                    ))
                                    .with_annotation(
                                        Annotation::primary(item.equals.join(item.value.span()))
                                            .with_message("Default value assigned here."),
                                    )
                                    .with_annotation(
                                        Annotation::secondary(hooked_property.span())
                                                .with_message("Property defined here.")
                                    )
                                    .with_annotation(
                                        Annotation::secondary(interface.span())
                                            .with_message(format!("Interface `{}` defined here.", interface_fqcn)),
                                    )
                                    .with_note(
                                        "Interface properties are virtual properties and cannot contain a default value.",
                                    ),
                                );
                            }

                            for hook in hooked_property.hooks.hooks.iter() {
                                if let PropertyHookBody::Concrete(property_hook_concrete_body) = &hook.body {
                                    context.report(
                                        Issue::error(format!(
                                            "Interface virtual property `{}::{}` must be abstract.",
                                            interface_name, property_name
                                        ))
                                        .with_annotation(
                                            Annotation::primary(property_hook_concrete_body.span())
                                                .with_message("Body defined here."),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(hooked_property.item.variable().span())
                                                .with_message("Property declared here."),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(interface.span())
                                                .with_message(format!("Interface `{}` defined here.", interface_fqcn)),
                                        )
                                        .with_note("Abstract hooked properties must not contain a body."),
                                    );
                                }
                            }
                        }
                    };

                    self.process_property(
                        property,
                        interface.span(),
                        "interface",
                        interface_name,
                        interface_fqcn,
                        true,
                        context,
                    );
                }
                ClassLikeMember::Constant(class_like_constant) => {
                    let mut non_public_read_visibility = vec![];
                    for modifier in class_like_constant.modifiers.iter() {
                        if matches!(modifier, Modifier::Private(_) | Modifier::Protected(_)) {
                            non_public_read_visibility.push(modifier);
                        }
                    }

                    for visibility in non_public_read_visibility.iter() {
                        let visibility_name = visibility.as_str(context.interner);

                        context.report(
                            Issue::error(format!(
                                "Interface constant cannot have `{}` visibility modifier.",
                                visibility_name,
                            ))
                            .with_annotation(
                                Annotation::primary(visibility.span())
                                    .with_message(format!("Visibility modifier `{}` applied here.", visibility_name)),
                            )
                            .with_help(format!(
                                "Remove the `{}` modifier from the constant to comply with interface requirements.",
                                visibility_name
                            ))
                            .with_note(
                                "Interface constants are implicitly public and cannot have a non-public visibility modifier.",
                            ),
                        );
                    }

                    self.process_class_like_constant(
                        class_like_constant,
                        interface.span(),
                        "interface",
                        interface_name,
                        interface_fqcn,
                        context,
                    );
                }
            }
        }
    }

    fn walk_in_trait(&self, r#trait: &Trait, context: &mut Context<'_>) {
        let class_like_name = context.interner.lookup(&r#trait.name.value);
        let class_like_fqcn = context.lookup_name(&r#trait.name.span.start);

        if RESERVED_KEYWORDS.iter().any(|keyword| keyword.eq_ignore_ascii_case(class_like_name))
            || SOFT_RESERVED_KEYWORDS_MINUS_SYMBOL_ALLOWED
                .iter()
                .any(|keyword| keyword.eq_ignore_ascii_case(class_like_name))
        {
            context.report(
                Issue::error(format!("Trait `{}` name cannot be a reserved keyword.", class_like_name))
                    .with_annotation(
                        Annotation::primary(r#trait.name.span())
                            .with_message(format!("Trait `{}` declared here.", class_like_name)),
                    )
                    .with_annotation(
                        Annotation::secondary(r#trait.span())
                            .with_message(format!("Trait `{}` defined here.", class_like_fqcn)),
                    )
                    .with_help("Rename the trait to a non-reserved keyword."),
            );
        }

        self.process_members(&r#trait.members, r#trait.span(), class_like_name, class_like_fqcn, "trait", context);

        for member in r#trait.members.iter() {
            match &member {
                ClassLikeMember::EnumCase(case) => {
                    context.report(
                        Issue::error(format!("Trait `{}` cannot contain enum cases.", class_like_name))
                            .with_annotation(Annotation::primary(case.span()).with_message("Enum case defined here."))
                            .with_annotation(
                                Annotation::secondary(r#trait.span())
                                    .with_message(format!("Trait `{}` defined here.", class_like_fqcn)),
                            )
                            .with_help("Remove the enum case from the trait."),
                    );
                }
                ClassLikeMember::Method(method) => {
                    let method_name = context.interner.lookup(&method.name.value);

                    self.process_method(
                        method,
                        method_name,
                        r#trait.span(),
                        class_like_name,
                        class_like_fqcn,
                        "trait",
                        false,
                        context,
                    );
                }
                ClassLikeMember::Property(property) => {
                    self.process_property(
                        property,
                        r#trait.span(),
                        "trait",
                        class_like_name,
                        class_like_fqcn,
                        false,
                        context,
                    );
                }
                ClassLikeMember::Constant(class_like_constant) => {
                    if !context.version.is_supported(Feature::ConstantsInTraits) {
                        context.report(
                            Issue::error("Constants in traits are only available in PHP 8.2 and above.")
                                .with_annotation(
                                    Annotation::primary(class_like_constant.span())
                                        .with_message("Constant defined in trait."),
                                )
                                .with_annotation(
                                    Annotation::secondary(r#trait.span())
                                        .with_message(format!("Trait `{}` defined here.", class_like_fqcn)),
                                ),
                        );
                    }

                    self.process_class_like_constant(
                        class_like_constant,
                        r#trait.span(),
                        "trait",
                        class_like_name,
                        class_like_fqcn,
                        context,
                    );
                }
                _ => {}
            }
        }
    }

    fn walk_in_enum(&self, r#enum: &Enum, context: &mut Context<'_>) {
        if !context.version.is_supported(Feature::Enums) {
            context.report(
                Issue::error("Enums are only available in PHP 8.1 and above.")
                    .with_annotation(Annotation::primary(r#enum.span()).with_message("Enum defined here.")),
            );
        }

        let enum_name = context.interner.lookup(&r#enum.name.value);
        let enum_fqcn = context.lookup_name(&r#enum.name.span.start);
        let enum_is_backed = r#enum.backing_type_hint.is_some();

        if RESERVED_KEYWORDS.iter().any(|keyword| keyword.eq_ignore_ascii_case(enum_name))
            || SOFT_RESERVED_KEYWORDS_MINUS_SYMBOL_ALLOWED.iter().any(|keyword| keyword.eq_ignore_ascii_case(enum_name))
        {
            context.report(
                Issue::error(format!("Enum `{}` name cannot be a reserved keyword.", enum_name))
                    .with_annotation(
                        Annotation::primary(r#enum.name.span())
                            .with_message(format!("Reserved keyword used as the enum name `{}`.", enum_name)),
                    )
                    .with_annotation(
                        Annotation::secondary(r#enum.span())
                            .with_message(format!("Enum `{}` defined here.", enum_fqcn)),
                    )
                    .with_help(format!("Rename the enum `{}` to a non-reserved keyword.", enum_name)),
            );
        }

        if let Some(EnumBackingTypeHint { hint, .. }) = &r#enum.backing_type_hint {
            if !matches!(hint, Hint::String(_) | Hint::Integer(_)) {
                let key = context.lookup_hint(hint);

                context.report(
                    Issue::error(format!(
                        "Enum `{}` backing type must be either `string` or `int`, but found `{}`.",
                        enum_name, key
                    ))
                    .with_annotation(
                        Annotation::primary(hint.span())
                            .with_message(format!("Invalid backing type `{}` specified here.", key)),
                    )
                    .with_annotation(
                        Annotation::secondary(r#enum.name.span())
                            .with_message(format!("Enum `{}` defined here.", enum_fqcn)),
                    )
                    .with_help("Change the backing type to either `string` or `int`."),
                );
            }
        }

        if let Some(implements) = &r#enum.implements {
            self.process_implements(implements, r#enum.span(), "enum", enum_name, enum_fqcn, true, context);
        }

        self.process_members(&r#enum.members, r#enum.span(), enum_name, enum_fqcn, "enum", context);

        for member in r#enum.members.iter() {
            match &member {
                ClassLikeMember::EnumCase(case) => {
                    let item_name_id = case.item.name().value;
                    let item_name = context.interner.lookup(&item_name_id);

                    match &case.item {
                        EnumCaseItem::Unit(_) => {
                            if enum_is_backed {
                                context.report(
                                    Issue::error(format!(
                                        "Case `{}` of backed enum `{}` must have a value.",
                                        item_name, enum_name
                                    ))
                                    .with_annotation(
                                        Annotation::primary(case.span())
                                            .with_message(format!("Case `{}` defined here.", item_name)),
                                    )
                                    .with_annotation(
                                        Annotation::secondary(r#enum.span())
                                            .with_message(format!("Enum `{}` defined here.", enum_fqcn)),
                                    )
                                    .with_help(format!(
                                        "Add a value to case `{}` or remove the backing from the enum `{}`.",
                                        item_name, enum_name
                                    )),
                                );
                            }
                        }
                        EnumCaseItem::Backed(item) => {
                            if !enum_is_backed {
                                context.report(
                                    Issue::error(format!(
                                        "Case `{}` of unbacked enum `{}` must not have a value.",
                                        item_name, enum_name
                                    ))
                                    .with_annotation(
                                        Annotation::primary(item.equals.span().join(item.value.span()))
                                            .with_message("Value assigned to the enum case."),
                                    )
                                    .with_annotations([
                                        Annotation::secondary(item.name.span()).with_message(format!(
                                            "Case `{}::{}` declared here.",
                                            enum_name, item_name
                                        )),
                                        Annotation::secondary(r#enum.span())
                                            .with_message(format!("Enum `{}` defined here.", enum_fqcn)),
                                    ])
                                    .with_help(format!(
                                        "Remove the value from case `{}` or make the enum `{}` backed.",
                                        item_name, enum_name
                                    )),
                                );
                            }
                        }
                    }
                }
                ClassLikeMember::Method(method) => {
                    let method_name_id = method.name.value;
                    let method_name = context.interner.lookup(&method_name_id);

                    if let Some(magic_method) =
                        MAGIC_METHODS.iter().find(|magic_method| magic_method.eq_ignore_ascii_case(method_name))
                    {
                        context.report(
                            Issue::error(format!(
                                "Enum `{}` cannot contain magic method `{}`.",
                                enum_name, magic_method
                            ))
                            .with_annotation(
                                Annotation::primary(method.name.span)
                                    .with_message(format!("Magic method `{}` declared here.", method_name)),
                            )
                            .with_annotation(
                                Annotation::secondary(r#enum.name.span())
                                    .with_message(format!("Enum `{}` declared here.", enum_fqcn)),
                            )
                            .with_help(format!(
                                "Remove the magic method `{}` from the enum `{}`.",
                                method_name, enum_name
                            )),
                        );
                    }

                    if let Some(abstract_modifier) = method.modifiers.get_abstract() {
                        context.report(
                            Issue::error(format!("Enum method `{}::{}` must not be abstract.", enum_name, method_name))
                                .with_annotation(
                                    Annotation::primary(abstract_modifier.span())
                                        .with_message("Abstract modifier found here."),
                                )
                                .with_annotations([
                                    Annotation::secondary(r#enum.span())
                                        .with_message(format!("Enum `{}` defined here.", enum_fqcn)),
                                    Annotation::secondary(method.span())
                                        .with_message(format!("Method `{}::{}` defined here.", enum_name, method_name)),
                                ])
                                .with_help(format!(
                                    "Remove the abstract modifier from the method `{}` in enum `{}`.",
                                    method_name, enum_name
                                )),
                        );
                    }

                    self.process_method(
                        method,
                        method_name,
                        r#enum.span(),
                        enum_name,
                        enum_fqcn,
                        "enum",
                        false,
                        context,
                    );
                }
                ClassLikeMember::Property(property) => {
                    context.report(
                        Issue::error(format!("Enum `{}` cannot have properties.", enum_name))
                            .with_annotation(
                                Annotation::primary(property.span()).with_message("Property defined here."),
                            )
                            .with_annotation(
                                Annotation::secondary(r#enum.span())
                                    .with_message(format!("Enum `{}` defined here.", enum_fqcn)),
                            )
                            .with_help(format!("Remove the property from the enum `{}`.", enum_name)),
                    );

                    self.process_property(property, r#enum.span(), "enum", enum_name, enum_fqcn, false, context);
                }
                ClassLikeMember::Constant(class_like_constant) => {
                    self.process_class_like_constant(
                        class_like_constant,
                        r#enum.span(),
                        "enum",
                        enum_name,
                        enum_fqcn,
                        context,
                    );
                }
                _ => {}
            }
        }
    }

    fn walk_in_anonymous_class(&self, anonymous_class: &AnonymousClass, context: &mut Context<'_>) {
        let mut last_final = None;
        let mut last_readonly = None;

        for modifier in anonymous_class.modifiers.iter() {
            match &modifier {
                Modifier::Static(_)
                | Modifier::Abstract(_)
                | Modifier::PrivateSet(_)
                | Modifier::ProtectedSet(_)
                | Modifier::PublicSet(_)
                | Modifier::Public(_)
                | Modifier::Protected(_)
                | Modifier::Private(_) => {
                    let modifier_name = modifier.as_str(context.interner);

                    context.report(
                        Issue::error(format!(
                            "Anonymous class `{}` cannot have the `{}` modifier.",
                            ANONYMOUS_CLASS_NAME, modifier_name
                        ))
                        .with_annotation(
                            Annotation::primary(modifier.span())
                                .with_message(format!("`{}` modifier applied here.", modifier_name)),
                        )
                        .with_annotation(
                            Annotation::secondary(anonymous_class.span())
                                .with_message(format!("Anonymous class `{}` defined here.", ANONYMOUS_CLASS_NAME)),
                        )
                        .with_help(format!("Remove the `{}` modifier from the class definition.", modifier_name)),
                    );
                }
                Modifier::Final(keyword) => {
                    if let Some(span) = last_final {
                        context.report(
                            Issue::error(format!(
                                "Anonymous class `{}` cannot have multiple `final` modifiers.",
                                ANONYMOUS_CLASS_NAME
                            ))
                            .with_annotation(
                                Annotation::primary(keyword.span())
                                    .with_message("Duplicate `final` modifier applied here."),
                            )
                            .with_annotation(
                                Annotation::secondary(span).with_message("Previous `final` modifier applied here."),
                            )
                            .with_annotation(
                                Annotation::secondary(anonymous_class.span())
                                    .with_message(format!("Anonymous class `{}` defined here.", ANONYMOUS_CLASS_NAME)),
                            )
                            .with_help("Remove the duplicate `final` modifier."),
                        );
                    }

                    last_final = Some(keyword.span);
                }
                Modifier::Readonly(keyword) => {
                    if let Some(span) = last_readonly {
                        context.report(
                            Issue::error(format!(
                                "Anonymous class `{}` cannot have multiple `readonly` modifiers.",
                                ANONYMOUS_CLASS_NAME
                            ))
                            .with_annotations([
                                Annotation::primary(keyword.span)
                                    .with_message("Duplicate `readonly` modifier applied here."),
                                Annotation::secondary(span).with_message("Previous `readonly` modifier applied here."),
                                Annotation::secondary(anonymous_class.span())
                                    .with_message(format!("Anonymous class `{}` defined here.", ANONYMOUS_CLASS_NAME)),
                            ])
                            .with_help("Remove the duplicate `readonly` modifier."),
                        );
                    }

                    last_readonly = Some(keyword.span);

                    if !context.version.is_supported(Feature::ReadonlyAnonymousClasses) {
                        context.report(
                            Issue::error("Readonly anonymous classes are only available in PHP 8.3 and above.")
                                .with_annotation(
                                    Annotation::primary(keyword.span).with_message("Readonly modifier used here."),
                                )
                                .with_annotation(
                                    Annotation::secondary(anonymous_class.span()).with_message(format!(
                                        "Anonymous class `{}` defined here.",
                                        ANONYMOUS_CLASS_NAME
                                    )),
                                ),
                        );
                    }
                }
            }
        }

        if let Some(extends) = &anonymous_class.extends {
            self.process_extends(
                extends,
                anonymous_class.span(),
                "class",
                ANONYMOUS_CLASS_NAME,
                ANONYMOUS_CLASS_NAME,
                true,
                context,
            );
        }

        if let Some(implements) = &anonymous_class.implements {
            self.process_implements(
                implements,
                anonymous_class.span(),
                "class",
                ANONYMOUS_CLASS_NAME,
                ANONYMOUS_CLASS_NAME,
                false,
                context,
            );
        }

        self.process_members(
            &anonymous_class.members,
            anonymous_class.span(),
            "class",
            ANONYMOUS_CLASS_NAME,
            ANONYMOUS_CLASS_NAME,
            context,
        );

        for member in anonymous_class.members.iter() {
            match &member {
                ClassLikeMember::EnumCase(case) => {
                    context.report(
                        Issue::error(format!("Anonymous class `{}` cannot contain enum cases.", ANONYMOUS_CLASS_NAME))
                            .with_annotations([
                                Annotation::primary(case.span()).with_message("Enum case defined here."),
                                Annotation::secondary(anonymous_class.span())
                                    .with_message(format!("Anonymous class `{}` defined here.", ANONYMOUS_CLASS_NAME)),
                            ])
                            .with_help("Remove the enum case from the anonymous class definition."),
                    );
                }
                ClassLikeMember::Method(method) => {
                    let method_name = context.interner.lookup(&method.name.value);

                    if let Some(abstract_modifier) = method.modifiers.get_abstract() {
                        context.report(
                            Issue::error(format!(
                                "Method `{}` in anonymous class `{}` must not be abstract.",
                                method_name, ANONYMOUS_CLASS_NAME
                            ))
                            .with_annotations([
                                Annotation::primary(abstract_modifier.span())
                                    .with_message("Abstract modifier applied here."),
                                Annotation::secondary(anonymous_class.span())
                                    .with_message(format!("Anonymous class `{}` defined here.", ANONYMOUS_CLASS_NAME)),
                                Annotation::secondary(method.span())
                                    .with_message(format!("Method `{}` defined here.", method_name)),
                            ])
                            .with_help("Remove the `abstract` modifier from the method."),
                        );
                    }

                    self.process_method(
                        method,
                        method_name,
                        anonymous_class.span(),
                        ANONYMOUS_CLASS_NAME,
                        ANONYMOUS_CLASS_NAME,
                        "class",
                        false,
                        context,
                    );
                }
                ClassLikeMember::Property(property) => {
                    self.process_property(
                        property,
                        anonymous_class.span(),
                        "class",
                        ANONYMOUS_CLASS_NAME,
                        ANONYMOUS_CLASS_NAME,
                        false,
                        context,
                    );
                }
                ClassLikeMember::Constant(class_like_constant) => {
                    self.process_class_like_constant(
                        class_like_constant,
                        anonymous_class.span(),
                        "class",
                        ANONYMOUS_CLASS_NAME,
                        ANONYMOUS_CLASS_NAME,
                        context,
                    );
                }
                _ => {}
            }
        }
    }

    fn walk_in_function(&self, function: &Function, context: &mut Context<'_>) {
        self.process_promoted_properties_outside_constructor(&function.parameter_list, context);

        let name = context.interner.lookup(&function.name.value);
        let fqfn = context.lookup_name(&function.name.span.start);

        let hint = if let Some(return_hint) = &function.return_type_hint {
            &return_hint.hint
        } else {
            return;
        };

        let returns = mago_ast_utils::find_returns_in_block(&function.body);

        match &hint {
            Hint::Void(_) => {
                for r#return in returns {
                    if let Some(val) = &r#return.value {
                        context.report(
                            Issue::error(format!(
                                "Function `{}` with return type `void` must not return a value.",
                                name
                            ))
                            .with_annotation(Annotation::primary(val.span()).with_message("Return value found here."))
                            .with_annotation(
                                Annotation::secondary(function.span())
                                    .with_message(format!("Function `{}` defined here.", fqfn)),
                            )
                            .with_help("Remove the return type hint or the return value."),
                        );
                    }
                }
            }
            Hint::Never(_) => {
                for r#return in returns {
                    context.report(
                        Issue::error(format!("Function `{}` with return type `never` must not return.", name))
                            .with_annotation(
                                Annotation::primary(r#return.span()).with_message("Return statement found here."),
                            )
                            .with_annotation(
                                Annotation::secondary(function.span())
                                    .with_message(format!("Function `{}` defined here.", fqfn)),
                            )
                            .with_help("Remove the return type hint or the return statement."),
                    );
                }
            }
            _ if !returns_generator(context, &function.body, hint) => {
                for r#return in returns {
                    if r#return.value.is_none() {
                        context.report(
                            Issue::error(format!("Function `{}` with a return type must return a value.", name))
                                .with_annotation(
                                    Annotation::primary(r#return.span())
                                        .with_message("Empty return statement found here."),
                                )
                                .with_annotation(
                                    Annotation::secondary(function.span())
                                        .with_message(format!("Function `{}` defined here.", fqfn)),
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

    fn walk_in_attribute_list(&self, attribute_list: &AttributeList, context: &mut Context<'_>) {
        if !context.version.is_supported(Feature::Attribute) {
            context.report(
                Issue::error("Attributes are only available in PHP 8.0 and above.")
                    .with_annotation(
                        Annotation::primary(attribute_list.span()).with_message("Attribute list used here."),
                    )
                    .with_help("Upgrade to PHP 8.0 or above to use attributes."),
            );
        }
    }

    fn walk_in_attribute(&self, attribute: &Attribute, context: &mut Context<'_>) {
        let name = context.interner.lookup(&attribute.name.value());
        if let Some(list) = &attribute.arguments {
            for argument in list.arguments.iter() {
                let (ellipsis, value) = match &argument {
                    Argument::Positional(positional_argument) => {
                        (positional_argument.ellipsis, &positional_argument.value)
                    }
                    Argument::Named(named_argument) => (named_argument.ellipsis, &named_argument.value),
                };

                if let Some(ellipsis) = ellipsis {
                    context.report(
                        Issue::error("Cannot use argument unpacking in attribute arguments.")
                            .with_annotation(
                                Annotation::primary(ellipsis.span()).with_message("Argument unpacking used here."),
                            )
                            .with_annotation(
                                Annotation::secondary(attribute.name.span())
                                    .with_message(format!("Attribute `{}` defined here.", name)),
                            )
                            .with_note("Unpacking arguments is not allowed in attribute arguments."),
                    );
                }

                if !value.is_constant(context.version, true) {
                    context.report(
                        Issue::error(format!("Attribute `{}` argument contains a non-constant expression.", name))
                            .with_annotations([
                                Annotation::primary(value.span()).with_message("Non-constant expression used here."),
                                Annotation::secondary(attribute.name.span())
                                    .with_message(format!("Attribute `{}` defined here.", name)),
                            ])
                            .with_note("Attribute arguments must be constant expressions."),
                    );
                }
            }
        }
    }

    fn walk_in_goto(&self, goto: &Goto, context: &mut Context<'_>) {
        let all_labels =
            context.program().filter_map(|node| if let Node::Label(label) = node { Some(*label) } else { None });

        if all_labels.iter().any(|l| l.name.value == goto.label.value) {
            return;
        }

        // If we reach this point, the label was not found.
        // Attempt to find a label with the same name but different case.
        // If found, suggest the correct label.
        let going_to = context.interner.lookup(&goto.label.value);
        let mut suggestions = vec![];

        for label in all_labels {
            let label_name = context.interner.lookup(&label.name.value);
            if label_name.eq_ignore_ascii_case(going_to) {
                suggestions.push((label_name, label.name.span));
            }
        }

        let mut issue =
            Issue::error(format!("Undefined `goto` label `{}`.", going_to))
                .with_annotation(Annotation::primary(goto.label.span).with_message("This `goto` label is not defined."))
                .with_annotations(suggestions.iter().map(|(name, span)| {
                    Annotation::secondary(*span).with_message(format!("Did you mean `{}`?", name))
                }));

        if suggestions.len() == 1 {
            issue = issue.with_note(format!(
                "The `goto` label `{}` was not found. Did you mean `{}`?",
                going_to, suggestions[0].0
            ));
        } else if !suggestions.is_empty() {
            let names = suggestions.iter().map(|(name, _)| format!("`{}`", name)).collect::<Vec<_>>().join(", ");
            issue = issue.with_note(format!(
                "The `goto` label `{}` was not found. Did you mean one of the following: {}?",
                going_to, names
            ));
        }

        context.report(issue);
    }

    fn walk_in_argument_list(&self, argument_list: &ArgumentList, context: &mut Context<'_>) {
        let mut last_named_argument: Option<Span> = None;
        let mut last_unpacking: Option<Span> = None;

        for argument in argument_list.arguments.iter() {
            match &argument {
                Argument::Positional(positional_argument) => {
                    if let Some(ellipsis) = positional_argument.ellipsis {
                        if let Some(last_named_argument) = last_named_argument {
                            context.report(
                                Issue::error("Cannot use argument unpacking after a named argument.")
                                    .with_annotation(
                                        Annotation::primary(ellipsis.span()).with_message("Unpacking argument here."),
                                    )
                                    .with_annotation(
                                        Annotation::secondary(last_named_argument).with_message("Named argument here."),
                                    )
                                    .with_note("Unpacking arguments must come before named arguments."),
                            );
                        }

                        last_unpacking = Some(ellipsis.span());
                    } else {
                        if let Some(named_argument) = last_named_argument {
                            context.report(
                                Issue::error("Cannot use positional argument after a named argument.")
                                    .with_annotation(
                                        Annotation::primary(positional_argument.span())
                                            .with_message("Positional argument defined here."),
                                    )
                                    .with_annotation(
                                        Annotation::secondary(named_argument).with_message("Named argument here."),
                                    )
                                    .with_note("Positional arguments must come before named arguments."),
                            );
                        }

                        if let Some(unpacking) = last_unpacking {
                            context.report(
                                Issue::error("Cannot use positional argument after argument unpacking.")
                                    .with_annotation(
                                        Annotation::primary(positional_argument.span())
                                            .with_message("Positional argument defined here."),
                                    )
                                    .with_annotation(
                                        Annotation::secondary(unpacking).with_message("Argument unpacking here."),
                                    )
                                    .with_note("Positional arguments must come before unpacking arguments."),
                            );
                        }
                    }
                }
                Argument::Named(named_argument) => {
                    if let Some(ellipsis) = named_argument.ellipsis {
                        context.report(
                            Issue::error("Cannot use argument unpacking in named arguments.")
                                .with_annotation(
                                    Annotation::primary(ellipsis.span())
                                        .with_message("Unpacking argument defined here."),
                                )
                                .with_annotation(
                                    Annotation::secondary(named_argument.span())
                                        .with_message("Named argument defined here."),
                                )
                                .with_note("Unpacking arguments is not allowed in named arguments."),
                        );
                    }

                    last_named_argument = Some(named_argument.span());
                }
            }
        }

        if !context.version.is_supported(Feature::TrailingCommaInFunctionCalls) {
            if let Some(last_comma) = argument_list.arguments.get_trailing_token() {
                context.report(
                    Issue::error("Trailing comma in function calls is only available in PHP 7.3 and later.")
                        .with_annotation(
                            Annotation::primary(last_comma.span).with_message("Trailing comma found here."),
                        )
                        .with_help(
                            "Remove the trailing comma to make the code compatible with PHP 7.2 and earlier versions, or upgrade to PHP 7.3 or later.",
                        )
                );
            }
        }
    }

    fn walk_in_closure(&self, closure: &Closure, context: &mut Context<'_>) {
        self.process_promoted_properties_outside_constructor(&closure.parameter_list, context);

        let hint = if let Some(return_hint) = &closure.return_type_hint {
            &return_hint.hint
        } else {
            return;
        };

        let returns = mago_ast_utils::find_returns_in_block(&closure.body);

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
                            .with_annotation(
                                Annotation::secondary(closure.span()).with_message("Closure defined here."),
                            )
                            .with_help(
                                "Remove the return statement, or change the return type hint to a compatible type.",
                            ),
                    );
                }
            }
            _ if !returns_generator(context, &closure.body, hint) => {
                for r#return in returns {
                    if r#return.value.is_none() {
                        context.report(
                            Issue::error("Closure with a return type must return a value.")
                                .with_annotation(
                                    Annotation::primary(r#return.span()).with_message("Missing return value."),
                                )
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

        if !context.version.is_supported(Feature::TrailingCommaInClosureUseList) {
            let Some(use_clause) = &closure.use_clause else {
                return;
            };

            let Some(trailing_comma) = use_clause.variables.get_trailing_token() else {
                return;
            };

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
    }

    fn walk_in_arrow_function(&self, arrow_function: &ArrowFunction, context: &mut Context<'_>) {
        if !context.version.is_supported(Feature::ArrowFunctions) {
            let issue = Issue::error("The `fn` keyword for arrow functions is only available in PHP 7.4 and later.")
                .with_annotation(
                    Annotation::primary(arrow_function.span()).with_message("Arrow function uses `fn` keyword."),
                );

            context.report(issue);
        }

        self.process_promoted_properties_outside_constructor(&arrow_function.parameter_list, context);

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
                        Issue::error(
                            "The `never` return type in arrow functions is only available in PHP 8.2 and later.",
                        )
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

    fn walk_in_function_like_parameter_list(
        &self,
        function_like_parameter_list: &FunctionLikeParameterList,
        context: &mut Context<'_>,
    ) {
        let mut last_variadic = None;
        let mut parameters_seen = vec![];
        for parameter in function_like_parameter_list.parameters.iter() {
            let name = context.interner.lookup(&parameter.variable.name);
            if let Some(prev_span) =
                parameters_seen.iter().find_map(|(n, s)| if parameter.variable.name.eq(n) { Some(s) } else { None })
            {
                context.report(
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
                        context.report(
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
                            context.report(
                                Issue::error(format!(
                                    "Parameter `{}` cannot have multiple `readonly` modifiers.",
                                    name
                                ))
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
                                Issue::error(format!(
                                    "Parameter `{}` cannot have multiple visibility modifiers.",
                                    name
                                ))
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
                                    "Parameter `{}` cannot have multiple write visibility modifiers.",
                                    name
                                ))
                                .with_annotation(
                                    Annotation::primary(modifier.span())
                                        .with_message("Duplicate write visibility modifier used here."),
                                )
                                .with_annotation(
                                    Annotation::secondary(s)
                                        .with_message("Previous write visibility modifier used here."),
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
                        "Invalid parameter order: parameter `{}` is defined after variadic parameter `{}`.",
                        name,
                        context.interner.lookup(&n)
                    ))
                    .with_annotation(
                        Annotation::primary(parameter.variable.span())
                            .with_message(format!("Parameter `{}` is defined here.", name)),
                    )
                    .with_annotation(
                        Annotation::secondary(s).with_message(format!(
                            "Variadic parameter `{}` is defined here.",
                            context.interner.lookup(&n)
                        )),
                    )
                    .with_help(
                        "Move all parameters following the variadic parameter to the end of the parameter list.",
                    ),
                );
            }

            if let Some(ellipsis) = parameter.ellipsis {
                if let Some(default) = &parameter.default_value {
                    context.report(
                        Issue::error(format!(
                            "Invalid parameter definition: variadic parameter `{}` cannot have a default value.",
                            name
                        ))
                        .with_annotation(
                            Annotation::primary(default.span()).with_message(format!(
                                "Default value is defined for variadic parameter `{}` here.",
                                name
                            )),
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
                    let hint_name = context.lookup_hint(hint);

                    context.report(
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
                }
            }
        }
    }

    fn walk_in_match(&self, r#match: &Match, context: &mut Context<'_>) {
        if !context.version.is_supported(Feature::MatchExpression) {
            context.report(
                Issue::error("Match expressions are only available in PHP 8.0 and above.")
                    .with_annotation(Annotation::primary(r#match.span()).with_message("Match expression defined here."))
                    .with_help("Upgrade to PHP 8.0 or above to use match expressions."),
            );
        }

        let mut last_default: Option<Span> = None;

        for arm in r#match.arms.iter() {
            if let MatchArm::Default(default_arm) = &arm {
                if let Some(previous) = last_default {
                    context.report(
                        Issue::error("A match expression can only have one default arm.")
                            .with_annotation(
                                Annotation::primary(default_arm.span())
                                    .with_message("This is a duplicate default arm."),
                            )
                            .with_annotation(
                                Annotation::secondary(previous).with_message("The first default arm is defined here."),
                            )
                            .with_annotation(
                                Annotation::secondary(r#match.span()).with_message("Match expression defined here."),
                            )
                            .with_help("Remove this duplicate default arm to ensure the match expression is valid."),
                    );
                } else {
                    last_default = Some(default_arm.default.span);
                }
            }
        }
    }

    fn walk_in_switch(&self, switch: &Switch, context: &mut Context<'_>) {
        let mut last_default: Option<Span> = None;

        for case in switch.body.cases() {
            if let SwitchCase::Default(default_case) = &case {
                if let Some(previous) = last_default {
                    context.report(
                        Issue::error("A switch statement can only have one default case.")
                            .with_annotation(
                                Annotation::primary(default_case.span()).with_message("This is a duplicate default case."),
                            )
                            .with_annotation(
                                Annotation::secondary(previous).with_message("The first default case is defined here."),
                            )
                            .with_annotation(
                                Annotation::secondary(switch.span()).with_message("Switch statement containing the duplicate cases."),
                            )
                            .with_help("Remove this duplicate default case to ensure the switch statement is valid and unambiguous."),
                    );
                } else {
                    last_default = Some(default_case.default.span);
                }
            }
        }
    }

    fn walk_in_assignment(&self, assignment: &Assignment, context: &mut Context<'_>) {
        let AssignmentOperator::Coalesce(operator) = assignment.operator else {
            return;
        };

        if context.version.is_supported(Feature::NullCoalesceAssign) {
            return;
        }

        context.report(
            Issue::error("The `??=` (null coalesce assignment) operator is only available in PHP 7.4 and later.")
                .with_annotation(
                    Annotation::primary(operator.span())
                        .with_message("Null coalesce assignment operator `??=` used here."),
                )
                .with_note(
                    "Use a manual check-and-assignment approach if you need compatibility with older PHP versions.",
                )
                .with_help("Replace `$var ??= <default>` with `$var = $var ?? <default>`."),
        );
    }

    fn walk_in_named_argument(&self, named_argument: &NamedArgument, context: &mut Context<'_>) {
        if context.version.is_supported(Feature::NamedArguments) {
            return;
        }

        context.report(
            Issue::error("Named arguments are only available in PHP 8.0 and above.")
                .with_annotation(Annotation::primary(named_argument.span()).with_message("Named argument used here.")),
        );
    }

    fn walk_in_function_like_parameter(
        &self,
        function_like_parameter: &FunctionLikeParameter,
        context: &mut Context<'_>,
    ) {
        if function_like_parameter.is_promoted_property() && !context.version.is_supported(Feature::PromotedProperties)
        {
            context.report(
                Issue::error("Promoted properties are only available in PHP 8.0 and above.").with_annotation(
                    Annotation::primary(function_like_parameter.span()).with_message("Promoted property used here."),
                ),
            );
        }

        if !context.version.is_supported(Feature::NativeUnionTypes) {
            if let Some(Hint::Union(union_hint)) = &function_like_parameter.hint {
                context.report(
                Issue::error(
                    "Union type hints (e.g. `int|float`) are only available in PHP 8.0 and above.",
                )
                .with_annotation(Annotation::primary(union_hint.span()).with_message("Union type hint used here."))
                .with_note(
                    "Union type hints are only available in PHP 8.0 and above. Consider using a different approach.",
                ),
            );
            }
        }
    }

    fn walk_in_function_like_return_type_hint(
        &self,
        function_like_return_type_hint: &FunctionLikeReturnTypeHint,
        context: &mut Context<'_>,
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

    fn walk_in_property(&self, property: &Property, context: &mut Context<'_>) {
        if !context.version.is_supported(Feature::ReadonlyProperties) {
            if let Some(readonly) = property.modifiers().get_readonly() {
                context.report(
                    Issue::error("Readonly properties are only available in PHP 8.1 and above.").with_annotation(
                        Annotation::primary(readonly.span()).with_message("Readonly modifier used here."),
                    ),
                );
            }
        }

        if !context.version.is_supported(Feature::TypedProperties) {
            if let Some(hint) = property.hint() {
                context.report(
                    Issue::error("Typed properties are only available in PHP 7.4 and above.")
                        .with_annotation(Annotation::primary(hint.span()).with_message("Type hint used here."))
                        .with_help("Remove the type hint to make the code compatible with PHP 7.3 and earlier versions, or upgrade to PHP 7.4 or later."),
                );
            }
        }
    }

    fn walk_in_plain_property(&self, plain_property: &PlainProperty, context: &mut Context<'_>) {
        if !context.version.is_supported(Feature::AsymmetricVisibility) {
            if let Some(write_visibility) = plain_property.modifiers.get_first_write_visibility() {
                context.report(
                    Issue::error("Asymmetric visibility is only available in PHP 8.4 and above.").with_annotation(
                        Annotation::primary(write_visibility.span()).with_message("Asymmetric visibility used here."),
                    ),
                );
            };
        }

        if !context.version.is_supported(Feature::NativeUnionTypes) {
            if let Some(Hint::Union(union_hint)) = &plain_property.hint {
                context.report(
                    Issue::error(
                        "Union type hints (e.g. `int|float`) are only available in PHP 8.0 and above.",
                    )
                    .with_annotation(Annotation::primary(union_hint.span()).with_message("Union type hint used here."))
                    .with_note(
                        "Union type hints are only available in PHP 8.0 and above. Consider using a different approach.",
                    ),
                );
            }
        }
    }

    fn walk_in_hooked_property(&self, hooked_property: &HookedProperty, context: &mut Context<'_>) {
        if !context.version.is_supported(Feature::PropertyHooks) {
            let issue = Issue::error("Hooked properties are only available in PHP 8.4 and above.").with_annotation(
                Annotation::primary(hooked_property.span()).with_message("Hooked property declaration used here."),
            );

            context.report(issue);
        }
    }

    fn walk_in_closure_creation(&self, closure_creation: &ClosureCreation, context: &mut Context<'_>) {
        if context.version.is_supported(Feature::ClosureCreation) {
            return;
        }

        context.report(
            Issue::error("The closure creation syntax is only available in PHP 8.1 and above.").with_annotation(
                Annotation::primary(closure_creation.span()).with_message("Closure creation syntax used here."),
            ),
        );
    }

    fn walk_in_class_like_constant(&self, class_like_constant: &ClassLikeConstant, context: &mut Context<'_>) {
        if !context.version.is_supported(Feature::TypedClassLikeConstants) {
            let Some(type_hint) = &class_like_constant.hint else {
                return;
            };

            context.report(
                Issue::error("Typed class constants are only available in PHP 8.3 and above.")
                    .with_annotation(Annotation::primary(type_hint.span()).with_message("Type hint used here.")),
            );
        }

        if !context.version.is_supported(Feature::FinalConstants) {
            if let Some(modifier) = class_like_constant.modifiers.get_final() {
                context.report(
                    Issue::error("Final class constants are only available in PHP 8.1 and above.").with_annotation(
                        Annotation::primary(modifier.span()).with_message("Final modifier used here."),
                    ),
                );
            }
        }

        if !context.version.is_supported(Feature::ClassLikeConstantVisibilityModifiers) {
            if let Some(visibility) = class_like_constant.modifiers.get_first_visibility() {
                context.report(
                    Issue::error("Visibility modifiers for class constants are only available in PHP 7.1 and above.")
                        .with_annotation(
                            Annotation::primary(visibility.span()).with_message("Visibility modifier used here."),
                        ),
                );
            }
        }
    }

    fn walk_in_list(&self, list: &List, context: &mut Context<'_>) {
        if !context.version.is_supported(Feature::TrailingCommaInListSyntax) {
            if let Some(token) = list.elements.get_trailing_token() {
                context.report(
                    Issue::error("Trailing comma in list syntax is only available in PHP 7.2 and above.")
                        .with_annotation(Annotation::primary(token.span).with_message("Trailing comma used here."))
                        .with_help("Upgrade to PHP 7.2 or later to use trailing commas in list syntax."),
                );
            }
        }

        if !context.version.is_supported(Feature::ListReferenceAssignment) {
            for element in list.elements.iter() {
                let value = match element {
                    ArrayElement::KeyValue(kv) => kv.value.as_ref(),
                    ArrayElement::Value(v) => v.value.as_ref(),
                    _ => continue,
                };

                if let Expression::UnaryPrefix(UnaryPrefix {
                    operator: UnaryPrefixOperator::Reference(reference),
                    ..
                }) = value
                {
                    context.report(
                        Issue::error("Reference assignment in list syntax is only available in PHP 7.3 and above.")
                            .with_annotation(
                                Annotation::primary(reference.span()).with_message("Reference assignment used here."),
                            )
                            .with_help("Upgrade to PHP 7.3 or later to use reference assignment in list syntax."),
                    );
                }
            }
        }
    }

    fn walk_in_method_call(&self, method_call: &MethodCall, context: &mut Context<'_>) {
        check_for_new_without_parenthesis(&method_call.object, context, "method call");
    }

    fn walk_in_null_safe_method_call(&self, null_safe_call: &NullSafeMethodCall, context: &mut Context<'_>) {
        if !context.version.is_supported(Feature::NullSafeOperator) {
            context.report(
                Issue::error("Nullsafe operator is available in PHP 8.0 and above.")
                    .with_annotation(
                        Annotation::primary(null_safe_call.question_mark_arrow)
                            .with_message("Nullsafe operator used here."),
                    )
                    .with_help("Upgrade to PHP 8.0 or later to use nullsafe method calls."),
            );
        }

        check_for_new_without_parenthesis(&null_safe_call.object, context, "nullsafe method call");
    }

    fn walk_in_property_access(&self, property_access: &PropertyAccess, context: &mut Context<'_>) {
        check_for_new_without_parenthesis(&property_access.object, context, "property access");
    }

    fn walk_in_null_safe_property_access(&self, null_safe_access: &NullSafePropertyAccess, context: &mut Context<'_>) {
        if !context.version.is_supported(Feature::NullSafeOperator) {
            context.report(
                Issue::error("Nullsafe operator is available in PHP 8.0 and above.")
                    .with_annotation(
                        Annotation::primary(null_safe_access.question_mark_arrow)
                            .with_message("Nullsafe operator used here."),
                    )
                    .with_help("Upgrade to PHP 8.0 or later to use nullsafe method calls."),
            );
        }

        check_for_new_without_parenthesis(&null_safe_access.object, context, "nullsafe property access");
    }

    fn walk_in_unary_prefix_operator(&self, unary_prefix_operator: &UnaryPrefixOperator, context: &mut Context<'_>) {
        if !context.version.is_supported(Feature::UnsetCast) {
            if let UnaryPrefixOperator::UnsetCast(span, _) = unary_prefix_operator {
                context.report(
                    Issue::error("The `unset` cast is no longer supported in PHP 8.0 and later.")
                        .with_annotation(Annotation::primary(*span).with_message("Unset cast used here.")),
                );
            }
        }
    }

    fn walk_in_literal_expression(&self, literal_expression: &Literal, context: &mut Context<'_>) {
        if context.version.is_supported(Feature::NumericLiteralSeparator) {
            return;
        }

        let value = match literal_expression {
            Literal::Integer(literal_integer) => &literal_integer.raw,
            Literal::Float(literal_float) => &literal_float.raw,
            _ => return,
        };

        if context.interner.lookup(value).contains('_') {
            context.report(
                Issue::error("Numeric literal separators are only available in PHP 7.4 and later.")
                    .with_annotation(Annotation::primary(literal_expression.span()).with_message("Numeric literal used here."))
                    .with_help("Remove the underscore separators to make the code compatible with PHP 7.3 and earlier versions, or upgrade to PHP 7.4 or later."),
            );
        }
    }

    fn walk_in_class_constant_access(&self, class_constant_access: &ClassConstantAccess, context: &mut Context<'_>) {
        if context.version.is_supported(Feature::AccessClassOnObject) {
            return;
        }

        // If the class is an identifier, static, self, or parent, it's fine.
        if let Expression::Identifier(_) | Expression::Static(_) | Expression::Self_(_) | Expression::Parent(_) =
            class_constant_access.class.as_ref()
        {
            return;
        }

        // If the constant is not an identifier, we don't care.
        let ClassLikeConstantSelector::Identifier(local_identifier) = &class_constant_access.constant else {
            return;
        };

        // If the constant is not `class`, we don't care.
        let value = context.interner.lookup(&local_identifier.value);
        if !value.eq_ignore_ascii_case("class") {
            return;
        }

        context.report(
            Issue::error("Accessing the `class` constant on an object is only available in PHP 8.0 and above.")
                .with_annotation(
                    Annotation::primary(class_constant_access.span()).with_message("`class` constant used here."),
                )
                .with_help("Use `get_class($object)` instead to make the code compatible with PHP 7.4 and earlier versions, or upgrade to PHP 8.0 or later."),
        );
    }

    fn walk_in_constant(&self, constant: &Constant, context: &mut Context<'_>) {
        if !context.version.is_supported(Feature::ConstantAttribute) {
            for attribute_list in constant.attribute_lists.iter() {
                context.report(
                    Issue::error("Constant attributes are only available in PHP 8.5 and above.")
                        .with_annotation(
                            Annotation::primary(attribute_list.span()).with_message("Attribute list used here."),
                        )
                        .with_help("Upgrade to PHP 8.5 or later to use constant attributes."),
                );
            }
        }

        for item in constant.items.iter() {
            if !item.value.is_constant(context.version, true) {
                context.report(
                    Issue::error("Constant value must be a constant expression.")
                        .with_annotation(
                            Annotation::primary(item.value.span()).with_message("This is not a constant expression."),
                        )
                        .with_help("Ensure the constant value is a constant expression."),
                );
            }
        }
    }
}

/// Defines the semantics of magic methods.
///
/// The tuple contains the following elements:
///
/// 1. The name of the magic method.
/// 2. The number of arguments the magic method accepts, or none if it can accept any number of arguments.
/// 3. Whether the magic method has to be public.
/// 4. Whether the magic method has to be static.
/// 5. Whether the magic method can contain a return type.
const MAGIC_METHOD_SEMANTICS: &[(&str, Option<usize>, bool, bool, bool)] = &[
    (CONSTRUCTOR_MAGIC_METHOD, None, false, false, false),
    (DESTRUCTOR_MAGIC_METHOD, None, false, false, false),
    (CLONE_MAGIC_METHOD, None, false, false, true),
    (CALL_MAGIC_METHOD, Some(2), true, false, true),
    (CALL_STATIC_MAGIC_METHOD, Some(2), true, true, true),
    (GET_MAGIC_METHOD, Some(1), true, false, true),
    (SET_MAGIC_METHOD, Some(2), true, false, true),
    (ISSET_MAGIC_METHOD, Some(1), true, false, true),
    (UNSET_MAGIC_METHOD, Some(1), true, false, true),
    (SLEEP_MAGIC_METHOD, Some(0), true, false, true),
    (WAKEUP_MAGIC_METHOD, Some(0), true, false, true),
    (SERIALIZE_MAGIC_METHOD, Some(0), true, false, true),
    (UNSERIALIZE_MAGIC_METHOD, Some(1), true, false, true),
    (TO_STRING_MAGIC_METHOD, Some(0), true, false, true),
    (INVOKE_MAGIC_METHOD, None, true, false, true),
    (SET_STATE_MAGIC_METHOD, Some(1), true, true, true),
    (DEBUG_INFO_MAGIC_METHOD, Some(0), true, false, true),
];

fn returns_generator<'ast>(context: &mut Context<'_>, block: &'ast Block, hint: &'ast Hint) -> bool {
    if hint_contains_generator(context, hint) {
        return true;
    }

    mago_ast_utils::block_has_yield(block)
}

fn hint_contains_generator(context: &mut Context<'_>, hint: &Hint) -> bool {
    match hint {
        Hint::Identifier(identifier) => {
            let symbol = context.lookup_name(&identifier.span().start);

            "generator".eq_ignore_ascii_case(symbol)
        }
        Hint::Parenthesized(parenthesized_hint) => hint_contains_generator(context, &parenthesized_hint.hint),
        Hint::Nullable(nullable_hint) => hint_contains_generator(context, &nullable_hint.hint),
        Hint::Union(union_hint) => {
            hint_contains_generator(context, &union_hint.left) || hint_contains_generator(context, &union_hint.right)
        }
        Hint::Intersection(intersection_hint) => {
            hint_contains_generator(context, &intersection_hint.left)
                || hint_contains_generator(context, &intersection_hint.right)
        }
        _ => false,
    }
}

#[inline]
fn check_for_new_without_parenthesis(object_expr: &Expression, context: &mut Context<'_>, operation: &str) {
    if context.version.is_supported(Feature::NewWithoutParentheses) {
        return;
    }

    let Expression::Instantiation(instantiation) = object_expr else {
        return;
    };

    context.report(
        Issue::error(format!(
            "Direct {operation} on `new` expressions without parentheses is only available in PHP 8.4 and above."
        ))
        .with_annotation(
            Annotation::primary(instantiation.span())
                .with_message(format!("Unparenthesized `new` expression used for {operation}.")),
        ),
    );
}
