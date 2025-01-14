use indoc::indoc;

use mago_ast::ast::*;
use mago_reporting::*;
use mago_span::*;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct RequireReturnTypeRule;

impl Rule for RequireReturnTypeRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Require Return Type", Level::Warning)
            .with_description(indoc! {"
                Detects functions, methods, closures, and arrow functions that are missing a return type hint.
            "})
            .with_example(RuleUsageExample::valid(
                "A closure with a return type hint",
                indoc! {r#"
                    <?php

                    $func = function(): int {
                        return 42;
                    };
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "A closure without a return type hint",
                indoc! {r#"
                    <?php

                    $func = function() {
                        return 42;
                    };
                "#},
            ))
    }
}

impl<'a> Walker<LintContext<'a>> for RequireReturnTypeRule {
    fn walk_in_function<'ast>(&self, function: &'ast Function, context: &mut LintContext<'a>) {
        if function.return_type_hint.is_some() {
            return;
        }

        let function_name = context.lookup(&function.name.value);
        let function_fqn = context.lookup_name(&function.name);

        context.report(
            Issue::new(context.level(), format!("Function `{}` is missing a return type hint.", function_name))
                .with_annotation(
                    Annotation::primary(function.span())
                        .with_message(format!("Function `{}` defined here.", function_fqn)),
                )
                .with_note("Type hints improve code readability and help prevent type-related errors.")
                .with_help(format!("Consider adding a return type hint to function `{}`.", function_name)),
        );
    }

    fn walk_in_closure<'ast>(&self, closure: &'ast Closure, context: &mut LintContext<'a>) {
        if closure.return_type_hint.is_some() {
            return;
        }

        context.report(
            Issue::new(context.level(), "Closure is missing a return type hint")
                .with_annotation(Annotation::primary(closure.span()).with_message("Closure defined here."))
                .with_note("Type hints improve code readability and help prevent type-related errors.")
                .with_help("Consider adding a return type hint to the closure."),
        );
    }

    fn walk_in_arrow_function<'ast>(&self, arrow_function: &'ast ArrowFunction, context: &mut LintContext<'a>) {
        if arrow_function.return_type_hint.is_some() {
            return;
        }

        context.report(
            Issue::new(context.level(), "Arrow function is missing a return type hint.")
                .with_annotation(
                    Annotation::primary(arrow_function.span()).with_message("Arrow function defined here."),
                )
                .with_note("Type hints improve code readability and help prevent type-related errors.")
                .with_help("Consider adding a return type hint to the arrow function."),
        );
    }

    fn walk_in_method<'ast>(&self, method: &'ast Method, context: &mut LintContext<'a>) {
        if method.return_type_hint.is_some() {
            return;
        }

        let method_name = context.lookup(&method.name.value);
        if "__construct" == method_name || "__destruct" == method_name {
            // constructors and destructors cannot have return types.
            return;
        }

        context.report(
            Issue::new(context.level(), format!("Method `{}` is missing a return type hint.", method_name))
                .with_annotation(
                    Annotation::primary(method.span()).with_message(format!("Method `{}` defined here", method_name)),
                )
                .with_note("Type hints improve code readability and help prevent type-related errors.")
                .with_help(format!("Consider adding a return type hint to method `{}`.", method_name)),
        );
    }
}
