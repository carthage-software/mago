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
pub struct RedundantClosingTagRule;

impl Rule for RedundantClosingTagRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Redundant Closing Tag", Level::Help)
            .with_description(indoc! {"
                Detects redundant closing tags ( `?>` ) at the end of a file.
            "})
            .with_example(RuleUsageExample::invalid(
                "A redundant closing tag at the end of a file",
                indoc! {r#"
                    <?php

                    echo "Hello, world!";

                    ?>
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Program(program) = node else { return LintDirective::Abort };

        check_statements(&program.statements, context);

        LintDirective::Abort
    }
}

fn check_statements(sequence: &Sequence<Statement>, context: &mut LintContext<'_>) {
    let Some(last_statement) = sequence.last() else {
        return;
    };

    if let Statement::ClosingTag(closing_tag) = last_statement {
        let issue = Issue::new(context.level(), "Redundant closing tag ( `?>` ).")
            .with_annotation(Annotation::primary(closing_tag.span()).with_message("This closing tag is redundant."))
            .with_help("Remove the redundant closing tag ( `?>` ).");

        context.propose(issue, |plan| plan.delete(closing_tag.span().to_range(), SafetyClassification::Safe));

        return;
    }

    if let Statement::Inline(inline) = last_statement {
        let stmts_len = sequence.len();
        if stmts_len < 2 {
            return;
        }

        let value = context.interner.lookup(&inline.value);
        if value.bytes().all(|b| b.is_ascii_whitespace()) {
            let Some(Statement::ClosingTag(tag)) = sequence.get(stmts_len - 2) else {
                return;
            };

            let issue = Issue::new(context.level(), "Redundant closing tag ( `?>` ) followed by trailing whitespace.")
                .with_annotation(Annotation::primary(tag.span()).with_message("This closing tag is redundant."))
                .with_annotation(
                    Annotation::secondary(inline.span())
                        .with_message("This inline statement is contains only whitespace."),
                )
                .with_help("Remove the redundant closing tag ( `?>` ) and trailing whitespace.");

            context.propose(issue, |plan| {
                plan.delete(inline.span().to_range(), SafetyClassification::Safe);
                plan.delete(tag.span().to_range(), SafetyClassification::Safe);
            });
        }
    }

    if let Statement::Namespace(namespace) = last_statement {
        match &namespace.body {
            NamespaceBody::Implicit(namespace_implicit_body) => {
                check_statements(&namespace_implicit_body.statements, context);
            }
            NamespaceBody::BraceDelimited(_) => {}
        }
    }
}
