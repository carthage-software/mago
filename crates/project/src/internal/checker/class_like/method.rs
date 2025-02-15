#![allow(clippy::too_many_arguments)]

use mago_ast::*;
use mago_reporting::*;
use mago_span::*;

use crate::internal::checker::function_like::check_for_promoted_properties_outside_constructor;
use crate::internal::checker::returns_generator;
use crate::internal::checker::MAGIC_METHOD_SEMANTICS;
use crate::internal::context::Context;

#[inline]
pub fn check_method(
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
                    context.issues.push(
                        Issue::error(format!(
                            "duplicate `static` modifier on method `{}::{}`",
                            class_like_name, method_name
                        ))
                        .with_annotation(
                            Annotation::primary(modifier.span()).with_message("duplicate `static` modifier"),
                        )
                        .with_annotation(Annotation::primary(last_static).with_message("previous `static` modifier"))
                        .with_annotation(
                            Annotation::secondary(method.span())
                                .with_message(format!("method `{}::{}` defined here.", class_like_name, method_name,)),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` is defined here.", class_like_kind, class_like_fqcn)),
                        ),
                    );
                }

                last_static = Some(modifier.span());
            }
            Modifier::Final(_) => {
                if let Some(abstract_modifier) = last_abstract {
                    context.issues.push(
                        Issue::error(format!(
                            "method `{}::{}` cannot be both `final` and `abstract`",
                            class_like_name, method_name
                        ))
                        .with_annotation(Annotation::primary(modifier.span()).with_message("`final` modifier"))
                        .with_annotation(Annotation::primary(abstract_modifier).with_message("`abstract` modifier"))
                        .with_annotation(
                            Annotation::secondary(method.span())
                                .with_message(format!("method `{}::{}` defined here.", class_like_name, method_name,)),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` is defined here.", class_like_kind, class_like_fqcn)),
                        ),
                    );
                }

                if let Some(last_final) = last_final {
                    context.issues.push(
                        Issue::error(format!(
                            "duplicate `final` modifier on method `{}::{}`",
                            class_like_name, method_name
                        ))
                        .with_annotation(
                            Annotation::primary(modifier.span()).with_message("duplicate `final` modifier"),
                        )
                        .with_annotation(Annotation::primary(last_final).with_message("previous `final` modifier"))
                        .with_annotation(
                            Annotation::secondary(method.span())
                                .with_message(format!("method `{}::{}` defined here.", class_like_name, method_name,)),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` is defined here.", class_like_kind, class_like_fqcn)),
                        ),
                    );
                }

                last_final = Some(modifier.span());
            }
            Modifier::Abstract(_) => {
                if let Some(final_modifier) = last_final {
                    context.issues.push(
                        Issue::error(format!(
                            "method `{}::{}` cannot be both `final` and `abstract`",
                            class_like_name, method_name
                        ))
                        .with_annotation(Annotation::primary(modifier.span()).with_message("`abstract` modifier"))
                        .with_annotation(Annotation::primary(final_modifier).with_message("`final` modifier"))
                        .with_annotation(
                            Annotation::secondary(method.span())
                                .with_message(format!("method `{}::{}` defined here.", class_like_name, method_name,)),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` is defined here.", class_like_kind, class_like_fqcn)),
                        ),
                    );
                }

                if let Some(last_abstract) = last_abstract {
                    context.issues.push(
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
                            Annotation::secondary(method.span())
                                .with_message(format!("method `{}::{}` defined here.", class_like_name, method_name,)),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` is defined here.", class_like_kind, class_like_fqcn)),
                        ),
                    );
                }

                last_abstract = Some(modifier.span());
            }
            Modifier::Readonly(_) => {
                context.issues.push(
                    Issue::error("`readonly` modifier is not allowed on methods".to_string())
                        .with_annotation(Annotation::primary(modifier.span()).with_message("`readonly` modifier"))
                        .with_annotation(
                            Annotation::secondary(method.span())
                                .with_message(format!("method `{}::{}` defined here.", class_like_name, method_name,)),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` is defined here.", class_like_kind, class_like_fqcn)),
                        ),
                );
            }
            Modifier::Private(_) | Modifier::Protected(_) | Modifier::Public(_) => {
                if let Some(last_visibility) = last_visibility {
                    context.issues.push(
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
                            Annotation::secondary(method.span())
                                .with_message(format!("method `{}::{}` defined here.", class_like_name, method_name,)),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` is defined here.", class_like_kind, class_like_fqcn)),
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

                context.issues.push(
                    Issue::error(format!("`{}` modifier is not allowed on methods", modifier_name))
                        .with_annotation(
                            Annotation::primary(modifier.span()).with_message(format!("`{}` modifier", modifier_name)),
                        )
                        .with_annotation(
                            Annotation::secondary(method.span())
                                .with_message(format!("method `{}::{}` defined here.", class_like_name, method_name,)),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` is defined here.", class_like_kind, class_like_fqcn)),
                        ),
                );
            }
        }
    }

    for (magic_method, parameter_count, must_be_public, must_be_static, can_have_return_type) in MAGIC_METHOD_SEMANTICS
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
                                "Magic method `{}::{}` must have exactly {} parameters, found more than {} due to variadic parameter.",
                                class_like_name,
                                method_name,
                                count,
                                found_count
                            )
                    } else {
                        format!(
                            "Magic method `{}::{}` must have exactly {} parameters, found {}.",
                            class_like_name, method_name, count, found_count
                        )
                    };

                    context.issues.push(
                        Issue::error(message)
                            .with_annotation(Annotation::primary(method.parameter_list.span()))
                            .with_annotation(
                                Annotation::secondary(method.span()).with_message(format!(
                                    "Method `{}::{}` defined here.",
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

            if *must_be_public && !is_public {
                context.issues.push(
                    Issue::error(format!("Magic method `{}::{}` must be public.", class_like_name, method_name))
                        .with_annotation(
                            Annotation::primary(last_visibility.unwrap())
                                .with_message("Non-Public visibility modifier."),
                        )
                        .with_annotation(
                            Annotation::secondary(method.span())
                                .with_message(format!("Method `{}::{}` defined here.", class_like_name, method_name,)),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` is defined here.", class_like_kind, class_like_fqcn)),
                        ),
                );
            }

            match last_static.as_ref() {
                Some(span) if !*must_be_static => {
                    context.issues.push(
                        Issue::error(format!("Magic method `{}::{}` cannot be static.", class_like_name, method_name))
                            .with_annotation(Annotation::primary(*span).with_message("`static` modifier"))
                            .with_annotation(
                                Annotation::secondary(method.span()).with_message(format!(
                                    "Method `{}::{}` defined here.",
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
                    context.issues.push(
                        Issue::error(format!("Magic method `{}::{}` must be static.", class_like_name, method_name))
                            .with_annotation(Annotation::primary(method.name.span()))
                            .with_annotation(
                                Annotation::secondary(class_like_span)
                                    .with_message(format!("{} `{}`", class_like_kind, class_like_fqcn)),
                            )
                            .with_annotation(
                                Annotation::secondary(method.span()).with_message(format!(
                                    "Method `{}::{}` defined here.",
                                    class_like_name, method_name,
                                )),
                            ),
                    );
                }
                _ => {}
            }

            if !*can_have_return_type {
                if let Some(hint) = &method.return_type_hint {
                    context.issues.push(
                        Issue::error(format!(
                            "Magic method `{}::{}` cannot have a return type hint.",
                            class_like_name, method_name
                        ))
                        .with_annotation(Annotation::primary(hint.span()))
                        .with_annotation(
                            Annotation::secondary(method.span())
                                .with_message(format!("Method `{}::{}` defined here.", class_like_name, method_name,)),
                        )
                        .with_annotation(
                            Annotation::secondary(class_like_span)
                                .with_message(format!("{} `{}` is defined here.", class_like_kind, class_like_fqcn)),
                        ),
                    );
                }
            }
        }
    }

    let mut is_abstract = false;
    match &method.body {
        MethodBody::Abstract(method_abstract_body) => {
            if !class_like_is_interface && last_abstract.is_none() {
                context.issues.push(
                    Issue::error(format!(
                        "Non-Abstract method `{}::{}` must have a concrete body.",
                        class_like_name, method_name,
                    ))
                    .with_annotation(Annotation::primary(method_abstract_body.span()))
                    .with_annotations([
                        Annotation::secondary(class_like_span)
                            .with_message(format!("{} `{}` is defined here.", class_like_kind, class_like_fqcn)),
                        Annotation::secondary(method.span())
                            .with_message(format!("Method `{}::{}` defined here.", class_like_name, method_name)),
                    ]),
                );
            }

            is_abstract = true;
        }
        MethodBody::Concrete(body) => {
            if let Some(abstract_modifier) = last_abstract {
                is_abstract = true;

                context.issues.push(
                    Issue::error(format!(
                        "Method `{}::{}` is abstract and cannot have a concrete body.",
                        class_like_name, method_name,
                    ))
                    .with_annotation(Annotation::primary(body.span()))
                    .with_annotations([
                        Annotation::primary(abstract_modifier.span()),
                        Annotation::secondary(class_like_span)
                            .with_message(format!("{} `{}` is defined here.", class_like_kind, class_like_fqcn)),
                        Annotation::secondary(method.span())
                            .with_message(format!("Method `{}::{}` defined here.", class_like_name, method_name)),
                    ]),
                );
            } else if class_like_is_interface {
                context.issues.push(
                    Issue::error(format!(
                        "Interface method `{}::{}` is implicitly abstract and cannot have a concrete body.",
                        class_like_name, method_name,
                    ))
                    .with_annotation(Annotation::primary(body.span()))
                    .with_annotations([
                        Annotation::secondary(class_like_span)
                            .with_message(format!("{} `{}` is defined here.", class_like_kind, class_like_fqcn)),
                        Annotation::secondary(method.span())
                            .with_message(format!("Method `{}::{}` defined here.", class_like_name, method_name)),
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
                            context.issues.push(
                                Issue::error(format!(
                                    "Method `{}::{}` with return type of `void` must not return a value.",
                                    class_like_name, method_name,
                                ))
                                .with_annotation(Annotation::primary(val.span()))
                                .with_annotations([
                                    Annotation::secondary(class_like_span).with_message(format!(
                                        "{} `{}` is defined here.",
                                        class_like_kind, class_like_fqcn
                                    )),
                                    Annotation::secondary(method.span()).with_message(format!(
                                        "Method `{}::{}` defined here.",
                                        class_like_name, method_name,
                                    )),
                                ])
                                .with_help("Remove the return type hint, or remove the return value."),
                            );
                        }
                    }
                }
                Hint::Never(_) => {
                    for r#return in returns {
                        context.issues.push(
                            Issue::error(format!(
                                "Function `{}::{}` with return type of `never` must not return.",
                                class_like_name, method_name,
                            ))
                            .with_annotation(Annotation::primary(r#return.span()))
                            .with_annotations([
                                Annotation::secondary(class_like_span).with_message(format!(
                                    "{} `{}` is defined here.",
                                    class_like_kind, class_like_fqcn
                                )),
                                Annotation::secondary(method.span()).with_message(format!(
                                    "Method `{}::{}` defined here.",
                                    class_like_name, method_name,
                                )),
                            ])
                            .with_help("Remove the return type hint, or remove the return statement."),
                        );
                    }
                }
                _ if !returns_generator(context, body, hint) => {
                    for r#return in returns {
                        if r#return.value.is_none() {
                            context.issues.push(
                                Issue::error(format!(
                                    "Method `{}::{}` with return type must return a value.",
                                    class_like_name, method_name,
                                ))
                                .with_annotation(Annotation::primary(r#return.span()))
                                .with_annotations([
                                    Annotation::secondary(class_like_span).with_message(format!(
                                        "{} `{}` is defined here.",
                                        class_like_kind, class_like_fqcn
                                    )),
                                    Annotation::secondary(method.span()).with_message(format!(
                                        "Method `{}::{}` defined here.",
                                        class_like_name, method_name,
                                    )),
                                ])
                                .with_note("Did you mean `return null;` instead of `return;`?")
                                .with_help("Add a return value to the statement."),
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    };

    if !method_name.eq_ignore_ascii_case("__construct") {
        check_for_promoted_properties_outside_constructor(&method.parameter_list, context);
    } else if is_abstract {
        for parameter in method.parameter_list.parameters.iter() {
            if parameter.is_promoted_property() {
                context.issues.push(
                    Issue::error("Promoted properties are not allowed in abstract constructors.")
                        .with_annotation(
                            Annotation::primary(parameter.span()).with_message("Promoted property used here."),
                        )
                        .with_help(
                            "Remove the promoted property from the constructor or make the constructor concrete.",
                        ),
                );
            }
        }
    }
}
