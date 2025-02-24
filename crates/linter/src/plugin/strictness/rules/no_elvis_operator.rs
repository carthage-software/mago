use indoc::indoc;

use mago_ast::*;
use mago_reporting::*;
use mago_span::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct NoElvisOperator;

impl Rule for NoElvisOperator {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Elvis Operator", Level::Warning)
            .with_description(indoc! {"
                Detects the use of the shorthand ternary operator (`?:`).

                The shorthand ternary operator (`?:`), aka elvis operator, relies on loose comparison.
            "})
            .with_example(RuleUsageExample::invalid(
                "Using the `?:` operator",
                indoc! {r#"
                    <?php

                    $value = $foo ?: $default;
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let expression @ Node::BinaryOperator(BinaryOperator::Elvis(_)) = node else { return LintDirective::default() };

        let issue = Issue::error("Use of the short-ternary operator (`?:`).")
            .with_annotation(
                Annotation::primary(expression.span()).with_message("Ambigous check due to `?:` loose comparison."),
            )
            .with_help("Use null coalesce operator (`??`) or ternary operator with explicit strict comparison.");
        context.report(issue);

        LintDirective::Prune
    }
}
