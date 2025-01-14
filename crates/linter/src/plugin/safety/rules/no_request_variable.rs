use indoc::indoc;

use mago_ast::*;
use mago_reporting::*;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::rule::Rule;

const REQUEST_VARIABLE: &str = "$_REQUEST";

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
}

impl<'a> Walker<LintContext<'a>> for NoRequestVariableRule {
    fn walk_in_direct_variable<'ast>(&self, direct_variable: &'ast DirectVariable, context: &mut LintContext<'a>) {
        let name = context.interner.lookup(&direct_variable.name);
        if !REQUEST_VARIABLE.eq(name) {
            return;
        }

        let issue = Issue::new(context.level(), "Unsafe use of `$_REQUEST` variable.")
            .with_annotation(
                Annotation::primary(direct_variable.span).with_message("The `$_REQUEST` variable is used here."),
            )
            .with_help("use `$_GET`, `$_POST`, or `$_COOKIE` instead for better clarity.");

        context.report(issue);
    }
}
