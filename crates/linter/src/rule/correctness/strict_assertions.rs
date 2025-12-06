use indoc::indoc;
use schemars::JsonSchema;
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
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

            if !NON_STRICT_ASSERTIONS.contains(&identifier.value) {
                continue;
            }

            let Some(argument_list) = reference.get_argument_list() else {
                continue;
            };

            let is_attribute_assertion =
                identifier.value == "assertAttributeEquals" || identifier.value == "assertAttributeNotEquals";

            let should_flag = if is_attribute_assertion {
                argument_list.arguments.get(1).is_some_and(|arg| is_non_bool_or_null_scalar(arg.value()))
            } else {
                argument_list.arguments.first().is_some_and(|arg| is_non_bool_or_null_scalar(arg.value()))
                    || argument_list.arguments.get(1).is_some_and(|arg| is_non_bool_or_null_scalar(arg.value()))
            };

            if !should_flag {
                continue;
            }

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

/// Checks if an expression is a scalar literal (null, bool, int, float, or string).
const fn is_non_bool_or_null_scalar(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::Literal(Literal::Integer(_))
            | Expression::Literal(Literal::Float(_))
            | Expression::Literal(Literal::String(_))
            | Expression::CompositeString(_)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = variable_comparison_not_flagged,
        rule = StrictAssertionsRule,
        code = indoc! {r#"
            <?php

            use PHPUnit\Framework\TestCase;

            final class Test extends TestCase
            {
                public function testFoo(): void
                {
                    $this->assertEquals($expected, $actual);
                }
            }
        "#}
    }

    test_lint_success! {
        name = method_call_not_flagged,
        rule = StrictAssertionsRule,
        code = indoc! {r#"
            <?php

            use PHPUnit\Framework\TestCase;

            final class Test extends TestCase
            {
                public function testFoo(): void
                {
                    self::assertEquals(Money::EUR('100'), $result);
                }
            }
        "#}
    }

    test_lint_success! {
        name = static_variable_comparison_not_flagged,
        rule = StrictAssertionsRule,
        code = indoc! {r#"
            <?php

            use PHPUnit\Framework\TestCase;

            final class Test extends TestCase
            {
                public function testFoo(): void
                {
                    static::assertNotEquals($obj1, $obj2);
                }
            }
        "#}
    }

    test_lint_success! {
        name = null_comparison_not_flagged,
        rule = StrictAssertionsRule,
        code = indoc! {r#"
            <?php

            use PHPUnit\Framework\TestCase;

            final class Test extends TestCase
            {
                public function testFoo(): void
                {
                    $this->assertEquals(null, $value);
                }
            }
        "#}
    }

    test_lint_success! {
        name = bool_comparison_not_flagged,
        rule = StrictAssertionsRule,
        code = indoc! {r#"
            <?php

            use PHPUnit\Framework\TestCase;

            final class Test extends TestCase
            {
                public function testFoo(): void
                {
                    $this->assertEquals(true, $flag);
                    self::assertEquals(false, $other);
                }
            }
        "#}
    }

    test_lint_failure! {
        name = integer_literal_flagged,
        rule = StrictAssertionsRule,
        count = 1,
        code = indoc! {r#"
            <?php

            use PHPUnit\Framework\TestCase;

            final class Test extends TestCase
            {
                public function testFoo(): void
                {
                    $this->assertEquals(42, $result);
                }
            }
        "#}
    }

    test_lint_failure! {
        name = string_literal_flagged,
        rule = StrictAssertionsRule,
        count = 1,
        code = indoc! {r#"
            <?php

            use PHPUnit\Framework\TestCase;

            final class Test extends TestCase
            {
                public function testFoo(): void
                {
                    self::assertEquals('foo', $bar);
                }
            }
        "#}
    }

    test_lint_failure! {
        name = float_literal_flagged,
        rule = StrictAssertionsRule,
        count = 1,
        code = indoc! {r#"
            <?php

            use PHPUnit\Framework\TestCase;

            final class Test extends TestCase
            {
                public function testFoo(): void
                {
                    static::assertEquals(3.14, $value);
                }
            }
        "#}
    }

    test_lint_failure! {
        name = literal_in_second_position_flagged,
        rule = StrictAssertionsRule,
        count = 1,
        code = indoc! {r#"
            <?php

            use PHPUnit\Framework\TestCase;

            final class Test extends TestCase
            {
                public function testFoo(): void
                {
                    $this->assertEquals($result, 42);
                }
            }
        "#}
    }

    test_lint_failure! {
        name = assert_not_equals_with_literal_flagged,
        rule = StrictAssertionsRule,
        count = 1,
        code = indoc! {r#"
            <?php

            use PHPUnit\Framework\TestCase;

            final class Test extends TestCase
            {
                public function testFoo(): void
                {
                    $this->assertNotEquals(42, $value);
                }
            }
        "#}
    }

    test_lint_failure! {
        name = multiple_literal_assertions_flagged,
        rule = StrictAssertionsRule,
        count = 3,
        code = indoc! {r#"
            <?php

            use PHPUnit\Framework\TestCase;

            final class Test extends TestCase
            {
                public function testFoo(): void
                {
                    $this->assertEquals(42, $a);
                    self::assertEquals('bar', $b);
                    static::assertNotEquals(1.5, $c);
                }
            }
        "#}
    }
}
