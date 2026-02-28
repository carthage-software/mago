use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Hint;
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
pub struct NoRegistryRule {
    meta: &'static RuleMeta,
    cfg: NoRegistryConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoRegistryConfig {
    pub level: Level,
}

impl Default for NoRegistryConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoRegistryConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRegistryRule {
    type Config = NoRegistryConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Registry",
            code: "no-registry",
            description: indoc! {"
                Flags usage of `Magento\\Framework\\Registry`, which is deprecated since Magento 2.3.
                The Registry singleton pattern is considered an anti-pattern. Use constructor dependency
                injection and proper state management instead.
            "},
            good_example: indoc! {r"
                <?php

                namespace Vendor\Module\Model;

                use Magento\Catalog\Api\ProductRepositoryInterface;

                class Example
                {
                    public function __construct(
                        private ProductRepositoryInterface $productRepository,
                    ) {}
                }
            "},
            bad_example: indoc! {r"
                <?php

                namespace Vendor\Module\Model;

                use Magento\Framework\Registry;

                class Example
                {
                    public function __construct(
                        private Registry $registry,
                    ) {}
                }
            "},
            category: Category::Deprecation,
            requirements: RuleRequirements::Integration(Integration::Magento),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Hint];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::Hint(Hint::Identifier(identifier)) = node else {
            return;
        };

        let fqcn = ctx.lookup_name(identifier);
        if !fqcn.eq_ignore_ascii_case("Magento\\Framework\\Registry") {
            return;
        }

        ctx.collector.report(
            Issue::new(
                self.cfg.level(),
                "`Magento\\Framework\\Registry` is deprecated since Magento 2.3.",
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(identifier.span()).with_message("Deprecated Registry usage"),
            )
            .with_help(
                "Use constructor dependency injection and proper state management \
                 instead of the Registry singleton.",
            ),
        );
    }
}
