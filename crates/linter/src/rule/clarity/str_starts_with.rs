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

const STR_STARTS_WITH: &str = "str_starts_with";
const STRPOS: &str = "strpos";

#[derive(Debug, Clone)]
pub struct StrStartsWithRule {
    meta: &'static RuleMeta,
    cfg: StrStartsWithConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct StrStartsWithConfig {
    pub level: Level,
}

impl Default for StrStartsWithConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for StrStartsWithConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for StrStartsWithRule {
    type Config = StrStartsWithConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Str Starts With",
            code: "str-starts-with",
            description: indoc! {"
                Detects `strpos($a, $b) === 0` comparisons and suggests replacing them with `str_starts_with($a, $b)`
                for improved readability and intent clarity.
            "},
            good_example: indoc! {"
                <?php

                $a = 'hello world';
                $b = 'hello';
                if (str_starts_with($a, $b)) {
                    echo 'Found';
                }
            "},
            bad_example: indoc! {"
                <?php

                $a = 'hello world';
                $b = 'hello';
                if (strpos($a, $b) === 0) {
                    echo 'Found';
                }
            "},
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
        let Node::Binary(binary) = node else {
            return;
        };

        let equal = match binary.operator {
            BinaryOperator::Identical(_) | BinaryOperator::Equal(_) => true,
            BinaryOperator::AngledNotEqual(_) | BinaryOperator::NotEqual(_) | BinaryOperator::NotIdentical(_) => false,
            _ => {
                return;
            }
        };

        // if one side is `0` and the other is a `strpos($a, $b)` call, we can suggest using `str_starts_with($a, $b)`
        let (left, call) = match (binary.lhs, binary.rhs) {
            (
                Expression::Literal(Literal::Integer(LiteralInteger { value: Some(0), .. })),
                Expression::Call(Call::Function(call @ FunctionCall { argument_list: arguments, .. })),
            ) if arguments.arguments.len() == 2 => (false, call),
            (
                Expression::Call(Call::Function(call @ FunctionCall { argument_list: arguments, .. })),
                Expression::Literal(Literal::Integer(LiteralInteger { value: Some(0), .. })),
            ) if arguments.arguments.len() == 2 => (true, call),
            _ => {
                return;
            }
        };

        if !function_call_matches(ctx, call, STRPOS) {
            return;
        }

        let issue = Issue::new(
            self.cfg.level,
            "Consider replacing `strpos` with `str_starts_with` for improved readability and intent clarity.",
        )
        .with_code(self.meta.code)
        .with_annotation(Annotation::secondary(binary.span()).with_message("This expression can be simplified."))
        .with_help("`strpos($a, $b) === 0` can be simplified to `str_starts_with($a, $b)`.")
        .with_note("Using `str_starts_with` makes the code easier to understand and more expressive.");

        ctx.collector.propose(issue, |plan| {
            if !equal {
                plan.insert(binary.span().start_position().offset, "!", SafetyClassification::Safe);
            }

            let function_span = call.function.span();

            plan.replace(function_span.to_range(), STR_STARTS_WITH.to_string(), SafetyClassification::Safe);

            if left {
                // delete the `=== 0` part
                plan.delete(binary.operator.span().join(binary.rhs.span()).to_range(), SafetyClassification::Safe);
            } else {
                // delete the `0 ===` part
                plan.delete(binary.lhs.span().join(binary.operator.span()).to_range(), SafetyClassification::Safe);
            }
        });
    }
}
