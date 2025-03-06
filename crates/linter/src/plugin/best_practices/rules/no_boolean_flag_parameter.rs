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
pub struct NoBooleanFlagParameterRule;

impl Rule for NoBooleanFlagParameterRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Boolean Flag Parameter", Level::Help)
            .with_description(indoc! {"
                Flags function-like parameters that use a boolean type.

                Boolean flag parameters can indicate a violation of the Single Responsibility Principle (SRP).
                Refactor by extracting the flag logic into its own class or method.
            "})
            .with_example(RuleUsageExample::valid(
                "Function without a boolean flag parameter",
                indoc! {r#"
                    <?php

                    function get_difference(string $a, string $b): string {
                        // ...
                    }

                    function get_difference_case_insensitive(string $a, string $b): string {
                        // ...
                    }

                    class Example {
                        public function __construct(
                            private bool $flag,
                        ) {}
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Function with a boolean flag parameter",
                indoc! {r#"
                    <?php

                    function get_difference(string $a, string $b, bool $ignoreCase): string {
                        // ...
                    }
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::FunctionLikeParameter(parameter) = node else { return LintDirective::default() };

        // Skip promoted properties
        if parameter.is_promoted_property() {
            return LintDirective::default();
        }

        let Some(Hint::Bool(bool_hint)) = &parameter.hint else { return LintDirective::default() };

        let issue = Issue::new(context.level(), "Avoid using boolean flag parameters")
            .with_annotation(
                Annotation::primary(parameter.variable.span())
                    .with_message("This parameter is declared with a boolean type"),
            )
            .with_annotation(Annotation::secondary(bool_hint.span).with_message("Boolean type declared here"))
            .with_note("Boolean flag parameters can indicate that a function is doing too much.")
            .with_help("Consider extracting the flag behavior into a separate method or class to adhere to SRP.");

        context.report(issue);

        LintDirective::default()
    }
}
