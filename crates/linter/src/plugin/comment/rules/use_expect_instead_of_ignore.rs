use indoc::indoc;

use mago_ast::*;
use mago_reporting::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;
use crate::utils::comment_lines;

#[derive(Clone, Debug)]
pub struct UseExpectInsteadOfIgnoreRule;

impl Rule for UseExpectInsteadOfIgnoreRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Use Expect Instead Of Ignore Rule", Level::Warning)
            .with_description(indoc! {"
                This rule suggests using `@mago-expect` instead of `@mago-ignore` to suppress issues.

                Using `@mago-expect` is generally preferred because it allows the linter to still
                scan the code and verify that the expected issue is actually present. This helps
                prevent unintended issues from being suppressed in the future if the code is
                modified.

                On the other hand, `@mago-ignore` completely silences the linter for the specified
                rule and node, which can lead to hidden issues if the code is changed later.
            "})
            .with_example(RuleUsageExample::valid(
                "Using `@mago-expect`",
                indoc! {r#"
                    <?php

                    // @mago-expect strictness/require-return-type
                    function foo() {}
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using `@mago-ignore` instead of `@mago-expect`",
                indoc! {r#"
                    <?php

                    // @mago-ignore strictness/require-return-type
                    function foo() {}
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
                if !trimmied.starts_with("@mago-ignore") {
                    continue;
                }

                context.report(
                    Issue::new(context.level(), "Use `@mago-expect` instead of `@mago-ignore`.")
                        .with_annotation(Annotation::primary(trivia.span).with_message(
                            "This comment should use `@mago-expect` instead of `@mago-ignore`.",
                        ))
                        .with_note("Using `@mago-ignore` can suppress potential issues in the future if the code is modified.")
                        .with_help("Use `@mago-expect` to ensure that the linter still scans the code and verifies the expected issue."),
                );

                break;
            }
        }

        LintDirective::Abort
    }
}
