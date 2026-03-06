use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::Position;
use mago_span::Span;
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
pub struct LineLengthLimitRule {
    meta: &'static RuleMeta,
    cfg: LineLengthLimitConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct LineLengthLimitConfig {
    pub level: Level,
    pub max_length: usize,
}

impl Default for LineLengthLimitConfig {
    fn default() -> Self {
        Self { level: Level::Warning, max_length: 120 }
    }
}

impl Config for LineLengthLimitConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for LineLengthLimitRule {
    type Config = LineLengthLimitConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Line Length Limit",
            code: "line-length-limit",
            description: indoc! {"
                Reports lines that exceed a configurable maximum character length. Long lines reduce
                readability and make side-by-side diffs harder to review. The default limit is 120 characters.
                Lines consisting entirely of a string literal or comment are excluded.
            "},
            good_example: indoc! {r#"
                <?php

                $shortVariable = 'hello';
            "#},
            bad_example: indoc! {r#"
                <?php

                $veryLongVariableNameHere = someFunction($parameterOne, $parameterTwo, $parameterThree, $parameterFour, $parameterFive, $parameterSix);
            "#},
            category: Category::Maintainability,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Program];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::Program(program) = node else {
            return;
        };

        let source = ctx.source_file.contents.as_ref();
        let file_id = program.file_id;
        let max_length = self.cfg.max_length;

        let mut offset: u32 = 0;
        for (line_num, line) in source.split('\n').enumerate() {
            // Strip trailing \r for Windows line endings
            let line_content = line.strip_suffix('\r').unwrap_or(line);
            let line_len = line_content.len();

            if line_len > max_length {
                let trimmed = line_content.trim();

                // Skip lines that are entirely a comment
                let is_comment_only = trimmed.starts_with("//")
                    || trimmed.starts_with('#')
                    || trimmed.starts_with('*')
                    || trimmed.starts_with("/*");

                // Skip lines that are entirely a string (quoted)
                let is_string_only = (trimmed.starts_with('\'') && trimmed.ends_with('\''))
                    || (trimmed.starts_with('"') && trimmed.ends_with('"'));

                if !is_comment_only && !is_string_only {
                    let line_start = Position::new(offset);
                    let line_end = Position::new(offset + line_len as u32);
                    let line_span = Span::new(file_id, line_start, line_end);

                    // Highlight the portion that exceeds the limit
                    let over_start = Position::new(offset + max_length as u32);
                    let over_span = Span::new(file_id, over_start, line_end);

                    let issue = Issue::new(
                        self.cfg.level(),
                        format!("Line {} exceeds {} characters ({} characters)", line_num + 1, max_length, line_len),
                    )
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(over_span)
                            .with_message(format!("{} characters over limit", line_len - max_length)),
                    )
                    .with_annotation(Annotation::secondary(line_span).with_message("This line is too long"))
                    .with_help("Consider breaking this line into multiple lines");

                    ctx.collector.report(issue);
                }
            }

            // +1 for the \n character
            offset += line.len() as u32 + 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::LineLengthLimitRule;
    use crate::settings::Settings;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = short_lines_are_ok,
        rule = LineLengthLimitRule,
        settings = |settings: &mut Settings| {
            settings.rules.line_length_limit.config.max_length = 120;
        },
        code = indoc! {r"
            <?php

            $x = 1;
            $y = 'hello world';
        "}
    }

    test_lint_failure! {
        name = long_code_line_is_bad,
        rule = LineLengthLimitRule,
        settings = |settings: &mut Settings| {
            settings.rules.line_length_limit.config.max_length = 40;
        },
        code = indoc! {r"
            <?php

            $veryLongVariable = someFunction($parameterOne, $parameterTwo);
        "}
    }

    test_lint_success! {
        name = long_comment_line_is_skipped,
        rule = LineLengthLimitRule,
        settings = |settings: &mut Settings| {
            settings.rules.line_length_limit.config.max_length = 40;
        },
        code = indoc! {r"
            <?php

            // This is a very long comment that exceeds the forty character limit but should be skipped
        "}
    }

    test_lint_success! {
        name = long_string_line_is_skipped,
        rule = LineLengthLimitRule,
        settings = |settings: &mut Settings| {
            settings.rules.line_length_limit.config.max_length = 40;
        },
        code = indoc! {r#"
            <?php

            $x = 1;
        "#}
    }

    test_lint_failure! {
        name = multiple_long_lines_reported,
        rule = LineLengthLimitRule,
        count = 2,
        settings = |settings: &mut Settings| {
            settings.rules.line_length_limit.config.max_length = 30;
        },
        code = indoc! {r"
            <?php

            $first = someFunction($parameterOne, $parameterTwo);
            $x = 1;
            $second = anotherFunction($parameterThree, $parameterFour);
        "}
    }

    test_lint_success! {
        name = line_at_exact_limit_is_ok,
        rule = LineLengthLimitRule,
        settings = |settings: &mut Settings| {
            // "<?php" is 5 chars, exactly at limit
            settings.rules.line_length_limit.config.max_length = 5;
        },
        code = "<?php\n"
    }
}
