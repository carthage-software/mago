use indoc::indoc;

use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct NoEmptyCatchClauseRule;

impl Rule for NoEmptyCatchClauseRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Empty Catch Clause", Level::Warning)
            .with_description(indoc! {"
                Warns when a catch clause is empty.

                An empty catch clause suppresses exceptions without handling or logging them,
                potentially hiding errors that should be addressed.
            "})
            .with_example(RuleUsageExample::valid(
                "Catch clause with error handling",
                indoc! {r#"
                    <?php
                    try {
                        // some code
                    } catch(Exception $e) {
                        error_log($e);

                        throw $e;
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Empty clause block suppresses error",
                indoc! {r#"
                    <?php
                    try {
                        // some code
                    } catch(Exception $e) {
                    }
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Try(r#try) = node else { return LintDirective::default() };

        for catch_clause in r#try.catch_clauses.iter() {
            if are_statements_empty(catch_clause.block.statements.as_slice()) {
                let issue = Issue::new(context.level(), "Empty catch clause suppresses errors.")
                    .with_annotation(
                        Annotation::primary(catch_clause.span()).with_message("This catch clause is empty."),
                    )
                    .with_note("Empty catch clauses hide exceptions and make debugging more difficult.")
                    .with_help("Add error handling or remove the catch clause.");

                context.report(issue);
            }
        }

        LintDirective::default()
    }
}

#[inline]
fn is_statement_empty(statement: &Statement) -> bool {
    match statement {
        Statement::Block(block) => are_statements_empty(block.statements.as_slice()),
        Statement::Noop(_) => true,
        _ => false,
    }
}

#[inline]
fn are_statements_empty(statements: &[Statement]) -> bool {
    statements.is_empty() || statements.iter().all(is_statement_empty)
}
