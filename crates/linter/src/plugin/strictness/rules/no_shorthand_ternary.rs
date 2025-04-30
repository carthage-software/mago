use indoc::indoc;

use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct NoShorthandTernary;

impl Rule for NoShorthandTernary {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Shorthand Ternary", Level::Warning)
            .with_description(indoc! {"
                Detects the use of the shorthand ternary and elvis operators.

                Both shorthand ternary operator (`$a ? : $b`) and elvis operator (`$a ?: $b`) relies on loose comparison.
            "})
            .with_example(RuleUsageExample::invalid(
                "Using the elvis operator (`?:`)",
                indoc! {r#"
                    <?php

                    $value = $foo ?: $default;
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using the shorthand ternary operator (`? :`)",
                indoc! {r#"
                    <?php

                    $value = $foo ? : $default;
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let issue = match node {
            Node::BinaryOperator(BinaryOperator::Elvis(_)) => Issue::new(context.level(), "Use of the elvis operator.")
                .with_annotation(
                    Annotation::primary(node.span()).with_message("Ambiguous check due to `?:` loose comparison."),
                ),
            Node::Conditional(Conditional { then: None, .. }) => {
                Issue::new(context.level(), "Use of the shorthand ternary operator.").with_annotation(
                    Annotation::primary(node.span()).with_message("Ambiguous check due to `? :` loose comparison."),
                )
            }
            _ => return LintDirective::default(),
        };

        context.report(
            issue.with_help("Use null coalesce operator (`??`) or ternary operator with explicit strict comparison."),
        );

        LintDirective::Prune
    }
}
