use indoc::indoc;

use mago_ast::*;
use mago_ast_utils::control_flow::find_control_flows_in_block;
use mago_ast_utils::control_flow::ControlFlow;
use mago_reporting::*;
use mago_span::HasSpan;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct NoUnsafeFinallyRule;

impl Rule for NoUnsafeFinallyRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Unsafe Finally", Level::Error)
            .with_description(indoc! {"
                Detects control flow statements in `finally` blocks.

                Control flow statements in `finally` blocks override control flows from `try` and `catch` blocks,
                leading to unexpected behavior.
            "})
            .with_example(RuleUsageExample::invalid(
                "A control flow statement in a `finally` block",
                indoc! {r#"
                    <?php

                    function example(): int
                    {
                        try {
                            return 1;
                        } finally {
                            throw new Exception();
                        }
                    }
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Try(r#try) = node else { return LintDirective::default() };

        let Some(finally) = r#try.finally_clause.as_ref() else {
            return LintDirective::default();
        };

        for control_flow in find_control_flows_in_block(&finally.block) {
            let kind = match control_flow {
                ControlFlow::Return(_) => "return",
                ControlFlow::Throw(_) => "throw",
                ControlFlow::Continue(_) => "continue",
                ControlFlow::Break(_) => "break",
            };

            let issue = Issue::new(context.level(), "Unsafe control flow in finally block.")
                .with_annotation(
                    Annotation::primary(control_flow.span())
                        .with_message(format!("Control flow statement `{}` in `finally` block.", kind)),
                )
                .with_annotation(
                    Annotation::secondary(r#try.span())
                        .with_message("This `finally` block is associated with this `try` block."),
                )
                .with_note(
                    "Control flow statements in `finally` blocks override control flows from `try` and `catch` blocks, leading to unexpected behavior.",
                )
                .with_help("Avoid using control flow statements in `finally` blocks.");

            context.report(issue);
        }

        LintDirective::default()
    }
}
