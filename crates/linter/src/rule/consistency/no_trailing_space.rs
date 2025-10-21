use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::*;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoTrailingSpaceRule {
    meta: &'static RuleMeta,
    cfg: NoTrailingSpaceConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoTrailingSpaceConfig {
    pub level: Level,
}

impl Default for NoTrailingSpaceConfig {
    fn default() -> Self {
        Self { level: Level::Note }
    }
}

impl Config for NoTrailingSpaceConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoTrailingSpaceRule {
    type Config = NoTrailingSpaceConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Trailing Space",
            code: "no-trailing-space",
            description: indoc! {"
                Detects trailing whitespace at the end of comments. Trailing whitespace can cause unnecessary
                diffs and formatting issues, so it is recommended to remove it.
            "},
            good_example: indoc! {r#"
                <?php

                // This is a good comment.
            "#},
            bad_example: indoc! {r#"
                <?php

                // This is a comment with trailing whitespace.  
            "#},
            category: Category::Consistency,
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

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Program(program) = node else {
            return;
        };

        for trivia in program.trivia.iter() {
            if !trivia.kind.is_comment() {
                continue;
            }

            let comment_span = trivia.span();
            let lines = trivia.value.lines().collect::<Vec<_>>();

            let mut offset = 0;
            for line in lines.iter() {
                let trimmed = line.trim_end();
                let trimmed_length = trimmed.len();
                let trailing_whitespace_length = line.len() - trimmed_length;
                if trailing_whitespace_length > 0 {
                    let whitespace_start = offset + trimmed_length;

                    let whitespace_span = Span::new(
                        comment_span.file_id,
                        comment_span.start.forward(whitespace_start as u32),
                        comment_span.start.forward(whitespace_start as u32 + trailing_whitespace_length as u32),
                    );

                    let issue = Issue::new(self.cfg.level(), "Trailing whitespace detected in comment.")
                        .with_code(self.meta.code)
                        .with_annotation(
                            Annotation::primary(whitespace_span).with_message("Trailing whitespace detected"),
                        )
                        .with_annotation(
                            Annotation::secondary(comment_span).with_message("Comment with trailing whitespace"),
                        )
                        .with_note("Trailing whitespaces can cause unnecessary diffs and formatting issues.")
                        .with_help("Remove the extra whitespace.");

                    ctx.collector.report(issue);
                }

                offset += line.len() + 1;
            }
        }
    }
}
