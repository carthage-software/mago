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
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches;
use crate::rule::utils::laravel::is_method_named;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct PreferViewArrayRule {
    meta: &'static RuleMeta,
    cfg: PreferViewArrayConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PreferViewArrayConfig {
    pub level: Level,
}

impl Default for PreferViewArrayConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for PreferViewArrayConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PreferViewArrayRule {
    type Config = PreferViewArrayConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prefer View Array",
            code: "prefer-view-array",
            description: indoc! {"
                Prefer passing data to views using the array parameter in the `view()` function,
                rather than chaining the `with()` method.`

                Using the array parameter directly is more concise and readable.
            "},
            good_example: indoc! {"
                <?php

                return view('user.profile', [
                    'user' => $user,
                    'profile' => $profile,
                ]);
            "},
            bad_example: indoc! {"
                <?php

                return view('user.profile')->with([
                    'user' => $user,
                    'profile' => $profile,
                ]);
            "},
            category: Category::BestPractices,

            requirements: RuleRequirements::Integration(Integration::Laravel),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::MethodCall];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::MethodCall(call @ MethodCall { object, method, .. }) = node else {
            return;
        };

        let Expression::Call(Call::Function(function_call)) = object else {
            return;
        };

        if !function_call_matches(ctx, function_call, "view") || !is_method_named(method, "with") {
            return;
        }

        ctx.collector.report(
            Issue::new(self.cfg.level(), "Use array parameter in `view()` instead of chaining `with()`.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(call.span())
                        .with_message("Chaining `with()` here is less readable and idiomatic"),
                )
                .with_note("Passing data directly as an array parameter to `view()` is preferred.")
                .with_help("Refactor the code to use the array parameter in the `view()` function."),
        );
    }
}
