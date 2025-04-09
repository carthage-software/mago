use indoc::indoc;

use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct UseWhileInsteadOfForRule;

impl Rule for UseWhileInsteadOfForRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Use While Instead Of For", Level::Note).with_description(indoc! {"
            Suggests using a `while` loop instead of a `for` loop when the `for` loop does not have any
            initializations or increments. This can make the code more readable and concise.
        "})
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::For(r#for) = node else { return LintDirective::default() };

        if !r#for.initializations.is_empty() || !r#for.increments.is_empty() {
            return LintDirective::default();
        }

        let issue = Issue::new(
            context.level(),
            "Use `while` loop instead of `for` loop.",
        )
        .with_annotation(Annotation::primary(r#for.span()).with_message("This `for` loop can be simplified to a `while` loop."))
        .with_note("This `for` loop can be simplified to a `while` loop since it doesn't have initializations or increments.")
        .with_help("Use a `while` loop instead of a `for` loop.");

        context.propose(issue, |plan| {
            plan.delete(r#for.r#for.span.to_range(), SafetyClassification::Safe);
            plan.insert(r#for.r#for.span.start.offset, "while", SafetyClassification::Safe);

            plan.delete(r#for.initializations_semicolon.to_range(), SafetyClassification::Safe);
            if r#for.conditions.is_empty() {
                plan.insert(r#for.initializations_semicolon.end.offset, "true", SafetyClassification::Safe);
            } else {
                for semicolon in r#for.conditions.tokens.iter() {
                    plan.replace(semicolon.span.to_range(), " && ", SafetyClassification::Safe);
                }
            }

            plan.delete(r#for.conditions_semicolon.to_range(), SafetyClassification::Safe);
            if let ForBody::ColonDelimited(for_colon_delimited_body) = &r#for.body {
                plan.replace(for_colon_delimited_body.end_for.span.to_range(), "endwhile", SafetyClassification::Safe);
            }
        });

        LintDirective::default()
    }
}
