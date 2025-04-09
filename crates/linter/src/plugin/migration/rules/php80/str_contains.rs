use indoc::indoc;

use mago_fixer::SafetyClassification;
use mago_php_version::PHPVersion;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

const STR_CONTAINS: &str = "str_contains";
const STRPOS: &str = "strpos";

#[derive(Clone, Debug)]
pub struct StrContainsRule;

impl Rule for StrContainsRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Str Contains", Level::Warning)
            .with_minimum_supported_php_version(PHPVersion::PHP80)
            .with_description(indoc! {"
                Detects `strpos($a, $b) !== false` comparisons and suggests replacing them with `str_contains($a, $b)`
                for improved readability and intent clarity.
            "})
            .with_example(RuleUsageExample::valid(
                "Using `str_contains` instead of `strpos`",
                indoc! {r#"
                    <?php

                    $a = 'hello world';
                    $b = 'world';

                    if (str_contains($a, $b)) {
                        echo 'Found';
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using `strpos` comparison",
                indoc! {r#"
                    <?php

                    $a = 'hello world';
                    $b = 'world';

                    if (strpos($a, $b) !== false) {
                        echo 'Found';
                    }
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Binary(binary) = node else { return LintDirective::default() };

        // Detect `strpos($a, $b) !== false`
        if !matches!(
            binary.operator,
            BinaryOperator::NotIdentical(_) | BinaryOperator::NotEqual(_) | BinaryOperator::AngledNotEqual(_)
        ) {
            return LintDirective::default();
        }

        let (left, call) = match (binary.lhs.as_ref(), binary.rhs.as_ref()) {
            (
                Expression::Call(Call::Function(call @ FunctionCall { argument_list: arguments, .. })),
                Expression::Literal(Literal::False(_)),
            ) if arguments.arguments.len() == 2 => (true, call),
            (
                Expression::Literal(Literal::False(_)),
                Expression::Call(Call::Function(call @ FunctionCall { argument_list: arguments, .. })),
            ) if arguments.arguments.len() == 2 => (false, call),
            _ => {
                return LintDirective::default();
            }
        };

        let Expression::Identifier(function_identifier) = call.function.as_ref() else {
            return LintDirective::default();
        };

        let function_name = context.resolve_function_name(function_identifier);
        if !function_name.eq_ignore_ascii_case(STRPOS) {
            return LintDirective::default();
        }

        let issue = Issue::new(
            context.level(),
            "Consider replacing `strpos` with `str_contains` for improved readability and intent clarity.",
        )
        .with_annotation(Annotation::primary(binary.span()).with_message("This comparison can be simplified."))
        .with_help("`strpos($a, $b) !== false` can be simplified to `str_contains($a, $b)`.")
        .with_note("Using `str_contains` makes the code easier to understand and more expressive.");

        context.propose(issue, |plan| {
            let function_span = function_identifier.span();

            // Replace `strpos` with `str_contains`
            plan.replace(function_span.to_range(), STR_CONTAINS.to_string(), SafetyClassification::Safe);

            // Remove `!== false` part
            if left {
                plan.delete(binary.operator.span().join(binary.rhs.span()).to_range(), SafetyClassification::Safe);
            } else {
                plan.delete(binary.lhs.span().join(binary.operator.span()).to_range(), SafetyClassification::Safe);
            }
        });

        LintDirective::Prune
    }
}
