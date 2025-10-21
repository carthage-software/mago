use std::str::FromStr;

use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_fixer::SafetyClassification;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::*;
use mago_syntax::utils::reference::MethodReference;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::phpunit::find_testing_or_assertion_references_in_method;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct AssertionStyleRule {
    meta: &'static RuleMeta,
    cfg: AssertionStyleConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssertionStyle {
    Static,
    #[serde(alias = "self")]
    Self_,
    #[serde(alias = "$this")]
    This,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct AssertionStyleConfig {
    pub level: Level,
    pub style: AssertionStyle,
}

impl Default for AssertionStyleConfig {
    fn default() -> Self {
        Self { level: Level::Warning, style: AssertionStyle::Static }
    }
}

impl Config for AssertionStyleConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for AssertionStyleRule {
    type Config = AssertionStyleConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Assertion Style",
            code: "assertion-style",
            description: indoc! {"
                Enforces a consistent style for PHPUnit assertion calls within test methods.

                Maintaining a consistent style (e.g., always using `static::` or `$this->`)
                improves code readability and helps enforce team-wide coding standards in test suites.
                This rule can be configured to enforce the preferred style.
            "},
            good_example: indoc! {r#"
                <?php
                // configured style: "static"
                final class SomeTest extends TestCase
                {
                    public function testSomething(): void
                    {
                        static::assertTrue(true);
                    }
                }
            "#},
            bad_example: indoc! {r#"
                <?php
                // configured style: "static"
                final class SomeTest extends TestCase
                {
                    public function testSomething(): void
                    {
                        $this->assertTrue(true); // Incorrect style
                        self::assertFalse(false); // Incorrect style
                    }
                }
            "#},
            category: Category::Consistency,

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

        for reference in find_testing_or_assertion_references_in_method(method) {
            let (to_replace, current_style) = match reference {
                MethodReference::MethodCall(c) => (c.object.span().join(c.arrow), AssertionStyle::This),
                MethodReference::MethodClosureCreation(c) => (c.object.span().join(c.arrow), AssertionStyle::This),
                MethodReference::StaticMethodClosureCreation(StaticMethodClosureCreation {
                    class,
                    double_colon,
                    ..
                }) => match class {
                    Expression::Static(_) => (class.span().join(*double_colon), AssertionStyle::Static),
                    Expression::Self_(_) => (class.span().join(*double_colon), AssertionStyle::Self_),
                    _ => continue,
                },
                MethodReference::StaticMethodCall(StaticMethodCall { class, double_colon, .. }) => match class {
                    Expression::Static(_) => (class.span().join(*double_colon), AssertionStyle::Static),
                    Expression::Self_(_) => (class.span().join(*double_colon), AssertionStyle::Self_),
                    _ => continue,
                },
            };

            if current_style == self.cfg.style {
                continue;
            }

            let current_syntax = match current_style {
                AssertionStyle::Static => "static::",
                AssertionStyle::Self_ => "self::",
                AssertionStyle::This => "$this->",
            };

            let desired_syntax = match self.cfg.style {
                AssertionStyle::Static => "static::",
                AssertionStyle::Self_ => "self::",
                AssertionStyle::This => "$this->",
            };

            let desired_style = match self.cfg.style {
                AssertionStyle::Static => "static",
                AssertionStyle::Self_ => "self",
                AssertionStyle::This => "this",
            };

            let issue = Issue::new(self.cfg.level, "Inconsistent assertions style.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(reference.span())
                        .with_message(format!("This assertion uses the `{current_syntax}` style.")),
                )
                .with_help(format!(
                    "Use `{desired_syntax}` instead of `{current_syntax}` to conform to the `{desired_style}` style.",
                ));

            ctx.collector.propose(issue, |plan| {
                plan.replace(
                    to_replace.to_range(),
                    desired_syntax.to_string(),
                    SafetyClassification::PotentiallyUnsafe,
                );
            });
        }
    }
}

impl FromStr for AssertionStyle {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("static") {
            Ok(Self::Static)
        } else if s.eq_ignore_ascii_case("self") {
            Ok(Self::Self_)
        } else if s.eq_ignore_ascii_case("this") || s.eq_ignore_ascii_case("$this") {
            Ok(Self::This)
        } else {
            Err(())
        }
    }
}
