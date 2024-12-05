use fennec_ast::*;
use fennec_fixer::SafetyClassification;
use fennec_reporting::*;
use fennec_span::HasPosition;
use fennec_span::HasSpan;
use fennec_walker::Walker;

use crate::context::LintContext;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct RedundantStringConcatRule;

impl Rule for RedundantStringConcatRule {
    fn get_name(&self) -> &'static str {
        "redundant-string-concat"
    }

    fn get_default_level(&self) -> Option<Level> {
        Some(Level::Help)
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

            let dangerous = match context.interner.lookup(&right.value)[1..].as_bytes() {
                [b'{', ..] => true,
                _ => false,
            };

            if dangerous {
                // $a = "\u" . "{1F418}";
                // $b = "\u{1F418}";

                return;
            }

            let issue = Issue::new(context.level(), "string concatenation can be simplified")
                .with_help("consider combining these strings into a single string")
                .with_annotations(vec![
                    Annotation::primary(operator.span()),
                    Annotation::secondary(left.span()),
                    Annotation::secondary(right.span()),
                ]);

            context.report_with_fix(issue, |plan| {
                let range = (left.span.end.offset - 1)..(right.span.start.offset + 1);

                plan.delete(range, SafetyClassification::Safe)
            });
        }
    }
}
