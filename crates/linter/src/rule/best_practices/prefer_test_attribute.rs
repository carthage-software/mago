use mago_allocator::Arena;
use std::borrow::Cow;

use indoc::indoc;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Method;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_text_edit::Safety;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct PreferTestAttributeRule {
    meta: &'static RuleMeta,
    cfg: PreferTestAttributeConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct PreferTestAttributeConfig {
    pub level: Level,
}

impl Default for PreferTestAttributeConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for PreferTestAttributeConfig {
    fn level(&self) -> Level {
        self.level
    }

    fn default_enabled() -> bool {
        false
    }
}

impl LintRule for PreferTestAttributeRule {
    type Config = PreferTestAttributeConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prefer Test Attribute",
            code: "prefer-test-attribute",
            description: indoc! {r#"
                Suggests using PHPUnit's `#[Test]` attribute instead of the `test` method name prefix.

                When a method name starts with `test`, it can be replaced with a `#[Test]` attribute
                and a shorter method name. This is the modern PHPUnit style (PHPUnit 10+).
            "#},
            good_example: indoc! {r"
                <?php

                use PHPUnit\Framework\TestCase;
                use PHPUnit\Framework\Attributes\Test;

                class UserTest extends TestCase
                {
                    #[Test]
                    public function itReturnsFullName(): void {}
                }
            "},
            bad_example: indoc! {r"
                <?php

                use PHPUnit\Framework\TestCase;

                class UserTest extends TestCase
                {
                    public function testItReturnsFullName(): void {}
                }
            "},
            category: Category::BestPractices,
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

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Method(method) = node else {
            return;
        };

        let name_bytes = method.name.value;
        let Some(name) = std::str::from_utf8(name_bytes).ok() else { return };

        if !name_bytes.starts_with(b"test") && !name_bytes.starts_with(b"Test") {
            return;
        }

        let after_test = &name_bytes[4..];
        if !after_test.is_empty() {
            let first = after_test[0];
            if first != b'_' && !first.is_ascii_uppercase() {
                return;
            }
        }

        if has_test_attribute(ctx, method) {
            return;
        }

        let new_name = compute_new_name(name);

        let issue =
            Issue::new(self.cfg.level, format!("Use `#[Test]` attribute instead of `test` prefix on method `{name}`."))
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(method.name.span())
                        .with_message(format!("Method `{name}` uses the `test` prefix.")),
                )
                .with_help(format!("Rename to `{new_name}` and add `#[Test]` attribute."));

        let indent = get_method_indent(ctx, method);

        ctx.collector.propose(issue, |edits| {
            edits.push(
                TextEdit::replace(method.name.span(), new_name.into_owned()).with_safety(Safety::PotentiallyUnsafe),
            );

            let attribute_text = format!("#[\\PHPUnit\\Framework\\Attributes\\Test]\n{indent}");
            edits.push(
                TextEdit::insert(method.span().start.offset, attribute_text).with_safety(Safety::PotentiallyUnsafe),
            );
        });
    }
}

fn has_test_attribute<A>(ctx: &LintContext<'_, '_, A>, method: &Method<'_>) -> bool
where
    A: Arena,
{
    for attribute_list in method.attribute_lists.iter() {
        for attribute in attribute_list.attributes.iter() {
            let resolved_name = ctx.resolved_names.get(&attribute.name);
            if resolved_name.eq_ignore_ascii_case(b"PHPUnit\\Framework\\Attributes\\Test") {
                return true;
            }
        }
    }

    false
}

fn compute_new_name(name: &str) -> Cow<'_, str> {
    let prefix_was_lowercase = name.starts_with("test");
    let after_test = &name[4..];

    if after_test.is_empty() || after_test == "_" {
        return Cow::Borrowed(name);
    }

    if let Some(new_name) = after_test.strip_prefix('_') {
        Cow::Borrowed(new_name)
    } else if prefix_was_lowercase {
        let mut chars = after_test.chars();
        let Some(first) = chars.next() else { return Cow::Borrowed(after_test) };
        Cow::Owned(format!("{}{}", first.to_ascii_lowercase(), chars.as_str()))
    } else {
        Cow::Borrowed(after_test)
    }
}

fn get_method_indent<'arena, A>(ctx: &LintContext<'_, 'arena, A>, method: &Method<'arena>) -> String
where
    A: Arena,
{
    let source: &[u8] = ctx.source_file.contents.as_ref();
    let start = method.span().start_offset() as usize;
    let line_start = memchr::memrchr(b'\n', &source[..start]).map_or(0, |pos| pos + 1);

    source[line_start..start]
        .iter()
        .take_while(|&&b| (b == b' ' || b == b'\t') && b != b'\n' && b != b'\r')
        .map(|&b| b as char)
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::PreferTestAttributeRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = already_has_test_attribute,
        rule = PreferTestAttributeRule,
        code = indoc! {r#"
            <?php

            class FooTest {
                #[\PHPUnit\Framework\Attributes\Test]
                public function itWorks(): void {}
            }
        "#}
    }

    test_lint_success! {
        name = non_test_method,
        rule = PreferTestAttributeRule,
        code = indoc! {r#"
            <?php

            class FooTest {
                public function setUp(): void {}
                public function helper(): void {}
            }
        "#}
    }

    test_lint_success! {
        name = method_named_testing,
        rule = PreferTestAttributeRule,
        code = indoc! {r#"
            <?php

            class FooTest {
                public function testing(): void {}
            }
        "#}
    }

    test_lint_success! {
        name = short_alias_attribute,
        rule = PreferTestAttributeRule,
        code = indoc! {r#"
            <?php

            use PHPUnit\Framework\Attributes\Test;

            class FooTest {
                #[Test]
                public function testItWorks(): void {}
            }
        "#}
    }

    test_lint_failure! {
        name = camel_case_test_method,
        rule = PreferTestAttributeRule,
        code = indoc! {r#"
            <?php

            class FooTest {
                public function testSomethingWorks(): void {}
            }
        "#}
    }

    test_lint_failure! {
        name = snake_case_test_method,
        rule = PreferTestAttributeRule,
        code = indoc! {r#"
            <?php

            class FooTest {
                public function test_something_works(): void {}
            }
        "#}
    }

    test_lint_failure! {
        name = uppercase_test_prefix,
        rule = PreferTestAttributeRule,
        code = indoc! {r#"
            <?php

            class FooTest {
                public function TestSomethingWorks(): void {}
            }
        "#}
    }

    test_lint_failure! {
        name = bare_test_method,
        rule = PreferTestAttributeRule,
        code = indoc! {r#"
            <?php

            class FooTest {
                public function test(): void {}
            }
        "#}
    }

    test_lint_failure! {
        name = multiple_test_methods,
        rule = PreferTestAttributeRule,
        count = 2,
        code = indoc! {r#"
            <?php

            class FooTest {
                public function testOne(): void {}
                public function testTwo(): void {}
            }
        "#}
    }

    #[test]
    fn test_compute_new_name() {
        use super::compute_new_name;

        assert_eq!(compute_new_name("testSomethingBad"), "somethingBad");
        assert_eq!(compute_new_name("TestSomethingBad"), "SomethingBad");
        assert_eq!(compute_new_name("test_something_bad"), "something_bad");
        assert_eq!(compute_new_name("Test_something_bad"), "something_bad");
        assert_eq!(compute_new_name("testX"), "x");
        assert_eq!(compute_new_name("TestX"), "X");
        assert_eq!(compute_new_name("test"), "test");
        assert_eq!(compute_new_name("test_"), "test_");
        assert_eq!(compute_new_name("Test_"), "Test_");
    }
}
