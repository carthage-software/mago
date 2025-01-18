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
pub struct VariadicFunctionsFeatureRule;

impl Rule for VariadicFunctionsFeatureRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Variadic Functions Feature", Level::Error)
            .with_maximum_supported_php_version(PHPVersion::PHP55)
            .with_description(indoc! {"
                Flags any usage of variadic functions, which were introduced in PHP 5.6.

                In environments running older versions of PHP, you can use the `func_get_args()` function instead.
            "})
            .with_example(RuleUsageExample::valid(
                "Using `func_get_args()` instead of variadic functions",
                indoc! {r#"
                    <?php

                    function sum() {
                        $args = func_get_args();
                        return array_sum($args);
                    }

                    echo sum(1, 2, 3);
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using variadic functions",
                indoc! {r#"
                    <?php

                    function sum(...$args) {
                        return array_sum($args);
                    }

                    echo sum(1, 2, 3);
                "#},
            ))
    }
}

impl<'a> Walker<LintContext<'a>> for VariadicFunctionsFeatureRule {
    fn walk_in_function<'ast>(&self, function: &'ast Function, context: &mut LintContext<'a>) {
        for param in function.parameters.parameters.iter() {
            if param.ellipsis.is_none() {
                continue;
            }

            let issue = Issue::new(
                context.level(),
                "Variadic functions are only available in PHP 5.6 and later.",
            )
            .with_annotation(
                Annotation::primary(param.span()).with_message("Variadic parameter used here."),
            )
            .with_note("Use `func_get_args()` if you need compatibility with older PHP versions.");

            context.report(issue);
        }
    }
}
