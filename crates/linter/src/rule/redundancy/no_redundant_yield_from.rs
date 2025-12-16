use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_fixer::SafetyClassification;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoRedundantYieldFromRule {
    meta: &'static RuleMeta,
    cfg: NoRedundantYieldFromConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoRedundantYieldFromConfig {
    pub level: Level,
}

impl Default for NoRedundantYieldFromConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoRedundantYieldFromConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRedundantYieldFromRule {
    type Config = NoRedundantYieldFromConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Redundant Yield From",
            code: "no-redundant-yield-from",
            description: indoc! {"
                Detects redundant use of `yield from` with single-element array literals.

                Using `yield from` with a single-element array literal creates unnecessary
                overhead in the generated opcodes. Direct `yield` is simpler and more efficient.
            "},
            good_example: indoc! {r"
                <?php

                function gen(): Generator {
                    yield 1;
                    yield 'foo' => new stdClass();
                }
            "},
            bad_example: indoc! {r"
                <?php

                function gen(): Generator {
                    yield from [1];
                    yield from ['foo' => new stdClass()];
                }
            "},
            category: Category::Redundancy,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::YieldFrom];
        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::YieldFrom(yield_from) = node else {
            return;
        };

        let elements = match yield_from.iterator {
            Expression::Array(array) => &array.elements,
            Expression::LegacyArray(legacy_array) => &legacy_array.elements,
            _ => return,
        };

        if elements.nodes.len() != 1 {
            return;
        }

        let element = &elements.nodes[0];

        if element.is_variadic() || element.is_missing() {
            return;
        }

        // Build the issue
        let issue = Issue::new(
            self.cfg.level(),
            "Redundant `yield from` with single-element array: use direct `yield` instead.",
        )
        .with_code(self.meta.code)
        .with_annotation(
            Annotation::primary(yield_from.from.span())
                .with_message("Unnecessary `from` keyword with single-element array"),
        )
        .with_annotation(
            Annotation::secondary(yield_from.iterator.span())
                .with_message("Single-element array creates unnecessary overhead"),
        )
        .with_note("Direct `yield` avoids the array allocation and iteration.")
        .with_note("This is more efficient and easier to read.")
        .with_help("Replace with direct `yield` for better performance.");

        ctx.collector.propose(issue, |plan| {
            // `yield from [...]` -> `yield [...]`
            plan.delete(yield_from.from.span.to_range(), SafetyClassification::Safe);

            match yield_from.iterator {
                // `yield [expr]` -> `yield expr`
                Expression::Array(array) => {
                    plan.delete(array.left_bracket.to_range(), SafetyClassification::Safe);
                    plan.delete(array.right_bracket.to_range(), SafetyClassification::Safe);
                }
                // `yield array(expr)` -> `yield expr`
                Expression::LegacyArray(legacy_array) => {
                    plan.delete(legacy_array.array.span.to_range(), SafetyClassification::Safe);
                    plan.delete(legacy_array.left_parenthesis.to_range(), SafetyClassification::Safe);
                    plan.delete(legacy_array.right_parenthesis.to_range(), SafetyClassification::Safe);
                }
                _ => unreachable!("Already filtered out non-array literals"),
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoRedundantYieldFromRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = direct_yield_is_fine,
        rule = NoRedundantYieldFromRule,
        code = indoc! {r"
            <?php

            function gen() {
                yield 1;
                yield 'key' => 2;
            }
        "}
    }

    test_lint_success! {
        name = yield_from_multi_element_array_is_fine,
        rule = NoRedundantYieldFromRule,
        code = indoc! {r"
            <?php

            function gen() {
                yield from [1, 2, 3];
                yield from array(4, 5, 6);
            }
        "}
    }

    test_lint_success! {
        name = yield_from_variable_is_fine,
        rule = NoRedundantYieldFromRule,
        code = indoc! {r"
            <?php

            function gen($iterator) {
                yield from $iterator;
            }
        "}
    }

    test_lint_success! {
        name = yield_from_function_call_is_fine,
        rule = NoRedundantYieldFromRule,
        code = indoc! {r"
            <?php

            function gen() {
                yield from getItems();
            }
        "}
    }

    test_lint_success! {
        name = yield_from_empty_array_is_fine,
        rule = NoRedundantYieldFromRule,
        code = indoc! {r"
            <?php

            function gen() {
                yield from [];
                yield from array();
            }
        "}
    }

    // Failure cases - SHOULD produce lint issues

    test_lint_failure! {
        name = yield_from_single_value_array,
        rule = NoRedundantYieldFromRule,
        code = indoc! {r"
            <?php

            function gen() {
                yield from [1];
            }
        "}
    }

    test_lint_failure! {
        name = yield_from_single_value_legacy_array,
        rule = NoRedundantYieldFromRule,
        code = indoc! {r"
            <?php

            function gen() {
                yield from array(1);
            }
        "}
    }

    test_lint_failure! {
        name = yield_from_single_keyed_array,
        rule = NoRedundantYieldFromRule,
        code = indoc! {r"
            <?php

            function gen() {
                yield from ['foo' => 'bar'];
            }
        "}
    }

    test_lint_failure! {
        name = yield_from_single_keyed_legacy_array,
        rule = NoRedundantYieldFromRule,
        code = indoc! {r"
            <?php

            function gen() {
                yield from array('foo' => 'bar');
            }
        "}
    }

    test_lint_failure! {
        name = yield_from_single_complex_value,
        rule = NoRedundantYieldFromRule,
        code = indoc! {r"
            <?php

            function gen() {
                yield from [new stdClass()];
            }
        "}
    }

    test_lint_failure! {
        name = yield_from_single_keyed_complex_value,
        rule = NoRedundantYieldFromRule,
        code = indoc! {r"
            <?php

            function gen() {
                yield from ['key' => new stdClass()];
            }
        "}
    }

    test_lint_failure! {
        name = multiple_single_element_yield_from,
        rule = NoRedundantYieldFromRule,
        count = 3,
        code = indoc! {r"
            <?php

            function gen() {
                yield from [1];
                yield from array(2);
                yield from ['key' => 3];
            }
        "}
    }

    test_lint_failure! {
        name = yield_from_single_numeric_key,
        rule = NoRedundantYieldFromRule,
        code = indoc! {r"
            <?php

            function gen() {
                yield from [0 => 'value'];
            }
        "}
    }
}
