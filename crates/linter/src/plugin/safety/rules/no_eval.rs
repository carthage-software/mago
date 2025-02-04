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
pub struct NoEvalRule;

impl Rule for NoEvalRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Eval", Level::Error)
            .with_description(indoc! {"
                Detects unsafe uses of the `eval` construct.

                The `eval` construct executes arbitrary code, which can be a major security risk if not used carefully.
            "})
            .with_example(RuleUsageExample::invalid(
                "An unsafe use of the `eval` construct",
                indoc! {r#"
                    <?php

                    eval('echo "Hello, world!";');
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::EvalConstruct(eval_construct) = node else { return LintDirective::default() };

        context.report(
            Issue::new(context.level(), "Unsafe use of `eval` construct.")
                .with_annotation(Annotation::primary(eval_construct.eval.span).with_message("this `eval` construct is unsafe."))
                .with_annotation(Annotation::secondary(eval_construct.value.span()).with_message("the evaluated code is here."))
                .with_note("The `eval` construct executes arbitrary code, which can be a major security risk if not used carefully.")
                .with_note("It can potentially lead to remote code execution vulnerabilities if the evaluated code is not properly sanitized.")
                .with_note("Consider using safer alternatives whenever possible.")
                .with_help("Avoid using `eval` unless absolutely necessary, and ensure that any dynamically generated code is properly validated and sanitized before execution.")
        );

        LintDirective::Prune
    }
}
