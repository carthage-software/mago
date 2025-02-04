use indoc::indoc;

use mago_ast::*;
use mago_reporting::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct NoRequestVariableRule;

impl Rule for NoRequestVariableRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Request Variable", Level::Error)
            .with_description(indoc! {"
                Detects the use of the `$_REQUEST` variable, which is considered unsafe.

                Use `$_GET`, `$_POST`, or `$_COOKIE` instead for better clarity.
            "})
            .with_example(RuleUsageExample::invalid(
                "Using the `$_REQUEST` variable",
                indoc! {r#"
                    <?php

                    $identifier = $_REQUEST['id'];
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::DirectVariable(direct_variable) = node else { return LintDirective::default() };

        let name = context.interner.lookup(&direct_variable.name);
        if !REQUEST_VARIABLE.eq(name) {
            return LintDirective::Prune;
        }

        context.report(
            Issue::new(context.level(), "Unsafe use of `$_REQUEST` variable.")
                .with_annotation(
                    Annotation::primary(direct_variable.span).with_message("The `$_REQUEST` variable is used here."),
                )
                .with_help("use `$_GET`, `$_POST`, or `$_COOKIE` instead for better clarity."),
        );

        LintDirective::Prune
    }
}

const REQUEST_VARIABLE: &str = "$_REQUEST";
