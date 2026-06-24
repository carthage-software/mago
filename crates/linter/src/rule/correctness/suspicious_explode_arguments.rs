use indoc::indoc;
use schemars::JsonSchema;

use mago_allocator::Arena;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::cst::Argument;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Literal;
use mago_syntax::cst::LiteralString;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct SuspiciousExplodeArgumentsRule {
    meta: &'static RuleMeta,
    cfg: SuspiciousExplodeArgumentsConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct SuspiciousExplodeArgumentsConfig {
    pub level: Level,
    /// Maximum length of the second argument's literal string for the call to be considered suspicious.
    ///
    /// The string being split is usually longer than the separator, so a short literal in the
    /// second position is a strong hint that the arguments were swapped.
    pub threshold: usize,
}

impl Default for SuspiciousExplodeArgumentsConfig {
    fn default() -> Self {
        Self { level: Level::Warning, threshold: 10 }
    }
}

impl Config for SuspiciousExplodeArgumentsConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for SuspiciousExplodeArgumentsRule {
    type Config = SuspiciousExplodeArgumentsConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Suspicious Explode Arguments",
            code: "suspicious-explode-arguments",
            description: indoc! {r"
                Detects `explode` calls whose arguments appear to be swapped.

                The signature is `explode(string $separator, string $string)`: the separator comes first
                and the string being split comes second. When the second argument is a short string literal
                while the first is not a shorter literal, the arguments were most likely passed in the wrong
                order, which produces a silently incorrect result instead of an error.
            "},
            good_example: indoc! {r"
                <?php

                $parts = explode(' ', $sentence);
            "},
            bad_example: indoc! {r"
                <?php

                $parts = explode($sentence, ' ');
            "},
            category: Category::Correctness,

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

        if !function_call_matches(ctx, function_call, "explode") {
            return;
        }

        let arguments = &function_call.argument_list.arguments.nodes;

        let (Some(Argument::Positional(separator_argument)), Some(Argument::Positional(string_argument))) =
            (arguments.first(), arguments.get(1))
        else {
            return;
        };

        if separator_argument.ellipsis.is_some() || string_argument.ellipsis.is_some() {
            return;
        }

        let Some(string_literal) = as_string_literal(string_argument.value) else {
            return;
        };

        let Some(string_value) = string_literal.value else {
            return;
        };

        if string_value.len() > self.cfg.threshold {
            return;
        }

        if let Some(separator_literal) = as_string_literal(separator_argument.value)
            && let Some(separator_value) = separator_literal.value
            && separator_value.len() <= string_value.len()
        {
            return;
        }

        let issue = Issue::new(self.cfg.level, "The arguments to `explode` may be swapped.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(string_argument.value.span())
                    .with_message("This short literal looks like the separator, but it is in the string position"),
            )
            .with_annotation(
                Annotation::secondary(separator_argument.value.span())
                    .with_message("...while this is being used as the separator"),
            )
            .with_note(
                "`explode` expects the separator first and the string to split second: `explode($separator, $string)`.",
            )
            .with_help("Swap the arguments so the separator comes first.");

        ctx.collector.report(issue);
    }
}

fn as_string_literal<'arena>(expression: &'arena Expression<'arena>) -> Option<&'arena LiteralString<'arena>> {
    match expression {
        Expression::Literal(Literal::String(literal)) => Some(literal),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;

    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_failure! {
        name = warns_when_second_argument_is_a_short_literal,
        rule = SuspiciousExplodeArgumentsRule,
        count = 1,
        code = indoc! {r"
            <?php

            $parts = explode($sentence, ' ');
        "},
    }

    test_lint_failure! {
        name = warns_when_both_literals_and_second_is_shorter,
        rule = SuspiciousExplodeArgumentsRule,
        count = 1,
        code = indoc! {r"
            <?php

            $parts = explode('hello', ',');
        "},
    }

    test_lint_success! {
        name = allows_separator_first,
        rule = SuspiciousExplodeArgumentsRule,
        code = indoc! {r"
            <?php

            $parts = explode(' ', $sentence);
        "},
    }

    test_lint_success! {
        name = allows_short_literal_separator_first,
        rule = SuspiciousExplodeArgumentsRule,
        code = indoc! {r"
            <?php

            $parts = explode(',', 'hello');
        "},
    }

    test_lint_success! {
        name = allows_non_literal_arguments,
        rule = SuspiciousExplodeArgumentsRule,
        code = indoc! {r"
            <?php

            $parts = explode($separator, $sentence);
        "},
    }

    test_lint_success! {
        name = allows_long_second_literal,
        rule = SuspiciousExplodeArgumentsRule,
        code = indoc! {r"
            <?php

            $parts = explode($separator, 'this is a long sentence');
        "},
    }
}
