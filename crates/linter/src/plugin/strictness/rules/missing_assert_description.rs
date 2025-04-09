use indoc::indoc;

use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct MissingAssertDescriptionRule;

impl Rule for MissingAssertDescriptionRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Missing Assert Description", Level::Warning)
            .with_description(indoc! {"
                Detects assert functions that do not have a description.
                Assert functions should have a description to make it easier to understand the purpose of the assertion.
            "})
            .with_example(RuleUsageExample::valid(
                "An assert function with a description",
                indoc! {r#"
                    <?php

                    // ...

                    assert($user->isActivated(), 'User MUST be activated at this point.');
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "An assert function without a description",
                indoc! {r#"
                    <?php

                    // ...

                    assert($user->isActivated());
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::FunctionCall(function_call) = node else {
            return LintDirective::default();
        };

        let Expression::Identifier(identifier) = function_call.function.as_ref() else {
            return LintDirective::default();
        };

        let function_name = context.resolve_function_name(identifier);
        // we only care about the "assert" function
        if !function_name.eq_ignore_ascii_case("assert") {
            return LintDirective::default();
        }

        if function_call.argument_list.arguments.get(1).is_none() {
            context.report(
                Issue::new(context.level(), "Missing description in assert function.")
                    .with_annotation(Annotation::primary(function_call.span()).with_message("`assert` function is called here."))
                    .with_help("Add a description to the assert function to make it easier to understand the purpose of the assertion.")
            );
        }

        LintDirective::default()
    }
}
