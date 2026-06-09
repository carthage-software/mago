use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Argument;
use mago_syntax::ast::FunctionCall;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::UnaryPrefixOperator;
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
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

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::FunctionCall(function_call) = node else {
            return;
        };

        if !function_call_matches(ctx, function_call, "is_null") {
            return;
        }

        let context = surrounding_context(ctx, function_call);

        let (title, help) = if context.negated {
            ("Use `!== null` instead of `!is_null()`.", "Replace with a strict `!== null` comparison.")
        } else {
            ("Use `=== null` instead of `is_null()`.", "Replace with a strict `=== null` comparison.")
        };

        let annotation_message = if context.negated { "`!is_null()` is redundant" } else { "`is_null()` is redundant" };

        let issue = Issue::new(self.cfg.level(), title)
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(context.span).with_message(annotation_message))
            .with_help(help);

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
            let arg_text =
                mago_bytes::BytesDisplay(&ctx.source_file.contents[arguments[0].value().span().to_range_usize()]);
            let operator = if context.negated { "!==" } else { "===" };
            edits.push(TextEdit::replace(context.span, format!("null {operator} {arg_text}")));
        });
    }
}

struct SurroundingContext {
    /// Span to replace: covers the call plus any surrounding `!`s and redundant
    /// parens that exist purely to wrap our expression.
    span: mago_span::Span,
    /// `true` when an odd number of `!` operators wrap the call.
    negated: bool,
}

/// Walk up from the `is_null(...)` call collecting consecutive `!` operators
/// and the redundant parens between them, so the fix can consume the whole run
/// in one edit and pick `===` vs `!==` from the parity.
fn surrounding_context<'arena, A>(
    ctx: &LintContext<'_, 'arena, A>,
    function_call: &'arena FunctionCall<'arena>,
) -> SurroundingContext
where
    A: Arena,
{
    let mut span = function_call.span();
    let mut negated = false;
    let mut n = 0;

    while let Some(parent) = ctx.get_nth_parent(n) {
        match parent {
            Node::Expression(_) | Node::Call(_) => {
                n += 1;
            }
            Node::Parenthesized(parenthesized) => {
                span = parenthesized.span();
                n += 1;
            }
            Node::UnaryPrefix(unary) if matches!(unary.operator, UnaryPrefixOperator::Not(_)) => {
                span = unary.span();
                negated = !negated;
                n += 1;
            }
            _ => break,
        }
    }

    SurroundingContext { span, negated }
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

            if (null === $value) {
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

            if (null === $value) {
                // ...
            }
        "#}
    }

    test_lint_fix! {
        name = fix_negated_is_null_to_strict_not_equal,
        rule = NoIsNullRule,
        code = indoc! {r#"
            <?php

            if (!is_null($value)) {
                // ...
            }
        "#},
        fixed = indoc! {r#"
            <?php

            if (null !== $value) {
                // ...
            }
        "#}
    }

    test_lint_fix! {
        name = fix_negated_is_null_with_whitespace,
        rule = NoIsNullRule,
        code = indoc! {r#"
            <?php

            echo ! is_null($x);
        "#},
        fixed = indoc! {r#"
            <?php

            echo null !== $x;
        "#}
    }

    test_lint_fix! {
        name = fix_negated_is_null_inside_parens,
        rule = NoIsNullRule,
        code = indoc! {r#"
            <?php

            $r = !(is_null($value));
        "#},
        fixed = indoc! {r#"
            <?php

            $r = null !== $value;
        "#}
    }

    test_lint_fix! {
        name = double_negation_strips_to_positive_form,
        rule = NoIsNullRule,
        code = indoc! {r#"
            <?php

            $r = !!is_null($value);
        "#},
        fixed = indoc! {r#"
            <?php

            $r = null === $value;
        "#}
    }

    test_lint_fix! {
        name = triple_negation_strips_to_negative_form,
        rule = NoIsNullRule,
        code = indoc! {r#"
            <?php

            $r = !!!is_null($value);
        "#},
        fixed = indoc! {r#"
            <?php

            $r = null !== $value;
        "#}
    }

    test_lint_fix! {
        name = redundant_outer_parens_dropped,
        rule = NoIsNullRule,
        code = indoc! {r#"
            <?php

            $r = ((is_null($value)));
        "#},
        fixed = indoc! {r#"
            <?php

            $r = null === $value;
        "#}
    }
}
