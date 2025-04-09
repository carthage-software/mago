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
pub struct StringInterpolationBracesRule;

impl Rule for StringInterpolationBracesRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("String Interpolation Braces", Level::Note)
            .with_description(indoc! {"
                Enforces the use of curly braces around variables within string interpolation.

                Using curly braces (`{$variable}`) within interpolated strings ensures clarity and avoids potential ambiguity, especially when variables are followed by alphanumeric characters. This rule promotes consistent and predictable code.
            "})
            .with_example(RuleUsageExample::valid(
                "Using braces within string interpolation",
                indoc! {r#"
                    <?php

                    $a = "Hello, {$name}!";
                    $b = "Hello, {$name}!";
                    $c = "Hello, {$$name}!";
                    $d = "Hello, {${$object->getMethod()}}!";
                "#},
            ))

            .with_example(RuleUsageExample::invalid(
                "Using variables without braces within string interpolation",
                indoc! {r#"
                    <?php

                    $a = "Hello, $name!";
                    $b = "Hello, ${name}!";
                    $c = "Hello, ${$name}!";
                    $d = "Hello, ${$object->getMethod()}!";
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::CompositeString(composite_string) = node else {
            return LintDirective::default();
        };

        let mut unbraced_expressions = vec![];
        for part in composite_string.parts().iter() {
            let StringPart::Expression(expression) = part else {
                // Either literal string part or braced expression, so continue.
                continue;
            };

            unbraced_expressions.push((
                expression.span(),
                !matches!(
                    expression.as_ref(),
                    Expression::Variable(Variable::Indirect(variable))
                    if matches!(
                        variable.expression.as_ref(),
                        Expression::Identifier(_) | Expression::Variable(_)
                    )
                ),
            ));
        }

        if unbraced_expressions.is_empty() {
            return LintDirective::default();
        }

        let mut issue = Issue::new(context.level(), "Unbraced variable in string interpolation").with_annotation(
            Annotation::primary(composite_string.span())
                .with_message("String interpolation contains unbraced variables."),
        );

        for (span, _) in &unbraced_expressions {
            issue = issue.with_annotation(
                Annotation::secondary(*span).with_message("Variable should be enclosed in curly braces."),
            );
        }

        issue = issue.with_note("Using curly braces around variables in interpolated strings improves readability and prevents potential parsing issues.")
            .with_help("Wrap the variable in curly braces, e.g., `{$variable}`.");

        context.propose(issue, |plan| {
            for (span, wrap_in_braces) in unbraced_expressions {
                if wrap_in_braces {
                    plan.insert(span.start.offset, "{", SafetyClassification::Safe);
                    plan.insert(span.end.offset, "}", SafetyClassification::Safe);
                } else {
                    plan.replace(span.start.offset..span.start.offset + 2, "{$", SafetyClassification::Safe);
                }
            }
        });

        LintDirective::default()
    }
}
