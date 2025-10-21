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
use crate::rule::utils::laravel::is_method_named;
use crate::rule::utils::laravel::is_this;
use crate::rule::utils::laravel::is_within_controller;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct MiddlewareInRoutesRule {
    meta: &'static RuleMeta,
    cfg: MiddlewareInRoutesConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct MiddlewareInRoutesConfig {
    pub level: Level,
}

impl Default for MiddlewareInRoutesConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for MiddlewareInRoutesConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for MiddlewareInRoutesRule {
    type Config = MiddlewareInRoutesConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Middleware In Routes",
            code: "middleware-in-routes",
            description: indoc! {r#"
                This rule warns against applying middlewares in controllers.

                Middlewares should be applied in the routes file, not in the controller.
            "#},
            good_example: indoc! {r#"
                <?php

                // routes/web.php
                Route::get('/user', 'UserController@index')->middleware('auth');
            "#},
            bad_example: indoc! {r#"
                <?php

                namespace App\Http\Controllers;

                class UserController extends Controller
                {
                    public function __construct()
                    {
                        $this->middleware('auth');
                    }
                }
            "#},
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

        if !is_within_controller(ctx) {
            return;
        }

        if !is_this(object) || !is_method_named(method, "middleware") {
            return;
        }

        let issue = Issue::new(self.cfg.level, "Avoid applying middlewares in controllers.")
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(call.span()).with_message("Middleware applied here."))
            .with_note("Middlewares should be applied in the routes file, not in the controller.")
            .with_help("Move the middleware to the routes file.");

        ctx.collector.report(issue);
    }
}
