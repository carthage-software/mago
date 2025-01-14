use indoc::indoc;

use mago_ast::*;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasPosition;
use mago_span::HasSpan;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
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
}

impl<'a> Walker<LintContext<'a>> for RedundantStringConcatRule {
    fn walk_in_binary<'ast>(&self, binary: &'ast Binary, context: &mut LintContext<'a>) {
        let Binary { lhs, operator, rhs } = binary;

        if !operator.is_concatenation() {
            return;
        }

        let (Expression::Literal(Literal::String(left)), Expression::Literal(Literal::String(right))) =
            (lhs.as_ref(), rhs.as_ref())
        else {
            return;
        };

        if left.kind == right.kind {
            if context.semantics.source.line_number(left.offset())
                != context.semantics.source.line_number(right.offset())
            {
                // strings are on different lines
                return;
            }

            let dangerous = matches!(context.interner.lookup(&right.value)[1..].as_bytes(), [b'{', ..]);
            if dangerous {
                // $a = "\u" . "{1F418}";
                // $b = "\u{1F418}";

                return;
            }

            let issue = Issue::new(context.level(), "String concatenation can be simplified.")
                .with_help("Consider combining these strings into a single string.")
                .with_annotations(vec![
                    Annotation::primary(operator.span()).with_message("Redundant string concatenation."),
                    Annotation::secondary(left.span()).with_message("Left string"),
                    Annotation::secondary(right.span()).with_message("Right string"),
                ]);

            context.report_with_fix(issue, |plan| {
                let range = (left.span.end.offset - 1)..(right.span.start.offset + 1);

                plan.delete(range, SafetyClassification::Safe)
            });
        }
    }
}
