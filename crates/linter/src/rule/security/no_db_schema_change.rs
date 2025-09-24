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
pub struct NoDbSchemaChangeRule {
    meta: &'static RuleMeta,
    cfg: NoDbSchemaChangeConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoDbSchemaChangeConfig {
    pub level: Level,
}

impl Default for NoDbSchemaChangeConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for NoDbSchemaChangeConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoDbSchemaChangeRule {
    type Config = NoDbSchemaChangeConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Database Schema Changes",
            code: "no-db-schema-change",
            description: indoc! {"
                This rule flags any attempt to alter the database schema (using `CREATE`, `ALTER`, or `DROP`)
                within a `$wpdb` call. Schema modifications must only occur within a plugin activation hook
                to prevent catastrophic performance issues and data corruption.
            "},
            good_example: indoc! {r#"
                <?php

                function my_plugin_activation_hook() {
                    global $wpdb;
                    $wpdb->query("ALTER TABLE {$wpdb->posts} ADD my_column VARCHAR(255)");
                }
                register_activation_hook(__FILE__, 'my_plugin_activation_hook');
            "#},
            bad_example: indoc! {r#"
                <?php

                // This schema change runs on every page load, which is very dangerous.
                global $wpdb;
                $wpdb->query("ALTER TABLE {$wpdb->posts} ADD my_column VARCHAR(255)");
            "#},
            category: Category::Security,
            requirements: RuleRequirements::Integration(Integration::WordPress),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::MethodCall];

        TARGETS
    }

    fn build(settings: RuleSettings<Self::Config>) -> Self {
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

        // Check if this is a query method
        let method_name = match &method_call.method {
            ClassLikeMemberSelector::Identifier(ident) => &ident.value,
            _ => return,
        };

        let method_name_lower = method_name.to_lowercase();
        if !matches!(method_name_lower.as_str(), "query" | "get_results" | "get_row" | "get_col" | "get_var") {
            return;
        }

        // Check the first argument for schema change keywords
        let argument_list = &method_call.argument_list;

        if let Some(Argument::Positional(first_arg)) = argument_list.arguments.first()
            && contains_schema_change_keywords(&first_arg.value)
        {
            let issue = Issue::new(self.cfg.level(), "Database schema change outside activation hook")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(method_call.span()).with_message("Schema change detected in query"),
                )
                .with_note("Schema changes outside hooks cause performance and corruption issues")
                .with_help("Move to activation hook: `register_activation_hook()`");

            ctx.collector.report(issue);
        }
    }
}

/// Check if an expression is a reference to the $wpdb variable
fn is_wpdb_variable(expr: &Expression) -> bool {
    match expr {
        Expression::Variable(Variable::Direct(var)) => var.name == "$wpdb",
        _ => false,
    }
}

/// Check if an expression contains SQL schema change keywords
fn contains_schema_change_keywords(expr: &Expression) -> bool {
    match expr {
        Expression::Literal(Literal::String(string_literal)) => {
            if let Some(value) = extract_string_value(string_literal) {
                let upper_value = value.to_uppercase();
                upper_value.contains("CREATE TABLE")
                    || upper_value.contains("ALTER TABLE")
                    || upper_value.contains("DROP TABLE")
                    || upper_value.contains("CREATE INDEX")
                    || upper_value.contains("DROP INDEX")
                    || upper_value.contains("ADD COLUMN")
                    || upper_value.contains("DROP COLUMN")
                    || upper_value.contains("MODIFY COLUMN")
                    || upper_value.contains("RENAME TABLE")
            } else {
                false
            }
        }
        Expression::Binary(binary) if matches!(binary.operator, BinaryOperator::StringConcat(_)) => {
            contains_schema_change_keywords(binary.lhs) || contains_schema_change_keywords(binary.rhs)
        }
        _ => false,
    }
}

/// Extract string value from a literal string
fn extract_string_value<'a>(string_literal: &'a LiteralString<'a>) -> Option<&'a str> {
    string_literal.value
}
