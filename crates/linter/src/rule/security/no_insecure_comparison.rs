use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Literal;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::security::get_password;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoInsecureComparisonRule {
    meta: &'static RuleMeta,
    cfg: NoInsecureComparisonConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct NoInsecureComparisonConfig {
    pub level: Level,
}

impl Default for NoInsecureComparisonConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for NoInsecureComparisonConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoInsecureComparisonRule {
    type Config = NoInsecureComparisonConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Insecure Comparison",
            code: "no-insecure-comparison",
            description: indoc! {r"
                Detects insecure comparison of passwords or tokens using `==`, `!=`, `===`, or `!==`.

                These operators are vulnerable to timing attacks, which can expose sensitive information.
                Instead, use `hash_equals` for comparing strings or `password_verify` for validating hashes.

                Sensitive data is identified by names, such as variables, array keys, and properties.
                String literal values such as `'refresh_token'` do not by themselves indicate sensitive data.
            "},
            good_example: indoc! {r"
                <?php

                if (hash_equals($storedToken, $userToken)) {
                    // Valid token
                }
            "},
            bad_example: indoc! {r"
                <?php

                if ($storedToken == $userToken) {
                    // Vulnerable to timing attacks
                }
            "},
            category: Category::Security,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Binary];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    #[allow(clippy::similar_names)]
    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Binary(binary) = node else {
            return;
        };

        if !binary.operator.is_equality() {
            return;
        }

        let lhs = get_sensitive_value(binary.lhs);
        let rhs = get_sensitive_value(binary.rhs);

        let is_lhs_like_password = lhs.is_some();
        let is_rhs_like_password = rhs.is_some();

        if !is_lhs_like_password && !is_rhs_like_password {
            return;
        }

        if (is_lhs_like_password && is_simple_literal(binary.rhs))
            || (is_rhs_like_password && is_simple_literal(binary.lhs))
        {
            return;
        }

        let mut issue = Issue::new(self.cfg.level(), "Insecure comparison of sensitive data.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(binary.operator.span()).with_message("This is the comparison operator."),
            )
            .with_note(
                "The `==`, `!=`, `===`, and `!==` operators are vulnerable to timing attacks when comparing sensitive data.",
            )
            .with_help("Use `hash_equals` for comparing strings or `password_verify` for validating hashes.");

        if let Some(span) = lhs {
            issue = issue.with_annotation(Annotation::secondary(span).with_message("This is sensitive data."));
        }

        if let Some(span) = rhs {
            issue = issue.with_annotation(Annotation::secondary(span).with_message("This is sensitive data."));
        }

        ctx.collector.report(issue);
    }
}

/// Distinguishes string values from names such as array keys and dynamic selectors.
#[inline]
#[must_use]
fn get_sensitive_value(expr: &Expression<'_>) -> Option<Span> {
    match expr {
        Expression::Parenthesized(parenthesized) => get_sensitive_value(parenthesized.expression),
        Expression::Assignment(assignment) => {
            get_sensitive_value(assignment.lhs).or_else(|| get_sensitive_value(assignment.rhs))
        }
        Expression::Literal(_) => None,
        _ => get_password(expr),
    }
}

#[inline]
#[must_use]
const fn is_simple_literal(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::Parenthesized(parenthesized) => is_simple_literal(parenthesized.expression),
        Expression::Literal(literal) => {
            if let Literal::String(literal_string) = literal {
                literal_string.raw.len() == 2
            } else {
                true
            }
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoInsecureComparisonRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = issue_2028_grant_type_comparison,
        rule = NoInsecureComparisonRule,
        code = indoc! {r"
            <?php

            declare(strict_types=1);

            namespace App;

            $grantType = 'someValue';
            if ($grantType === 'refresh_token') {
                echo 'Refresh Token selected';
            }
        "}
    }

    test_lint_success! {
        name = string_literals_are_not_sensitive_values,
        rule = NoInsecureComparisonRule,
        code = indoc! {r#"
            <?php

            $request['grant_type'] == 'refresh_token';
            $request['grant_type'] != 'refresh_token';
            $request['grant_type'] === "refresh_token";
            $request['grant_type'] !== "refresh_token";
            'refresh_token' === $grantType;
            ('refresh_token') === ($grantType);
            $grantType === 'password';
            $kind === 'secret';
            $kind === 'apikey';
            $kind === 'api_key';
        "#}
    }

    test_lint_success! {
        name = assigned_literals_are_not_sensitive_values,
        rule = NoInsecureComparisonRule,
        code = indoc! {r"
            <?php

            ($grantType = 'refresh_token') === $expected;
            $expected !== ($grantType = ('refresh_token'));
            ($grantType = $other = 'password') == $expected;
        "}
    }

    test_lint_failure! {
        name = sensitive_variable_comparisons,
        rule = NoInsecureComparisonRule,
        count = 8,
        code = indoc! {r"
            <?php

            $storedToken == $userToken;
            $storedToken != $userToken;
            $storedToken === $userToken;
            $storedToken !== $userToken;
            $password === 'hardcoded';
            'hardcoded' !== ($password);
            ($password = 'hardcoded') === $provided;
            $provided === ($value = $password);
        "}
    }

    test_lint_failure! {
        name = sensitive_names_in_keys_and_selectors,
        rule = NoInsecureComparisonRule,
        count = 12,
        code = indoc! {r"
            <?php

            $data['refresh_token'] === $provided;
            $provided === $data[('password')];
            $object->password === $provided;
            $object?->password === $provided;
            $object->{'password'} === $provided;
            $object?->{'password'} === $provided;
            $object->getToken() === $provided;
            $object->{'getToken'}() === $provided;
            Credentials::getToken() === $provided;
            Credentials::$password === $provided;
            Credentials::PASSWORD === $provided;
            ${'password'} === $provided;
        "}
    }

    test_lint_success! {
        name = simple_literal_checks_and_secure_comparisons,
        rule = NoInsecureComparisonRule,
        code = indoc! {r#"
            <?php

            $password === '';
            "" !== $token;
            ($password) === (null);
            false === $token;
            $password == 0;
            $password != 0.0;
            hash_equals($storedToken, $userToken);
            password_verify($password, $hash);
        "#}
    }
}
