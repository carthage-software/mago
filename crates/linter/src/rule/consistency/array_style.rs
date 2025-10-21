use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_fixer::SafetyClassification;
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
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct ArrayStyleRule {
    meta: &'static RuleMeta,
    cfg: ArrayStyleConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArrayStyleOption {
    Short,
    Long,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct ArrayStyleConfig {
    pub level: Level,
    pub style: ArrayStyleOption,
}

impl Default for ArrayStyleConfig {
    fn default() -> Self {
        Self { level: Level::Note, style: ArrayStyleOption::Short }
    }
}

impl Config for ArrayStyleConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for ArrayStyleRule {
    type Config = ArrayStyleConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Array Style",
            code: "array-style",
            description: indoc! {"
                Suggests using the short array style `[..]` instead of the long array style `array(..)`,
                or vice versa, depending on the configuration. The short array style is more concise and
                is the preferred way to define arrays in PHP.
            "},
            good_example: indoc! {r#"
                <?php

                // By default, `style` is 'short', so this snippet is valid:
                $arr = [1, 2, 3];
            "#},
            bad_example: indoc! {r#"
                <?php

                // By default, 'short' is enforced, so array(...) triggers a warning:
                $arr = array(1, 2, 3);
            "#},
            category: Category::Consistency,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::LegacyArray, NodeKind::Array];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        match node {
            Node::LegacyArray(arr) if ArrayStyleOption::Short == self.cfg.style => {
                let issue = Issue::new(self.cfg.level(), "Short array style `[..]` is preferred over `array(..)`.")
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(arr.span())
                            .with_message("This array uses the long array style `array(..)`"),
                    )
                    .with_help("Use the short array style `[..]` instead.");

                ctx.collector.propose(issue, |plan| {
                    plan.delete(arr.array.span.to_range(), SafetyClassification::Safe);
                    plan.replace(arr.left_parenthesis.to_range(), "[", SafetyClassification::Safe);
                    plan.replace(arr.right_parenthesis.to_range(), "]", SafetyClassification::Safe);
                });
            }
            Node::Array(arr) if ArrayStyleOption::Long == self.cfg.style => {
                let issue = Issue::new(self.cfg.level(), "Long array style `array(..)` is preferred over `[..]`.")
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(arr.span()).with_message("This array uses the short array style `[..]`"),
                    )
                    .with_help("Use the long array style `array(..)` instead.");

                ctx.collector.propose(issue, |plan| {
                    plan.replace(arr.left_bracket.to_range(), "array(", SafetyClassification::Safe);
                    plan.replace(arr.right_bracket.to_range(), ")", SafetyClassification::Safe);
                });
            }
            _ => {}
        }
    }
}
