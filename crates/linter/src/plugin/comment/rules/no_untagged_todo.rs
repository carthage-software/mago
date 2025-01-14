use std::sync::LazyLock;

use indoc::indoc;
use regex::Regex;

use mago_ast::Program;
use mago_reporting::*;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::plugin::comment::rules::utils::comment_content;
use crate::rule::Rule;

static TAGGED_TODO_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"todo\((#|@)?\S+\)").unwrap());

#[derive(Clone, Debug)]
pub struct NoUntaggedTodoRule;

impl Rule for NoUntaggedTodoRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Untagged TODO", Level::Warning).with_description(indoc! {"
            Detects TODO comments that are not tagged with a user or issue reference. Untagged TODOs
            can be difficult to track and may be forgotten. Tagging TODOs with a user or issue reference
            makes it easier to track progress and ensures that tasks are not forgotten.
        "})
    }
}

impl<'a> Walker<LintContext<'a>> for NoUntaggedTodoRule {
    fn walk_program<'ast>(&self, program: &'ast Program, context: &mut LintContext<'a>) {
        for trivia in program.trivia.iter() {
            if let Some(content) = comment_content(trivia, context) {
                let content = content.to_ascii_lowercase();
                if !content.contains("todo") {
                    continue;
                }

                for line in content.lines() {
                    let trimmied = line.trim_start();
                    if !trimmied.starts_with("todo") {
                        continue;
                    }

                    if (*TAGGED_TODO_REGEX).is_match(trimmied) {
                        continue;
                    }

                    context.report(
                        Issue::new(context.level(), "TODO should be tagged with (@username) or (#issue).")
                            .with_annotation(Annotation::primary(trivia.span))
                            .with_help(
                                "Add a user tag or issue reference to the TODO comment, e.g. TODO(@azjezz), TODO(azjezz), TODO(#123).",
                            )
                    );

                    break;
                }
            }
        }
    }
}
