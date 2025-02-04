use indoc::indoc;
use toml::Value;

use mago_ast::*;
use mago_php_version::PHPVersion;
use mago_reporting::*;
use mago_span::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleOptionDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

pub const IGNORE_RETURN_TYPE_FOR_CLOSURE: &str = "ignore_closure";
pub const IGNORE_RETURN_TYPE_FOR_CLOSURE_DEFAULT: bool = false;
pub const IGNORE_RETURN_TYPE_FOR_ARROW_FUNCTION: &str = "ignore_arrow_function";
pub const IGNORE_RETURN_TYPE_FOR_ARROW_FUNCTION_DEFAULT: bool = false;

#[derive(Clone, Debug)]
pub struct RequireReturnTypeRule;

impl Rule for RequireReturnTypeRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Require Return Type", Level::Warning)
            .with_minimum_supported_php_version(PHPVersion::PHP70)
            .with_description(indoc! {"
                Detects functions, methods, closures, and arrow functions that are missing a return type hint.
            "})
            .with_option(RuleOptionDefinition {
                name: IGNORE_RETURN_TYPE_FOR_CLOSURE,
                r#type: "boolean",
                description: "Whether to ignore return types in closures.",
                default: Value::Boolean(IGNORE_RETURN_TYPE_FOR_CLOSURE_DEFAULT),
            })
            .with_option(RuleOptionDefinition {
                name: IGNORE_RETURN_TYPE_FOR_ARROW_FUNCTION,
                r#type: "boolean",
                description: "Whether to ignore return types in arrow functions.",
                default: Value::Boolean(IGNORE_RETURN_TYPE_FOR_ARROW_FUNCTION_DEFAULT),
            })
            .with_example(RuleUsageExample::valid(
                "A closure with a return type hint",
                indoc! {r#"
                    <?php

                    $func = function(): int {
                        return 42;
                    };
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "A closure without a return type hint",
                indoc! {r#"
                    <?php

                    $func = function() {
                        return 42;
                    };
                "#},
            ))
            .with_example(
                RuleUsageExample::valid(
                    "A closure without a return type hint, but with the ignore option set",
                    indoc! {r#"
                    <?php

                    $func = function() {
                        return 42;
                    };
                "#},
                )
                .with_option(IGNORE_RETURN_TYPE_FOR_CLOSURE, Value::Boolean(true)),
            )
            .with_example(RuleUsageExample::valid(
                "An arrow function with a return type hint",
                indoc! {r#"
                    <?php

                    $func = fn(): int => 42;
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "An arrow function without a return type hint",
                indoc! {r#"
                    <?php

                    $func = fn() => 42;
                "#},
            ))
            .with_example(
                RuleUsageExample::valid(
                    "An arrow function without a return type hint, but with the ignore option set",
                    indoc! {r#"
                    <?php

                    $func = fn() => 42;
                "#},
                )
                .with_option(IGNORE_RETURN_TYPE_FOR_ARROW_FUNCTION, Value::Boolean(true)),
            )
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        match node {
            Node::Function(function) => {
                if function.return_type_hint.is_some() {
                    return LintDirective::default();
                }

                let function_name = context.lookup(&function.name.value);
                let function_fqn = context.lookup_name(&function.name);

                context.report(
                    Issue::new(context.level(), format!("Function `{}` is missing a return type hint.", function_name))
                        .with_annotation(
                            Annotation::primary(function.span())
                                .with_message(format!("Function `{}` defined here.", function_fqn)),
                        )
                        .with_note("Type hints improve code readability and help prevent type-related errors.")
                        .with_help(format!("Consider adding a return type hint to function `{}`.", function_name)),
                );
            }
            Node::Closure(closure) => {
                if closure.return_type_hint.is_some() {
                    return LintDirective::default();
                }

                let ignore_return_type_for_closure = context
                    .option(IGNORE_RETURN_TYPE_FOR_CLOSURE)
                    .and_then(|o| o.as_bool())
                    .unwrap_or(IGNORE_RETURN_TYPE_FOR_CLOSURE_DEFAULT);

                if ignore_return_type_for_closure {
                    return LintDirective::default();
                }

                context.report(
                    Issue::new(context.level(), "Closure is missing a return type hint")
                        .with_annotation(Annotation::primary(closure.span()).with_message("Closure defined here."))
                        .with_note("Type hints improve code readability and help prevent type-related errors.")
                        .with_help("Consider adding a return type hint to the closure."),
                );
            }
            Node::ArrowFunction(arrow_function) => {
                if arrow_function.return_type_hint.is_some() {
                    return LintDirective::default();
                }

                let ignore_return_type_for_arrow_function = context
                    .option(IGNORE_RETURN_TYPE_FOR_ARROW_FUNCTION)
                    .and_then(|o| o.as_bool())
                    .unwrap_or(IGNORE_RETURN_TYPE_FOR_ARROW_FUNCTION_DEFAULT);

                if ignore_return_type_for_arrow_function {
                    return LintDirective::default();
                }

                context.report(
                    Issue::new(context.level(), "Arrow function is missing a return type hint.")
                        .with_annotation(
                            Annotation::primary(arrow_function.span()).with_message("Arrow function defined here."),
                        )
                        .with_note("Type hints improve code readability and help prevent type-related errors.")
                        .with_help("Consider adding a return type hint to the arrow function."),
                );
            }
            Node::Method(method) => {
                if method.return_type_hint.is_some() {
                    return LintDirective::default();
                }

                let method_name = context.lookup(&method.name.value);
                if "__construct" == method_name || "__destruct" == method_name {
                    // constructors and destructors cannot have return types.
                    return LintDirective::default();
                }

                context.report(
                    Issue::new(context.level(), format!("Method `{}` is missing a return type hint.", method_name))
                        .with_annotation(
                            Annotation::primary(method.span())
                                .with_message(format!("Method `{}` defined here", method_name)),
                        )
                        .with_note("Type hints improve code readability and help prevent type-related errors.")
                        .with_help(format!("Consider adding a return type hint to method `{}`.", method_name)),
                );
            }
            _ => (),
        }

        LintDirective::default()
    }
}
