use indoc::indoc;
use mago_text_edit::Safety;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::ClassLikeMemberSelector;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Literal;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::phpunit::find_all_assertion_references_in_method;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

const GENERIC_ASSERTIONS: [&str; 4] = ["assertEquals", "assertNotEquals", "assertSame", "assertNotSame"];
const EQUALITY_ASSERTIONS: [&str; 2] = ["assertEquals", "assertNotEquals"];

/// Position of the literal in the assertion call.
#[derive(Debug, Clone, Copy)]
enum LiteralPosition {
    /// Literal is the first argument: `assertEquals(null, $x)` → remove from open paren to second arg start
    First,
    /// Literal is the second argument: `assertEquals($x, null)` → remove from first arg end to close paren
    Second,
}

#[derive(Debug, Clone)]
pub struct UseSpecificAssertionsRule {
    meta: &'static RuleMeta,
    cfg: UseSpecificAssertionsConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct UseSpecificAssertionsConfig {
    pub level: Level,
}

impl Default for UseSpecificAssertionsConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for UseSpecificAssertionsConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for UseSpecificAssertionsRule {
    type Config = UseSpecificAssertionsConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Use Specific Assertions",
            code: "use-specific-assertions",
            description: indoc! {"
                Suggests using specific PHPUnit assertions instead of generic equality assertions
                when comparing with `null`, `true`, or `false`.

                Using specific assertions like `assertNull`, `assertTrue`, and `assertFalse`
                provides clearer error messages and makes test intent more explicit.
            "},
            good_example: indoc! {r"
                <?php

                declare(strict_types=1);

                use PHPUnit\Framework\TestCase;

                final class SomeTest extends TestCase
                {
                    public function testSomething(): void
                    {
                        $this->assertNull($value);
                        $this->assertTrue($flag);
                        $this->assertFalse($condition);
                    }
                }
            "},
            bad_example: indoc! {r"
                <?php

                declare(strict_types=1);

                use PHPUnit\Framework\TestCase;

                final class SomeTest extends TestCase
                {
                    public function testSomething(): void
                    {
                        $this->assertEquals(null, $value);
                        $this->assertSame(true, $flag);
                        $this->assertEquals(false, $condition);
                    }
                }
            "},
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

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
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

            if !GENERIC_ASSERTIONS.contains(&identifier.value) {
                continue;
            }

            let Some(argument_list) = reference.get_argument_list() else {
                continue;
            };

            // Need at least two arguments
            let (Some(first_arg), Some(second_arg)) = (argument_list.arguments.first(), argument_list.arguments.get(1))
            else {
                continue;
            };

            let first_expr = first_arg.value();
            let second_expr = second_arg.value();

            let is_equality_assertion = EQUALITY_ASSERTIONS.contains(&identifier.value);

            let Some((specific_assertion, literal_position)) =
                get_specific_assertion(identifier.value, first_expr, second_expr)
            else {
                continue;
            };

            let mut issue = Issue::new(
                self.cfg.level(),
                format!("Use `{}` instead of `{}` for clearer test assertions.", specific_assertion, identifier.value),
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(reference.span())
                    .with_message(format!("This can be simplified to `{specific_assertion}`.")),
            )
            .with_help(format!(
                "Replace `{}(...)` with `{}(...)` for a more specific assertion.",
                identifier.value, specific_assertion
            ));

            if is_equality_assertion {
                issue = issue.with_help(format!(
                    "`{}` performs a non-strict comparison, while `{specific_assertion}` performs a strict comparison.",
                    identifier.value
                ));

                issue = issue.with_help("Ensure that this change does not affect the test behavior.");
            }

            ctx.collector.propose(issue, |edits| {
                let safety = if is_equality_assertion { Safety::PotentiallyUnsafe } else { Safety::Safe };

                edits.push(TextEdit::replace(reference.get_selector().span(), specific_assertion).with_safety(safety));

                match literal_position {
                    LiteralPosition::First => {
                        edits.push(
                            TextEdit::replace(
                                argument_list.span().start_offset()..second_expr.span().start_offset(),
                                "(",
                            )
                            .with_safety(safety),
                        );
                    }
                    LiteralPosition::Second => {
                        edits.push(
                            TextEdit::replace(first_expr.span().end_offset()..argument_list.span().end_offset(), ")")
                                .with_safety(safety),
                        );
                    }
                }
            });
        }
    }
}

/// Checks what kind of literal an expression is.
enum LiteralKind {
    Null,
    True,
    False,
    Other,
}

fn get_literal_kind(expr: &Expression<'_>) -> Option<LiteralKind> {
    match expr {
        Expression::Literal(Literal::Null(_)) => Some(LiteralKind::Null),
        Expression::Literal(Literal::True(_)) => Some(LiteralKind::True),
        Expression::Literal(Literal::False(_)) => Some(LiteralKind::False),
        Expression::Literal(_) => Some(LiteralKind::Other),
        _ => None,
    }
}

/// Returns the specific assertion name and the position of the literal argument.
fn get_specific_assertion(
    assertion_name: &str,
    first_arg: &Expression<'_>,
    second_arg: &Expression<'_>,
) -> Option<(&'static str, LiteralPosition)> {
    let is_not_assertion = assertion_name.contains("Not");

    // Check first argument for literal
    if let Some(kind) = get_literal_kind(first_arg) {
        match kind {
            LiteralKind::Null => {
                return Some((if is_not_assertion { "assertNotNull" } else { "assertNull" }, LiteralPosition::First));
            }
            LiteralKind::True => {
                if !is_not_assertion {
                    return Some(("assertTrue", LiteralPosition::First));
                }
            }
            LiteralKind::False => {
                if !is_not_assertion {
                    return Some(("assertFalse", LiteralPosition::First));
                }
            }
            LiteralKind::Other => {}
        }
    }

    // Check second argument for literal
    if let Some(kind) = get_literal_kind(second_arg) {
        match kind {
            LiteralKind::Null => {
                return Some((if is_not_assertion { "assertNotNull" } else { "assertNull" }, LiteralPosition::Second));
            }
            LiteralKind::True => {
                if !is_not_assertion {
                    return Some(("assertTrue", LiteralPosition::Second));
                }
            }
            LiteralKind::False => {
                if !is_not_assertion {
                    return Some(("assertFalse", LiteralPosition::Second));
                }
            }
            LiteralKind::Other => {}
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = specific_assertions_not_flagged,
        rule = UseSpecificAssertionsRule,
        code = indoc! {r"
            <?php

            use PHPUnit\Framework\TestCase;

            final class Test extends TestCase
            {
                public function testFoo(): void
                {
                    $this->assertNull($value);
                    $this->assertTrue($flag);
                    $this->assertFalse($condition);
                }
            }
        "}
    }

    test_lint_success! {
        name = non_null_bool_literals_not_flagged,
        rule = UseSpecificAssertionsRule,
        code = indoc! {r"
            <?php

            use PHPUnit\Framework\TestCase;

            final class Test extends TestCase
            {
                public function testFoo(): void
                {
                    $this->assertEquals(42, $value);
                    self::assertSame('foo', $bar);
                }
            }
        "}
    }

    test_lint_success! {
        name = variable_comparisons_not_flagged,
        rule = UseSpecificAssertionsRule,
        code = indoc! {r"
            <?php

            use PHPUnit\Framework\TestCase;

            final class Test extends TestCase
            {
                public function testFoo(): void
                {
                    $this->assertEquals($expected, $actual);
                }
            }
        "}
    }

    test_lint_failure! {
        name = assert_equals_null_first_position,
        rule = UseSpecificAssertionsRule,
        count = 1,
        code = indoc! {r"
            <?php

            use PHPUnit\Framework\TestCase;

            final class Test extends TestCase
            {
                public function testFoo(): void
                {
                    $this->assertEquals(null, $value);
                }
            }
        "}
    }

    test_lint_failure! {
        name = assert_equals_null_second_position,
        rule = UseSpecificAssertionsRule,
        count = 1,
        code = indoc! {r"
            <?php

            use PHPUnit\Framework\TestCase;

            final class Test extends TestCase
            {
                public function testFoo(): void
                {
                    self::assertEquals($value, null);
                }
            }
        "}
    }

    test_lint_failure! {
        name = assert_same_null,
        rule = UseSpecificAssertionsRule,
        count = 1,
        code = indoc! {r"
            <?php

            use PHPUnit\Framework\TestCase;

            final class Test extends TestCase
            {
                public function testFoo(): void
                {
                    static::assertSame(null, $value);
                }
            }
        "}
    }

    test_lint_failure! {
        name = assert_not_equals_null,
        rule = UseSpecificAssertionsRule,
        count = 1,
        code = indoc! {r"
            <?php

            use PHPUnit\Framework\TestCase;

            final class Test extends TestCase
            {
                public function testFoo(): void
                {
                    $this->assertNotEquals(null, $value);
                }
            }
        "}
    }

    test_lint_failure! {
        name = assert_not_same_null,
        rule = UseSpecificAssertionsRule,
        count = 1,
        code = indoc! {r"
            <?php

            use PHPUnit\Framework\TestCase;

            final class Test extends TestCase
            {
                public function testFoo(): void
                {
                    self::assertNotSame($value, null);
                }
            }
        "}
    }

    test_lint_failure! {
        name = assert_equals_true,
        rule = UseSpecificAssertionsRule,
        count = 1,
        code = indoc! {r"
            <?php

            use PHPUnit\Framework\TestCase;

            final class Test extends TestCase
            {
                public function testFoo(): void
                {
                    $this->assertEquals(true, $flag);
                }
            }
        "}
    }

    test_lint_failure! {
        name = assert_same_true_second_position,
        rule = UseSpecificAssertionsRule,
        count = 1,
        code = indoc! {r"
            <?php

            use PHPUnit\Framework\TestCase;

            final class Test extends TestCase
            {
                public function testFoo(): void
                {
                    self::assertSame($result, true);
                }
            }
        "}
    }

    test_lint_failure! {
        name = assert_equals_false,
        rule = UseSpecificAssertionsRule,
        count = 1,
        code = indoc! {r"
            <?php

            use PHPUnit\Framework\TestCase;

            final class Test extends TestCase
            {
                public function testFoo(): void
                {
                    $this->assertEquals(false, $flag);
                }
            }
        "}
    }

    test_lint_failure! {
        name = assert_same_false,
        rule = UseSpecificAssertionsRule,
        count = 1,
        code = indoc! {r"
            <?php

            use PHPUnit\Framework\TestCase;

            final class Test extends TestCase
            {
                public function testFoo(): void
                {
                    static::assertSame(false, $value);
                }
            }
        "}
    }

    test_lint_failure! {
        name = multiple_specific_assertions,
        rule = UseSpecificAssertionsRule,
        count = 3,
        code = indoc! {r"
            <?php

            use PHPUnit\Framework\TestCase;

            final class Test extends TestCase
            {
                public function testFoo(): void
                {
                    $this->assertEquals(null, $a);
                    self::assertSame(true, $b);
                    static::assertEquals(false, $c);
                }
            }
        "}
    }
}
