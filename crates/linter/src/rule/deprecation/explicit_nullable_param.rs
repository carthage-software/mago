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
pub struct ExplicitNullableParamRule {
    meta: &'static RuleMeta,
    cfg: ExplicitNullableParamConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct ExplicitNullableParamConfig {
    pub level: Level,
}

impl Default for ExplicitNullableParamConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for ExplicitNullableParamConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for ExplicitNullableParamRule {
    type Config = ExplicitNullableParamConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Explicit Nullable Param",
            code: "explicit-nullable-param",
            description: indoc! {"
                Detects parameters that are implicitly nullable and rely on a deprecated feature.

                Such parameters are considered deprecated; an explicit nullable type hint is recommended.
            "},
            good_example: indoc! {"
                <?php

                function foo(?string $param) {}

                function bar(null|string $param) {}

                function baz(null|object $param = null) {}
            "},
            bad_example: indoc! {"
                <?php

                function foo(string $param = null) {}

                function bar(string $param = NULL) {}

                function baz(object $param = null) {}
            "},
            category: Category::Deprecation,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP84)),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::FunctionLikeParameter];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::FunctionLikeParameter(function_like_parameter) = node else {
            return;
        };

        let Some(hint) = function_like_parameter.hint.as_ref() else {
            return;
        };

        if hint.contains_null() {
            return;
        }

        let Some(default_value) = function_like_parameter.default_value.as_ref() else {
            return;
        };

        let Expression::Literal(Literal::Null(_)) = default_value.value else {
            return;
        };

        let parameter_name = function_like_parameter.variable.name;

        let issue = Issue::new(
            self.cfg.level(),
            format!("Parameter `{}` is implicitly nullable and relies on a deprecated feature.", parameter_name),
        )
        .with_code(self.meta.code)
        .with_annotation(
            Annotation::primary(function_like_parameter.span())
                .with_message(format!("Parameter `{}` is declared here", parameter_name)),
        )
        .with_note("Updating this will future-proof your code and align it with PHP 8.4 standards.")
        .with_help("Consider using an explicit nullable type hint or replacing the default value.");

        ctx.collector.report(issue);
    }
}
