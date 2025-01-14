use indoc::indoc;

use mago_ast::*;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct LoopDoesNotIterateRule;

impl Rule for LoopDoesNotIterateRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Loop Does Not Iterate", Level::Warning)
            .with_description(indoc! {"
                Detects loops (for, foreach, while, do-while) that unconditionally break or return
                before executing even a single iteration. Such loops are misleading or redundant
                since they give the impression of iteration but never actually do so.
            "})
            .with_example(RuleUsageExample::valid(
                "A `for` loop that executes at least one iteration",
                indoc! {r#"
                    <?php

                    for ($i = 0; $i < 3; $i++) {
                        echo $i;
                        // The loop isn't unconditionally exited at the start.
                        // This will iterate until $i == 3 or a conditional break occurs later.
                    }
                "#},
            ))
            .with_example(RuleUsageExample::valid(
                "A `while` loop with a conditional break",
                indoc! {r#"
                    <?php

                    $i = 0;
                    while ($i < 5) {
                        echo $i++;
                        if ($i === 2) {
                            // This break is conditional, so the loop isn't unconditionally terminated.
                            break;
                        }
                    }
                "#},
            ))
            .with_example(RuleUsageExample::valid(
                "A `foreach` that processes an array",
                indoc! {r#"
                    <?php

                    $items = [1, 2, 3];
                    foreach ($items as $item) {
                        echo $item;
                        // No unconditional break/return here, so it iterates through items.
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "A `for` loop with an unconditional break immediately",
                indoc! {r#"
                    <?php

                    for ($i = 0; $i < 3; $i++) {
                        break; // The loop never truly iterates, as this break is unconditional.
                        echo "Unreachable";
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "A `while` loop that unconditionally returns",
                indoc! {r#"
                    <?php

                    while (true) {
                        return; // The loop never iterates, since we return on the first pass.
                        echo "Unreachable";
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "A `do-while` loop that unconditionally breaks in the body",
                indoc! {r#"
                    <?php

                    do {
                        break; // Even though 'do-while' typically guarantees one iteration,
                               // this unconditional break prevents the loop from actually iterating.
                        echo "Unreachable";
                    } while (false);
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "A `foreach` loop that unconditionally returns",
                indoc! {r#"
                    <?php

                    foreach ([1, 2, 3] as $item) {
                        return; // No iteration occurs, as the function ends immediately.
                        echo "Unreachable";
                    }
                "#},
            ))
    }
}

impl LoopDoesNotIterateRule {
    fn report(&self, r#loop: impl HasSpan, terminator: LoopTerminator<'_>, context: &mut LintContext<'_>) {
        let loop_span = r#loop.span();
        let terminator_span = match terminator {
            LoopTerminator::Break(break_stmt) => break_stmt.span(),
            LoopTerminator::Return(return_stmt) => return_stmt.span(),
        };

        let issue = Issue::new(context.level(), "Loop does not iterate.")
            .with_annotations([
                Annotation::primary(loop_span).with_message("This loop does not iterate."),
                Annotation::secondary(terminator_span)
                    .with_message("This statement unconditionally terminates the loop."),
            ])
            .with_help("Remove or refactor the loop to avoid redundant or misleading code.");

        context.report(issue);
    }
}

impl<'a> Walker<LintContext<'a>> for LoopDoesNotIterateRule {
    fn walk_in_foreach(&self, foreach: &Foreach, context: &mut LintContext<'a>) {
        if let Some(terminator) = match &foreach.body {
            ForeachBody::Statement(stmt) => get_loop_terminator_from_statement(stmt),
            ForeachBody::ColonDelimited(block) => get_loop_terminator_from_statements(block.statements.as_slice()),
        } {
            self.report(foreach, terminator, context);
        }
    }

    fn walk_in_for(&self, for_loop: &For, context: &mut LintContext<'a>) {
        if let Some(terminator) = match &for_loop.body {
            ForBody::Statement(stmt) => get_loop_terminator_from_statement(stmt),
            ForBody::ColonDelimited(block) => get_loop_terminator_from_statements(block.statements.as_slice()),
        } {
            self.report(for_loop, terminator, context);
        }
    }

    fn walk_in_while(&self, while_loop: &While, context: &mut LintContext<'a>) {
        if let Some(terminator) = match &while_loop.body {
            WhileBody::Statement(stmt) => get_loop_terminator_from_statement(stmt),
            WhileBody::ColonDelimited(block) => get_loop_terminator_from_statements(block.statements.as_slice()),
        } {
            self.report(while_loop, terminator, context);
        }
    }

    fn walk_in_do_while(&self, do_while: &DoWhile, context: &mut LintContext<'a>) {
        if let Some(terminator) = get_loop_terminator_from_statement(&do_while.statement) {
            self.report(do_while, terminator, context);
        }
    }
}

enum LoopTerminator<'a> {
    Break(&'a Break),
    Return(&'a Return),
}

fn get_loop_terminator_from_statements(statements: &[Statement]) -> Option<LoopTerminator<'_>> {
    for statement in statements.iter().rev() {
        if let Some(terminator) = get_loop_terminator_from_statement(statement) {
            return Some(terminator);
        }
    }

    None
}

fn get_loop_terminator_from_statement(statement: &Statement) -> Option<LoopTerminator<'_>> {
    match statement {
        Statement::Block(block) => get_loop_terminator_from_statements(block.statements.as_slice()),
        Statement::Break(break_stmt) => match break_stmt.level {
            None | Some(Expression::Literal(Literal::Integer(LiteralInteger { value: Some(1), .. }))) => {
                Some(LoopTerminator::Break(break_stmt))
            }
            Some(_) => None,
        },
        Statement::Return(return_stmt) => Some(LoopTerminator::Return(return_stmt)),
        _ => None,
    }
}
