use indoc::indoc;
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
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
            good_example: indoc! {r#"
                <?php

                // This is a useful comment.
                # This is also a useful comment.
                /**
                 * This is a docblock.
                 */
            "#},
            bad_example: indoc! {r#"
                <?php

                //
                #
                /**/
            "#},
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

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Program(program) = node else {
            return;
        };

        for trivia in program.trivia.iter() {
            if !trivia.kind.is_comment() {
                continue;
            }

            if trivia.kind.is_single_line_comment() && self.cfg.preserve_single_line_comments {
                continue;
            }

            let is_empty = comment_lines(trivia).iter().all(|(_, line)| line.trim().is_empty());
            if !is_empty {
                continue;
            }

            let issue = Issue::new(self.cfg.level(), "Empty comments are not allowed.")
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(trivia.span).with_message("This is an empty comment"))
                .with_help("Consider removing this comment.");

            if trivia.kind.is_single_line_comment() {
                ctx.collector.report(issue);

                continue;
            }

            ctx.collector.propose(issue, |plan| {
                plan.delete(trivia.span.to_range(), SafetyClassification::Safe);
            });
        }
    }
}
