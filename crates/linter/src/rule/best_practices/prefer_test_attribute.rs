use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Identifier;
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
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

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::Method(method) = node else {
            return;
        };

        let name = method.name.value;

        // Must start with "test" or "Test"
        if !name.starts_with("test") && !name.starts_with("Test") {
            return;
        }

        let after_test = &name[4..];
        if !after_test.is_empty() {
            let first = after_test.as_bytes()[0];
            // testfoo is not a test method — needs testFoo or test_foo
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
            edits.push(TextEdit::replace(method.name.span(), new_name.clone()).with_safety(Safety::PotentiallyUnsafe));

            let attribute_text = format!("#[\\PHPUnit\\Framework\\Attributes\\Test]\n{indent}");
            edits.push(
                TextEdit::insert(method.span().start.offset, attribute_text).with_safety(Safety::PotentiallyUnsafe),
            );
        });
    }
}

fn has_test_attribute(ctx: &mut LintContext<'_, '_>, method: &Method<'_>) -> bool {
    for attribute_list in method.attribute_lists.iter() {
        for attribute in attribute_list.attributes.iter() {
            let resolved_name = ctx.resolved_names.get(&attribute.name);
            if resolved_name.eq_ignore_ascii_case("PHPUnit\\Framework\\Attributes\\Test") {
                return true;
            }
        }
    }

    false
}

fn compute_new_name(name: &str) -> String {
    let prefix_was_lowercase = name.starts_with("test");
    let after_test = &name[4..];

    if after_test.is_empty() || after_test == "_" {
        return name.to_string();
    }

    if after_test.starts_with('_') {
        // test_something_bad -> something_bad
        after_test[1..].to_string()
    } else if prefix_was_lowercase {
        // testSomethingBad -> somethingBad
        let mut chars = after_test.chars();
        let first = chars.next().unwrap();
        format!("{}{}", first.to_ascii_lowercase(), chars.as_str())
    } else {
        // TestSomethingBad -> SomethingBad
        after_test.to_string()
    }
}

fn get_method_indent<'arena>(ctx: &LintContext<'_, 'arena>, method: &Method<'arena>) -> String {
    let source = ctx.source_file.contents.as_ref();
    let start = method.span().start_offset() as usize;
    let line_start = source[..start].rfind('\n').map_or(0, |pos| pos + 1);

    source[line_start..start]
        .chars()
        .take_while(|c: &char| c.is_whitespace() && *c != '\n' && *c != '\r')
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
