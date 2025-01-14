use indoc::indoc;

use mago_ast::*;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
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
}

impl<'a> Walker<LintContext<'a>> for RedundantLabelRule {
    fn walk_program<'ast>(&self, program: &'ast Program, context: &mut LintContext<'a>) {
        let node = Node::Program(program);

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

            context.report_with_fix(issue, |plan| plan.delete(label_span.to_range(), SafetyClassification::Safe));
        }
    }
}
