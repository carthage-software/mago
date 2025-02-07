use indoc::indoc;

use mago_ast::*;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct RedundantLabelRule;

impl Rule for RedundantLabelRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Redundant Label", Level::Help)
            .with_description(indoc! {"
                Detects redundant `goto` labels that are declared but not used.
            "})
            .with_example(RuleUsageExample::invalid(
                "A redundant `goto` label",
                indoc! {r#"
                    <?php

                    label:
                    echo "Hello, world!";
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Program(_) = node else { return LintDirective::Abort };

        let labels =
            node.filter_map(
                |node| {
                    if let Node::Label(label) = node {
                        Some((label.name.value, label.span()))
                    } else {
                        None
                    }
                },
            );

        let gotos = node.filter_map(|node| if let Node::Goto(goto) = node { Some(goto.label.value) } else { None });

        for (label_id, label_span) in labels.into_iter() {
            if gotos.contains(&label_id) {
                continue;
            }

            let label_name = context.interner.lookup(&label_id);

            let issue = Issue::new(context.level(), format!("Redundant goto label `{}`.", label_name))
                .with_annotation(Annotation::primary(label_span).with_message("This label is declared but not used."))
                .with_note(format!("Label `{}` is declared but not used by any `goto` statement.", label_name))
                .with_help("Remove the redundant label.");

            context.propose(issue, |plan| plan.delete(label_span.to_range(), SafetyClassification::Safe));
        }

        LintDirective::Abort
    }
}
