use indoc::indoc;
use mago_span::HasSpan;
use mago_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_fixer::SafetyClassification;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_syntax::ast::*;
use mago_syntax::comments::comment_lines;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoEmptyCommentRule {
    meta: &'static RuleMeta,
    cfg: NoEmptyCommentConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoEmptyCommentConfig {
    pub level: Level,
    #[serde(alias = "preserve-single-line-comments")]
    pub preserve_single_line_comments: bool,
}

impl Default for NoEmptyCommentConfig {
    fn default() -> Self {
        Self { level: Level::Note, preserve_single_line_comments: false }
    }
}

impl Config for NoEmptyCommentConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoEmptyCommentRule {
    type Config = NoEmptyCommentConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Empty Comment",
            code: "no-empty-comment",
            description: indoc! {"
                Detects empty comments in the codebase. Empty comments are not useful and should be removed
                to keep the codebase clean and maintainable.
            "},
            good_example: indoc! {r"
                <?php

                // This is a useful comment.
                //
                // And so is this whole single line comment block, including the enclosed empty line.
                # This is also a useful comment.
                /**
                 * This is a docblock.
                 */
            "},
            bad_example: indoc! {r"
                <?php

                //
                #
                /**/
            "},
            category: Category::Redundancy,

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

        // Keep track of ongoing blocks of single line comments and only complain about empty
        // single line comments if they are at the start or end of such a block
        let mut current_block = None;
        let mut pending = Vec::new();
        let mut submit = |pending: &mut Vec<Span>| {
            for span in pending.drain(..) {
                let issue = Issue::new(self.cfg.level(), "Empty comments are not allowed.")
                    .with_code(self.meta.code)
                    .with_annotation(Annotation::primary(span).with_message("This is an empty comment"))
                    .with_help("Consider removing this comment.");

                ctx.collector.propose(issue, |plan| {
                    plan.delete(span.to_range(), SafetyClassification::Safe);
                });
            }
        };

        for trivia in program.trivia.iter() {
            // Check if we're still in the same block of single line comments
            if let Some((kind, end)) = &mut current_block {
                if (trivia.kind == *kind || trivia.kind == TriviaKind::WhiteSpace) && trivia.start_position() == *end {
                    *end = trivia.end_position();
                } else {
                    current_block = None;
                }
            }

            if current_block.is_none() {
                submit(&mut pending);
            }

            if !trivia.kind.is_comment() {
                continue;
            }

            if trivia.kind.is_single_line_comment() && self.cfg.preserve_single_line_comments {
                continue;
            }

            let is_empty = comment_lines(trivia).iter().all(|(_, line)| line.trim().is_empty());
            if !is_empty {
                if trivia.kind.is_single_line_comment() {
                    current_block = Some((trivia.kind, trivia.end_position()));
                    pending.clear();
                }
                continue;
            }

            pending.push(trivia.span);
        }

        submit(&mut pending);
    }
}
