use indoc::indoc;
use mago_ast::*;
use mago_fixer::SafetyClassification;
use mago_php_version::feature::Feature;
use mago_reporting::*;
use mago_span::HasSpan;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

/// A rule that discourages using the `#️⃣` emoji in place of the ASCII `#`
/// for comments or attribute declarations.
///
/// Although `#️⃣` may work in PHP or Mago, it can confuse readers and may
/// break third-party tools that expect a simple ASCII hash symbol.
#[derive(Clone, Debug)]
pub struct NoHashEmojiRule;

impl Rule for NoHashEmojiRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Hash Emoji", Level::Warning)
            .with_description(indoc! {"
                Discourages usage of the `#️⃣` emoji in place of the ASCII `#`.

                While this emoji might look visually appealing, it can cause confusion for
                other developers and potentially break external tools that do not handle
                emoji characters in code.
            "})
            .with_example(RuleUsageExample::valid(
                "Using a normal `#` symbol for comments",
                indoc! {r#"
                    <?php

                    # This is a comment
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using the `#️⃣` emoji for comments",
                indoc! {r#"
                    <?php

                    #️⃣ This is a comment
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using the `#️⃣` emoji for an attribute declaration",
                indoc! {r#"
                    <?php

                    #️⃣[MyAttribute]
                    class Foo {}
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        match node {
            Node::Program(program) => {
                for trivia in &program.trivia.nodes {
                    let Trivia { kind: TriviaKind::HashComment, value, .. } = trivia else {
                        continue;
                    };

                    let comment = context.interner.lookup(value);
                    if comment.starts_with("#️⃣") {
                        let issue = Issue::new(context.level(), "Emoji-based hash (`#️⃣`) used instead of ASCII `#`.")
                            .with_annotation(
                                Annotation::primary(trivia.span()).with_message("This uses an emoji in place of `#`."),
                            )
                            .with_note(
                                "While this may work in PHP, it can confuse readers and may break external tools.",
                            )
                            .with_help("Replace `#️⃣` with the normal ASCII `#`.");

                        context.report_with_fix(issue, |plan| {
                            plan.replace(
                                trivia.span().start.offset..(trivia.span().start.offset + "#️⃣".len()),
                                "#".to_string(),
                                SafetyClassification::Safe,
                            );
                        });
                    }
                }

                // Continue traversing the program to check for attribute declarations.
                LintDirective::Continue
            }
            Node::AttributeList(attribute_list) => {
                // Grab snippet for the attribute prefix
                let code = context.interner.lookup(&context.semantics.source.content);
                let range_start = attribute_list.hash_left_bracket.start.offset;
                let range_end = attribute_list.hash_left_bracket.end.offset;
                let attribute_list_code = &code[range_start..range_end];

                if attribute_list_code.starts_with("#️⃣") {
                    let issue =
                        Issue::new(context.level(), "Emoji-based hash (`#️⃣`) used for an attribute declaration.")
                            .with_annotation(
                                Annotation::primary(attribute_list.hash_left_bracket.span())
                                    .with_message("This uses an emoji in place of `#`."),
                            )
                            .with_note(
                                "While this may work in PHP, it can confuse readers and may break external tools.",
                            )
                            .with_help("Use `#[` instead of `#️⃣[` for attributes.");

                    context.report_with_fix(issue, |plan| {
                        plan.replace(
                            attribute_list.hash_left_bracket.span().to_range(),
                            "#[".to_string(),
                            SafetyClassification::Safe,
                        );
                    });
                }

                if context.php_version.is_supported(Feature::ClosureInConstantExpressions) {
                    // PHP 8.5+ supports closures in constant expressions, which might contain
                    // attributes inside them, therefore we need to traverse this node.
                    LintDirective::Continue
                } else {
                    // No need to traverse further.
                    LintDirective::Prune
                }
            }
            _ => LintDirective::default(),
        }
    }
}
