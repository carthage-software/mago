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
pub struct NoObjectManagerTypeHintRule {
    meta: &'static RuleMeta,
    cfg: NoObjectManagerTypeHintConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoObjectManagerTypeHintConfig {
    pub level: Level,
}

impl Default for NoObjectManagerTypeHintConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoObjectManagerTypeHintConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoObjectManagerTypeHintRule {
    type Config = NoObjectManagerTypeHintConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No ObjectManagerInterface Type Hint",
            code: "no-object-manager-type-hint",
            description: indoc! {"
                Flags type hints that reference `Magento\\Framework\\ObjectManagerInterface` directly.
                Injecting the ObjectManager into classes is considered an anti-pattern in Magento 2.
                Instead, inject the specific interfaces or classes you need and let the DI container
                handle the wiring.
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

                use Magento\Framework\ObjectManagerInterface;

                class Example
                {
                    public function __construct(
                        private ObjectManagerInterface $objectManager,
                    ) {}
                }
            "},
            category: Category::BestPractices,
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
        if !fqcn.eq_ignore_ascii_case("Magento\\Framework\\ObjectManagerInterface") {
            return;
        }

        ctx.collector.report(
            Issue::new(
                self.cfg.level(),
                "Injecting `ObjectManagerInterface` directly is discouraged.",
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(identifier.span())
                    .with_message("Inject the specific interface or class you need instead"),
            )
            .with_help(
                "Instead of injecting `ObjectManagerInterface`, inject the specific \
                 interfaces or classes your code depends on.",
            ),
        );
    }
}
