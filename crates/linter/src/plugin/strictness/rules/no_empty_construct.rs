use indoc::indoc;

use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

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

        let issue = Issue::new(context.level(), "Use of the `empty` construct.")
            .with_annotation(
                Annotation::primary(construct.span()).with_message("Ambiguous check due to `empty()` loose semantic."),
            )
            .with_note("`empty()` exhibits unexpected behavior on specific value.")
            .with_note("It is unclear what condition is being treated with `empty()`.")
            .with_help("Use strict comparison or specific predicate function to clearly convey your intent.");

        context.report(issue);

        LintDirective::Prune
    }
}
