use indoc::indoc;
use mago_text_edit::TextEdit;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches_any;
use crate::rule::utils::call::get_method_name;
use crate::rule::utils::pest::find_expect_call;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

const PEST_TEST_FUNCTIONS: &[&str] =
    &["test", "it", "describe", "todo", "beforeEach", "afterEach", "beforeAll", "afterAll"];

#[derive(Debug, Clone)]
pub struct NoOnlyRule {
    meta: &'static RuleMeta,
    cfg: NoOnlyConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoOnlyConfig {
    pub level: Level,
}

impl Default for NoOnlyConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for NoOnlyConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoOnlyRule {
    type Config = NoOnlyConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Only",
            code: "no-only",
            description: indoc! {"
                Detects usage of `->only()` in Pest tests which should not be committed.

                The `->only()` modifier causes only that specific test to run, which can lead to
                incomplete test coverage if accidentally committed to the repository.
            "},
            good_example: indoc! {r#"
                <?php

                test('example test', function () {
                    expect(true)->toBeTrue();
                });

                it('does something', function () {
                    expect(1)->toBe(1);
                });
            "#},
            bad_example: indoc! {r#"
                <?php

                test('example test', function () {
                    expect(true)->toBeTrue();
                })->only();

                it('does something', function () {
                    expect(1)->toBe(1);
                })->only();
            "#},
            category: Category::Correctness,
            requirements: RuleRequirements::Integration(Integration::Pest),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::MethodCall];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::MethodCall(method_call) = node else {
            return;
        };

        // Check if method is `only`
        let Some(method_name) = get_method_name(method_call) else {
            return;
        };

        if method_name != "only" {
            return;
        }

        // Check if this is part of a Pest test function chain
        let Some(func_call) = find_expect_call(method_call) else {
            return;
        };

        // Check if it's one of the Pest test functions
        if function_call_matches_any(ctx, func_call, PEST_TEST_FUNCTIONS).is_none() {
            return;
        }

        let issue =
            Issue::new(self.cfg.level(), "Remove `->only()` before committing - it prevents other tests from running.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(method_call.span()).with_message("This `->only()` call should be removed."),
                )
                .with_help("Remove `->only()` to ensure all tests run.");

        ctx.collector.propose(issue, |edits| {
            edits.push(TextEdit::delete(method_call.arrow.join(method_call.argument_list.span())));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = test_without_only,
        rule = NoOnlyRule,
        code = indoc! {r#"
            <?php

            test('example', function () {
                expect(true)->toBeTrue();
            });

            it('does something', function () {
                expect(1)->toBe(1);
            });
        "#}
    }

    test_lint_failure! {
        name = test_with_only,
        rule = NoOnlyRule,
        count = 1,
        code = indoc! {r#"
            <?php

            test('example', function () {
                expect(true)->toBeTrue();
            })->only();
        "#}
    }

    test_lint_failure! {
        name = it_with_only,
        rule = NoOnlyRule,
        count = 1,
        code = indoc! {r#"
            <?php

            it('does something', function () {
                expect(1)->toBe(1);
            })->only();
        "#}
    }

    test_lint_failure! {
        name = describe_with_only,
        rule = NoOnlyRule,
        count = 1,
        code = indoc! {r#"
            <?php

            describe('group', function () {
                test('test', function () {});
            })->only();
        "#}
    }

    test_lint_failure! {
        name = multiple_only,
        rule = NoOnlyRule,
        count = 2,
        code = indoc! {r#"
            <?php

            test('first', function () {})->only();
            it('second', function () {})->only();
        "#}
    }
}
