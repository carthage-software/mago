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
pub struct NoGlobalRule;

impl Rule for NoGlobalRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Global", Level::Error)
            .with_description(indoc! {"
                Detects the use of the `global` keyword and the `$GLOBALS` variable.

                The `global` keyword introduces global state into your function, making it harder to reason about and test.
            "})
            .with_example(RuleUsageExample::invalid(
                "Using the `global` keyword",
                indoc! {r#"
                    <?php

                    function foo(): void
                    {
                        global $bar;

                        // ...
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using the `$GLOBALS` variable",
                indoc! {r#"
                    <?php

                    function foo(): void
                    {
                        // ...

                        $GLOBALS['bar'] = $value;
                    }
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        match node {
            Node::Global(global) => {
                let mut issue = Issue::new(context.level(), "Unsafe use of `global` keyword.")
                    .with_annotation(Annotation::primary(global.global.span).with_message("This `global` keyword is used here."))
                    .with_note("The `global` keyword introduces global state into your function, making it harder to reason about and test.")
                    .with_note("It can also lead to unexpected behavior and make your code more prone to errors.")
                    .with_note("Consider using dependency injection or other techniques to manage state and avoid relying on global variables.")
                    .with_help("Refactor your code to avoid using the `global` keyword.");

                for variable in global.variables.iter() {
                    issue = issue.with_annotation(
                        Annotation::secondary(variable.span()).with_message("This variable is declared as global."),
                    );
                }

                context.report(issue);

                LintDirective::Prune
            }
            Node::DirectVariable(direct_variable) => {
                let name = context.interner.lookup(&direct_variable.name);
                if !GLOBALS_VARIABLE.eq(name) {
                    return LintDirective::default();
                }

                let issue = Issue::new(context.level(), "Unsafe use of `$GLOBAL` variable.")
                    .with_annotation(Annotation::primary(direct_variable.span).with_message("The `$GLOBALS` variable is used here."))
                    .with_note("Accessing the `$GLOBALS` array directly can lead to similar issues as using the `global` keyword.")
                    .with_note("It can make your code harder to understand, test, and maintain due to the implicit global state.")
                    .with_note("Consider using dependency injection or other techniques to manage state and avoid relying on global variables.")
                    .with_help("Refactor your code to avoid using the `$GLOBALS` variable directly.");

                context.report(issue);

                LintDirective::Prune
            }
            _ => LintDirective::default(),
        }
    }
}

const GLOBALS_VARIABLE: &str = "$GLOBALS";
