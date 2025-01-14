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
pub struct RequireBlockStatementBodyRule;

impl Rule for RequireBlockStatementBodyRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Require Block Statement Body", Level::Note)
            .with_description(indoc! {"
                Enforces that loop bodies (`for`, `while`, `foreach`) are enclosed in braces `{}`.
                Using single statements without braces can lead to confusion or errors if new
                statements are later inserted.
            "})
            .with_example(RuleUsageExample::valid(
                "Using braces around a loop body",
                indoc! {r#"
                    <?php

                    for ($i = 0; $i < 5; $i++) {
                        echo $i;
                    }

                    while ($condition) {
                        doSomething();
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Loop body without braces",
                indoc! {r#"
                    <?php

                    for ($i = 0; $i < 5; $i++)
                        echo $i; // Error: Should be wrapped in {}

                    while ($condition)
                        doSomething(); // Error: Should be wrapped in {}
                "#},
            ))
    }
}

impl RequireBlockStatementBodyRule {
    fn report(&self, r#loop: &impl HasSpan, statement: &Statement, context: &mut LintContext<'_>) {
        if matches!(statement, Statement::Block(_)) {
            return;
        }

        let issue = Issue::new(context.level(), "Loop body should be enclosed in a block.")
            .with_annotations([Annotation::primary(statement.span()), Annotation::secondary(r#loop.span())])
            .with_note(
                "Enclosing the loop body in a block improves readability and prevents potential errors \
                when adding or modifying statements within the loop.",
            )
            .with_help("Enclose the loop body in a block for clarity and error prevention.");

        context.report_with_fix(issue, |plan| {
            if matches!(statement, Statement::Noop(_)) {
                plan.replace(statement.span().to_range(), "{}", SafetyClassification::Safe);
            } else {
                plan.insert(statement.span().start.offset, "{", SafetyClassification::Safe);
                plan.insert(statement.span().end.offset, "}", SafetyClassification::Safe);
            }
        });
    }
}
impl<'a> Walker<LintContext<'a>> for RequireBlockStatementBodyRule {
    fn walk_in_for<'ast>(&self, r#for: &'ast For, context: &mut LintContext<'a>) {
        let ForBody::Statement(statement) = &r#for.body else {
            return;
        };

        self.report(r#for, statement, context);
    }

    fn walk_in_while<'ast>(&self, r#while: &'ast While, context: &mut LintContext<'a>) {
        let WhileBody::Statement(statement) = &r#while.body else {
            return;
        };

        self.report(r#while, statement, context);
    }

    fn walk_in_foreach<'ast>(&self, foreach: &'ast Foreach, context: &mut LintContext<'a>) {
        let ForeachBody::Statement(statement) = &foreach.body else {
            return;
        };

        self.report(foreach, statement, context);
    }
}
