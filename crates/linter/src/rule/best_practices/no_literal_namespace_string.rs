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
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoLiteralNamespaceStringRule {
    meta: &'static RuleMeta,
    cfg: NoLiteralNamespaceStringConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoLiteralNamespaceStringConfig {
    pub level: Level,
}

impl Default for NoLiteralNamespaceStringConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoLiteralNamespaceStringConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoLiteralNamespaceStringRule {
    type Config = NoLiteralNamespaceStringConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Literal Namespace String",
            code: "no-literal-namespace-string",
            description: indoc! {"
                Flags hardcoded fully qualified class name strings. Use `::class` notation
                instead for better IDE support, refactoring safety, and static analysis.
            "},
            good_example: indoc! {r"
                <?php

                $className = \Magento\Catalog\Model\Product::class;
            "},
            bad_example: indoc! {r"
                <?php

                $className = 'Magento\Catalog\Model\Product';
            "},
            category: Category::BestPractices,
            requirements: RuleRequirements::Integration(Integration::Magento),
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

        if !looks_like_class_name(value) {
            return;
        }

        ctx.collector.report(
            Issue::new(
                self.cfg.level(),
                "Use `::class` notation instead of a hardcoded namespace string.",
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(literal.span)
                    .with_message("Hardcoded namespace string"),
            )
            .with_help("Replace this string with `ClassName::class` for better IDE support and refactoring safety."),
        );
    }
}

/// Checks if a string value looks like a fully qualified class name.
///
/// Matches patterns like `Vendor\Module\ClassName` with at least 3 segments,
/// each starting with an uppercase letter.
fn looks_like_class_name(value: &str) -> bool {
    // Normalize double backslashes to single
    let normalized: String;
    let s = if value.contains("\\\\") {
        normalized = value.replace("\\\\", "\\");
        &normalized
    } else {
        value
    };

    // Strip optional leading backslash
    let s = s.strip_prefix('\\').unwrap_or(s);

    // Must not end with backslash
    if s.ends_with('\\') {
        return false;
    }

    let segments: Vec<&str> = s.split('\\').collect();

    // Need at least 3 segments (e.g. Vendor\Module\Class)
    if segments.len() < 3 {
        return false;
    }

    // Each segment must start with an uppercase letter and contain only alphanumeric + underscore
    segments.iter().all(|seg| {
        !seg.is_empty()
            && seg.starts_with(|c: char| c.is_ascii_uppercase())
            && seg.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = normal_string,
        rule = NoLiteralNamespaceStringRule,
        code = r#"
            <?php

            $name = 'hello world';
        "#
    }

    test_lint_success! {
        name = two_segment_string,
        rule = NoLiteralNamespaceStringRule,
        code = r#"
            <?php

            $name = 'Vendor\Module';
        "#
    }

    test_lint_failure! {
        name = three_segment_namespace_string,
        rule = NoLiteralNamespaceStringRule,
        code = r#"
            <?php

            $className = 'Magento\Catalog\Model\Product';
        "#
    }

    test_lint_failure! {
        name = double_backslash_namespace_string,
        rule = NoLiteralNamespaceStringRule,
        code = r#"
            <?php

            $className = 'Magento\\Catalog\\Model\\Product';
        "#
    }
}
