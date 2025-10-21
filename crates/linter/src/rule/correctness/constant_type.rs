use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct ConstantTypeRule {
    meta: &'static RuleMeta,
    cfg: ConstantTypeConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct ConstantTypeConfig {
    pub level: Level,
}

impl Default for ConstantTypeConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for ConstantTypeConfig {
    fn default_enabled() -> bool {
        // This rule is disabled by default due to its deprecation.
        // Use the analyzer's `check-missing-type-hints` setting instead.
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for ConstantTypeRule {
    type Config = ConstantTypeConfig;

    fn deprecated() -> bool {
        true
    }

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Constant Type",
            code: "constant-type",
            description: indoc! {"
                Detects class constants that are missing a type hint.
            "},
            good_example: indoc! {r#"
                <?php

                declare(strict_types=1);

                namespace Psl\IO\Internal;

                use Psl\IO;

                class ResourceHandle implements IO\CloseSeekReadWriteStreamHandleInterface {
                    use IO\ReadHandleConvenienceMethodsTrait;
                    use IO\WriteHandleConvenienceMethodsTrait;

                    public const int DEFAULT_READ_BUFFER_SIZE = 4096;
                    public const int MAXIMUM_READ_BUFFER_SIZE = 786432;

                    // ...
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                declare(strict_types=1);

                namespace Psl\IO\Internal;

                use Psl\IO;

                class ResourceHandle implements IO\CloseSeekReadWriteStreamHandleInterface {
                    use IO\ReadHandleConvenienceMethodsTrait;
                    use IO\WriteHandleConvenienceMethodsTrait;

                    public const DEFAULT_READ_BUFFER_SIZE = 4096;
                    public const MAXIMUM_READ_BUFFER_SIZE = 786432;

                    // ...
                }
            "#},
            category: Category::Correctness,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP83)),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::ClassLikeConstant];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::ClassLikeConstant(class_like_constant) = node else {
            return;
        };

        if class_like_constant.hint.is_some() {
            return;
        }

        let item = class_like_constant.first_item();

        let constant_name = item.name.value;

        ctx.collector.report(
            Issue::new(self.cfg.level(), format!("Class constant `{}` is missing a type hint.", constant_name))
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(class_like_constant.span())
                        .with_message(format!("Class constant `{}` is defined here", constant_name)),
                )
                .with_note("Adding a type hint to constants improves code readability and helps prevent type errors.")
                .with_help(format!("Consider specifying a type hint for `{}`.", constant_name)),
        );
    }
}
