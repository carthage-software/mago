use indoc::indoc;

use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasPosition;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct RedundantStringConcatRule;

impl Rule for RedundantStringConcatRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Redundant String Concat", Level::Help)
            .with_description(indoc! {"
                Detects redundant string concatenation expressions.
            "})
            .with_example(RuleUsageExample::invalid(
                "A redundant string concatenation expression",
                indoc! {r#"
                    <?php

                    $foo = "Hello" . " World";
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Binary(binary) = node else { return LintDirective::default() };

        let Binary { lhs, operator, rhs } = binary;

        if !operator.is_concatenation() {
            return LintDirective::default();
        }

        let (Expression::Literal(Literal::String(left)), Expression::Literal(Literal::String(right))) =
            (lhs.as_ref(), rhs.as_ref())
        else {
            return LintDirective::default();
        };

        if left.kind == right.kind {
            if context.source.line_number(left.offset()) != context.source.line_number(right.offset()) {
                // strings are on different lines
                return LintDirective::Prune;
            }

            let dangerous = matches!(&context.interner.lookup(&right.raw).as_bytes()[1..], [b'{', ..]);
            if dangerous {
                // $a = "\u" . "{1F418}";
                // $b = "\u{1F418}";

                return LintDirective::Prune;
            }

            let issue = Issue::new(context.level(), "String concatenation can be simplified.")
                .with_help("Consider combining these strings into a single string.")
                .with_annotations(vec![
                    Annotation::primary(operator.span()).with_message("Redundant string concatenation."),
                    Annotation::secondary(left.span()).with_message("Left string"),
                    Annotation::secondary(right.span()).with_message("Right string"),
                ]);

            context.propose(issue, |plan| {
                let range = (left.span.end.offset - 1)..(right.span.start.offset + 1);

                plan.delete(range, SafetyClassification::Safe)
            });
        }

        LintDirective::default()
    }
}
