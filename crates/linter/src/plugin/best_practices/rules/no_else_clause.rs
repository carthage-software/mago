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
pub struct NoElseClauseRule;

impl Rule for NoElseClauseRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("No Else Clause", Level::Help)
            .with_description(indoc! {"
                Flags if statements that include an else branch, including else-if chains.

                Often, else clauses can be removed to simplify your control flow. By using early returns or by splitting
                the logic into smaller functions, your code becomes easier to read and maintain. For simple assignments,
                consider using the ternary operator.
            "})
            .with_example(RuleUsageExample::valid(
                "If statement with early return",
                indoc! {r#"
                    <?php

                    if ($condition) {
                        doSomething();

                        return;
                    }

                    doSomethingElse();
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "If statement with an else clause",
                indoc! {r#"
                    <?php

                    if ($condition) {
                        doSomething();
                    } else {
                        doSomethingElse();
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Else-if chain",
                indoc! {r#"
                    <?php

                    if ($x) {
                        doX();
                    } elseif ($y) {
                        doY();
                    }
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::If(if_statement) = node else { return LintDirective::default() };

        match &if_statement.body {
            IfBody::Statement(if_statement_body) => {
                for else_if_clause in if_statement_body.else_if_clauses.iter() {
                    context.report(
                        Issue::new(
                            context.level(),
                            "Avoid else-if clauses for clearer control flow"
                        )
                        .with_annotation(
                            Annotation::primary(else_if_clause.span())
                                .with_message("Else-if clause detected here")
                        )
                        .with_note("Else-if clauses often indicate that multiple paths can be handled with early returns or separate functions")
                        .with_help("Consider refactoring to eliminate the else-if branch")
                    );
                }

                if let Some(else_clause) = &if_statement_body.else_clause {
                    context.report(
                        Issue::new(
                            context.level(),
                            "Avoid else clauses for simpler control flow"
                        )
                        .with_annotation(
                            Annotation::primary(else_clause.span())
                                .with_message("Else clause detected here")
                        )
                        .with_note("Else clauses can usually be removed by using early returns or splitting the logic into smaller functions")
                        .with_help("Refactor your code to remove the else branch and simplify control flow")
                    );
                }
            }
            IfBody::ColonDelimited(if_colon_delimited_body) => {
                for else_if_clause in if_colon_delimited_body.else_if_clauses.iter() {
                    context.report(
                        Issue::new(context.level(), "Avoid else-if clauses for clearer control flow")
                            .with_annotation(
                                Annotation::primary(else_if_clause.span()).with_message("Else-if clause detected here"),
                            )
                            .with_note("Else-if clauses can often be replaced with early returns or separate functions")
                            .with_help("Consider refactoring to eliminate the else-if branch"),
                    );
                }

                if let Some(else_clause) = &if_colon_delimited_body.else_clause {
                    context.report(
                        Issue::new(
                            context.level(),
                            "Avoid else clauses for simpler control flow"
                        )
                        .with_annotation(
                            Annotation::primary(else_clause.span())
                                .with_message("Else clause detected here")
                        )
                        .with_note("Else clauses can usually be removed by using early returns or splitting logic into smaller functions")
                        .with_help("Refactor your code to remove the else branch and simplify control flow")
                    );
                }
            }
        }

        LintDirective::default()
    }
}
