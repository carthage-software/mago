use indoc::indoc;

use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::plugin::phpunit::rules::utils::find_all_assertion_references_in_method;
use crate::rule::Rule;

const NON_STRICT_ASSERTIONS: [&str; 4] =
    ["assertAttributeEquals", "assertAttributeNotEquals", "assertEquals", "assertNotEquals"];

/// A PHPUnit rule that enforces the use of strict assertions.
#[derive(Clone, Debug)]
pub struct StrictAssertionsRule;

impl Rule for StrictAssertionsRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Strict Assertions", Level::Warning)
            .with_description(indoc! {"
                Detects non-strict assertions in test methods.
                Assertions should use strict comparison methods, such as `assertSame` or `assertNotSame`
                instead of `assertEquals` or `assertNotEquals`.
            "})
            .with_example(RuleUsageExample::valid(
                "A strict assertion using the `assertSame` method",
                indoc! {r#"
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
            ))
            .with_example(RuleUsageExample::invalid(
                "A non-strict assertion using the `assertEquals` method",
                indoc! {r#"
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
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Method(method) = node else { return LintDirective::default() };

        let name = context.lookup(&method.name.value);
        if !name.starts_with("test") || name.chars().nth(4).is_none_or(|c| c != '_' && !c.is_uppercase()) {
            return LintDirective::Prune;
        }

        for reference in find_all_assertion_references_in_method(method, context) {
            let ClassLikeMemberSelector::Identifier(identifier) = reference.get_selector() else {
                continue;
            };

            let name = context.lookup(&identifier.value);
            if NON_STRICT_ASSERTIONS.contains(&name) {
                let strict_name = name.replacen("Equals", "Same", 1);

                let issue = Issue::new(context.level(), "Use strict assertions in PHPUnit tests.")
                    .with_annotation(
                        Annotation::primary(reference.span())
                            .with_message(format!("Non-strict assertion `{name}` is used here.")),
                    )
                    .with_help(format!(
                        "Replace `{name}` with `{strict_name}` to enforce strict comparisons in your tests."
                    ));

                context.propose(issue, |plan| {
                    plan.replace(
                        reference.get_selector().span().to_range(),
                        strict_name,
                        SafetyClassification::PotentiallyUnsafe,
                    );
                });
            }
        }

        LintDirective::Prune
    }
}
