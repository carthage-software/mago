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
            .with_example(RuleUsageExample::valid(
                "A `while` that condtionally continues",
                indoc! {r#"
                    <?php

                    function get_first_command(MessageStream $stream): Message {
                        while (true) {
                            $message = $stream->next();

                            if (!$message->isCommand()) {
                                continue; // This conditional continue doesn't unconditionally terminate the loop.
                            }

                            return $message; // This return is conditional, so the loop isn't unconditionally terminated.
                        }
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

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        match node {
            Node::Foreach(foreach) => {
                if let Some(terminator) = match &foreach.body {
                    ForeachBody::Statement(stmt) => get_loop_terminator_from_statement(stmt),
                    ForeachBody::ColonDelimited(block) => {
                        get_loop_terminator_from_statements(block.statements.as_slice())
                    }
                } {
                    check_loop(foreach, terminator, context);
                }
            }
            Node::For(for_loop) => {
                if let Some(terminator) = match &for_loop.body {
                    ForBody::Statement(stmt) => get_loop_terminator_from_statement(stmt),
                    ForBody::ColonDelimited(block) => get_loop_terminator_from_statements(block.statements.as_slice()),
                } {
                    check_loop(for_loop, terminator, context);
                }
            }
            Node::While(while_loop) => {
                if let Some(terminator) = match &while_loop.body {
                    WhileBody::Statement(stmt) => get_loop_terminator_from_statement(stmt),
                    WhileBody::ColonDelimited(block) => {
                        get_loop_terminator_from_statements(block.statements.as_slice())
                    }
                } {
                    check_loop(while_loop, terminator, context);
                }
            }
            Node::DoWhile(do_while) => {
                if let Some(terminator) = get_loop_terminator_from_statement(&do_while.statement) {
                    check_loop(do_while, terminator, context);
                }
            }
            _ => {}
        }

        LintDirective::default()
    }
}

fn check_loop(r#loop: impl HasSpan, terminator: LoopTerminator<'_>, context: &mut LintContext<'_>) {
    let loop_span = r#loop.span();
    let terminator_span = match terminator {
        LoopTerminator::Break(break_stmt) => break_stmt.span(),
        LoopTerminator::Return(return_stmt) => return_stmt.span(),
    };

    let issue = Issue::new(context.level(), "Loop does not iterate.")
        .with_annotations([
            Annotation::primary(loop_span).with_message("This loop does not iterate."),
            Annotation::secondary(terminator_span).with_message("This statement unconditionally terminates the loop."),
        ])
        .with_help("Remove or refactor the loop to avoid redundant or misleading code.");

    context.report(issue);
}

#[derive(Debug)]
enum LoopTerminator<'a> {
    Break(&'a Break),
    Return(&'a Return),
}

#[inline]
fn get_loop_terminator_from_statements(statements: &[Statement]) -> Option<LoopTerminator<'_>> {
    for statement in statements.iter() {
        if might_skip_terminator(statement) {
            return None;
        }

        if let Some(terminator) = get_loop_terminator_from_statement(statement) {
            return Some(terminator);
        }
    }

    None
}

#[inline]
fn get_loop_terminator_from_statement(statement: &Statement) -> Option<LoopTerminator<'_>> {
    match statement {
        Statement::Block(block) => get_loop_terminator_from_statements(block.statements.as_slice()),
        Statement::Break(break_stmt) => match break_stmt.level {
            None | Some(Expression::Literal(Literal::Integer(LiteralInteger { value: 1, .. }))) => {
                Some(LoopTerminator::Break(break_stmt))
            }
            Some(_) => None,
        },
        Statement::Return(return_stmt) => Some(LoopTerminator::Return(return_stmt)),
        _ => None,
    }
}

#[inline]
fn might_skip_terminator(statement: &Statement) -> bool {
    match statement {
        Statement::Continue(_) | Statement::Goto(_) => true,
        Statement::Block(block) => block.statements.iter().any(might_skip_terminator),
        Statement::If(if_stmt) => match &if_stmt.body {
            IfBody::Statement(body) => {
                if might_skip_terminator(&body.statement) {
                    return true;
                }

                if body.else_clause.as_ref().is_some_and(|clause| might_skip_terminator(&clause.statement)) {
                    return true;
                }

                body.else_if_clauses.iter().any(|clause| might_skip_terminator(&clause.statement))
            }
            IfBody::ColonDelimited(body) => {
                if body.statements.iter().any(might_skip_terminator) {
                    return true;
                }

                if body.else_clause.as_ref().is_some_and(|clause| clause.statements.iter().any(might_skip_terminator)) {
                    return true;
                }

                body.else_if_clauses.iter().any(|clause| clause.statements.iter().any(might_skip_terminator))
            }
        },
        Statement::While(while_stmt) => match &while_stmt.body {
            WhileBody::Statement(body) => might_skip_terminator(body.as_ref()),
            WhileBody::ColonDelimited(body) => body.statements.iter().any(might_skip_terminator),
        },
        Statement::DoWhile(do_while_stmt) => might_skip_terminator(&do_while_stmt.statement),
        Statement::For(for_stmt) => match &for_stmt.body {
            ForBody::Statement(body) => might_skip_terminator(body.as_ref()),
            ForBody::ColonDelimited(body) => body.statements.iter().any(might_skip_terminator),
        },
        Statement::Foreach(foreach_stmt) => match &foreach_stmt.body {
            ForeachBody::Statement(body) => might_skip_terminator(body.as_ref()),
            ForeachBody::ColonDelimited(body) => body.statements.iter().any(might_skip_terminator),
        },
        Statement::Namespace(namespace) => namespace.statements().iter().any(might_skip_terminator),
        Statement::Declare(declare) => match &declare.body {
            DeclareBody::Statement(body) => might_skip_terminator(body.as_ref()),
            DeclareBody::ColonDelimited(body) => body.statements.iter().any(might_skip_terminator),
        },
        Statement::Try(try_stmt) => {
            if try_stmt.block.statements.iter().any(might_skip_terminator) {
                return true;
            }

            if try_stmt.catch_clauses.iter().any(|clause| clause.block.statements.iter().any(might_skip_terminator)) {
                return true;
            }

            try_stmt
                .finally_clause
                .as_ref()
                .is_some_and(|clause| clause.block.statements.iter().any(might_skip_terminator))
        }
        Statement::Switch(switch_stmt) => {
            switch_stmt.body.cases().iter().any(|case| case.statements().iter().any(might_skip_terminator))
        }
        _ => false,
    }
}
