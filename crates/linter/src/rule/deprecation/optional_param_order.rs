use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
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
pub struct OptionalParamOrderRule {
    meta: &'static RuleMeta,
    cfg: OptionalParamOrderConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
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
            good_example: indoc! {r"                <?php

                function foo(string $required, ?string $optional = null): void {}
            "},
            bad_example: indoc! {r"                <?php

                function foo(?string $optional = null, string $required): void {}
            "},
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

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::FunctionLikeParameterList(function_like_parameter_list) = node else {
            return;
        };

        let mut optional_parameters = Vec::new();

        for parameter in &function_like_parameter_list.parameters {
            if parameter.default_value.is_some() || parameter.ellipsis.is_some() {
                optional_parameters.push((parameter.variable.name, parameter.variable.span()));
            } else if !optional_parameters.is_empty() {
                let opt_names_joined: Vec<u8> = {
                    let mut out: Vec<u8> = Vec::new();
                    for (i, (opt_name, _)) in optional_parameters.iter().enumerate() {
                        if i > 0 {
                            out.extend_from_slice(b"`, `");
                        }
                        out.extend_from_slice(opt_name);
                    }
                    out
                };
                let opt_names_display = mago_bytes::BytesDisplay(&opt_names_joined);
                let req_name_display = mago_bytes::BytesDisplay(parameter.variable.name);
                let issue = Issue::new(
                    self.cfg.level(),
                    format!(
                        "Optional parameter(s) `{opt_names_display}` defined before required parameter `{req_name_display}`.",
                    ),
                )
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(parameter.variable.span())
                        .with_message(format!("Required parameter `{req_name_display}` defined here")),
                )
                .with_annotations(optional_parameters.iter().map(|(opt_name, opt_span)| {
                    let opt_display = mago_bytes::BytesDisplay(opt_name);
                    Annotation::secondary(*opt_span)
                        .with_message(format!("Optional parameter `{opt_display}` defined here"))
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
