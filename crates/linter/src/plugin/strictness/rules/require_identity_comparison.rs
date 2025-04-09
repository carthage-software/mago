use indoc::indoc;

use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct RequireIdentityComparisonRule;

impl Rule for RequireIdentityComparisonRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Require Identity Comparison", Level::Warning)
            .with_description(indoc! {"
                Detects equality and inequality comparisons that should use identity comparison operators.
            "})
            .with_example(RuleUsageExample::valid(
                "An identity comparison operator",
                indoc! {r#"
                    <?php

                    // ...

                    if ($a === $b) {
                        echo '$a is same as $b';
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "An equality comparison operator",
                indoc! {r#"
                    <?php

                    // ...

                    if ($a == $b) {
                        echo '$a is same as $b';
                    }
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Binary(binary) = node else { return LintDirective::default() };

        match &binary.operator {
            // `==` -> `===`
            BinaryOperator::Equal(span) => {
                let issue =
                    Issue::new(context.level(), "Use identity comparison `===` instead of equality comparison `==`.")
                        .with_annotation(Annotation::primary(*span).with_message("Equality operator is used here."))
                        .with_note(
                            "Identity comparison `===` checks for both value and type equality, \
                    while equality comparison `==` performs type coercion, which can lead to unexpected results.",
                        )
                        .with_help("Use `===` to ensure both value and type are equal.");

                context.propose(issue, |plan| plan.replace(span.to_range(), "===", SafetyClassification::Unsafe));
            }
            // `!=` -> `!==`
            BinaryOperator::NotEqual(span) => {
                let issue =
                    Issue::new(context.level(), "Use identity inequality `!==` instead of inequality comparison `!=`.")
                        .with_annotation(Annotation::primary(*span).with_message("Inequality operator is used here."))
                        .with_note(
                            "Identity inequality `!==` checks for both value and type inequality, \
                        while inequality comparison `!=` performs type coercion, which can lead to unexpected results.",
                        )
                        .with_help("Use `!==` to ensure both value and type are different.");

                context.propose(issue, |plan| plan.replace(span.to_range(), "!==", SafetyClassification::Unsafe));
            }
            // `<>` -> `!==`
            BinaryOperator::AngledNotEqual(span) => {
                let issue = Issue::new(
                    context.level(),
                    "Use identity inequality `!==` instead of angled inequality comparison `<>`.",
                )
                .with_annotation(Annotation::primary(*span).with_message("Angled inequality operator is used here."))
                .with_note(
                    "Identity inequality `!==` checks for both value and type inequality, \
                    while angled inequality comparison `<>` performs type coercion, which can lead to unexpected results.",
                )
                .with_help("Use `!==` to ensure both value and type are different.");

                context.propose(issue, |plan| plan.replace(span.to_range(), "!==", SafetyClassification::Unsafe));
            }
            _ => {}
        }

        LintDirective::default()
    }
}
