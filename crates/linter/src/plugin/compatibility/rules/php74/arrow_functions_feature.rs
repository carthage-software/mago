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
pub struct ArrowFunctionsFeatureRule;

impl Rule for ArrowFunctionsFeatureRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Arrow Functions Feature", Level::Error)
            .with_maximum_supported_php_version(PHPVersion::PHP74)
            .with_description(indoc! {"
                Flags any usage of the `fn` keyword for arrow functions, which was introduced in PHP 7.4.

                In environments running older versions of PHP, you can use an anonymous function.
            "})
            .with_example(RuleUsageExample::valid(
                "Using an anonymous function with `use` keyword",
                indoc! {r#"
                    <?php

                    $y = 2;

                    // Works in all PHP versions (pre-7.4 included):
                    $fn = function ($x) use ($y) {
                        return $x + $y;
                    };
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using the `fn` keyword for arrow functions",
                indoc! {r#"
                    <?php

                    $y = 2;

                    // Only valid in PHP 7.4+:
                    $fn = fn ($x) => $x + $y;
                "#},
            ))
    }
}

impl<'a> Walker<LintContext<'a>> for ArrowFunctionsFeatureRule {
    fn walk_in_arrow_function<'ast>(&self, arrow_function: &'ast ArrowFunction, context: &mut LintContext<'a>) {
        let issue =
            Issue::new(context.level(), "The `fn` keyword for arrow functions is only available in PHP 7.4 and later.")
                .with_annotation(
                    Annotation::primary(arrow_function.span()).with_message("Arrow function uses `fn` keyword."),
                );

        context.report(issue);
    }
}
