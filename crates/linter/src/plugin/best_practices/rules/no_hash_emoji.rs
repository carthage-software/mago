use indoc::indoc;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct NoHashEmojiRule;

impl Rule for NoHashEmojiRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Hash Emoji", Level::Warning)
            .with_description(indoc! {"
                Discourages usage of the `#️⃣` emoji in place of the ASCII `#`.

                While PHP allows the use of emojis in comments, it is generally discouraged to use them in place of the normal ASCII `#` symbol.
                This is because it can confuse readers and may break external tools that expect the normal ASCII `#` symbol.
            "})
            .with_example(RuleUsageExample::valid(
                "Using a normal `#` symbol for comments.",
                indoc! {r#"
                    <?php

                    # This is a comment
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using the `#️⃣` emoji for comments.",
                indoc! {r#"
                    <?php

                    #️⃣ This is a comment
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Trying to use `#️⃣` for attribute declarations ( parsed as a comment ).",
                indoc! {r#"
                    <?php

                    #️⃣[MyAttribute]
                    class Foo {}
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Program(program) = node else { return LintDirective::default() };

        for trivia in &program.trivia.nodes {
            let Trivia { kind: TriviaKind::HashComment, value, .. } = trivia else {
                continue;
            };

            let comment = context.interner.lookup(value);
            if !comment.starts_with("#️⃣") {
                continue;
            }

            let mut issue = Issue::new(context.level(), "Emoji-based hash (`#️⃣`) used instead of ASCII `#`.")
                .with_annotation(Annotation::primary(trivia.span()).with_message("This uses an emoji in place of `#`."))
                .with_note(
                    "While this might render similarly in some editors, it can confuse readers or break tooling.",
                )
                .with_help("Replace `#️⃣` with `#`.");

            if comment.starts_with("#️⃣[") {
                issue = issue.with_note("`#️⃣[` does not parse as an attribute in PHP; use `#[` instead.");
            }

            context.propose(issue, |plan| {
                plan.replace(
                    trivia.span().start.offset..(trivia.span().start.offset + "#️⃣".len()),
                    "#".to_string(),
                    SafetyClassification::Safe,
                );
            });
        }

        // We don't need to walk further for this rule
        LintDirective::Abort
    }
}
