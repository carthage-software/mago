use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Argument;
use mago_syntax::ast::ArrayElement;
use mago_syntax::ast::Expression;
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
pub struct PreferArraySpreadRule {
    meta: &'static RuleMeta,
    cfg: PreferArraySpreadConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PreferArraySpreadConfig {
    pub level: Level,
}

impl Default for PreferArraySpreadConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for PreferArraySpreadConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PreferArraySpreadRule {
    type Config = PreferArraySpreadConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prefer Array Spread",
            code: "prefer-array-spread",
            description: indoc! {r"
                Detects calls to `array_merge()` and suggests using the array spread operator (`...`)
                in an array literal instead.

                The spread operator is more concise, avoids a function call, and makes the intent
                of the merge clear at the call site. Since PHP 8.1, the spread operator supports
                string keys in addition to integer keys, making it a complete replacement for
                `array_merge()`.
            "},
            good_example: indoc! {r"
                <?php

                $merged = [...$first, ...$second, ...$third];
            "},
            bad_example: indoc! {r"
                <?php

                $merged = array_merge($first, $second, $third);
            "},
            category: Category::BestPractices,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP81)),
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

        if !function_call_matches(ctx, function_call, "array_merge") {
            return;
        }

        let arguments = &function_call.argument_list.arguments.nodes;

        // `array_merge(...$arrays)` would need a loop to express as a spread literal,
        // so leave such calls alone.
        let has_unspreadable_arg = arguments.iter().any(|arg| match arg {
            Argument::Named(_) => true,
            Argument::Positional(positional) => positional.ellipsis.is_some(),
        });

        if has_unspreadable_arg {
            return;
        }

        let issue = Issue::new(self.cfg.level(), "Use the array spread operator instead of `array_merge()`.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(function_call.span())
                    .with_message("`array_merge()` can be replaced with array spread (`[...$a, ...$b]`)"),
            )
            .with_note(
                "The spread operator avoids a function call and makes the merge explicit at the call site. Since PHP 8.1, it supports both integer and string keys.",
            )
            .with_help("Replace this `array_merge` call with an array literal that spreads each argument.");

        ctx.collector.propose(issue, |edits| {
            if arguments.is_empty() {
                edits.push(TextEdit::replace(function_call.span(), "[]"));

                return;
            }

            edits.push(TextEdit::replace(
                function_call.start_offset()..function_call.argument_list.left_parenthesis.end_offset(),
                "[",
            ));

            let tokens = &function_call.argument_list.arguments.tokens;
            let right_paren_start = function_call.argument_list.right_parenthesis.start_offset();
            // Tracks whether each comma in `tokens` has been swallowed by an adjacent empty-array
            // arg, so two consecutive empties don't try to delete the same comma.
            let mut consumed_comma = vec![false; tokens.len()];

            for (idx, arg) in arguments.iter().enumerate() {
                let Argument::Positional(positional) = arg else {
                    continue;
                };

                if is_empty_array_literal(positional.value) {
                    // Prefer to swallow the trailing comma; fall back to the leading one;
                    // and if neither is available (single-arg call), just delete the arg.
                    if tokens.get(idx).is_some() && !consumed_comma[idx] {
                        consumed_comma[idx] = true;
                        let next_start = arguments.get(idx + 1).map(|a| a.start_offset()).unwrap_or(right_paren_start);
                        edits.push(TextEdit::delete(positional.value.start_offset()..next_start));
                    } else if idx > 0 && !consumed_comma[idx - 1] {
                        consumed_comma[idx - 1] = true;
                        edits.push(TextEdit::delete(tokens[idx - 1].start.offset..positional.value.end_offset()));
                    } else {
                        edits.push(TextEdit::delete(positional.value.span()));
                    }

                    continue;
                }

                if !try_inline_array_literal(edits, positional.value) {
                    edits.push(TextEdit::insert(positional.value.start_offset(), "..."));
                }
            }

            edits.push(TextEdit::replace(function_call.argument_list.right_parenthesis, "]"));
        });
    }
}

fn is_empty_array_literal(value: &Expression<'_>) -> bool {
    match value {
        Expression::Array(arr) => arr.elements.is_empty(),
        Expression::LegacyArray(arr) => arr.elements.is_empty(),
        _ => false,
    }
}

/// If `value` is a non-empty array literal that can be safely lifted into the surrounding array
/// literal, push edits that strip its delimiters (and any trailing comma) and return `true`.
///
/// `[1, 2, 3]` becomes `1, 2, 3` so the caller produces `[..., 1, 2, 3, ...]` rather than
/// `[..., ...[1, 2, 3], ...]`. Empty literals and ones with missing elements (`[1, , 3]`) are
/// left for the spread fallback because lifting them would create stray commas.
fn try_inline_array_literal(edits: &mut Vec<TextEdit>, value: &Expression<'_>) -> bool {
    let elements = match value {
        Expression::Array(arr) => &arr.elements,
        Expression::LegacyArray(arr) => &arr.elements,
        _ => return false,
    };

    if elements.iter().any(|e| matches!(e, ArrayElement::Missing(_))) {
        return false;
    }

    let (Some(first), Some(last)) = (elements.first(), elements.last()) else {
        return false;
    };

    // Strip the opening delimiter (`[` or `array(`) plus any whitespace before the first element.
    edits.push(TextEdit::delete(value.start_offset()..first.start_offset()));
    // Strip an optional trailing comma, any whitespace, and the closing delimiter (`]` or `)`).
    edits.push(TextEdit::delete(last.end_offset()..value.end_offset()));

    true
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::PreferArraySpreadRule;
    use crate::test_lint_failure;
    use crate::test_lint_fix;
    use crate::test_lint_success;

    test_lint_success! {
        name = spread_operator_is_fine,
        rule = PreferArraySpreadRule,
        code = indoc! {r#"
            <?php

            $merged = [...$first, ...$second];
        "#}
    }

    test_lint_success! {
        name = call_with_unpacked_argument_is_not_reported,
        rule = PreferArraySpreadRule,
        code = indoc! {r#"
            <?php

            $merged = array_merge(...$arrays);
        "#}
    }

    test_lint_failure! {
        name = multiple_argument_call,
        rule = PreferArraySpreadRule,
        code = indoc! {r#"
            <?php

            $merged = array_merge($a, $b, $c);
        "#}
    }

    test_lint_failure! {
        name = single_argument_call,
        rule = PreferArraySpreadRule,
        code = indoc! {r#"
            <?php

            $copied = array_merge($a);
        "#}
    }

    test_lint_failure! {
        name = no_argument_call,
        rule = PreferArraySpreadRule,
        code = indoc! {r#"
            <?php

            $empty = array_merge();
        "#}
    }

    test_lint_fix! {
        name = fix_multiple_argument_call,
        rule = PreferArraySpreadRule,
        code = indoc! {r#"
            <?php

            $merged = array_merge($a, $b, $c);
        "#},
        fixed = indoc! {r#"
            <?php

            $merged = [...$a, ...$b, ...$c];
        "#}
    }

    test_lint_fix! {
        name = fix_single_argument_call,
        rule = PreferArraySpreadRule,
        code = indoc! {r#"
            <?php

            $copied = array_merge($a);
        "#},
        fixed = indoc! {r#"
            <?php

            $copied = [...$a];
        "#}
    }

    test_lint_fix! {
        name = fix_no_argument_call,
        rule = PreferArraySpreadRule,
        code = indoc! {r#"
            <?php

            $empty = array_merge();
        "#},
        fixed = indoc! {r#"
            <?php

            $empty = [];
        "#}
    }

    test_lint_fix! {
        name = fix_array_literal_argument_is_inlined,
        rule = PreferArraySpreadRule,
        code = indoc! {r#"
            <?php

            $merged = array_merge($a, [1, 2, 3]);
        "#},
        fixed = indoc! {r#"
            <?php

            $merged = [...$a, 1, 2, 3];
        "#}
    }

    test_lint_fix! {
        name = fix_legacy_array_literal_is_inlined,
        rule = PreferArraySpreadRule,
        code = indoc! {r#"
            <?php

            $merged = array_merge($a, array(1, 2, 3));
        "#},
        fixed = indoc! {r#"
            <?php

            $merged = [...$a, 1, 2, 3];
        "#}
    }

    test_lint_fix! {
        name = fix_array_literal_with_string_keys_is_inlined,
        rule = PreferArraySpreadRule,
        code = indoc! {r#"
            <?php

            $merged = array_merge($a, ['x' => 1, 'y' => 2]);
        "#},
        fixed = indoc! {r#"
            <?php

            $merged = [...$a, 'x' => 1, 'y' => 2];
        "#}
    }

    test_lint_fix! {
        name = fix_first_argument_array_literal_is_inlined,
        rule = PreferArraySpreadRule,
        code = indoc! {r#"
            <?php

            $merged = array_merge([1, 2], $b, [3, 4]);
        "#},
        fixed = indoc! {r#"
            <?php

            $merged = [1, 2, ...$b, 3, 4];
        "#}
    }

    test_lint_fix! {
        name = fix_inlined_literal_drops_trailing_comma,
        rule = PreferArraySpreadRule,
        code = indoc! {r#"
            <?php

            $merged = array_merge([1, 2,], $b);
        "#},
        fixed = indoc! {r#"
            <?php

            $merged = [1, 2, ...$b];
        "#}
    }

    test_lint_fix! {
        name = fix_empty_array_at_end_is_dropped,
        rule = PreferArraySpreadRule,
        code = indoc! {r#"
            <?php

            $merged = array_merge($a, []);
        "#},
        fixed = indoc! {r#"
            <?php

            $merged = [...$a];
        "#}
    }

    test_lint_fix! {
        name = fix_empty_array_at_start_is_dropped,
        rule = PreferArraySpreadRule,
        code = indoc! {r#"
            <?php

            $merged = array_merge([], $a);
        "#},
        fixed = indoc! {r#"
            <?php

            $merged = [...$a];
        "#}
    }

    test_lint_fix! {
        name = fix_empty_array_in_middle_is_dropped,
        rule = PreferArraySpreadRule,
        code = indoc! {r#"
            <?php

            $merged = array_merge($a, [], $b);
        "#},
        fixed = indoc! {r#"
            <?php

            $merged = [...$a, ...$b];
        "#}
    }

    test_lint_fix! {
        name = fix_consecutive_empty_arrays_are_dropped,
        rule = PreferArraySpreadRule,
        code = indoc! {r#"
            <?php

            $merged = array_merge($a, [], [], $b);
        "#},
        fixed = indoc! {r#"
            <?php

            $merged = [...$a, ...$b];
        "#}
    }

    test_lint_fix! {
        name = fix_only_empty_array_becomes_empty_literal,
        rule = PreferArraySpreadRule,
        code = indoc! {r#"
            <?php

            $merged = array_merge([]);
        "#},
        fixed = indoc! {r#"
            <?php

            $merged = [];
        "#}
    }

    test_lint_fix! {
        name = fix_all_empty_arrays_becomes_empty_literal,
        rule = PreferArraySpreadRule,
        code = indoc! {r#"
            <?php

            $merged = array_merge([], []);
        "#},
        fixed = indoc! {r#"
            <?php

            $merged = [];
        "#}
    }

    test_lint_fix! {
        name = fix_legacy_empty_array_is_dropped,
        rule = PreferArraySpreadRule,
        code = indoc! {r#"
            <?php

            $merged = array_merge($a, array());
        "#},
        fixed = indoc! {r#"
            <?php

            $merged = [...$a];
        "#}
    }

    test_lint_fix! {
        name = fix_call_with_function_call_argument,
        rule = PreferArraySpreadRule,
        code = indoc! {r#"
            <?php

            $merged = array_merge($a, get_extras());
        "#},
        fixed = indoc! {r#"
            <?php

            $merged = [...$a, ...get_extras()];
        "#}
    }

    test_lint_fix! {
        name = fix_namespaced_call,
        rule = PreferArraySpreadRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $merged = \array_merge($a, $b);
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            $merged = [...$a, ...$b];
        "#}
    }
}
