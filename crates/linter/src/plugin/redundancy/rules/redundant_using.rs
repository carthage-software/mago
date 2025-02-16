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
pub struct RedundantUsingRule;

impl Rule for RedundantUsingRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Redundant Using", Level::Help)
            .with_description(indoc! {"
                Detects redundant using statements.
            "})
            .with_example(RuleUsageExample::invalid(
                "A redundant using statement with noop",
                indoc! {r#"
                    <?php

                    using($a = "hello");
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "A redundant using statement with an empty block",
                indoc! {r#"
                    <?php

                    using($a = "hello") {}
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Using(using) = node else {
            return LintDirective::default();
        };

        if is_statement_useful(&using.statement) {
            return LintDirective::default();
        }

        context.report(
            Issue::new(context.level(), "Redundant using statements")
                .with_annotation(Annotation::primary(using.span()).with_message("This using statement is redundant."))
                .with_help("Remove this using statement or replace it with an `unset` statement."),
        );

        LintDirective::default()
    }
}

#[inline]
fn is_statement_useful(statement: &'_ Statement) -> bool {
    match statement {
        Statement::Block(block) => block.statements.iter().any(is_statement_useful),
        Statement::Noop(_) => false,
        _ => true, // we can do better here.
    }
}
