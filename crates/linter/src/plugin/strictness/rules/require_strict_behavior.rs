use indoc::indoc;

use mago_ast::*;
use mago_php_version::PHPVersion;
use mago_reporting::*;
use mago_span::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct RequireStrictBehavior;

impl Rule for RequireStrictBehavior {
    fn get_definition(&self) -> crate::definition::RuleDefinition {
        RuleDefinition::enabled("Require Strict Behavior", Level::Warning)
            .with_description(indoc! {"
                Detects functions relying on loose comparison unless the `$strict` parameter is specified.

                The use of loose comparison for these functions may lead to hard-to-debug, unexpected behaviors.
            "})
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

        let mut is_strict = false;

        for (position, argument) in func_call.argument_list.arguments.iter().enumerate() {
            match argument {
                Argument::Positional(argument) if position == expected_position => {
                    if matches!(argument.value, Expression::Literal(Literal::True(_))) {
                        is_strict = true;
                        break;
                    }
                }
                Argument::Named(argument) => {
                    let name = context.interner.lookup(&argument.name.value);
                    if name != "strict" {
                        continue;
                    }
                    if matches!(argument.value, Expression::Literal(Literal::True(_))) {
                        is_strict = true;
                        break;
                    }
                }
                _ => {
                    continue;
                }
            }
        }

        if is_strict {
            return LintDirective::default();
        }

        let issue = Issue::error(format!("Call to `{}` must enforce strict comparison.", function_name))
            .with_annotation(Annotation::primary(identifier.span()).with_message(format!(
                "Function `{}` relies on loose comparison which can lead to unexpected behavior.",
                function_name
            )))
            .with_help(format!("Call the function `{}` with the `$strict` parameter set to `true`.", function_name));
        context.report(issue);

        LintDirective::default()
    }
}
