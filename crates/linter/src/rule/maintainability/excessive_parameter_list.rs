use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct ExcessiveParameterListRule {
    meta: &'static RuleMeta,
    cfg: ExcessiveParameterListConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct ExcessiveParameterListConfig {
    pub level: Level,
    pub threshold: u8,
}

impl Default for ExcessiveParameterListConfig {
    fn default() -> Self {
        Self { level: Level::Error, threshold: 5 }
    }
}

impl Config for ExcessiveParameterListConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for ExcessiveParameterListRule {
    type Config = ExcessiveParameterListConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Excessive Parameter List",
            code: "excessive-parameter-list",
            description: indoc! {r#"
                Detects functions, closures, and methods with too many parameters.

                If the number of parameters exceeds a configurable threshold, an issue is reported.
            "#},
            good_example: indoc! {r#"
                <?php

                function processOrder($orderId, $userId, $total, $status, $date) {
                    return true;
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                function createUser($name, $email, $password, $age, $country, $city, $zipCode) {
                    return true;
                }
            "#},
            category: Category::Maintainability,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::FunctionLikeParameterList];

        TARGETS
    }

    fn build(settings: &RuleSettings<ExcessiveParameterListConfig>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::FunctionLikeParameterList(parameter_list) = node else {
            return;
        };

        let threshold = self.cfg.threshold;

        if parameter_list.parameters.len() as u8 > threshold {
            let issue = Issue::new(self.cfg.level, "Parameter list is too long.".to_string())
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(parameter_list.span()).with_message(format!(
                    "This list has {} parameters, which exceeds the threshold of {}.",
                    parameter_list.parameters.len(),
                    threshold
                )))
                .with_note("Having a large number of parameters can make functions harder to understand and maintain.")
                .with_help("Try reducing the number of parameters, or consider passing an object or a shape instead.");

            ctx.collector.report(issue);
        }
    }
}
