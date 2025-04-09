use indoc::indoc;

use mago_php_version::PHPVersion;
use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;
use toml::Value;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleOptionDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct RequireStrictBehavior;

pub const ALLOW_LOOSE_BEHAVIOR: &str = "allow-loose-behavior";
pub const ALLOW_LOOSE_BEHAVIOR_DEFAULT: bool = false;

impl Rule for RequireStrictBehavior {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Require Strict Behavior", Level::Warning)
            .with_description(indoc! {"
                Detects functions relying on loose comparison unless the `$strict` parameter is specified.

                The use of loose comparison for these functions may lead to hard-to-debug, unexpected behaviors.
            "})
            .with_option(RuleOptionDefinition {
                name: ALLOW_LOOSE_BEHAVIOR,
                r#type: "boolean",
                description: "Allow explicitly enabling loose behavior by specifying `false` for `$strict` parameter.",
                default: Value::Boolean(ALLOW_LOOSE_BEHAVIOR_DEFAULT),
            })
            .with_minimum_supported_php_version(PHPVersion::PHP70)
            .with_example(RuleUsageExample::invalid(
                "A call to `in_array()` with implicit loose behavior",
                indoc! {r#"
                    <?php

                    in_array(1, ['foo', 'bar', 'baz']);
                "#},
            ))
            .with_example(RuleUsageExample::valid(
                "A call to `in_array()` with explicit strict behavior through named parameter",
                indoc! {r#"
                    <?php

                    in_array(1, ['foo', 'bar', 'baz'], strict: true);
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "A call to `array_search()` with explicit loose behavior through positional parameter",
                indoc! {r#"
                    <?php

                    array_search(true, [0 => 'foo', 1 => 'bar', 2 => 'baz'], false);
                "#},
            ))
            .with_example(RuleUsageExample::valid(
                "A call to `array_search()` with explicit strict behavior through positional parameter",
                indoc! {r#"
                    <?php

                    array_search(true, [0 => 'foo', 1 => 'bar', 2 => 'baz'], true);
                "#},
            ))
            .with_example(
                RuleUsageExample::valid(
                    "A call to `array_search()` with loose behavior option through name parameter",
                    indoc! {r#"
                    <?php

                    array_search(true, [0 => 'foo', 1 => 'bar', 2 => 'baz'], strict: false);
                "#},
                )
                .with_option(ALLOW_LOOSE_BEHAVIOR, Value::Boolean(true)),
            )
            .with_example(
                RuleUsageExample::valid(
                    "A call to `array_search()` with loose behavior option through positional parameter",
                    indoc! {r#"
                    <?php

                    array_search(true, [0 => 'foo', 1 => 'bar', 2 => 'baz'], false);
                "#},
                )
                .with_option(ALLOW_LOOSE_BEHAVIOR, Value::Boolean(true)),
            )
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::FunctionCall(func_call) = node else { return LintDirective::default() };

        let Expression::Identifier(identifier) = func_call.function.as_ref() else {
            return LintDirective::default();
        };

        let function_name = context.resolve_function_name(identifier);

        let expected_position = match function_name {
            "base64_decode" => 1,
            "in_array" | "array_search" => 2,
            "array_keys" if func_call.argument_list.arguments.len() > 1 => 2,
            _ => {
                return LintDirective::default();
            }
        };

        let allow_loose_behavior = context
            .option(ALLOW_LOOSE_BEHAVIOR)
            .and_then(|option| option.as_bool())
            .unwrap_or(ALLOW_LOOSE_BEHAVIOR_DEFAULT);

        let mut correct = false;
        for (position, argument) in func_call.argument_list.arguments.iter().enumerate() {
            match argument {
                Argument::Positional(argument) if position == expected_position => {
                    if matches!(argument.value, Expression::Literal(Literal::True(_)))
                        || (allow_loose_behavior && matches!(argument.value, Expression::Literal(Literal::False(_))))
                    {
                        correct = true;
                        break;
                    }
                }
                Argument::Named(argument) => {
                    let name = context.interner.lookup(&argument.name.value);
                    if name != "strict" {
                        continue;
                    }

                    if matches!(argument.value, Expression::Literal(Literal::True(_)))
                        || (allow_loose_behavior && matches!(argument.value, Expression::Literal(Literal::False(_))))
                    {
                        correct = true;
                        break;
                    }
                }
                _ => {
                    continue;
                }
            }
        }

        if correct {
            return LintDirective::default();
        }

        let mut issue =
            Issue::new(context.level(), format!("Call to `{}` must enforce strict comparison.", function_name))
                .with_annotation(Annotation::primary(identifier.span()).with_message(format!(
                    "Function `{}` relies on loose comparison which can lead to unexpected behavior.",
                    function_name
                )))
                .with_help(format!(
                    "Call the function `{}` with the `$strict` parameter set to `true`.",
                    function_name
                ));

        if allow_loose_behavior {
            issue = issue.with_note(format!(
                "The `{}` option is enabled; you may set the `$strict` parameter to `false`.",
                ALLOW_LOOSE_BEHAVIOR
            ));
        }

        context.report(issue);

        LintDirective::default()
    }
}
