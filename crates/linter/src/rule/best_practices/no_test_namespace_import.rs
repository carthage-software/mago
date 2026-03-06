use indoc::indoc;
use schemars::JsonSchema;
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
        const TARGETS: &[NodeKind] = &[NodeKind::UseItem];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::UseItem(use_item) = node else {
            return;
        };

        // Use the raw identifier value since use items contain the full path
        let fqcn = use_item.name.value();

        // Check for common test namespace patterns
        if !is_test_namespace(fqcn) {
            return;
        }

        ctx.collector.report(
            Issue::new(
                self.cfg.level(),
                "Importing classes from test namespaces in production code is discouraged.",
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(use_item.name.span())
                    .with_message("This import references a test namespace"),
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
}
