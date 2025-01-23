use indoc::indoc;

use mago_ast::ast::*;
use mago_php_version::PHPVersion;
use mago_reporting::*;
use mago_span::*;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::rule::Rule;

#[derive(Clone, Copy, Debug)]
pub struct FinallyFeatureRule;

impl Rule for FinallyFeatureRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Finally Feature", Level::Error)
            .with_maximum_supported_php_version(PHPVersion::PHP54)
            .with_description(indoc! {"
                Flags any usage of the `finally` keyword, which was introduced in PHP 5.5.

                In environments running older versions of PHP, you can use a `try`/`catch` block without a `finally` block instead.
            "})
            .with_example(RuleUsageExample::valid(
                "Using a `try`/`catch` block without a `finally` block",
                indoc! {r#"
                    <?php

                    try {
                        // Code that might throw an exception.
                    } catch (Exception $e) {
                        // Handle the exception.
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using a `try`/`catch` block with a `finally` block",
                indoc! {r#"
                    <?php

                    try {
                        // Code that might throw an exception.
                    } catch (Exception $e) {
                        // Handle the exception.
                    } finally {
                        // Code that should always run.
                    }
                "#},
            ))
    }
}

impl<'a> Walker<LintContext<'a>> for FinallyFeatureRule {
    fn walk_in_try<'ast>(&self, r#try: &'ast Try, context: &mut LintContext<'a>) {
        let Some(finally) = r#try.finally_clause.as_ref() else {
            return;
        };

        let issue = Issue::new(
            context.level(),
            "The `finally` block is only available in PHP 5.5 and later.",
        )
        .with_annotation(
            Annotation::primary(finally.span()).with_message("Finally block used here."),
        );

        context.report(issue);
    }
}
