use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::*;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct OptionalParamOrderRule {
    meta: &'static RuleMeta,
    cfg: OptionalParamOrderConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct OptionalParamOrderConfig {
    pub level: Level,
}

impl Default for OptionalParamOrderConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for OptionalParamOrderConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for OptionalParamOrderRule {
    type Config = OptionalParamOrderConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Optional Parameter Before Required",
            code: "optional-param-order",
            description: indoc! {"                Detects optional parameters defined before required parameters in function-like declarations.
                Such parameter order is considered deprecated; required parameters should precede optional parameters.
            "},
            good_example: indoc! {r#"                <?php

                function foo(string $required, ?string $optional = null): void {}
            "#},
            bad_example: indoc! {r#"                <?php

                function foo(?string $optional = null, string $required): void {}
            "#},
            category: Category::Deprecation,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP80)),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::FunctionLikeParameterList];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::FunctionLikeParameterList(function_like_parameter_list) = node else {
            return;
        };

        let mut optional_parameters = Vec::new();

        for parameter in function_like_parameter_list.parameters.iter() {
            if parameter.default_value.is_some() || parameter.ellipsis.is_some() {
                optional_parameters.push((parameter.variable.name, parameter.variable.span()));
            } else if !optional_parameters.is_empty() {
                let issue = Issue::new(
                    self.cfg.level(),
                    format!(
                        "Optional parameter(s) `{}` defined before required parameter `{}`.",
                        optional_parameters.iter().map(|(opt_name, _)| *opt_name).collect::<Vec<_>>().join("`, `"),
                        parameter.variable.name
                    ),
                )
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(parameter.variable.span())
                        .with_message(format!("Required parameter `{}` defined here", parameter.variable.name)),
                )
                .with_annotations(optional_parameters.iter().map(|(opt_name, opt_span)| {
                    Annotation::secondary(*opt_span)
                        .with_message(format!("Optional parameter `{}` defined here", opt_name))
                }))
                .with_note("Parameters after an optional one are implicitly required.")
                .with_note("Defining optional parameters before required ones has been deprecated since PHP 8.0.")
                .with_help("Move all optional parameters to the end of the parameter list to resolve this issue.");

                ctx.collector.report(issue);

                optional_parameters.clear();
            }
        }
    }
}
