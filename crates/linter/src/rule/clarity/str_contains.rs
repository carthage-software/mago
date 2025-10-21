use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_fixer::SafetyClassification;
use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
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
use crate::rule::utils::call::function_call_matches;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

const STR_CONTAINS: &str = "str_contains";
const STRPOS: &str = "strpos";

#[derive(Debug, Clone)]
pub struct StrContainsRule {
    meta: &'static RuleMeta,
    cfg: StrContainsConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct StrContainsConfig {
    pub level: Level,
}

impl Default for StrContainsConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for StrContainsConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for StrContainsRule {
    type Config = StrContainsConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Str Contains",
            code: "str-contains",
            description: indoc! {"
                Detects `strpos($a, $b) !== false` comparisons and suggests replacing them with `str_contains($a, $b)`
                for improved readability and intent clarity.
            "},
            good_example: indoc! {r#"
                <?php

                $a = 'hello world';
                $b = 'world';

                if (str_contains($a, $b)) {
                    echo 'Found';
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                $a = 'hello world';
                $b = 'world';

                if (strpos($a, $b) !== false) {
                    echo 'Found';
                }
            "#},
            category: Category::Clarity,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP80)),
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
        let Node::Binary(binary) = node else { return };

        if !matches!(
            binary.operator,
            BinaryOperator::NotIdentical(_) | BinaryOperator::NotEqual(_) | BinaryOperator::AngledNotEqual(_)
        ) {
            return;
        }

        let (left, call) = match (binary.lhs, binary.rhs) {
            (
                Expression::Call(Call::Function(call @ FunctionCall { argument_list: arguments, .. })),
                Expression::Literal(Literal::False(_)),
            ) if arguments.arguments.len() == 2 => (true, call),
            (
                Expression::Literal(Literal::False(_)),
                Expression::Call(Call::Function(call @ FunctionCall { argument_list: arguments, .. })),
            ) if arguments.arguments.len() == 2 => (false, call),
            _ => {
                return;
            }
        };

        if !function_call_matches(ctx, call, STRPOS) {
            return;
        }

        let issue = Issue::new(
            self.cfg.level,
            "Consider replacing `strpos` with `str_contains` for improved readability and intent clarity.",
        )
        .with_code(self.meta.code)
        .with_annotation(Annotation::primary(binary.span()).with_message("This comparison can be simplified."))
        .with_help("`strpos($a, $b) !== false` can be simplified to `str_contains($a, $b)`.")
        .with_note("Using `str_contains` makes the code easier to understand and more expressive.");

        ctx.collector.propose(issue, |plan| {
            let function_span = call.function.span();

            // Replace `strpos` with `str_contains`
            plan.replace(function_span.to_range(), STR_CONTAINS.to_string(), SafetyClassification::Safe);

            // Remove `!== false` part
            if left {
                plan.delete(binary.operator.span().join(binary.rhs.span()).to_range(), SafetyClassification::Safe);
            } else {
                plan.delete(binary.lhs.span().join(binary.operator.span()).to_range(), SafetyClassification::Safe);
            }
        });
    }
}
