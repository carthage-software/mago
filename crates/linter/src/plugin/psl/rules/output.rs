use indoc::indoc;

use mago_ast::*;
use mago_reporting::*;
use mago_span::HasSpan;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::plugin::psl::rules::utils::format_replacements;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct OutputRule;

impl Rule for OutputRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Output", Level::Error)
            .with_description(indoc! {"
                This rule enforces the usage of Psl output functions over their PHP counterparts.

                Psl output functions are preferred because they are type-safe and provide more consistent behavior.
            "})
            .with_example(RuleUsageExample::valid(
                "Using `Psl\\IO\\write_line` instead of `echo`.",
                indoc! {r#"
                    <?php

                    Psl\IO\write_line("Hello, world!");
                "#},
            ))
            .with_example(RuleUsageExample::valid(
                "Using `Psl\\IO\\write_error_line` instead of `fwrite(STDERR, ...)`.",
                indoc! {r#"
                    <?php

                    Psl\IO\write_error_line("Error message.");
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using `echo`.",
                indoc! {r#"
                    <?php

                    echo "Hello, world!";
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using `fwrite(STDERR, ...)`.",
                indoc! {r#"
                    <?php

                    fwrite(STDERR, "Hello, world!");
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let (used_directive, is_stdout) = match node {
            Node::Echo(_) => ("echo", true),
            Node::PrintConstruct(_) => ("print", true),
            Node::FunctionCall(function_call) => {
                let Expression::Identifier(identifier) = function_call.function.as_ref() else {
                    return LintDirective::default();
                };

                let function_name = context.resolve_function_name(identifier);

                match true {
                    _ if function_name.eq_ignore_ascii_case("printf") => ("printf", true),
                    _ if function_name.eq_ignore_ascii_case("fwrite") => {
                        let Some(argument) = function_call.argument_list.arguments.get(0) else {
                            return LintDirective::default();
                        };

                        let Expression::ConstantAccess(constant) = argument.value() else {
                            return LintDirective::default();
                        };

                        let name = context.resolve_constant_name(&constant.name);

                        match true {
                            _ if name.eq_ignore_ascii_case("STDOUT") => ("fwrite", true),
                            _ if name.eq_ignore_ascii_case("STDERR") => ("fwrite", false),
                            _ => return LintDirective::default(),
                        }
                    }
                    _ => {
                        return LintDirective::default();
                    }
                }
            }
            _ => return LintDirective::default(),
        };

        let replacements = if is_stdout { &STDOUT_FUNCTIONS } else { &STDERR_FUNCTIONS };

        context.report(
            Issue::new(
                context.level(),
                "Use the Psl output function instead of the PHP counterpart.",
            )
            .with_annotation(
                Annotation::primary(node.span()).with_message(format!("Using PHP's `{used_directive}`.")),
            )
            .with_note(
                "Psl output functions are preferred because they are type-safe and provide more consistent behavior.",
            )
            .with_help(format!(
                "Use `{}` instead.",
                format_replacements(replacements)
            )),
        );

        LintDirective::default()
    }
}

const STDOUT_FUNCTIONS: [&str; 2] = ["Psl\\IO\\write", "Psl\\IO\\write_line"];
const STDERR_FUNCTIONS: [&str; 2] = ["Psl\\IO\\write_error", "Psl\\IO\\write_error_line"];
