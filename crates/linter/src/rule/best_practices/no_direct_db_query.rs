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
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoDirectDbQueryRule {
    meta: &'static RuleMeta,
    cfg: NoDirectDbQueryConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoDirectDbQueryConfig {
    pub level: Level,
}

impl Default for NoDirectDbQueryConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoDirectDbQueryConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoDirectDbQueryRule {
    type Config = NoDirectDbQueryConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Direct Database Queries",
            code: "no-direct-db-query",
            description: indoc! {"
                This rule flags all direct method calls on the global `$wpdb` object. Direct database queries
                bypass the WordPress object cache, which can lead to poor performance. Using high-level functions
                like `get_posts()` is safer and more efficient.
            "},
            good_example: indoc! {r#"
                <?php

                $posts = get_posts(['author' => $author_id]);
            "#},
            bad_example: indoc! {r#"
                <?php

                global $wpdb;
                $posts = $wpdb->get_results("SELECT * FROM {$wpdb->posts} WHERE post_author = 1");
            "#},
            category: Category::BestPractices,
            requirements: RuleRequirements::Integration(Integration::WordPress),
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
        let Node::MethodCall(method_call) = node else {
            return;
        };

        // Check if this is a method call on $wpdb
        if !is_wpdb_variable(method_call.object) {
            return;
        }

        // Flag any method call on $wpdb
        let issue = Issue::new(self.cfg.level(), "Direct database query using `$wpdb` is discouraged.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(method_call.span()).with_message("Use high-level WordPress functions instead"),
            )
            .with_note("Direct queries bypass object cache and reduce performance")
            .with_help("Use `get_posts()`, `get_users()`, `get_terms()`, etc.");

        ctx.collector.report(issue);
    }
}

/// Check if an expression is a reference to the $wpdb variable
fn is_wpdb_variable(expr: &Expression) -> bool {
    match expr {
        Expression::Variable(Variable::Direct(var)) => var.name == "$wpdb",
        _ => false,
    }
}
