use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Argument;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoIsNullRule {
    meta: &'static RuleMeta,
    cfg: NoIsNullConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoIsNullConfig {
    pub level: Level,
}

impl Default for NoIsNullConfig {
    fn default() -> Self {
        Self { level: Level::Note }
    }
}

impl Config for NoIsNullConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoIsNullRule {
    type Config = NoIsNullConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Is Null",
            code: "no-is-null",
            description: indoc! {"
                Detects usage of the `is_null()` function and suggests using a strict `=== null` comparison instead.

                The `is_null()` function is redundant because `=== null` achieves the same result with clearer intent
                and without the overhead of a function call.
            "},
            good_example: indoc! {r"
                <?php

                if ($value === null) {
                    // ...
                }
            "},
            bad_example: indoc! {r"
                <?php

                if (is_null($value)) {
                    // ...
                }
            "},
            category: Category::Redundancy,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::FunctionCall];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::FunctionCall(function_call) = node else {
            return;
        };

        if !function_call_matches(ctx, function_call, "is_null") {
            return;
        }

        let issue = Issue::new(self.cfg.level(), "Use `=== null` instead of `is_null()`.")
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(function_call.span()).with_message("`is_null()` is redundant"))
            .with_help("Replace with a strict `=== null` comparison.");

        // Only fix single-argument calls without spread.
        let arguments = &function_call.argument_list.arguments;
        if arguments.len() != 1 {
            ctx.collector.report(issue);

            return;
        }

        // Don't auto-fix spread arguments like is_null(...$v).
        if let Some(Argument::Positional(positional)) = arguments.first()
            && positional.ellipsis.is_some()
        {
            ctx.collector.report(issue);

            return;
        }

        ctx.collector.propose(issue, |edits| {
            edits.push(TextEdit::replace(
                function_call.start_offset()..function_call.argument_list.left_parenthesis.end_offset(),
                "(null === ",
            ));
        });
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoIsNullRule;
    use crate::test_lint_failure;
    use crate::test_lint_fix;
    use crate::test_lint_success;

    test_lint_success! {
        name = null_comparison,
        rule = NoIsNullRule,
        code = indoc! {r"
            <?php

            if (null === $value) {
                // ...
            }
        "}
    }

    test_lint_failure! {
        name = is_null_call,
        rule = NoIsNullRule,
        code = indoc! {r"
            <?php

            if (is_null($value)) {
                // ...
            }
        "}
    }

    test_lint_failure! {
        name = is_null_uppercase,
        rule = NoIsNullRule,
        code = indoc! {r"
            <?php

            if (IS_NULL($value)) {
                // ...
            }
        "}
    }

    test_lint_failure! {
        name = is_null_in_expression,
        rule = NoIsNullRule,
        code = indoc! {r"
            <?php

            $result = is_null($foo) ? 'yes' : 'no';
        "}
    }

    test_lint_fix! {
        name = fix_is_null_to_null_comparison,
        rule = NoIsNullRule,
        code = indoc! {r#"
            <?php

            if (is_null($value)) {
                // ...
            }
        "#},
        fixed = indoc! {r#"
            <?php

            if ((null === $value)) {
                // ...
            }
        "#}
    }

    test_lint_fix! {
        name = fix_is_null_to_null_comparison_alias,
        rule = NoIsNullRule,
        code = indoc! {r#"
            <?php

            use function is_null as is_that_one_value;

            if (is_that_one_value($value)) {
                // ...
            }
        "#},
        fixed = indoc! {r#"
            <?php

            use function is_null as is_that_one_value;

            if ((null === $value)) {
                // ...
            }
        "#}
    }
}
