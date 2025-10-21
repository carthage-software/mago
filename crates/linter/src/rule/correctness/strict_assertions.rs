use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::phpunit::find_all_assertion_references_in_method;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

const NON_STRICT_ASSERTIONS: [&str; 4] =
    ["assertAttributeEquals", "assertAttributeNotEquals", "assertEquals", "assertNotEquals"];

#[derive(Debug, Clone)]
pub struct StrictAssertionsRule {
    meta: &'static RuleMeta,
    cfg: StrictAssertionsConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct StrictAssertionsConfig {
    pub level: Level,
}

impl Default for StrictAssertionsConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for StrictAssertionsConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for StrictAssertionsRule {
    type Config = StrictAssertionsConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Strict Assertions",
            code: "strict-assertions",
            description: indoc! {"
                Detects non-strict assertions in test methods.
                Assertions should use strict comparison methods, such as `assertSame` or `assertNotSame`
                instead of `assertEquals` or `assertNotEquals`.
            "},
            good_example: indoc! {r#"
                <?php

                declare(strict_types=1);

                use PHPUnit\Framework\TestCase;

                final class SomeTest extends TestCase
                {
                    public function testSomething(): void
                    {
                        $this->assertSame(42, 42);
                    }
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                declare(strict_types=1);

                use PHPUnit\Framework\TestCase;

                final class SomeTest extends TestCase
                {
                    public function testSomething(): void
                    {
                        $this->assertEquals(42, 42);
                    }
                }
            "#},
            category: Category::Correctness,

            requirements: RuleRequirements::Integration(Integration::PHPUnit),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Method];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Method(method) = node else {
            return;
        };

        if !method.name.value.starts_with("test")
            || method.name.value.chars().nth(4).is_none_or(|c| c != '_' && !c.is_uppercase())
        {
            return;
        }

        for reference in find_all_assertion_references_in_method(method) {
            let ClassLikeMemberSelector::Identifier(identifier) = reference.get_selector() else {
                continue;
            };

            if NON_STRICT_ASSERTIONS.contains(&identifier.value) {
                let strict_name = identifier.value.replacen("Equals", "Same", 1);

                let issue = Issue::new(self.cfg.level(), "Use strict assertions in PHPUnit tests.")
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(reference.span())
                            .with_message(format!("Non-strict assertion `{}` is used here.", identifier.value)),
                    )
                    .with_help(format!(
                        "Replace `{}` with `{}` to enforce strict comparisons in your tests.",
                        identifier.value, strict_name
                    ));

                ctx.collector.propose(issue, |plan| {
                    plan.replace(
                        reference.get_selector().span().to_range(),
                        strict_name,
                        SafetyClassification::PotentiallyUnsafe,
                    );
                });
            }
        }
    }
}
