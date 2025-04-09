use indoc::indoc;

use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::plugin::laravel::rules::utils::is_function_call_to;
use crate::plugin::laravel::rules::utils::is_method_named;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct ViewArrayParameterRule;

impl Rule for ViewArrayParameterRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("View Array Parameter", Level::Help)
            .with_description(indoc! {"
                Prefer passing data to views using the array parameter in the `view()` function,
                rather than chaining the `with()` method.

                Using the array parameter directly is more concise and readable.
            "})
            .with_example(RuleUsageExample::valid(
                "Using `view(..., [...])`",
                indoc! {r#"
                    <?php

                    return view('user.profile', [
                        'user' => $user,
                        'profile' => $profile,
                    ]);
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using `view(...)->with([...])`",
                indoc! {r#"
                    <?php

                    return view('user.profile')->with([
                        'user' => $user,
                        'profile' => $profile,
                    ]);
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::MethodCall(call @ MethodCall { object, method, .. }) = node else {
            return LintDirective::default();
        };

        if !is_function_call_to(context, object.as_ref(), "view") || !is_method_named(context, method, "with") {
            return LintDirective::default();
        }

        context.report(
            Issue::new(context.level(), "Use array parameter in `view()` instead of chaining `with()`.")
                .with_annotation(
                    Annotation::primary(call.span())
                        .with_message("Chaining `with()` here is less readable and idiomatic."),
                )
                .with_note("Passing data directly as an array parameter to `view()` is preferred.")
                .with_help("Refactor the code to use the array parameter in the `view()` function."),
        );

        LintDirective::Prune
    }
}
