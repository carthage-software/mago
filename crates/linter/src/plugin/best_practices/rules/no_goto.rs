use indoc::indoc;

use mago_ast::*;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct NoGotoRule;

impl Rule for NoGotoRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No GOTO", Level::Note).with_description(indoc! {"
            Detects the use of `goto` statements in code. The `goto` statement can make code harder to read,
            understand, and maintain. It can lead to spaghetti code and make it difficult to follow the flow
            of execution.
        "})
    }
}

impl<'a> Walker<LintContext<'a>> for NoGotoRule {
    fn walk_in_goto<'ast>(&self, goto: &'ast Goto, context: &mut LintContext<'a>) {
        let issue = Issue::new(context.level(), "Avoid using `goto`.")
            .with_annotation(Annotation::primary(goto.goto.span()).with_message("This `goto` statement is used here."))
            .with_annotation(Annotation::secondary(goto.label.span()))
            .with_note("The `goto` statement can make code harder to read, understand, and maintain.")
            .with_note("It can lead to spaghetti code and make it difficult to follow the flow of execution.")
            .with_note(
                "Consider using structured control flow statements like `if`, `else`, `for`, and `while` instead.",
            )
            .with_help("Refactor your code to avoid using `goto`.");

        context.report(issue);
    }

    fn walk_in_label<'ast>(&self, label: &'ast Label, context: &mut LintContext<'a>) {
        let name = context.lookup(&label.name.value);

        let issue = Issue::new(context.level(), "Avoid using labels")
            .with_annotation(
                Annotation::primary(label.span()).with_message(format!("Label `{}` is declared here.", name)),
            )
            .with_note("Labels are often used with `goto` statements, which can make code harder to read and maintain.")
            .with_note(
                "Consider using structured control flow statements like `if`, `else`, `for`, and `while` instead.",
            )
            .with_help("Refactor your code to avoid using labels.");

        context.report(issue);
    }
}
