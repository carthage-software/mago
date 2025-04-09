use std::sync::LazyLock;

use indoc::indoc;
use regex::Regex;

use mago_reporting::*;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::directive::LintDirective;
use crate::rule::Rule;
use crate::utils::comment_lines;

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

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Program(program) = node else { return LintDirective::Abort };

        for trivia in program.trivia.iter() {
            if !trivia.kind.is_comment() {
                continue;
            }

            for line in comment_lines(trivia, context.interner) {
                let trimmied = line.trim_start().to_lowercase();
                if !trimmied.starts_with("todo") {
                    continue;
                }

                if (*TAGGED_TODO_REGEX).is_match(&trimmied) {
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

        LintDirective::Abort
    }
}
