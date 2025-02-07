use indoc::indoc;

use mago_ast::*;
use mago_fixer::SafetyClassification;
use mago_php_version::PHPVersion;
use mago_reporting::*;
use mago_span::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Copy, Debug)]
pub struct ImplicitlyNullableParameterRule;

impl Rule for ImplicitlyNullableParameterRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Implicitly Nullable Parameter", Level::Warning)
            .with_minimum_supported_php_version(PHPVersion::PHP84)
            .with_description(indoc! {"
                Detects parameters that are implicitly nullable and rely on a deprecated feature.
                Such parameters are considered deprecated; an explicit nullable type hint is recommended.
            "})
            .with_example(RuleUsageExample::valid(
                "Using an explicit nullable type hint",
                indoc! {r#"
                    <?php

                    function foo(?string $param) {}

                    function bar(null|string $param) {}

                    function baz(null|object $param = null) {}
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using an implicit nullable parameter",
                indoc! {r#"
                    <?php

                    function foo(string $param = null) {}

                    function bar(string $param = NULL) {}

                    function baz(object $param = null) {}
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::FunctionLikeParameter(function_like_parameter) = node else { return LintDirective::default() };
        let Some(hint) = function_like_parameter.hint.as_ref() else {
            return LintDirective::default();
        };

        if hint.contains_null() {
            return LintDirective::default();
        }

        let Some(default_value) = function_like_parameter.default_value.as_ref() else {
            return LintDirective::default();
        };

        let Expression::Literal(Literal::Null(_)) = default_value.value else {
            return LintDirective::default();
        };

        let parameter_name = context.lookup(&function_like_parameter.variable.name);
        let current_hint = context.get_readable_hint(hint);
        let (prefix, resulting_hint) = match hint {
            Hint::Union(_) => ("null|", format!("null|{}", current_hint)),
            Hint::Intersection(_) => ("null|", format!("null|({})", current_hint)),
            Hint::Parenthesized(_) => ("null|", format!("null|{}", current_hint)),
            _ => ("null|", format!("?{}", current_hint)),
        };

        let issue = Issue::new(
            context.level(),
            format!("Parameter `{}` is implicitly nullable and relies on a deprecated feature.", parameter_name),
        )
        .with_annotation(
            Annotation::primary(function_like_parameter.span())
                .with_message(format!("Parameter `{}` is declared here.", parameter_name)),
        )
        .with_help(format!(
            "Consider using an explicit nullable type hint ( `{}` ) or replacing the default value.",
            resulting_hint
        ))
        .with_note("Updating this will future-proof your code and align it with PHP 8.4 standards.");

        context.propose(issue, |plan| {
            plan.insert(hint.span().start_position().offset, prefix, SafetyClassification::Safe);
        });

        LintDirective::default()
    }
}
