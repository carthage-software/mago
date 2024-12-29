use mago_ast::Program;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::plugin::comment::rules::utils::comment_content;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct NoEmptyCommentsRule;

impl Rule for NoEmptyCommentsRule {
    #[inline]
    fn get_name(&self) -> &'static str {
        "no-empty-comments"
    }

    #[inline]
    fn get_default_level(&self) -> Option<Level> {
        Some(Level::Note)
    }
}

impl<'a> Walker<LintContext<'a>> for NoEmptyCommentsRule {
    fn walk_program<'ast>(&self, program: &'ast Program, context: &mut LintContext<'a>) {
        let preseve_single_line =
            context.option("preserve-single-line-comments").and_then(|c| c.as_bool()).unwrap_or(false);

        for trivia in program.trivia.iter() {
            if trivia.kind.is_single_line_comment() && preseve_single_line {
                continue;
            }

            if let Some(content) = comment_content(trivia, context) {
                let content = content.trim();
                if !content.is_empty() {
                    continue;
                }

                let issue = Issue::new(context.level(), "Empty comments are not allowed.")
                    .with_annotation(Annotation::primary(trivia.span).with_message("This is an empty comment."))
                    .with_help("Consider removing this comment.");

                context.report_with_fix(issue, |plan| {
                    plan.delete(trivia.span.to_range(), SafetyClassification::Safe);
                });
            }
        }
    }
}
