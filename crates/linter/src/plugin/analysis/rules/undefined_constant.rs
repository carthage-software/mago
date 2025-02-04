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
pub struct UndefinedConstantRule;

impl Rule for UndefinedConstantRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Undefined Constant", Level::Error)
            .with_description(indoc! {"
                Checks for usage of constants that have not been defined. This typically occurs
                when a constant is referenced by name (e.g., `FOO`) without being declared via
                `define` or `const` in the same namespace, or imported from another namespace.
            "})
            .with_example(RuleUsageExample::valid(
                "Defining a constant via `const`",
                indoc! {r#"
                    <?php

                    const GREETING = 'Hello, world!';

                    echo GREETING; // Valid
                "#},
            ))
            .with_example(RuleUsageExample::valid(
                "Defining a constant via `define()`",
                indoc! {r#"
                    <?php

                    define('GREETING', 'Hello, world!');

                    echo GREETING; // Valid
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Accessing an undefined constant",
                indoc! {r#"
                    <?php

                    echo GREETING; // Error: Undefined constant `GREETING`
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::ConstantAccess(constant_access) = node else { return LintDirective::default() };

        let identifier = &constant_access.name;
        let constant_name = context.resolve_constant_name(identifier);
        let constant_name_id = context.interner.intern(constant_name);
        if context.codebase.constant_exists(context.interner, &constant_name_id) {
            return LintDirective::default();
        }

        let issue = Issue::error(format!("Use of undefined constant `{}`.", constant_name))
            .with_annotation(
                Annotation::primary(identifier.span())
                    .with_message(format!("Constant `{}` does not exist.", constant_name)),
            )
            .with_help(format!("Ensure the constant `{}` is defined or imported before using it.", constant_name));

        context.report(issue);

        LintDirective::Prune
    }
}
