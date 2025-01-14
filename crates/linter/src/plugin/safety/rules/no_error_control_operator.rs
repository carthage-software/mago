use indoc::indoc;

use mago_ast::*;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct NoErrorControlOperatorRule;

impl Rule for NoErrorControlOperatorRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Error Control Operator", Level::Error)
            .with_description(indoc! {"
                Detects the use of the error control operator `@`.

                The error control operator suppresses errors and makes debugging more difficult.
            "})
            .with_example(RuleUsageExample::invalid(
                "An unsafe use of the error control operator `@`",
                indoc! {r#"
                    <?php

                    $result = @file_get_contents('example.txt');
                "#},
            ))
    }
}

impl<'a> Walker<LintContext<'a>> for NoErrorControlOperatorRule {
    fn walk_in_unary_prefix<'ast>(&self, unary_prefix: &'ast UnaryPrefix, context: &mut LintContext<'a>) {
        if let UnaryPrefixOperator::ErrorControl(_) = unary_prefix.operator {
            let issue = Issue::new(context.level(), "Unsafe use of error control operator `@`.")
                .with_annotation(
                    Annotation::primary(unary_prefix.operator.span()).with_message("This operator suppresses errors."),
                )
                .with_annotation(
                    Annotation::secondary(unary_prefix.operand.span())
                        .with_message("This expression is being suppressed."),
                )
                .with_note("Error control operator hide potential errors and make debugging more difficult.")
                .with_help("Remove the `@` and use `set_error_handler` to handle errors instead.");

            context.report_with_fix(issue, |plan| {
                plan.delete(unary_prefix.operator.span().to_range(), SafetyClassification::Safe)
            });
        }
    }
}
