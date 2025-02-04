use indoc::indoc;

use mago_ast::*;
use mago_reporting::*;
use mago_span::HasSpan;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct NoShellExecuteStringRule;

impl Rule for NoShellExecuteStringRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Shell Execute String", Level::Error)
            .with_description(indoc! {"
                Detects the use of shell execute strings (`...`) in PHP code.
            "})
            .with_example(RuleUsageExample::invalid(
                "Using a shell execute string",
                indoc! {r#"
                    <?php

                    $output = `ls -l`;
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::ShellExecuteString(shell_execute_string) = node else { return LintDirective::default() };

        let mut is_interpolated = false;
        for part in shell_execute_string.parts.iter() {
            if !matches!(part, StringPart::Literal(..)) {
                is_interpolated = true;

                break;
            }
        }

        let issue = if is_interpolated {
            Issue::new(context.level(), "Unsafe use of interpolated shell execute string.")
                        .with_annotation(Annotation::primary(shell_execute_string.span()).with_message("This shell execute string is interpolated."))
                        .with_note("Interpolating shell execute strings (`...`) is a potential security vulnerability, as it allows executing arbitrary shell commands.")
                        .with_help(
                            "Consider using `shell_exec()` along with `escapeshellarg()` or `escapeshellcmd()` to escape arguments instead."
                        )
        } else {
            Issue::new(context.level(), "Potentilly unsafe use of shell execute string.")
                .with_annotation(
                    Annotation::primary(shell_execute_string.span()).with_message("Shell execute string used here."),
                )
                .with_note("Shell execute strings (`...`) can often be replaced with safer alternatives.")
                .with_help("Consider using `shell_exec()` instead.")
        };

        context.report(issue);

        LintDirective::Abort
    }
}
