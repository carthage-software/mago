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
pub struct NoThisInTemplateRule {
    meta: &'static RuleMeta,
    cfg: NoThisInTemplateConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoThisInTemplateConfig {
    pub level: Level,
}

impl Default for NoThisInTemplateConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoThisInTemplateConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoThisInTemplateRule {
    type Config = NoThisInTemplateConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No $this In Template",
            code: "no-this-in-template",
            description: indoc! {"
                Flags usage of `$this` in `.phtml` template files. In Magento 2, `$this` is
                deprecated in templates since Magento 2.1. Use `$block` instead to access
                block methods, or better yet, use a ViewModel.
            "},
            good_example: indoc! {r#"
                <?php
                /** @var \Magento\Framework\View\Element\Template $block */
                $block->getChildHtml('child');
            "#},
            bad_example: indoc! {r#"
                <?php
                // In a .phtml template:
                $this->getChildHtml('child');
            "#},
            category: Category::BestPractices,
            requirements: RuleRequirements::Integration(Integration::Magento),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::DirectVariable];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::DirectVariable(var) = node else {
            return;
        };

        if var.name != "$this" {
            return;
        }

        // Flag $this when used outside a class-like scope (common in .phtml templates)
        // or when explicitly in a .phtml file
        let in_class = ctx.scope.get_class_like_scope().is_some();
        let is_template = ctx.source_file.name.ends_with(".phtml");

        if in_class && !is_template {
            return;
        }

        ctx.collector.report(
            Issue::new(
                self.cfg.level(),
                "The use of `$this` in templates is deprecated.",
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(var.span())
                    .with_message("Use `$block` instead of `$this`"),
            )
            .with_help("Replace `$this` with `$block`, or use a ViewModel for complex logic."),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = block_variable_in_phtml,
        rule = NoThisInTemplateRule,
        filename = "template.phtml",
        code = r#"
            <?php
            $block->getChildHtml('child');
        "#
    }

    test_lint_success! {
        name = this_in_class_method_is_ok,
        rule = NoThisInTemplateRule,
        code = r#"
            <?php
            class Foo {
                public function bar(): void {
                    $this->doSomething();
                }
            }
        "#
    }

    test_lint_failure! {
        name = this_in_phtml_template,
        rule = NoThisInTemplateRule,
        filename = "template.phtml",
        code = r#"
            <?php
            $this->getChildHtml('child');
        "#
    }
}
