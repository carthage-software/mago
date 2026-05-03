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
use mago_text_edit::TextEdit;
use mago_text_edit::TextRange;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct PreferExplodeOverPregSplitRule {
    meta: &'static RuleMeta,
    cfg: PreferExplodeOverPregSplitConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PreferExplodeOverPregSplitConfig {
    pub level: Level,
}

impl Default for PreferExplodeOverPregSplitConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for PreferExplodeOverPregSplitConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PreferExplodeOverPregSplitRule {
    type Config = PreferExplodeOverPregSplitConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prefer Explode Over Preg Split",
            code: "prefer-explode-over-preg-split",
            description: indoc! {r"
                Detects calls to `preg_split()` whose pattern has no regex meta-characters and no
                modifiers, which means the split could be done with `explode()` and no regex engine
                at all.

                `explode()` is faster (no compilation step), easier to read, and expresses the
                intent more directly when the separator is a plain string.

                The rule only fires when:

                - the pattern argument is a string literal,
                - the pattern has no flags after the closing delimiter,
                - the content between the delimiters contains no regex meta-characters,
                - and the `flags` argument (if present) is literal `0`.
            "},
            good_example: indoc! {r#"
                <?php

                $parts = explode(', ', $csv);
            "#},
            bad_example: indoc! {r#"
                <?php

                $parts = preg_split('/, /', $csv);
            "#},
            category: Category::BestPractices,
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

        if !function_call_matches(ctx, function_call, "preg_split") {
            return;
        }

        let arguments = &function_call.argument_list.arguments.nodes;
        if arguments.len() < 2 || arguments.len() > 4 {
            return;
        }

        if arguments.iter().any(|arg| !arg.is_positional()) {
            return;
        }

        let Some(Argument::Positional(pattern_arg)) = arguments.first() else {
            return;
        };

        let Expression::Literal(Literal::String(pattern_literal)) = pattern_arg.value else {
            return;
        };

        let Some(pattern_value) = pattern_literal.value else {
            return;
        };
        let Some(separator) = try_extract_plain_separator(pattern_value) else {
            return;
        };

        if let Some(Argument::Positional(flags_arg)) = arguments.get(3)
            && !is_literal_zero(flags_arg.value)
        {
            return;
        }

        let issue = Issue::new(self.cfg.level(), "`preg_split` with a plain separator can be replaced with `explode`.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(function_call.function.span())
                    .with_message("This `preg_split` call uses a pattern with no regex meta-characters"),
            )
            .with_annotation(
                Annotation::secondary(pattern_arg.value.span()).with_message("...this separator is a plain string"),
            )
            .with_note(
                "`explode()` performs a plain substring split without compiling a regex, so it is faster and easier to read when the separator has no special meaning.",
            )
            .with_help("Replace the `preg_split` call with `explode` and unwrap the pattern delimiters.");

        let rewritten_separator = quote_separator(separator);

        ctx.collector.propose(issue, |edits| {
            let has_namespace = !ctx.scope.get_namespace().is_empty();

            edits.push(TextEdit::replace(
                function_call.function.span(),
                if has_namespace { "\\explode" } else { "explode" },
            ));
            edits.push(TextEdit::replace(pattern_arg.value.span(), rewritten_separator));

            if let Some(Argument::Positional(flags_arg)) = arguments.get(3) {
                let source = ctx.source_file.contents.as_bytes();
                let mut comma_offset = flags_arg.start_offset() as usize;
                let mut found_comma = false;
                while comma_offset > 0 {
                    comma_offset -= 1;
                    match source.get(comma_offset) {
                        Some(&b',') => {
                            found_comma = true;
                            break;
                        }
                        Some(&b) if (b as char).is_whitespace() => {}
                        _ => return,
                    }
                }

                if !found_comma {
                    return;
                }

                edits.push(TextEdit::delete(TextRange::new(comma_offset as u32, flags_arg.end_offset())));
            }
        });
    }
}

fn is_literal_zero(expression: &Expression<'_>) -> bool {
    matches!(
        expression,
        Expression::Literal(Literal::Integer(integer)) if integer.value == Some(0),
    )
}

/// Extracts the plain-text separator from a PHP regex pattern literal when the pattern has no
/// regex meta-characters and no modifiers.
fn try_extract_plain_separator(pattern: &str) -> Option<&str> {
    let mut chars = pattern.chars();
    let delimiter = chars.next()?;

    // Only accept common symmetric delimiters that are not themselves regex meta-characters.
    // Paired delimiters (e.g., `{...}`, `[...]`) and alphanumeric delimiters are rejected to
    // keep the rule conservative.
    if !matches!(delimiter, '/' | '#' | '~' | '!' | '@' | '%' | ',' | ';' | '|') {
        return None;
    }

    let closing = pattern.rfind(delimiter)?;
    if closing == 0 {
        return None;
    }

    // Any trailing modifier (e.g., `i`, `u`, `s`) changes match semantics and cannot be
    // represented by `explode`.
    if closing != pattern.len() - 1 {
        return None;
    }

    let inner = &pattern[1..closing];

    // Characters that carry regex meaning. Any of these in the pattern means we cannot safely
    // convert to a plain `explode` separator.
    const META_CHARS: &[char] =
        &['\\', '.', '+', '*', '?', '[', '^', ']', '$', '(', ')', '{', '}', '=', '!', '<', '>', '|', ':', '-', '#'];

    if inner.chars().any(|c| META_CHARS.contains(&c)) {
        return None;
    }

    Some(inner)
}

/// Wraps `content` in a PHP string literal. Uses single quotes for printable content, and
/// falls back to double quotes with escape sequences when the separator contains ASCII
/// control bytes (newlines, tabs, etc.) so the emitted source doesn't get split across lines.
fn quote_separator(content: &str) -> String {
    if content.bytes().any(is_ascii_control_byte) { quote_as_double(content) } else { quote_as_single(content) }
}

#[inline]
const fn is_ascii_control_byte(byte: u8) -> bool {
    byte < 0x20 || byte == 0x7F
}

/// Wraps `content` in single quotes, escaping any characters that single-quoted PHP strings
/// require escaping (`\` and `'`).
fn quote_as_single(content: &str) -> String {
    let mut out = String::with_capacity(content.len() + 2);
    out.push('\'');
    for c in content.chars() {
        match c {
            '\\' | '\'' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }

    out.push('\'');
    out
}

/// Wraps `content` in double quotes, emitting PHP escape sequences for ASCII control bytes,
/// backslashes, and characters that are special inside double-quoted strings (`"`, `$`).
fn quote_as_double(content: &str) -> String {
    let mut out = String::with_capacity(content.len() + 2);
    out.push('"');
    for c in content.chars() {
        match c {
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x0b' => out.push_str("\\v"),
            '\x0c' => out.push_str("\\f"),
            '\x1b' => out.push_str("\\e"),
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '$' => out.push_str("\\$"),
            ctrl if (ctrl as u32) < 0x80 && is_ascii_control_byte(ctrl as u8) => {
                use std::fmt::Write as _;
                let _ = write!(out, "\\x{:02x}", ctrl as u32);
            }
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::PreferExplodeOverPregSplitRule;
    use crate::test_lint_failure;
    use crate::test_lint_fix;
    use crate::test_lint_success;

    test_lint_success! {
        name = pattern_with_meta_chars_is_not_flagged,
        rule = PreferExplodeOverPregSplitRule,
        code = indoc! {r#"
            <?php

            $parts = preg_split('/\s+/', $s);
        "#}
    }

    test_lint_success! {
        name = pattern_with_modifier_is_not_flagged,
        rule = PreferExplodeOverPregSplitRule,
        code = indoc! {r#"
            <?php

            $parts = preg_split('/, /i', $s);
        "#}
    }

    test_lint_success! {
        name = pattern_with_nonzero_flags_is_not_flagged,
        rule = PreferExplodeOverPregSplitRule,
        code = indoc! {r#"
            <?php

            $parts = preg_split('/, /', $s, -1, PREG_SPLIT_NO_EMPTY);
        "#}
    }

    test_lint_success! {
        name = non_literal_pattern_is_not_flagged,
        rule = PreferExplodeOverPregSplitRule,
        code = indoc! {r#"
            <?php

            $parts = preg_split($pattern, $s);
        "#}
    }

    test_lint_success! {
        name = pattern_with_dash_is_not_flagged,
        rule = PreferExplodeOverPregSplitRule,
        code = indoc! {r#"
            <?php

            $parts = preg_split('/a-z/', $s);
        "#}
    }

    test_lint_failure! {
        name = simple_comma_separator_is_flagged,
        rule = PreferExplodeOverPregSplitRule,
        code = indoc! {r#"
            <?php

            $parts = preg_split('/, /', $s);
        "#}
    }

    test_lint_failure! {
        name = hash_delimited_separator_is_flagged,
        rule = PreferExplodeOverPregSplitRule,
        code = indoc! {r#"
            <?php

            $parts = preg_split('#abc#', $s);
        "#}
    }

    test_lint_failure! {
        name = zero_flags_argument_is_flagged,
        rule = PreferExplodeOverPregSplitRule,
        code = indoc! {r#"
            <?php

            $parts = preg_split('/, /', $s, -1, 0);
        "#}
    }

    test_lint_fix! {
        name = fix_simple_comma_separator,
        rule = PreferExplodeOverPregSplitRule,
        code = indoc! {r#"
            <?php

            $parts = preg_split('/, /', $s);
        "#},
        fixed = indoc! {r#"
            <?php

            $parts = explode(', ', $s);
        "#}
    }

    test_lint_fix! {
        name = fix_with_limit,
        rule = PreferExplodeOverPregSplitRule,
        code = indoc! {r#"
            <?php

            $parts = preg_split('/, /', $s, 3);
        "#},
        fixed = indoc! {r#"
            <?php

            $parts = explode(', ', $s, 3);
        "#}
    }

    test_lint_fix! {
        name = fix_drops_zero_flags,
        rule = PreferExplodeOverPregSplitRule,
        code = indoc! {r#"
            <?php

            $parts = preg_split('/, /', $s, -1, 0);
        "#},
        fixed = indoc! {r#"
            <?php

            $parts = explode(', ', $s, -1);
        "#}
    }

    test_lint_fix! {
        name = fix_hash_delimited_pattern,
        rule = PreferExplodeOverPregSplitRule,
        code = indoc! {r#"
            <?php

            $parts = preg_split('#abc#', $s);
        "#},
        fixed = indoc! {r#"
            <?php

            $parts = explode('abc', $s);
        "#}
    }

    test_lint_fix! {
        name = fix_hash_delimited_pattern_in_namespace,
        rule = PreferExplodeOverPregSplitRule,
        code = indoc! {r#"
            <?php

            namespace App;

            $parts = preg_split('#abc#', $s);
        "#},
        fixed = indoc! {r#"
            <?php

            namespace App;

            $parts = \explode('abc', $s);
        "#}
    }

    test_lint_fix! {
        name = fix_separator_with_newline_escapes_to_double_quoted,
        rule = PreferExplodeOverPregSplitRule,
        code = indoc! {r#"
            <?php

            $parts = preg_split("/\n\n/", $s);
        "#},
        fixed = indoc! {r#"
            <?php

            $parts = explode("\n\n", $s);
        "#}
    }

    test_lint_fix! {
        name = fix_separator_with_tab_escapes_to_double_quoted,
        rule = PreferExplodeOverPregSplitRule,
        code = indoc! {r#"
            <?php

            $parts = preg_split("/\t/", $s);
        "#},
        fixed = indoc! {r#"
            <?php

            $parts = explode("\t", $s);
        "#}
    }

    test_lint_fix! {
        name = fix_separator_with_crlf_escapes_to_double_quoted,
        rule = PreferExplodeOverPregSplitRule,
        code = indoc! {r#"
            <?php

            $parts = preg_split("/\r\n/", $s);
        "#},
        fixed = indoc! {r#"
            <?php

            $parts = explode("\r\n", $s);
        "#}
    }

    test_lint_fix! {
        name = fix_separator_with_arbitrary_control_byte_uses_hex_escape,
        rule = PreferExplodeOverPregSplitRule,
        code = indoc! {r#"
            <?php

            $parts = preg_split("/\x01/", $s);
        "#},
        fixed = indoc! {r#"
            <?php

            $parts = explode("\x01", $s);
        "#}
    }
}
