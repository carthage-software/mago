use indoc::indoc;
use schemars::JsonSchema;
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
use crate::scope::FunctionLikeScope;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct ExcessiveParameterListRule {
    meta: &'static RuleMeta,
    cfg: ExcessiveParameterListConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct ExcessiveParameterListConfig {
    pub level: Level,
    pub threshold: u8,
    pub constructor_threshold: Option<u8>,
}

impl Default for ExcessiveParameterListConfig {
    fn default() -> Self {
        Self { level: Level::Error, threshold: 5, constructor_threshold: None }
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
            description: indoc! {r"
                Detects functions, closures, and methods with too many parameters.

                If the number of parameters exceeds a configurable threshold, an issue is reported.
            "},
            good_example: indoc! {r"
                <?php

                function processOrder($orderId, $userId, $total, $status, $date) {
                    return true;
                }
            "},
            bad_example: indoc! {r"
                <?php

                function createUser($name, $email, $password, $age, $country, $city, $zipCode) {
                    return true;
                }
            "},
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

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::FunctionLikeParameterList(parameter_list) = node else {
            return;
        };

        let is_constructor = matches!(
            ctx.scope.get_function_like_scope(),
            Some(FunctionLikeScope::Method(name, _)) if name.eq_ignore_ascii_case("__construct")
        );

        let threshold = if is_constructor {
            self.cfg.constructor_threshold.unwrap_or(self.cfg.threshold)
        } else {
            self.cfg.threshold
        };

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

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::ExcessiveParameterListRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = constructor_within_constructor_threshold,
        rule = ExcessiveParameterListRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.excessive_parameter_list.config.threshold = 3;
            s.rules.excessive_parameter_list.config.constructor_threshold = Some(8);
        },
        code = indoc! {r"
            <?php

            class User {
                public function __construct($a, $b, $c, $d, $e, $f) {}
            }
        "}
    }

    test_lint_failure! {
        name = constructor_exceeds_constructor_threshold,
        rule = ExcessiveParameterListRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.excessive_parameter_list.config.threshold = 3;
            s.rules.excessive_parameter_list.config.constructor_threshold = Some(4);
        },
        code = indoc! {r"
            <?php

            class User {
                public function __construct($a, $b, $c, $d, $e) {}
            }
        "}
    }

    test_lint_failure! {
        name = method_still_uses_regular_threshold,
        rule = ExcessiveParameterListRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.excessive_parameter_list.config.threshold = 3;
            s.rules.excessive_parameter_list.config.constructor_threshold = Some(8);
        },
        code = indoc! {r"
            <?php

            class User {
                public function doSomething($a, $b, $c, $d) {}
            }
        "}
    }

    test_lint_success! {
        name = constructor_falls_back_to_threshold_when_unset,
        rule = ExcessiveParameterListRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.excessive_parameter_list.config.threshold = 6;
        },
        code = indoc! {r"
            <?php

            class User {
                public function __construct($a, $b, $c, $d, $e, $f) {}
            }
        "}
    }

    test_lint_failure! {
        name = constructor_exceeds_fallback_threshold,
        rule = ExcessiveParameterListRule,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.excessive_parameter_list.config.threshold = 3;
        },
        code = indoc! {r"
            <?php

            class User {
                public function __construct($a, $b, $c, $d) {}
            }
        "}
    }
}
