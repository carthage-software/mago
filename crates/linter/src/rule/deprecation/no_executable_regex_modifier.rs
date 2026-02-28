use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Argument;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Literal;
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
pub struct NoExecutableRegexModifierRule {
    meta: &'static RuleMeta,
    cfg: NoExecutableRegexModifierConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoExecutableRegexModifierConfig {
    pub level: Level,
}

impl Default for NoExecutableRegexModifierConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for NoExecutableRegexModifierConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoExecutableRegexModifierRule {
    type Config = NoExecutableRegexModifierConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Executable Regex Modifier",
            code: "no-executable-regex-modifier",
            description: indoc! {"
                Flags the use of the `e` (executable) modifier in `preg_replace()` patterns.
                The `e` modifier causes the replacement string to be evaluated as PHP code,
                which is a security vulnerability. It was deprecated in PHP 5.5 and removed in PHP 7.0.
                Use `preg_replace_callback()` instead.
            "},
            good_example: indoc! {r#"
                <?php

                $result = preg_replace_callback('/pattern/', function ($matches) {
                    return strtoupper($matches[0]);
                }, $subject);
            "#},
            bad_example: indoc! {r#"
                <?php

                $result = preg_replace('/pattern/e', 'strtoupper("$1")', $subject);
            "#},
            category: Category::Deprecation,

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

        if !function_call_matches(ctx, function_call, "preg_replace") {
            return;
        }

        // Get the first argument (the regex pattern)
        let Some(first_arg) = function_call.argument_list.arguments.first() else {
            return;
        };

        let pattern_str = match first_arg {
            Argument::Positional(arg) => extract_string_value(arg.value),
            Argument::Named(arg) => extract_string_value(arg.value),
        };

        let Some(pattern) = pattern_str else {
            return;
        };

        if has_executable_modifier(pattern) {
            ctx.collector.report(
                Issue::new(
                    self.cfg.level(),
                    "The `e` modifier in `preg_replace()` is deprecated and dangerous.",
                )
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(first_arg.span())
                        .with_message("This pattern uses the executable `e` modifier"),
                )
                .with_help("Use `preg_replace_callback()` instead of the `e` modifier."),
            );
        }
    }
}

fn extract_string_value<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    match expr {
        Expression::Literal(Literal::String(s)) => s.value,
        _ => None,
    }
}

/// Checks if a regex pattern string contains the `e` modifier.
///
/// The pattern format is: `<delimiter><pattern><delimiter><modifiers>`
/// Common delimiters: `/`, `#`, `~`, `|`
fn has_executable_modifier(pattern: &str) -> bool {
    // Find the delimiter (first character)
    let mut chars = pattern.chars();
    let Some(delimiter) = chars.next() else {
        return false;
    };

    // Find the closing delimiter (last occurrence)
    if let Some(last_delim_pos) = pattern.rfind(delimiter).filter(|&pos| pos > 0) {
        let modifiers = &pattern[last_delim_pos + delimiter.len_utf8()..];
        modifiers.contains('e')
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = preg_replace_without_e_modifier,
        rule = NoExecutableRegexModifierRule,
        code = r#"
            <?php

            $result = preg_replace('/pattern/i', 'replacement', $subject);
        "#
    }

    test_lint_success! {
        name = preg_replace_callback,
        rule = NoExecutableRegexModifierRule,
        code = r#"
            <?php

            $result = preg_replace_callback('/pattern/', function ($m) { return $m[0]; }, $subject);
        "#
    }

    test_lint_failure! {
        name = preg_replace_with_e_modifier,
        rule = NoExecutableRegexModifierRule,
        code = r#"
            <?php

            $result = preg_replace('/pattern/e', 'strtoupper("$1")', $subject);
        "#
    }

    test_lint_failure! {
        name = preg_replace_with_e_and_other_modifiers,
        rule = NoExecutableRegexModifierRule,
        code = r#"
            <?php

            $result = preg_replace('/pattern/ie', 'strtoupper("$1")', $subject);
        "#
    }
}
