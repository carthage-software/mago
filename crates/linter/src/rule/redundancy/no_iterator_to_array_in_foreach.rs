use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Call;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoIteratorToArrayInForeachRule {
    meta: &'static RuleMeta,
    cfg: NoIteratorToArrayInForeachConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoIteratorToArrayInForeachConfig {
    pub level: Level,
}

impl Default for NoIteratorToArrayInForeachConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoIteratorToArrayInForeachConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoIteratorToArrayInForeachRule {
    type Config = NoIteratorToArrayInForeachConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Iterator To Array In Foreach",
            code: "no-iterator-to-array-in-foreach",
            description: indoc! {"
                Detects `iterator_to_array()` calls used directly as a `foreach` expression.

                Since `foreach` natively supports any `Traversable`, wrapping an iterator in
                `iterator_to_array()` is redundant and causes unnecessary memory allocation.
            "},
            good_example: indoc! {r"
                <?php

                foreach ($iterator as $value) {
                    // ...
                }
            "},
            bad_example: indoc! {r"
                <?php

                foreach (iterator_to_array($iterator) as $value) {
                    // ...
                }
            "},
            category: Category::Redundancy,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Foreach];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::Foreach(foreach) = node else {
            return;
        };

        let Expression::Call(Call::Function(function_call)) = foreach.expression else {
            return;
        };

        if !function_call_matches(ctx, function_call, "iterator_to_array") {
            return;
        }

        ctx.collector.report(
            Issue::new(self.cfg.level(), "Unnecessary `iterator_to_array()` in `foreach`.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(function_call.span())
                        .with_message("`foreach` can iterate over any `Traversable` directly"),
                )
                .with_note("Wrapping an iterator in `iterator_to_array()` materializes the entire dataset into an array, wasting memory and CPU.")
                .with_help("Remove `iterator_to_array()` and pass the iterator directly to `foreach`."),
        );
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoIteratorToArrayInForeachRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = direct_iterator,
        rule = NoIteratorToArrayInForeachRule,
        code = indoc! {r"
            <?php

            foreach ($iterator as $value) {
                echo $value;
            }
        "}
    }

    test_lint_success! {
        name = iterator_to_array_outside_foreach,
        rule = NoIteratorToArrayInForeachRule,
        code = indoc! {r"
            <?php

            $items = iterator_to_array($iterator);
        "}
    }

    test_lint_failure! {
        name = iterator_to_array_in_foreach,
        rule = NoIteratorToArrayInForeachRule,
        code = indoc! {r"
            <?php

            foreach (iterator_to_array($iterator) as $value) {
                echo $value;
            }
        "}
    }

    test_lint_failure! {
        name = iterator_to_array_with_preserve_keys,
        rule = NoIteratorToArrayInForeachRule,
        code = indoc! {r"
            <?php

            foreach (iterator_to_array($iterator, false) as $value) {
                echo $value;
            }
        "}
    }

    test_lint_failure! {
        name = iterator_to_array_uppercase,
        rule = NoIteratorToArrayInForeachRule,
        code = indoc! {r"
            <?php

            foreach (ITERATOR_TO_ARRAY($iterator) as $value) {
                echo $value;
            }
        "}
    }

    test_lint_failure! {
        name = iterator_to_array_colon_delimited_with_alias,
        rule = NoIteratorToArrayInForeachRule,
        code = indoc! {r"
            <?php

            use function iterator_to_array as to_Array;

            foreach (to_Array($iterator) as $value):
                echo $value;
            endforeach;
        "}
    }
}
