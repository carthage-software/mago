use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_text_edit::Safety;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

const INVALID_TAGS: &[&str] = &["<php?", "<ph?p", "<p?hp", "<ph?", "<p?"];

#[derive(Debug, Clone)]
pub struct InvalidOpenTagRule {
    meta: &'static RuleMeta,
    cfg: InvalidOpenTagConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct InvalidOpenTagConfig {
    pub level: Level,
}

impl Default for InvalidOpenTagConfig {
    fn default() -> Self {
        Self { level: Level::Note }
    }
}

impl Config for InvalidOpenTagConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for InvalidOpenTagRule {
    type Config = InvalidOpenTagConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Invalid Open Tag",
            code: "invalid-open-tag",
            description: indoc! {"
                Detects misspelled PHP opening tags like `<php?` instead of `<?php`.

                A misspelled opening tag will cause the PHP interpreter to treat the
                following code as plain text, leading to the code being output directly
                to the browser instead of being executed. This can cause unexpected
                behavior and potential security vulnerabilities.
            "},
            good_example: indoc! {r"
                <?php

                echo 'Hello, world!';
            "},
            bad_example: indoc! {r"
                <php?

                echo 'Hello, world!';
            "},
            category: Category::Correctness,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Inline];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Inline(inline_stmt) = node else {
            return;
        };

        let content = inline_stmt.value;
        let trim_start_idx = content.iter().position(|&b| !b.is_ascii_whitespace()).unwrap_or(content.len());
        let trimmed_content = &content[trim_start_idx..];

        for &invalid_tag in INVALID_TAGS {
            let tag_bytes = invalid_tag.as_bytes();
            let mut matches = true;
            let mut prefix_byte_len = 0;

            for (i, &tag_byte) in tag_bytes.iter().enumerate() {
                if let Some(&content_byte) = trimmed_content.get(i) {
                    if !tag_byte.eq_ignore_ascii_case(&content_byte) {
                        matches = false;
                        break;
                    }

                    prefix_byte_len += 1;
                } else {
                    matches = false;
                    break;
                }
            }

            if matches {
                let start_offset = content.len() - trimmed_content.len();
                let invalid_tag_span = inline_stmt.span().subspan(start_offset as u32, prefix_byte_len as u32);

                let issue = Issue::new(
                    self.cfg.level(),
                    format!(
                        "Misspelled PHP opening tag `{}`.",
                        mago_bytes::BytesDisplay(&trimmed_content[..prefix_byte_len])
                    ),
                )
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(invalid_tag_span).with_message("This looks like a typo for `<?php`."),
                )
                .with_note("Code following a misspelled tag will be treated as plain text and output directly.")
                .with_help("Replace this with the correct `<?php` opening tag.");

                ctx.collector.propose(issue, |edits| {
                    edits.push(TextEdit::replace(invalid_tag_span, "<?php").with_safety(Safety::PotentiallyUnsafe));
                });

                break;
            }
        }
    }
}
