use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::ClassLikeMemberSelector;
use mago_syntax::ast::Expression;
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
pub struct NoObjectManagerSingletonRule {
    meta: &'static RuleMeta,
    cfg: NoObjectManagerSingletonConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoObjectManagerSingletonConfig {
    pub level: Level,
}

impl Default for NoObjectManagerSingletonConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoObjectManagerSingletonConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoObjectManagerSingletonRule {
    type Config = NoObjectManagerSingletonConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No ObjectManager Singleton",
            code: "no-object-manager-singleton",
            description: indoc! {"
                Flags direct usage of `ObjectManager::getInstance()`. In Magento 2, the ObjectManager
                singleton should never be used directly. Dependencies should be injected via constructor
                dependency injection instead.
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

                use Magento\Framework\App\ObjectManager;

                class Example
                {
                    public function doSomething(): void
                    {
                        $objectManager = ObjectManager::getInstance();
                    }
                }
            "},
            category: Category::BestPractices,
            requirements: RuleRequirements::Integration(Integration::Magento),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::StaticMethodCall];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::StaticMethodCall(call) = node else {
            return;
        };

        // Check the method name is "getInstance"
        let ClassLikeMemberSelector::Identifier(method_id) = &call.method else {
            return;
        };

        if !method_id.value.eq_ignore_ascii_case("getInstance") {
            return;
        }

        // Check the class is ObjectManager
        let Expression::Identifier(class_id) = call.class else {
            return;
        };

        let fqcn = ctx.lookup_name(class_id);
        if !fqcn.eq_ignore_ascii_case("Magento\\Framework\\App\\ObjectManager") {
            return;
        }

        ctx.collector.report(
            Issue::new(
                self.cfg.level(),
                "Direct usage of `ObjectManager::getInstance()` is discouraged.",
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(call.span())
                    .with_message("Use constructor dependency injection instead"),
            )
            .with_help("Inject dependencies via the constructor instead of using the ObjectManager singleton."),
        );
    }
}
