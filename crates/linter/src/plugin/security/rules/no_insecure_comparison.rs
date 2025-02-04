use indoc::indoc;

use mago_ast::*;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::plugin::security::rules::utils::get_password;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct NoInsecureComparisonRule;

impl Rule for NoInsecureComparisonRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Insecure Comparison", Level::Error)
            .with_description(indoc! {r#"
                Detects insecure comparison of passwords or tokens using `==`, `!=`, `===`, or `!==`.
                These operators are vulnerable to timing attacks, which can expose sensitive information.
                Instead, use `hash_equals` for comparing strings or `password_verify` for validating hashes.
            "#})
            .with_example(RuleUsageExample::valid(
                "Secure password comparison using `hash_equals`",
                indoc! {r#"
                    <?php

                    if (hash_equals($storedToken, $userToken)) {
                        // Valid token
                    }
                "#},
            ))
            .with_example(RuleUsageExample::valid(
                "Secure password validation using `password_verify`",
                indoc! {r#"
                    <?php

                    if (password_verify($userPassword, $storedHash)) {
                        // Valid password
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Insecure password comparison using `==`",
                indoc! {r#"
                    <?php

                    if ($storedToken == $userToken) {
                        // Vulnerable to timing attacks
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Insecure password comparison using `===`",
                indoc! {r#"
                    <?php

                    if ($input === $user->getToken()) {
                        // Vulnerable to timing attacks
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Insecure password comparison with array access",
                indoc! {r#"
                    <?php

                    if ($credentials['token'] === $user->getToken()) {
                        // Vulnerable to timing attacks
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Insecure password comparison with object property",
                indoc! {r#"
                    <?php

                    if ($user->token === $storedToken) {
                        // Vulnerable to timing attacks
                    }
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Binary(binary) = node else { return LintDirective::default() };

        if !binary.operator.is_equality() {
            return LintDirective::default();
        }

        let lhs = get_password(context, &binary.lhs);
        let rhs = get_password(context, &binary.rhs);

        let is_lhs_like_password = lhs.is_some();
        let is_rhs_like_password = rhs.is_some();

        // Skip the check if neither side is a password-like value
        if !is_lhs_like_password && !is_rhs_like_password {
            return LintDirective::default();
        }

        // Skip the check if  one side is a password-like value and the other side
        //  is a simple literal (e.g. a number, a boolean, null)
        if (is_lhs_like_password && is_simple_literal(context, &binary.rhs))
            || (is_rhs_like_password && is_simple_literal(context, &binary.lhs))
        {
            return LintDirective::Prune;
        }

        let mut issue = Issue::new(context.level(), "Insecure comparison of sensitive data.")
            .with_annotation(
                Annotation::primary(binary.operator.span()).with_message("This is the comparison operator."),
            )
            .with_note("The `==`, `!=`, `===`, and `!==` operators are vulnerable to timing attacks when comparing sensitive data.")
            .with_help("Use `hash_equals` for comparing strings or `password_verify` for validating hashes.");

        if let Some(span) = lhs {
            issue = issue.with_annotation(Annotation::secondary(span).with_message("This is sensitive data."));
        }

        if let Some(span) = rhs {
            issue = issue.with_annotation(Annotation::secondary(span).with_message("This is sensitive data."));
        }

        context.report(issue);

        LintDirective::Prune
    }
}

#[inline]
#[must_use]
fn is_simple_literal(context: &mut LintContext, expr: &Expression) -> bool {
    match expr {
        Expression::Parenthesized(parenthesized) => is_simple_literal(context, &parenthesized.expression),
        Expression::Literal(literal) => {
            if let Literal::String(literal_string) = literal {
                let value = context.interner.lookup(&literal_string.value);

                value.len() == 2 // empty string
            } else {
                true
            }
        }
        _ => false,
    }
}
