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
pub struct NoEmptyConstruct;

impl Rule for NoEmptyConstruct {
    fn get_definition(&self) -> crate::definition::RuleDefinition {
        RuleDefinition::enabled("No Empty Construct", Level::Warning)
            .with_description(indoc! {"
                Detects the use of the `empty()` construct.

                The `empty()` language construct can lead to ambiguous and potentially buggy code due to
                loose and counterintuitive definition of emptiness. It fails to clearly convey
                developer's intent or expectation, making it preferable to use explicit checks.
            "})
            .with_example(RuleUsageExample::invalid(
                "Using the `empty()` construct",
                indoc! {r#"
                    <?php

                    // ...

                    if (!empty($myArray)) {
                        // ...
                    }
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::EmptyConstruct(construct) = node else { return LintDirective::default() };

        let issue = Issue::error("Use of the `empty` construct.")
            .with_annotation(
                Annotation::primary(construct.span()).with_message("Ambigous check due to `empty()` loose semantic."),
            )
            .with_help(indoc! {"
                Use explicit checks instead of `empty()` to clearly convey your intent.

                Specifically, use strict comparison operators and explicit functions like `is_null()`
                or `array_key_exist()`. When handling multiple cases, consider combining these predicates
                with logical operators.
            "});

        context.report(issue);

        LintDirective::Prune
    }
}
