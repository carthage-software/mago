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
pub struct CombineConsecutiveIssetsRule;

impl Rule for CombineConsecutiveIssetsRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Combine Consecutive Issets", Level::Warning)
            .with_description(indoc! {"
                Suggests combining consecutive calls to `isset()` when they are joined by a logical AND.
                For example, `isset($a) && isset($b)` can be turned into `isset($a, $b)`, which is more concise
                and avoids repeated function calls. If one or both `isset()` calls are wrapped in parentheses,
                the rule will still warn, but it will not attempt an automated fix.
            "})
            .with_example(RuleUsageExample::valid(
                "Using `isset()` with multiple variables in a single call",
                indoc! {r#"
                    <?php

                    if (isset($a, $b)) {
                        // ...
                    }
                "#},
            ))
            .with_example(RuleUsageExample::valid(
                "Calls to `isset()` separated by other expressions",
                indoc! {r#"
                    <?php

                    // This won't be flagged, because the isset() calls are not consecutive:
                    if (isset($a) && $b && isset($c)) {
                        // ...
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Two consecutive `isset()` calls using `&&`",
                indoc! {r#"
                    <?php

                    if (isset($a) && isset($b)) {
                        // ...
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Parenthesized `isset()` on one side",
                indoc! {r#"
                    <?php

                    if ((isset($a)) && isset($b)) {
                        // ...
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Three consecutive `isset()` calls using `&&`",
                indoc! {r#"
                    <?php

                    if ((isset($a)) && isset($b) && isset($c)) {
                        // ...
                    }
                "#},
            ))
    }
}

impl<'a> Walker<LintContext<'a>> for CombineConsecutiveIssetsRule {
    fn walk_in_binary(&self, binary: &Binary, context: &mut LintContext<'a>) {
        let BinaryOperator::And(_) = binary.operator else {
            return;
        };

        let Some((left_parenthesized, left_isset)) = get_isset_construct(binary.lhs.as_ref(), true) else {
            return;
        };

        let Some((right_parenthesized, right_isset)) = get_isset_construct(binary.rhs.as_ref(), false) else {
            return;
        };

        let issue = Issue::new(context.level(), "Consecutive isset calls can be combined.")
            .with_annotation(Annotation::primary(left_isset.span()))
            .with_annotation(Annotation::primary(right_isset.span()))
            .with_annotation(Annotation::secondary(binary.span()))
            .with_help("Combine the isset calls into a single call, e.g. `isset($a, $b)`.");

        // don't bother fixing if either of the isset calls is already parenthesized
        // this can be messy to fix and is not worth the effort.
        if left_parenthesized || right_parenthesized {
            return context.report(issue);
        }

        context.report_with_fix(issue, |plan| {
            let to_replace = left_isset.right_parenthesis.join(binary.operator.span());
            let to_delete = right_isset.isset.span.join(right_isset.left_parenthesis);

            plan.replace(to_replace.to_range(), ",".to_string(), SafetyClassification::Safe);
            plan.delete(to_delete.to_range(), SafetyClassification::Safe);
        });
    }
}

fn get_isset_construct(mut expression: &Expression, select_binary_rhs: bool) -> Option<(bool, &IssetConstruct)> {
    let mut between_parentheses = false;

    while let Expression::Parenthesized(parenthesized) = expression {
        expression = parenthesized.expression.as_ref();
        between_parentheses = true;
    }

    match expression {
        Expression::Construct(construct) => {
            if let Construct::Isset(isset) = construct {
                Some((between_parentheses, isset))
            } else {
                None
            }
        }
        Expression::Binary(binary) if select_binary_rhs => {
            if let BinaryOperator::And(_) = binary.operator {
                let (lhs_between_parentheses, lhs_isset) = get_isset_construct(binary.rhs.as_ref(), true)?;

                Some((between_parentheses || lhs_between_parentheses, lhs_isset))
            } else {
                None
            }
        }
        _ => None,
    }
}
