use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct SortedIntegerKeysRule {
    meta: &'static RuleMeta,
    cfg: SortedIntegerKeysConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct SortedIntegerKeysConfig {
    pub level: Level,
}

impl Default for SortedIntegerKeysConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for SortedIntegerKeysConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for SortedIntegerKeysRule {
    type Config = SortedIntegerKeysConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Sorted Integer Keys",
            code: "sorted-integer-keys",
            description: indoc! {"
                Detects array literals with integer keys that are not in ascending order.

                PHP internally uses a \"packed array\" optimization for arrays with integer
                keys in natural ascending order, which consumes significantly less memory
                and is faster. When integer keys are out of order, PHP falls back to a
                regular hash table.
            "},
            good_example: indoc! {r"
                <?php

                $weights = [
                    2  => 0.06011,
                    3  => 0.506,
                    4  => 0.01233,
                    5  => 0.21246,
                    10 => 0.10823,
                    20 => 0.06206,
                ];
            "},
            bad_example: indoc! {r"
                <?php

                $weights = [
                    3  => 0.506,
                    5  => 0.21246,
                    10 => 0.10823,
                    20 => 0.06206,
                    2  => 0.06011,
                    4  => 0.01233,
                ];
            "},
            category: Category::BestPractices,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Array, NodeKind::LegacyArray];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let (elements, array_span) = match node {
            Node::Array(array) => (&array.elements, array.span()),
            Node::LegacyArray(array) => (&array.elements, array.span()),
            _ => return,
        };

        if elements.len() < 2 {
            return;
        }

        let mut keys: Vec<(u64, Span)> = Vec::with_capacity(elements.len());
        for element in elements.iter() {
            let ArrayElement::KeyValue(kv) = element else {
                return;
            };

            let Some(value) = extract_literal_int(kv.key) else {
                return;
            };

            keys.push((value, kv.key.span()));
        }

        let is_sorted = keys.windows(2).all(|w| {
            let [a, b] = w else { return true };
            a.0 <= b.0
        });
        if is_sorted {
            return;
        }

        ctx.collector.report(
            Issue::new(self.cfg.level(), "Integer array keys are not in ascending order.")
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(array_span).with_message("keys are out of order"))
                .with_note("PHP uses a memory-efficient \"packed array\" for integer keys in ascending order.")
                .with_note("Out-of-order keys force a hash table fallback with higher memory and CPU overhead.")
                .with_help("Sort the array entries by key in ascending order to enable the packed array optimization."),
        );
    }
}

fn extract_literal_int(expr: &Expression<'_>) -> Option<u64> {
    match expr {
        Expression::Literal(Literal::Integer(LiteralInteger { value: Some(v), .. })) => Some(*v),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::SortedIntegerKeysRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = already_sorted,
        rule = SortedIntegerKeysRule,
        code = indoc! {r"
            <?php

            $items = [0 => 'a', 1 => 'b', 2 => 'c'];
        "}
    }

    test_lint_success! {
        name = string_keys,
        rule = SortedIntegerKeysRule,
        code = indoc! {r#"
            <?php

            $config = ['host' => 'localhost', 'port' => 3306];
        "#}
    }

    test_lint_success! {
        name = no_keys,
        rule = SortedIntegerKeysRule,
        code = indoc! {r"
            <?php

            $items = ['a', 'b', 'c'];
        "}
    }

    test_lint_success! {
        name = single_element,
        rule = SortedIntegerKeysRule,
        code = indoc! {r"
            <?php

            $items = [1 => 'a'];
        "}
    }

    test_lint_success! {
        name = empty_array,
        rule = SortedIntegerKeysRule,
        code = indoc! {r"
            <?php

            $items = [];
        "}
    }

    test_lint_success! {
        name = mixed_keys,
        rule = SortedIntegerKeysRule,
        code = indoc! {r#"
            <?php

            $items = [0 => 'a', 'key' => 'b', 1 => 'c'];
        "#}
    }

    test_lint_success! {
        name = equal_keys_is_ok,
        rule = SortedIntegerKeysRule,
        code = indoc! {r"
            <?php

            $items = [1 => 'a', 1 => 'b', 2 => 'c'];
        "}
    }

    test_lint_failure! {
        name = unsorted_keys,
        rule = SortedIntegerKeysRule,
        code = indoc! {r"
            <?php

            $weights = [
                3  => 0.506,
                5  => 0.21246,
                10 => 0.10823,
                20 => 0.06206,
                2  => 0.06011,
                4  => 0.01233,
            ];
        "}
    }

    test_lint_failure! {
        name = unsorted_legacy_array,
        rule = SortedIntegerKeysRule,
        code = indoc! {r"
            <?php

            $items = array(3 => 'c', 1 => 'a', 2 => 'b');
        "}
    }

    test_lint_failure! {
        name = reversed_keys,
        rule = SortedIntegerKeysRule,
        code = indoc! {r"
            <?php

            $items = [2 => 'b', 1 => 'a', 0 => 'c'];
        "}
    }

    test_lint_success! {
        name = variadic_element_skips,
        rule = SortedIntegerKeysRule,
        code = indoc! {r"
            <?php

            $items = [3 => 'c', ...$other, 1 => 'a'];
        "}
    }
}
