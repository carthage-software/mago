use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_text_edit::TextEdit;
use mago_text_edit::TextRange;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoRedundantBinaryStringPrefixRule {
    meta: &'static RuleMeta,
    cfg: NoRedundantBinaryStringPrefixConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoRedundantBinaryStringPrefixConfig {
    pub level: Level,
}

impl Default for NoRedundantBinaryStringPrefixConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoRedundantBinaryStringPrefixConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRedundantBinaryStringPrefixRule {
    type Config = NoRedundantBinaryStringPrefixConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Redundant Binary String Prefix",
            code: "no-redundant-binary-string-prefix",
            description: indoc! {"
                Detects the redundant `b`/`B` prefix on string literals. The binary string prefix
                has no effect in PHP and can be safely removed.
            "},
            good_example: indoc! {r#"
                <?php

                $foo = 'hello';
                $bar = "world";
            "#},
            bad_example: indoc! {r#"
                <?php

                $foo = b'hello';
                $bar = b"world";
            "#},
            category: Category::Redundancy,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::LiteralString, NodeKind::InterpolatedString, NodeKind::DocumentString];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let prefix_span = match node {
            Node::LiteralString(literal) if (literal.raw.starts_with('b') || literal.raw.starts_with('B')) => {
                let span = literal.span();
                Span { start: span.start, end: span.start.forward(1), ..span }
            }
            Node::InterpolatedString(interpolated) => {
                if let Some(prefix) = &interpolated.prefix {
                    prefix.span
                } else {
                    return;
                }
            }
            Node::DocumentString(document) => {
                if let Some(prefix) = &document.prefix {
                    prefix.span
                } else {
                    return;
                }
            }
            _ => return,
        };

        let issue = Issue::new(self.cfg.level(), "Redundant binary string prefix.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(prefix_span).with_message("This `b` prefix has no effect and can be removed"),
            )
            .with_note("The `b` prefix on strings is a no-op in PHP and has no effect on the string value.")
            .with_help("Remove the `b` prefix.");

        ctx.collector.propose(issue, |edits| {
            edits.push(TextEdit::delete(TextRange::from(prefix_span)));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;

    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = plain_single_quoted_string,
        rule = NoRedundantBinaryStringPrefixRule,
        code = indoc! {r#"
            <?php

            $foo = 'hello';
        "#}
    }

    test_lint_success! {
        name = plain_double_quoted_string,
        rule = NoRedundantBinaryStringPrefixRule,
        code = indoc! {r#"
            <?php

            $foo = "hello";
        "#}
    }

    test_lint_success! {
        name = plain_interpolated_string,
        rule = NoRedundantBinaryStringPrefixRule,
        code = indoc! {r#"
            <?php

            $foo = "hello $name";
        "#}
    }

    test_lint_success! {
        name = plain_heredoc,
        rule = NoRedundantBinaryStringPrefixRule,
        code = "<?php\n\n$foo = <<<EOT\nhello\nEOT;\n"
    }

    test_lint_success! {
        name = plain_nowdoc,
        rule = NoRedundantBinaryStringPrefixRule,
        code = "<?php\n\n$foo = <<<'EOT'\nhello\nEOT;\n"
    }

    test_lint_failure! {
        name = b_prefix_single_quoted,
        rule = NoRedundantBinaryStringPrefixRule,
        count = 1,
        code = indoc! {r#"
            <?php

            $foo = b'hello';
        "#}
    }

    test_lint_failure! {
        name = upper_b_prefix_single_quoted,
        rule = NoRedundantBinaryStringPrefixRule,
        count = 1,
        code = indoc! {r#"
            <?php

            $foo = B'hello';
        "#}
    }

    test_lint_failure! {
        name = b_prefix_double_quoted,
        rule = NoRedundantBinaryStringPrefixRule,
        count = 1,
        code = indoc! {r#"
            <?php

            $foo = b"hello";
        "#}
    }

    test_lint_failure! {
        name = b_prefix_interpolated,
        rule = NoRedundantBinaryStringPrefixRule,
        count = 1,
        code = indoc! {r#"
            <?php

            $foo = b"hello $name";
        "#}
    }

    test_lint_failure! {
        name = b_prefix_heredoc,
        rule = NoRedundantBinaryStringPrefixRule,
        count = 1,
        code = "<?php\n\n$foo = b<<<EOT\nhello\nEOT;\n"
    }

    test_lint_failure! {
        name = b_prefix_nowdoc,
        rule = NoRedundantBinaryStringPrefixRule,
        count = 1,
        code = "<?php\n\n$foo = b<<<'EOT'\nhello\nEOT;\n"
    }

    test_lint_failure! {
        name = multiple_b_prefixes,
        rule = NoRedundantBinaryStringPrefixRule,
        count = 3,
        code = indoc! {r#"
            <?php

            $a = b'hello';
            $b = B"world";
            $c = b"foo $bar";
        "#}
    }
}
