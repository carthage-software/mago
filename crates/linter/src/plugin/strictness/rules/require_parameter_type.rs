use indoc::indoc;
use toml::Value;

use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::*;
use mago_php_version::PHPVersion;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleOptionDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

pub const IGNORE_PARAMETER_TYPE_FOR_CLOSURE: &str = "ignore_closure";
pub const IGNORE_PARAMETER_TYPE_FOR_CLOSURE_DEFAULT: bool = false;
pub const IGNORE_PARAMETER_TYPE_FOR_ARROW_FUNCTION: &str = "ignore_arrow_function";
pub const IGNORE_PARAMETER_TYPE_FOR_ARROW_FUNCTION_DEFAULT: bool = false;

#[derive(Clone, Debug)]
pub struct RequireParameterTypeRule;

impl Rule for RequireParameterTypeRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Require Parameter Type", Level::Warning)
            .with_minimum_supported_php_version(PHPVersion::PHP70)
            .with_description(indoc! {"
                Detects parameters that are missing a type hint.
            "})
            .with_option(RuleOptionDefinition {
                name: IGNORE_PARAMETER_TYPE_FOR_CLOSURE,
                r#type: "boolean",
                description: "Whether to ignore parameters in closures.",
                default: Value::Boolean(IGNORE_PARAMETER_TYPE_FOR_CLOSURE_DEFAULT),
            })
            .with_option(RuleOptionDefinition {
                name: IGNORE_PARAMETER_TYPE_FOR_ARROW_FUNCTION,
                r#type: "boolean",
                description: "Whether to ignore parameters in arrow functions.",
                default: Value::Boolean(IGNORE_PARAMETER_TYPE_FOR_ARROW_FUNCTION_DEFAULT),
            })
            .with_example(RuleUsageExample::valid(
                "A function with a parameter that has a type hint",
                indoc! {r#"
                    <?php

                    function foo(string $bar): void
                    {
                        // ...
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "A function with a parameter that is missing a type hint",
                indoc! {r#"
                    <?php

                    function foo($bar): void
                    {
                        // ...
                    }
                "#},
            ))
            .with_example(RuleUsageExample::valid(
                "A Closure with a parameter that has a type hint",
                indoc! {r#"
                    <?php

                    $func = function (string $bar): void {
                        // ...
                    };
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "A Closure with a parameter that is missing a type hint",
                indoc! {r#"
                    <?php

                    $func = function ($bar): void {
                        // ...
                    };
                "#},
            ))
            .with_example(
                RuleUsageExample::valid(
                    "A Closure with a parameter that is missing a type hint, but the rule is configured to ignore it",
                    indoc! {r#"
                    <?php

                    $func = function ($bar): void {
                        // ...
                    };
                "#},
                )
                .with_option(IGNORE_PARAMETER_TYPE_FOR_CLOSURE, Value::Boolean(true)),
            )
            .with_example(RuleUsageExample::valid(
                "An arrow function with a parameter that has a type hint",
                indoc! {r#"
                    <?php

                    $func = fn(string $bar): string => $bar;
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "An arrow function with a parameter that is missing a type hint",
                indoc! {r#"<?php

                    $func = fn($bar): string => $bar;
                "#},
            ))
            .with_example(
                RuleUsageExample::valid(
                    "An arrow function with a parameter that is missing a type hint, but the rule is configured to ignore it",
                    indoc! {r#"
                    <?php

                    $func = fn($bar): string => $bar;
                "#},
                )
                .with_option(IGNORE_PARAMETER_TYPE_FOR_ARROW_FUNCTION, Value::Boolean(true)),
            )
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        match node {
            Node::Function(function) => {
                for parameter in function.parameter_list.parameters.iter() {
                    check_function_like_parameter(parameter, context);
                }
            }
            Node::Closure(closure) => {
                let ignore_parameter_type_for_closure = context
                    .option(IGNORE_PARAMETER_TYPE_FOR_CLOSURE)
                    .and_then(|o| o.as_bool())
                    .unwrap_or(IGNORE_PARAMETER_TYPE_FOR_CLOSURE_DEFAULT);

                if ignore_parameter_type_for_closure {
                    return LintDirective::Abort;
                }

                for parameter in closure.parameter_list.parameters.iter() {
                    check_function_like_parameter(parameter, context);
                }
            }
            Node::ArrowFunction(arrow_function) => {
                let ignore_parameter_type_for_arrow_function = context
                    .option(IGNORE_PARAMETER_TYPE_FOR_ARROW_FUNCTION)
                    .and_then(|o| o.as_bool())
                    .unwrap_or(IGNORE_PARAMETER_TYPE_FOR_ARROW_FUNCTION_DEFAULT);

                if ignore_parameter_type_for_arrow_function {
                    return LintDirective::Abort;
                }

                for parameter in arrow_function.parameter_list.parameters.iter() {
                    check_function_like_parameter(parameter, context);
                }
            }
            Node::Interface(interface) => {
                let name = context.resolved_names.get(&interface.name);
                let Some(metadata) = get_interface(context.codebase, context.interner, name) else {
                    return LintDirective::default();
                };

                check_class_like_members(metadata, interface.members.as_slice(), context);
            }
            Node::Class(class) => {
                let name = context.resolved_names.get(&class.name);
                let Some(metadata) = get_class(context.codebase, context.interner, name) else {
                    return LintDirective::default();
                };

                check_class_like_members(metadata, class.members.as_slice(), context);
            }
            Node::Enum(r#enum) => {
                let name = context.resolved_names.get(&r#enum.name);
                let Some(metadata) = get_enum(context.codebase, context.interner, name) else {
                    return LintDirective::default();
                };

                check_class_like_members(metadata, r#enum.members.as_slice(), context);
            }
            Node::Trait(r#trait) => {
                let name = context.resolved_names.get(&r#trait.name);
                let Some(metadata) = get_trait(context.codebase, context.interner, name) else {
                    return LintDirective::default();
                };

                check_class_like_members(metadata, r#trait.members.as_slice(), context);
            }
            _ => (),
        }

        LintDirective::default()
    }
}

#[inline]
fn check_function_like_parameter(function_like_parameter: &FunctionLikeParameter, context: &mut LintContext<'_>) {
    if function_like_parameter.hint.is_some() {
        return;
    }

    let parameter_name = context.lookup(&function_like_parameter.variable.name);

    context.report(
        Issue::new(context.level(), format!("Parameter `{parameter_name}` is missing a type hint."))
            .with_annotation(
                Annotation::primary(function_like_parameter.span())
                    .with_message(format!("Parameter `{parameter_name}` is declared here")),
            )
            .with_note("Type hints improve code readability and help prevent type-related errors.")
            .with_help(format!("Consider adding a type hint to parameter `{parameter_name}`.")),
    );
}

#[inline]
fn check_class_like_members(metadata: &ClassLikeMetadata, members: &[ClassLikeMember], context: &mut LintContext<'_>) {
    for member in members {
        let ClassLikeMember::Method(method) = member else {
            continue;
        };

        if is_method_overriding(context.codebase, context.interner, &metadata.name, &method.name.value) {
            continue;
        }

        for parameter in method.parameter_list.parameters.iter() {
            check_function_like_parameter(parameter, context);
        }
    }
}
