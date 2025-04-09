use std::sync::LazyLock;

use indoc::indoc;
use regex::Regex;

use mago_reporting::*;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;
use crate::utils::comment_lines;

static TAGGED_FIXME_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"fixme\((#|@)?\S+\)").unwrap());

#[derive(Clone, Debug)]
pub struct NoUntaggedFixmeRule;

impl Rule for NoUntaggedFixmeRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Untagged FIXME", Level::Warning)
            .with_description(indoc! {"
            Detects FIXME comments that are not tagged with a user or issue reference. Untagged FIXME comments
            are not actionable and can be easily missed by the team. Tagging the FIXME comment with a user or
            issue reference ensures that the issue is tracked and resolved.
        "})
            .with_example(RuleUsageExample::valid(
                "Correctly tagged FIXME comment",
                indoc! {r#"
                    <?php

                    // FIXME(@azjezz) This is a valid FIXME comment.
                    // FIXME(azjezz) This is a valid FIXME comment.
                    // FIXME(#123) This is a valid FIXME comment.
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Untagged FIXME comment",
                indoc! {r#"
                    <?php

                    // FIXME: This is an invalid FIXME comment.
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Program(program) = node else { return LintDirective::Abort };

        for trivia in program.trivia.iter() {
            if !trivia.kind.is_comment() {
                continue;
            }

            for line in comment_lines(trivia, context.interner) {
                let trimmied = line.trim_start().to_lowercase();
                if !trimmied.starts_with("fixme") {
                    continue;
                }

                if (*TAGGED_FIXME_REGEX).is_match(&trimmied) {
                    continue;
                }

                context.report(
                    Issue::new(context.level(), "FIXME comment should be tagged with (@username) or (#issue).")
                        .with_annotation(Annotation::primary(trivia.span))
                        .with_help(
                            "Add a user tag or issue reference to the FIXME comment, e.g. FIXME(@azjezz), FIXME(azjezz), FIXME(#123).",
                        )
                );

                break;
            }
        }

        LintDirective::Abort
    }
}
