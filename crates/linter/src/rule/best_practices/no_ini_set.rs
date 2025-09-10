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
use crate::rule::utils::call::function_call_matches;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoIniSetRule {
    meta: &'static RuleMeta,
    cfg: NoIniSetConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoIniSetConfig {
    pub level: Level,
}

impl Default for NoIniSetConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoIniSetConfig {
    fn level(&self) -> Level {
        self.level
    }

    fn default_enabled() -> bool {
        true
    }
}

impl LintRule for NoIniSetRule {
    type Config = NoIniSetConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No ini_set",
            code: "no-ini-set",
            description: indoc! {r#"
                Enforces that ini_set is not used.

                Runtime configuration changes via ini_set make application behavior unpredictable and environment-dependent. They can mask misconfigured servers, introduce subtle bugs, and lead to inconsistent behavior between development, testing, and production environments.

                Modern applications should rely on well-defined configuration through php.ini or framework specific configuration. This ensures that configuration is explicit, consistent, and controlled across all environments.

                If a setting truly needs to vary between contexts, it should be handled at the infrastructure or framework configuration level, never by calling ini_set within the application code.
            "#},
            good_example: indoc! {r#"
                // In framework config files (e.g., wp-config.php), use constants.
                define( 'WP_DEBUG', true );

                // Use framework-provided functions where available.
                wp_raise_memory_limit( 'admin' );
            "#},
            bad_example: indoc! {r#"
                // This can override server settings in an unpredictable way.
                ini_set( 'display_errors', 1 );
                ini_set( 'memory_limit', '256M' );
            "#},
            category: Category::BestPractices,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::FunctionCall];

        TARGETS
    }

    fn build(settings: RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::FunctionCall(function_call) = node else {
            return;
        };

        if !function_call_matches(ctx, function_call, "ini_set")
            && !function_call_matches(ctx, function_call, "ini_alter")
        {
            return;
        }

        let issue = Issue::new(self.cfg.level, "ini_set should not be used.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(function_call.span())
                    .with_message("ini_set() is discouraged. Use framework-level constants or dedicated functions where available to avoid runtime configuration conflicts."),
            )
            .with_note("Framework level constant or dedicated configuration are preferred.")
            .with_help("Use a framework specific configuration instead of `ini_set`.");

        ctx.collector.report(issue);
    }
}
