use indoc::indoc;
use toml::Value;

use mago_ast::Program;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleOptionDefinition;
use crate::plugin::comment::rules::utils::comment_content;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct NoEmptyCommentsRule;

const PRESERVE_SINGLE_LINE: &str = "preserve-single-line-comments";
const PRESERVE_SINGLE_LINE_DEFAULT: bool = false;

impl Rule for NoEmptyCommentsRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Empty Comments", Level::Note)
            .with_description(indoc! {"
            Detects empty comments in the codebase. Empty comments are not useful and should be removed
            to keep the codebase clean and maintainable.
        "})
            .with_option(RuleOptionDefinition {
                name: PRESERVE_SINGLE_LINE,
                r#type: "boolean",
                description: "Whether to preserve empty single-line comments.",
                default: Value::Boolean(PRESERVE_SINGLE_LINE_DEFAULT),
            })
    }
}

impl<'a> Walker<LintContext<'a>> for NoEmptyCommentsRule {
    fn walk_program<'ast>(&self, program: &'ast Program, context: &mut LintContext<'a>) {
        let preseve_single_line =
            context.option(PRESERVE_SINGLE_LINE).and_then(|c| c.as_bool()).unwrap_or(PRESERVE_SINGLE_LINE_DEFAULT);

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
