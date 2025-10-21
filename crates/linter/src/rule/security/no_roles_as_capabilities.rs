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
use crate::rule::utils::call::function_call_matches_any;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

const CAPABILITY_FUNCTIONS: [&str; 5] =
    ["current_user_can", "user_can", "current_user_can_for_blog", "author_can", "map_meta_cap"];

#[derive(Debug, Clone)]
pub struct NoRolesAsCapabilitiesRule {
    meta: &'static RuleMeta,
    cfg: NoRolesAsCapabilitiesConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoRolesAsCapabilitiesConfig {
    pub level: Level,
}

impl Default for NoRolesAsCapabilitiesConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoRolesAsCapabilitiesConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRolesAsCapabilitiesRule {
    type Config = NoRolesAsCapabilitiesConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Roles As Capabilities",
            code: "no-roles-as-capabilities",
            description: indoc! {"
                This rule flags the use of user roles (e.g., `'administrator'`) in functions that expect a
                granular capability (e.g., `'edit_posts'`). Checking against specific capabilities is a
                core security principle in WordPress.
            "},
            good_example: indoc! {r#"
                <?php

                if ( current_user_can( 'edit_posts' ) ) { /* ... */ }
            "#},
            bad_example: indoc! {r#"
                <?php

                // This check is brittle and will fail if roles are customized.
                if ( current_user_can( 'editor' ) ) { /* ... */ }
            "#},
            category: Category::Security,
            requirements: RuleRequirements::Integration(Integration::WordPress),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::FunctionCall];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::FunctionCall(function_call) = node else {
            return;
        };

        if function_call_matches_any(ctx, function_call, &CAPABILITY_FUNCTIONS).is_none() {
            return;
        }

        let argument_list = &function_call.argument_list;

        // Check the first argument for WordPress roles
        if let Some(Argument::Positional(first_arg)) = argument_list.arguments.first()
            && let Some(role_name) = extract_string_literal(&first_arg.value)
            && is_wordpress_role(role_name)
        {
            let issue = Issue::new(
                        self.cfg.level(),
                        "Use capabilities instead of roles in checks",
                    )
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(first_arg.value.span())
                            .with_message(format!("Role '{}' used instead of capability", role_name)),
                    )
                    .with_note("Checking against specific capabilities instead of roles makes code more flexible and secure.")
                    .with_help("Use `'edit_posts'`, `'manage_options'`, etc. instead of roles like `'administrator'` or `'editor'`.");

            ctx.collector.report(issue);
        }
    }
}

/// Check if a string is a known WordPress role
fn is_wordpress_role(role: &str) -> bool {
    matches!(
        role,
        "administrator" | "editor" | "author" | "contributor" | "subscriber" |
        "super_admin" | // Multisite
        "shop_manager" | "customer" // WooCommerce common roles
    )
}

/// Extract string literal value from an expression
fn extract_string_literal<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    match expr {
        Expression::Literal(Literal::String(string_literal)) => string_literal.value,
        _ => None,
    }
}
