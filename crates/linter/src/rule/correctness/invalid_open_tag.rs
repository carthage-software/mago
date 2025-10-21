use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_fixer::SafetyClassification;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::*;

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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
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
            good_example: indoc! {r#"
                <?php

                echo 'Hello, world!';
            "#},
            bad_example: indoc! {r#"
                <php?

                echo 'Hello, world!';
            "#},
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

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Inline(inline_stmt) = node else {
            return;
        };

        let content = inline_stmt.value;
        let trimmed_content = content.trim_start();

        for &invalid_tag in INVALID_TAGS {
            let mut content_chars = trimmed_content.chars();
            let mut matches = true;
            let mut prefix_byte_len = 0;

            for tag_char in invalid_tag.chars() {
                if let Some(content_char) = content_chars.next() {
                    if !tag_char.eq_ignore_ascii_case(&content_char) {
                        matches = false;
                        break;
                    }

                    prefix_byte_len += content_char.len_utf8();
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
                    format!("Misspelled PHP opening tag `{}`.", &trimmed_content[..prefix_byte_len]),
                )
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(invalid_tag_span).with_message("This looks like a typo for `<?php`."),
                )
                .with_note("Code following a misspelled tag will be treated as plain text and output directly.")
                .with_help("Replace this with the correct `<?php` opening tag.");

                ctx.collector.propose(issue, |plan| {
                    plan.replace(invalid_tag_span.to_range(), "<?php", SafetyClassification::PotentiallyUnsafe);
                });

                break;
            }
        }
    }
}
