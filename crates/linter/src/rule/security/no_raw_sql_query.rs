use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

const SQL_KEYWORDS: &[&str] = &[
    "SELECT ", "INSERT ", "UPDATE ", "DELETE ", "CREATE ", "ALTER ", "DROP ", "TRUNCATE ",
];

#[derive(Debug, Clone)]
pub struct NoRawSqlQueryRule {
    meta: &'static RuleMeta,
    cfg: NoRawSqlQueryConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoRawSqlQueryConfig {
    pub level: Level,
}

impl Default for NoRawSqlQueryConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoRawSqlQueryConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRawSqlQueryRule {
    type Config = NoRawSqlQueryConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Raw SQL Query",
            code: "no-raw-sql-query",
            description: indoc! {"
                Flags string literals that contain raw SQL queries. Raw SQL in application code
                bypasses the database abstraction layer and can introduce SQL injection
                vulnerabilities. Use parameterized queries or the framework's query builder instead.
            "},
            good_example: indoc! {r"
                <?php

                $collection->addFieldToFilter('status', 'active');
            "},
            bad_example: indoc! {r#"
                <?php

                $sql = "SELECT * FROM catalog_product_entity WHERE status = 1";
            "#},
            category: Category::Security,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::LiteralString];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::LiteralString(literal) = node else {
            return;
        };

        let Some(value) = literal.value else {
            return;
        };

        let trimmed = value.trim();

        let starts_with_sql = SQL_KEYWORDS
            .iter()
            .any(|kw| trimmed.len() >= kw.len() && trimmed[..kw.len()].eq_ignore_ascii_case(kw));

        if !starts_with_sql {
            return;
        }

        ctx.collector.report(
            Issue::new(
                self.cfg.level(),
                "Raw SQL query detected in string literal.",
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(literal.span)
                    .with_message("This string contains a raw SQL query"),
            )
            .with_help("Use parameterized queries or the framework's query builder/repository pattern instead."),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = normal_string,
        rule = NoRawSqlQueryRule,
        code = r#"
            <?php

            $message = 'Hello world';
        "#
    }

    test_lint_success! {
        name = string_starting_with_select_word,
        rule = NoRawSqlQueryRule,
        code = r#"
            <?php

            $label = 'Selected items';
        "#
    }

    test_lint_failure! {
        name = select_query,
        rule = NoRawSqlQueryRule,
        code = r#"
            <?php

            $sql = "SELECT * FROM users WHERE id = 1";
        "#
    }

    test_lint_failure! {
        name = insert_query,
        rule = NoRawSqlQueryRule,
        code = r#"
            <?php

            $sql = "INSERT INTO users (name) VALUES ('test')";
        "#
    }

    test_lint_failure! {
        name = delete_query,
        rule = NoRawSqlQueryRule,
        code = r#"
            <?php

            $sql = 'DELETE FROM users WHERE id = 1';
        "#
    }
}
