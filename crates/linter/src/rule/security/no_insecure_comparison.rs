use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::*;

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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
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
            description: indoc! {r#"
                Detects insecure comparison of passwords or tokens using `==`, `!=`, `===`, or `!==`.

                These operators are vulnerable to timing attacks, which can expose sensitive information.
                Instead, use `hash_equals` for comparing strings or `password_verify` for validating hashes.
            "#},
            good_example: indoc! {r#"
                <?php

                if (hash_equals($storedToken, $userToken)) {
                    // Valid token
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                if ($storedToken == $userToken) {
                    // Vulnerable to timing attacks
                }
            "#},
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

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Binary(binary) = node else {
            return;
        };

        if !binary.operator.is_equality() {
            return;
        }

        let lhs = get_password(binary.lhs);
        let rhs = get_password(binary.rhs);

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

#[inline]
#[must_use]
const fn is_simple_literal<'ast, 'arena>(expr: &'ast Expression<'arena>) -> bool {
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
