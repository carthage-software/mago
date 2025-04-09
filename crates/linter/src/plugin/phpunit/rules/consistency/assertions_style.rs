use indoc::indoc;
use toml::Value;

use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;
use mago_syntax::utils::reference::MethodReference;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleOptionDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::plugin::phpunit::rules::utils::find_testing_or_assertion_references_in_method;
use crate::rule::Rule;

const STATIC_STYLES: &str = "static";
const SELF_STYLES: &str = "self";
const THIS_STYLES: &str = "this";

const STYLES: [&str; 3] = [STATIC_STYLES, SELF_STYLES, THIS_STYLES];

const STYLE: &str = "style";
const STYLE_DEFAULT: &str = STATIC_STYLES;

#[derive(Clone, Debug)]
pub struct AssertionsStyleRule;

impl Rule for AssertionsStyleRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Assertions Style", Level::Warning)
            .with_description(indoc! {"
                Detects inconsistent assertions style in test methods.
                Assertions should use the same style, either `static::`, `self::`, or `$this->`.
            "})
            .with_option(RuleOptionDefinition {
                name: STYLE,
                r#type: "string",
                description: "The desired assertions style, either `static`, `self`, or `this`.",
                default: Value::String(STYLE_DEFAULT.to_string()),
            })
            .with_example(RuleUsageExample::valid(
                "An assertion using the `static::` style",
                indoc! {r#"
                    <?php

                    declare(strict_types=1);

                    use PHPUnit\Framework\TestCase;

                    final class SomeTest extends TestCase
                    {
                        public function testSomething(): void
                        {
                            static::assertTrue(true);
                        }
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "An assertion using the `$this->` style",
                indoc! {r#"
                    <?php

                    declare(strict_types=1);

                    use PHPUnit\Framework\TestCase;

                    final class SomeTest extends TestCase
                    {
                        public function testSomething(): void
                        {
                            $this->assertTrue(true);
                        }
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "An assertion using the `self::` style",
                indoc! {r#"
                    <?php

                    declare(strict_types=1);

                    use PHPUnit\Framework\TestCase;

                    final class SomeTest extends TestCase
                    {
                        public function testSomething(): void
                        {
                            self::assertTrue(true);
                        }
                    }
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Method(method) = node else { return LintDirective::default() };

        let name = context.lookup(&method.name.value);
        if !name.starts_with("test") || name.chars().nth(4).is_none_or(|c| c != '_' && !c.is_uppercase()) {
            return LintDirective::Prune;
        }

        let desired_style = context
            .option(STYLE)
            .and_then(|o| o.as_str())
            .filter(|s| STYLES.contains(&s.to_ascii_lowercase().as_str()))
            .unwrap_or(STYLE_DEFAULT)
            .to_string();

        let desired_syntax = match desired_style.as_str() {
            STATIC_STYLES => "static::",
            SELF_STYLES => "self::",
            THIS_STYLES => "$this->",
            _ => unreachable!(),
        };

        for reference in find_testing_or_assertion_references_in_method(method, context) {
            let (to_replace, current_style) = match reference {
                MethodReference::MethodCall(c) => (c.object.span().join(c.arrow), THIS_STYLES),
                MethodReference::MethodClosureCreation(c) => (c.object.span().join(c.arrow), THIS_STYLES),
                MethodReference::StaticMethodClosureCreation(StaticMethodClosureCreation {
                    class,
                    double_colon,
                    ..
                }) => match class.as_ref() {
                    Expression::Static(_) => (class.span().join(*double_colon), STATIC_STYLES),
                    Expression::Self_(_) => (class.span().join(*double_colon), SELF_STYLES),
                    _ => continue,
                },
                MethodReference::StaticMethodCall(StaticMethodCall { class, double_colon, .. }) => match class.as_ref()
                {
                    Expression::Static(_) => (class.span().join(*double_colon), STATIC_STYLES),
                    Expression::Self_(_) => (class.span().join(*double_colon), SELF_STYLES),
                    _ => continue,
                },
            };

            if current_style == desired_style {
                continue;
            }

            let current_syntax = match current_style {
                STATIC_STYLES => "static::",
                SELF_STYLES => "self::",
                THIS_STYLES => "$this->",
                _ => unreachable!(),
            };

            let issue = Issue::new(context.level(), "Inconsistent assertions style.")
                .with_annotation(
                    Annotation::primary(reference.span())
                        .with_message(format!("This assertion uses the `{}` style.", current_syntax)),
                )
                .with_help(format!(
                    "Use `{}` instead of `{}` to conform to the `{}` style.",
                    desired_syntax, current_syntax, desired_style,
                ));

            context.propose(issue, |plan| {
                plan.replace(
                    to_replace.to_range(),
                    desired_syntax.to_string(),
                    SafetyClassification::PotentiallyUnsafe,
                );
            });
        }

        LintDirective::Prune
    }
}
