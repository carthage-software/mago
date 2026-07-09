use std::sync::LazyLock;

use foldhash::HashMap;
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
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches_any;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoMissingFormatArgumentRule {
    meta: &'static RuleMeta,
    cfg: NoMissingFormatArgumentConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct NoMissingFormatArgumentConfig {
    pub level: Level,
}

impl Default for NoMissingFormatArgumentConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for NoMissingFormatArgumentConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoMissingFormatArgumentRule {
    type Config = NoMissingFormatArgumentConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Missing Format Argument",
            code: "no-missing-format-argument",
            description: indoc! {"
                Detects `sprintf`-style format calls where the number of placeholders
                in the format string exceeds the number of provided values.

                When the format string is a literal, this rule parses the placeholders
                (regular `%s`/`%d` and positional `%1$s`) and compares the
                required argument count against the actual values given.

                This catches runtime `ArgumentCountError`-class errors at lint time.
            "},
            good_example: indoc! {r#"
                <?php

                $result = sprintf('Hello %s, you have %d messages', $name, $count);
            "#},
            bad_example: indoc! {r#"
                <?php

                // Missing second value; will trigger a runtime error.
                $result = sprintf('Hello %s, you have %d messages', $name);
            "#},
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

        let function_names: Vec<&str> = FORMAT_FUNCTION_MODELS.keys().copied().collect();
        let Some(function_name) = function_call_matches_any(ctx, function_call, &function_names) else {
            return;
        };

        let info = &FORMAT_FUNCTION_MODELS[function_name];

        let arguments = &function_call.argument_list.arguments.nodes;
        let total_args = arguments.len();

        let Some(format_arg) = arguments.get(info.format_index) else {
            return;
        };
        let Argument::Positional(format_arg) = format_arg else {
            return;
        };

        if format_arg.ellipsis.is_some() {
            return;
        }

        let Some(format_str) = extract_string_value(format_arg.value) else {
            return;
        };

        let parsed = parse_format_string(format_str);

        match info.values_model {
            ValuesModel::Variadic => {
                let values_start = info.format_index + 1;
                let values_count = total_args.saturating_sub(values_start);

                if values_count >= parsed.required_args {
                    return;
                }

                ctx.collector.report(
                    Issue::new(
                        self.cfg.level,
                        format!(
                            "Call to `{}` contains {} {}, {} value{} given.",
                            function_name,
                            parsed.placeholder_count,
                            if parsed.placeholder_count == 1 { "placeholder" } else { "placeholders" },
                            parsed.required_args,
                            if parsed.required_args == 1 { "" } else { "s" },
                        ),
                    )
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(function_call.argument_list.span())
                            .with_message("Not enough arguments for the format string"),
                    )
                    .with_annotation(
                        Annotation::secondary(format_arg.value.span())
                            .with_message("This format string requires more values"),
                    )
                    .with_note(
                        "A mismatch between format placeholders and provided values will cause a runtime error (ArgumentCountError).",
                    )
                    .with_help(
                        format!(
                            "Add the missing value{} to the call, or remove the corresponding %s placeholders from the format string.",
                            if parsed.required_args - values_count == 1 { "" } else { "s" },
                        ),
                    ),
                );
            }
            ValuesModel::Array => {
                if total_args <= info.format_index + 1 {
                    ctx.collector.report(
                        Issue::new(
                            self.cfg.level,
                            format!(
                                "Call to `{}` expects a format string and a values array, but only {} argument{} given.",
                                function_name,
                                total_args,
                                if total_args == 1 { "" } else { "s" },
                            ),
                        )
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(function_call.argument_list.span())
                                .with_message(
                                    format!(
                                        "A values array is required as argument {}",
                                        info.format_index + 2,
                                    ),
                                ),
                        )
                        .with_note(
                            "Functions that accept a values array still require the array argument to be present.",
                        )
                        .with_help("Provide an array of values as the second argument."),
                    );
                }
            }
        }
    }
}

struct FormatInfo {
    placeholder_count: usize,
    required_args: usize,
}

struct FormatFunctionModel {
    format_index: usize,
    values_model: ValuesModel,
}

enum ValuesModel {
    Variadic,
    Array,
}

fn extract_string_value<'arena>(expression: &'arena Expression<'arena>) -> Option<&'arena [u8]> {
    match expression {
        Expression::Literal(Literal::String(literal)) => literal.value,
        _ => None,
    }
}

fn parse_format_string(format_str: &[u8]) -> FormatInfo {
    let bytes = format_str;
    let len = bytes.len();
    let mut i = 0;
    let mut placeholder_count: usize = 0;
    let mut max_position: usize = 0;
    let mut has_positional = false;
    let mut seq_count: usize = 0;

    while i < len {
        if bytes[i] != b'%' {
            i += 1;
            continue;
        }

        i += 1;

        if i >= len {
            break;
        }

        if bytes[i] == b'%' {
            i += 1;
            continue;
        }

        placeholder_count += 1;

        let pos = parse_position(bytes, &mut i, len);
        if let Some(p) = pos {
            has_positional = true;
            if p > max_position {
                max_position = p;
            }
        } else {
            seq_count += 1;
        }

        skip_format_spec(bytes, &mut i, len);
    }

    let required_args = if has_positional { max_position.max(seq_count) } else { seq_count };

    FormatInfo { placeholder_count, required_args }
}

fn parse_position(bytes: &[u8], i: &mut usize, len: usize) -> Option<usize> {
    if *i >= len || !bytes[*i].is_ascii_digit() {
        return None;
    }

    let start = *i;
    let mut n: usize = 0;
    while *i < len && bytes[*i].is_ascii_digit() {
        n = n * 10 + (bytes[*i] - b'0') as usize;
        *i += 1;
    }

    if *i < len && bytes[*i] == b'$' {
        *i += 1;
        return Some(n);
    }

    *i = start;

    None
}

fn skip_format_spec(bytes: &[u8], i: &mut usize, len: usize) {
    loop {
        if *i >= len {
            return;
        }

        match bytes[*i] {
            b'-' | b'+' | b' ' | b'0' => {
                *i += 1;
            }
            b'\'' => {
                *i += 2;
                if *i > len {
                    return;
                }
            }
            _ => break,
        }
    }

    parse_number(bytes, i, len);

    if *i < len && bytes[*i] == b'.' {
        *i += 1;
        parse_number(bytes, i, len);
    }

    if *i < len {
        let c = bytes[*i];
        match c {
            b's' | b'd' | b'u' | b'f' | b'F' | b'e' | b'E' | b'x' | b'X' | b'o' | b'b' | b'c' => {
                *i += 1;
            }
            _ => {}
        }
    }
}

fn parse_number(bytes: &[u8], i: &mut usize, len: usize) -> usize {
    let mut n: usize = 0;
    while *i < len && bytes[*i].is_ascii_digit() {
        n = n * 10 + (bytes[*i] - b'0') as usize;
        *i += 1;
    }

    n
}

static FORMAT_FUNCTION_MODELS: LazyLock<HashMap<&'static str, FormatFunctionModel>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("sprintf", FormatFunctionModel { format_index: 0, values_model: ValuesModel::Variadic }),
        ("printf", FormatFunctionModel { format_index: 0, values_model: ValuesModel::Variadic }),
        ("fprintf", FormatFunctionModel { format_index: 1, values_model: ValuesModel::Variadic }),
        ("vsprintf", FormatFunctionModel { format_index: 0, values_model: ValuesModel::Array }),
        ("vprintf", FormatFunctionModel { format_index: 0, values_model: ValuesModel::Array }),
        ("vfprintf", FormatFunctionModel { format_index: 1, values_model: ValuesModel::Array }),
        ("Psl\\Str\\format", FormatFunctionModel { format_index: 0, values_model: ValuesModel::Variadic }),
        ("Psl\\IO\\write", FormatFunctionModel { format_index: 0, values_model: ValuesModel::Variadic }),
        ("Psl\\IO\\write_line", FormatFunctionModel { format_index: 0, values_model: ValuesModel::Variadic }),
        ("Psl\\IO\\write_error", FormatFunctionModel { format_index: 0, values_model: ValuesModel::Variadic }),
        ("Psl\\IO\\write_error_line", FormatFunctionModel { format_index: 0, values_model: ValuesModel::Variadic }),
    ])
});

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;

    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = sprintf_all_args_provided,
        rule = NoMissingFormatArgumentRule,
        code = indoc! {r"
            <?php

            $result = sprintf('Hello %s, you have %d messages', $name, $count);
        "},
    }

    test_lint_success! {
        name = sprintf_no_placeholders,
        rule = NoMissingFormatArgumentRule,
        code = indoc! {r"
            <?php

            $result = sprintf('Hello world');
        "},
    }

    test_lint_success! {
        name = sprintf_escaped_percent,
        rule = NoMissingFormatArgumentRule,
        code = indoc! {r"
            <?php

            $result = sprintf('100%% complete');
        "},
    }

    test_lint_success! {
        name = sprintf_positional_with_enough_args,
        rule = NoMissingFormatArgumentRule,
        code = indoc! {r"
            <?php

            $result = sprintf('Hello %2$s, you have %1$d messages', $count, $name);
        "},
    }

    test_lint_success! {
        name = sprintf_positional_reused,
        rule = NoMissingFormatArgumentRule,
        code = indoc! {r"
            <?php

            $result = sprintf('Hello %1$s, again %1$s', $name);
        "},
    }

    test_lint_success! {
        name = sprintf_non_literal_format,
        rule = NoMissingFormatArgumentRule,
        code = indoc! {r"
            <?php

            $result = sprintf($format, $value);
        "},
    }

    test_lint_success! {
        name = printf_enough_args,
        rule = NoMissingFormatArgumentRule,
        code = indoc! {r"
            <?php

            printf('Hello %s', $name);
        "},
    }

    test_lint_success! {
        name = vsprintf_has_array,
        rule = NoMissingFormatArgumentRule,
        code = indoc! {r"
            <?php

            $result = vsprintf('Hello %s', $values);
        "},
    }

    test_lint_failure! {
        name = sprintf_missing_argument,
        rule = NoMissingFormatArgumentRule,
        count = 1,
        code = indoc! {r"
            <?php

            $result = sprintf('Hello %s, you have %d messages', $name);
        "},
    }

    test_lint_failure! {
        name = printf_no_values,
        rule = NoMissingFormatArgumentRule,
        count = 1,
        code = indoc! {r"
            <?php

            printf('Hello %s');
        "},
    }

    test_lint_failure! {
        name = sprintf_positional_missing_higher,
        rule = NoMissingFormatArgumentRule,
        count = 1,
        code = indoc! {r"
            <?php

            $result = sprintf('Hello %2$s, you have %1$d messages', $count);
        "},
    }

    test_lint_success! {
        name = fprintf_enough_args,
        rule = NoMissingFormatArgumentRule,
        code = indoc! {r"
            <?php

            fprintf($handle, 'Hello %s', $name);
        "},
    }

    test_lint_failure! {
        name = fprintf_missing_format_arg,
        rule = NoMissingFormatArgumentRule,
        count = 1,
        code = indoc! {r"
            <?php

            fprintf($handle, 'Hello %s %s', $name);
        "},
    }

    test_lint_success! {
        name = sprintf_with_extra_args_is_ok,
        rule = NoMissingFormatArgumentRule,
        code = indoc! {r"
            <?php

            $result = sprintf('Hello %s', $name, $extra);
        "},
    }

    test_lint_failure! {
        name = vsprintf_missing_array,
        rule = NoMissingFormatArgumentRule,
        count = 1,
        code = indoc! {r"
            <?php

            $result = vsprintf('Hello %s');
        "},
    }

    #[test]
    fn parse_empty_string() {
        let info = parse_format_string(b"");

        assert_eq!(info.placeholder_count, 0);
        assert_eq!(info.required_args, 0);
    }

    #[test]
    fn parse_no_placeholders() {
        let info = parse_format_string(b"Hello world");

        assert_eq!(info.placeholder_count, 0);
        assert_eq!(info.required_args, 0);
    }

    #[test]
    fn parse_all_escaped_percents() {
        let info = parse_format_string(b"100%%%% complete%%");

        assert_eq!(info.placeholder_count, 0);
        assert_eq!(info.required_args, 0);
    }

    #[test]
    fn parse_trailing_percent() {
        let info = parse_format_string(b"Hello %");

        assert_eq!(info.placeholder_count, 0);
        assert_eq!(info.required_args, 0);
    }

    #[test]
    fn parse_trailing_percent_after_valid() {
        let info = parse_format_string(b"Hello %s %");

        assert_eq!(info.placeholder_count, 1);
        assert_eq!(info.required_args, 1);
    }

    #[test]
    fn parse_sequential_placeholders() {
        let info = parse_format_string(b"Hello %s, you have %d messages");

        assert_eq!(info.placeholder_count, 2);
        assert_eq!(info.required_args, 2);
    }

    #[test]
    fn parse_positional_placeholders() {
        let info = parse_format_string(b"Hello %2$s, you have %1$d messages");

        assert_eq!(info.placeholder_count, 2);
        assert_eq!(info.required_args, 2);
    }

    #[test]
    fn parse_positional_reuse() {
        let info = parse_format_string(b"%1$s %1$s %1$s");

        assert_eq!(info.placeholder_count, 3);
        assert_eq!(info.required_args, 1);
    }

    #[test]
    fn parse_positional_nonsequential() {
        let info = parse_format_string(b"%5$d %1$s");

        assert_eq!(info.placeholder_count, 2);
        assert_eq!(info.required_args, 5);
    }

    #[test]
    fn parse_positional_nonsequential_checked() {
        let info = parse_format_string(b"%1$s %5$d");

        assert_eq!(info.placeholder_count, 2);
        assert_eq!(info.required_args, 5);
    }

    #[test]
    fn parse_all_specifiers() {
        let info = parse_format_string(b"%s %d %u %f %F %e %E %x %X %o %b %c");

        assert_eq!(info.placeholder_count, 12);
        assert_eq!(info.required_args, 12);
    }

    #[test]
    fn parse_flags_and_width() {
        let info = parse_format_string(b"%-10s %+d %0d %'x5s");

        assert_eq!(info.placeholder_count, 4);
        assert_eq!(info.required_args, 4);
    }

    #[test]
    fn parse_precision() {
        let info = parse_format_string(b"%.2f %5.2f %.0d");

        assert_eq!(info.placeholder_count, 3);
        assert_eq!(info.required_args, 3);
    }

    #[test]
    fn parse_positional_with_width_and_precision() {
        let info = parse_format_string(b"%1$10s %2$05d %1$.2f");

        assert_eq!(info.placeholder_count, 3);
        assert_eq!(info.required_args, 2);
    }

    #[test]
    fn parse_ignored_padding_character_specifier() {
        let info = parse_format_string(b"%'*10s %' 05d");

        assert_eq!(info.placeholder_count, 2);
        assert_eq!(info.required_args, 2);
    }

    #[test]
    fn parse_format_flag_minus() {
        let info = parse_format_string(b"%-s %-10d");

        assert_eq!(info.placeholder_count, 2);
        assert_eq!(info.required_args, 2);
    }

    #[test]
    fn parse_format_flag_plus() {
        let info = parse_format_string(b"%+d %+f");

        assert_eq!(info.placeholder_count, 2);
        assert_eq!(info.required_args, 2);
    }

    #[test]
    fn parse_format_flag_space() {
        let info = parse_format_string(b"% d % f");

        assert_eq!(info.placeholder_count, 2);
        assert_eq!(info.required_args, 2);
    }

    #[test]
    fn parse_adjacent_percents() {
        let info = parse_format_string(b"%%%%");

        assert_eq!(info.placeholder_count, 0);
        assert_eq!(info.required_args, 0);
    }
}
