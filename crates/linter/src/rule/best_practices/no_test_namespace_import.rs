use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Identifier;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::Statement;
use mago_syntax::ast::UseItem;
use mago_syntax::ast::UseItems;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoTestNamespaceImportRule {
    meta: &'static RuleMeta,
    cfg: NoTestNamespaceImportConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoTestNamespaceImportConfig {
    pub level: Level,
}

impl Default for NoTestNamespaceImportConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoTestNamespaceImportConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoTestNamespaceImportRule {
    type Config = NoTestNamespaceImportConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Test Namespace Import",
            code: "no-test-namespace-import",
            description: indoc! {"
                Flags `use` statements that import classes from test namespaces.
                Application modules should not depend on test classes in production code.
            "},
            good_example: indoc! {r"
                <?php

                namespace Vendor\Module\Model;

                use Magento\Catalog\Api\ProductRepositoryInterface;
            "},
            bad_example: indoc! {r"
                <?php

                namespace Vendor\Module\Model;

                use Magento\TestFramework\Helper\Bootstrap;
            "},
            category: Category::BestPractices,
            requirements: RuleRequirements::Integration(Integration::Magento),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Program];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::Program(program) = node else {
            return;
        };

        for stmt in &program.statements {
            if let Statement::Namespace(ns) = stmt {
                // If the file's own namespace is a test namespace, skip entirely
                if let Some(name) = &ns.name {
                    if is_test_namespace(name.value()) {
                        continue;
                    }
                }

                for ns_stmt in ns.statements() {
                    self.check_use_statement(ctx, ns_stmt);
                }
            } else {
                self.check_use_statement(ctx, stmt);
            }
        }
    }
}

impl NoTestNamespaceImportRule {
    fn check_use_statement<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, stmt: &Statement<'arena>) {
        let Statement::Use(use_stmt) = stmt else {
            return;
        };

        match &use_stmt.items {
            UseItems::Sequence(s) => {
                for item in &s.items.nodes {
                    self.report_if_test_import(ctx, item);
                }
            }
            UseItems::TypedSequence(s) => {
                for item in &s.items.nodes {
                    self.report_if_test_import(ctx, item);
                }
            }
            UseItems::TypedList(list) => {
                // For grouped imports like `use Magento\TestFramework\{A, B}`, check the prefix
                let prefix = list.namespace.value();
                if is_test_namespace(prefix) {
                    for item in &list.items.nodes {
                        self.report_use_item(ctx, &item.name);
                    }
                }
            }
            UseItems::MixedList(list) => {
                let prefix = list.namespace.value();
                if is_test_namespace(prefix) {
                    for item in &list.items.nodes {
                        self.report_use_item(ctx, &item.item.name);
                    }
                }
            }
        }
    }

    fn report_if_test_import<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, use_item: &UseItem<'arena>) {
        let fqcn = use_item.name.value();
        if is_test_namespace(fqcn) {
            self.report_use_item(ctx, &use_item.name);
        }
    }

    fn report_use_item<'arena>(
        &self,
        ctx: &mut LintContext<'_, 'arena>,
        name: &Identifier<'arena>,
    ) {
        ctx.collector.report(
            Issue::new(self.cfg.level(), "Importing classes from test namespaces in production code is discouraged.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(name.span()).with_message("This import references a test namespace"),
                )
                .with_help("Remove this import and use proper dependency injection or mocking in tests instead."),
        );
    }
}

fn is_test_namespace(fqcn: &str) -> bool {
    let lower = fqcn.to_ascii_lowercase();

    // Magento test namespaces
    lower.contains("\\testframework\\")
        || lower.contains("\\test\\")
        || lower.contains("\\tests\\")
        || lower.starts_with("magento\\testframework\\")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = normal_import,
        rule = NoTestNamespaceImportRule,
        code = r#"
            <?php

            namespace Vendor\Module\Model;

            use Magento\Catalog\Api\ProductRepositoryInterface;
        "#
    }

    test_lint_failure! {
        name = test_framework_import,
        rule = NoTestNamespaceImportRule,
        code = r#"
            <?php

            namespace Vendor\Module\Model;

            use Magento\TestFramework\Helper\Bootstrap;
        "#
    }

    test_lint_failure! {
        name = test_namespace_import,
        rule = NoTestNamespaceImportRule,
        code = r#"
            <?php

            namespace Vendor\Module\Model;

            use Vendor\Module\Test\Unit\SomeHelper;
        "#
    }

    test_lint_success! {
        name = test_import_from_test_namespace,
        rule = NoTestNamespaceImportRule,
        code = r#"
            <?php

            namespace Vendor\Module\Test\Unit;

            use Magento\TestFramework\Helper\Bootstrap;
        "#
    }
}
