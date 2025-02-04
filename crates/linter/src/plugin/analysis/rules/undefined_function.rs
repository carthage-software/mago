use indoc::indoc;

use mago_ast::*;
use mago_reporting::*;
use mago_span::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct UndefinedFunctionRule;

impl Rule for UndefinedFunctionRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Undefined Function", Level::Error)
            .with_description(indoc! {"
                Flags any calls to functions that are not defined or imported. This often indicates
                a typo, or a function that hasn't been declared yet.
            "})
            .with_example(RuleUsageExample::valid(
                "Calling a defined function",
                indoc! {r#"
                    <?php

                    function greet(): void {
                        echo 'Hello World';
                    }

                    greet(); // Valid: 'greet' is defined
                "#},
            ))
            .with_example(RuleUsageExample::valid(
                "Calling a global function from a namespace",
                indoc! {r#"
                    <?php

                    namespace {
                        // Mock implementation of `strlen` for demonstration purposes
                        function strlen(string $string): int {
                            return 0;
                        }
                    }

                    namespace App {
                        $length = strlen('Hello, world!'); // OK: `strlen` is a global function
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Calling an undefined function",
                indoc! {r#"
                    <?php

                    greet(); // Error: Call to undefined function `greet`.
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::FunctionCall(function_call) = node else { return LintDirective::default() };

        let Expression::Identifier(identifier) = function_call.function.as_ref() else {
            return LintDirective::default();
        };

        let function_name = context.resolve_function_name(identifier);
        let function_name_id = context.interner.intern(function_name);
        if context.codebase.function_exists(context.interner, &function_name_id) {
            return LintDirective::default();
        }

        let issue = Issue::error(format!("Call to undefined function `{}`.", function_name))
            .with_annotation(
                Annotation::primary(identifier.span())
                    .with_message(format!("Function `{}` does not exist.", function_name)),
            )
            .with_help(format!("Ensure the function `{}` is defined or imported before calling it.", function_name));

        context.report(issue);

        LintDirective::Continue
    }
}
