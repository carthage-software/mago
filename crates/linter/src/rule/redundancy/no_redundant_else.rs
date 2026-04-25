use indoc::indoc;
use mago_syntax::ast::UnaryPostfix;
use mago_syntax::ast::UnaryPrefix;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoRedundantElseRule {
    meta: &'static RuleMeta,
    cfg: NoRedundantElseConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoRedundantElseConfig {
    pub level: Level,
}

impl Default for NoRedundantElseConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoRedundantElseConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoRedundantElseRule {
    type Config = NoRedundantElseConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Redundant Else",
            code: "no-redundant-else",
            description: indoc! {"
                Flags `if`/`else` statements where the `if` branch always terminates
                control flow (via `return`, `throw`, `exit`, `die`, `continue`, or `break`).

                When the `if` branch unconditionally terminates, the `else` branch becomes
                unnecessary nesting. Extracting the `else` body to follow the `if` flattens
                the control flow without changing semantics.
            "},
            good_example: indoc! {r#"
                <?php

                function process($user) {
                    if (!$user->isVerified()) {
                        return;
                    }

                    $user->login();
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                function process($user) {
                    if (!$user->isVerified()) {
                        return;
                    } else {
                        $user->login();
                    }
                }
            "#},
            category: Category::Redundancy,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::If];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::If(if_stmt) = node else {
            return;
        };

        match &if_stmt.body {
            IfBody::Statement(body) => {
                let has_branches = !body.else_if_clauses.is_empty() || body.else_clause.is_some();
                if !has_branches {
                    return;
                }

                if !statement_always_terminates(body.statement) {
                    return;
                }

                self.report_statement_form(ctx, if_stmt.r#if.span(), body.statement, body);
            }
            IfBody::ColonDelimited(body) => {
                let has_branches = !body.else_if_clauses.is_empty() || body.else_clause.is_some();
                if !has_branches {
                    return;
                }

                if !statements_always_terminate(body.statements.as_slice()) {
                    return;
                }

                let trailing_kw_span = body
                    .else_clause
                    .as_ref()
                    .map(|c| c.r#else.span())
                    .or_else(|| body.else_if_clauses.iter().next().map(|c| c.elseif.span()))
                    .expect("has_branches implies at least one elseif or else");

                ctx.collector.report(self.make_issue(if_stmt.r#if.span(), trailing_kw_span));
            }
        }
    }
}

impl NoRedundantElseRule {
    fn make_issue(&self, if_kw_span: Span, trailing_kw_span: Span) -> Issue {
        Issue::new(self.cfg.level(), "The `if` branch always terminates; the trailing branches can be extracted.")
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(trailing_kw_span)
                    .with_message("This branch is redundant because the `if` branch always exits"),
            )
            .with_annotation(
                Annotation::secondary(if_kw_span).with_message("The matching `if` branch unconditionally terminates"),
            )
            .with_note("Extracting the trailing branches reduces nesting without changing behavior.")
            .with_help("Hoist the `else` body after the `if`, and convert any `elseif` into a fresh `if` statement.")
    }

    fn report_statement_form<'ast, 'arena>(
        &self,
        ctx: &mut LintContext<'_, 'arena>,
        if_kw_span: Span,
        if_stmt_body: &'ast Statement<'arena>,
        body: &'ast IfStatementBody<'arena>,
    ) {
        let trailing_kw_span = body
            .else_clause
            .as_ref()
            .map(|c| c.r#else.span())
            .or_else(|| body.else_if_clauses.iter().next().map(|c| c.elseif.span()))
            .expect("caller checked at least one trailing branch");

        let issue = self.make_issue(if_kw_span, trailing_kw_span);

        if !matches!(if_stmt_body, Statement::Block(_)) {
            ctx.collector.report(issue);
            return;
        }

        let if_body_end = if_stmt_body.end_offset();

        if let Some(first_elseif) = body.else_if_clauses.iter().next() {
            ctx.collector.propose(issue, |edits| {
                edits.push(TextEdit::replace(if_body_end..first_elseif.elseif.end_offset(), "\n\nif"));
            });

            return;
        }

        let else_clause = body.else_clause.as_ref().expect("caller checked");
        let else_stmt = else_clause.statement;

        ctx.collector.propose(issue, |edits| match else_stmt {
            Statement::Block(else_block) => {
                if else_block.statements.is_empty() {
                    edits.push(TextEdit::delete(if_body_end..else_block.right_brace.end_offset()));
                } else {
                    let last_stmt_end = else_block.statements.nodes.last().unwrap().end_offset();

                    edits.push(TextEdit::replace(if_body_end..else_block.left_brace.end_offset(), "\n"));
                    edits.push(TextEdit::delete(last_stmt_end..else_block.right_brace.end_offset()));
                }
            }
            other => {
                edits.push(TextEdit::replace(if_body_end..other.start_offset(), "\n\n"));
            }
        });
    }
}

fn statement_always_terminates(stmt: &Statement<'_>) -> bool {
    match stmt {
        Statement::Return(_) | Statement::Continue(_) | Statement::Break(_) => true,
        Statement::Expression(expr_stmt) => expression_always_terminates(expr_stmt.expression),
        Statement::Block(Block { statements, .. }) => statements_always_terminate(statements.as_slice()),
        _ => false,
    }
}

fn statements_always_terminate(stmts: &[Statement<'_>]) -> bool {
    stmts.iter().rev().find(|s| !matches!(s, Statement::Noop(_))).is_some_and(statement_always_terminates)
}

fn expression_always_terminates(expr: &Expression<'_>) -> bool {
    if let Expression::Parenthesized(paren) = expr {
        return expression_always_terminates(paren.expression);
    }

    if matches!(expr, Expression::Throw(_) | Expression::Construct(Construct::Exit(_) | Construct::Die(_))) {
        return true;
    }

    if let Expression::UnaryPostfix(UnaryPostfix { operand, .. })
    | Expression::UnaryPrefix(UnaryPrefix { operand, .. }) = expr
    {
        return expression_always_terminates(operand);
    }

    if let Expression::Assignment(assignment) = expr {
        return expression_always_terminates(assignment.rhs);
    }

    if let Expression::Call(call) = expr {
        return call.get_argument_list().arguments.iter().map(|arg| arg.value()).any(expression_always_terminates);
    }

    if let Expression::Conditional(conditional) = expr {
        return conditional.then.is_some_and(expression_always_terminates)
            && expression_always_terminates(conditional.r#else);
    }

    if let Expression::Match(r#match) = expr {
        return expression_always_terminates(r#match.expression)
            || r#match.arms.iter().map(|arm| arm.expression()).all(expression_always_terminates);
    }

    false
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoRedundantElseRule;
    use crate::test_lint_failure;
    use crate::test_lint_fix;
    use crate::test_lint_success;

    test_lint_failure! {
        name = if_returns_then_else_block,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            function process($user) {
                if (!$user->isVerified()) {
                    return;
                } else {
                    $user->login();
                }
            }
        "#}
    }

    test_lint_failure! {
        name = if_throws_then_else_block,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            function process($x) {
                if ($x === null) {
                    throw new InvalidArgumentException();
                } else {
                    do_something($x);
                }
            }
        "#}
    }

    test_lint_failure! {
        name = if_exit_then_else_block,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            if ($fatal) {
                exit(1);
            } else {
                continue_running();
            }
        "#}
    }

    test_lint_failure! {
        name = if_die_then_else_block,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            if ($fatal) {
                die('boom');
            } else {
                keep_going();
            }
        "#}
    }

    test_lint_failure! {
        name = if_continue_in_loop,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            foreach ($items as $i) {
                if ($i->skip) {
                    continue;
                } else {
                    handle($i);
                }
            }
        "#}
    }

    test_lint_failure! {
        name = if_break_in_loop,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            while ($cond) {
                if ($done) {
                    break;
                } else {
                    step();
                }
            }
        "#}
    }

    test_lint_failure! {
        name = if_throw_in_parens_then_else,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            if ($x === null) {
                (throw new InvalidArgumentException());
            } else {
                use_value($x);
            }
        "#}
    }

    test_lint_failure! {
        name = if_ternary_both_branches_throw_then_else,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            if ($x === null) {
                $useA ? throw new ErrorA() : throw new ErrorB();
            } else {
                use_value($x);
            }
        "#}
    }

    test_lint_failure! {
        name = if_match_all_arms_terminate_then_else,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            if ($x === null) {
                match ($code) {
                    1 => throw new ErrorA(),
                    2 => throw new ErrorB(),
                    default => exit(1),
                };
            } else {
                use_value($x);
            }
        "#}
    }

    test_lint_failure! {
        name = if_nested_ternary_in_match_arm_terminates,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            if ($bad) {
                match ($code) {
                    1 => throw new ErrorA(),
                    default => $useDie ? die('x') : exit(1),
                };
            } else {
                keep_going();
            }
        "#}
    }

    test_lint_success! {
        name = if_ternary_one_branch_continues_is_kept,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            if ($x === null) {
                $useA ? throw new ErrorA() : log_warning();
            } else {
                use_value($x);
            }
        "#}
    }

    test_lint_success! {
        name = if_match_arm_does_not_terminate_is_kept,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            if ($bad) {
                match ($code) {
                    1 => throw new ErrorA(),
                    default => log_warning(),
                };
            } else {
                keep_going();
            }
        "#}
    }

    test_lint_success! {
        name = if_short_ternary_does_not_always_terminate,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            if ($bad) {
                $value ?: throw new InvalidArgumentException();
            } else {
                keep_going();
            }
        "#}
    }

    test_lint_failure! {
        name = if_assignment_to_throw_then_else,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            if ($bad) {
                $error = throw new RuntimeException();
            } else {
                keep_going();
            }
        "#}
    }

    test_lint_failure! {
        name = if_property_assignment_to_throw_then_else,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            if ($bad) {
                $this->error = throw new RuntimeException();
            } else {
                keep_going();
            }
        "#}
    }

    test_lint_failure! {
        name = if_array_assignment_to_throw_then_else,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            if ($bad) {
                $errors[$key] = throw new RuntimeException();
            } else {
                keep_going();
            }
        "#}
    }

    test_lint_failure! {
        name = if_compound_assignment_to_throw_then_else,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            if ($bad) {
                $count += throw new RuntimeException();
            } else {
                keep_going();
            }
        "#}
    }

    test_lint_failure! {
        name = if_unary_not_over_throw_then_else,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            if ($bad) {
                !(throw new RuntimeException());
            } else {
                keep_going();
            }
        "#}
    }

    test_lint_failure! {
        name = if_unary_negate_over_die_then_else,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            if ($bad) {
                -(die('boom'));
            } else {
                keep_going();
            }
        "#}
    }

    test_lint_failure! {
        name = if_match_scrutinee_throws_then_else,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            if ($bad) {
                match (throw new RuntimeException()) {
                    1 => 'a',
                    default => 'b',
                };
            } else {
                keep_going();
            }
        "#}
    }

    test_lint_failure! {
        name = if_function_call_with_terminating_arg_then_else,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            if ($bad) {
                handle($ctx, throw new RuntimeException(), $extra);
            } else {
                keep_going();
            }
        "#}
    }

    test_lint_success! {
        name = if_function_call_without_terminating_arg_is_kept,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            if ($bad) {
                handle($ctx, $error, $extra);
            } else {
                keep_going();
            }
        "#}
    }

    test_lint_success! {
        name = if_assignment_to_non_terminating_rhs_is_kept,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            if ($bad) {
                $error = build_error();
            } else {
                keep_going();
            }
        "#}
    }

    test_lint_success! {
        name = if_without_else_is_fine,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            function process($user) {
                if (!$user->isVerified()) {
                    return;
                }

                $user->login();
            }
        "#}
    }

    test_lint_success! {
        name = if_does_not_terminate_so_else_kept,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            if ($a) {
                do_a();
            } else {
                do_b();
            }
        "#}
    }

    test_lint_failure! {
        name = if_with_elseif_chain_is_flagged,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            if ($a) {
                return;
            } elseif ($b) {
                handle_b();
            } else {
                handle_other();
            }
        "#}
    }

    test_lint_fix! {
        name = fix_promotes_elseif_to_if,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            if ($a) {
                return;
            } elseif ($b) {
                handle_b();
            } else {
                handle_other();
            }
        "#},
        fixed = indoc! {r#"
            <?php

            if ($a) {
                return;
            }

            if ($b) {
                handle_b();
            } else {
                handle_other();
            }
        "#}
    }

    test_lint_success! {
        name = if_body_terminates_partially_only,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            if ($a) {
                if ($b) {
                    return;
                }
                fallthrough();
            } else {
                handle();
            }
        "#}
    }

    test_lint_fix! {
        name = fix_block_else,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            function process($user) {
                if (!$user->isVerified()) {
                    return;
                } else {
                    $user->login();
                }
            }
        "#},
        fixed = indoc! {r#"
            <?php

            function process($user) {
                if (!$user->isVerified()) {
                    return;
                }

                    $user->login();
            }
        "#}
    }

    test_lint_fix! {
        name = fix_empty_else_block_is_dropped,
        rule = NoRedundantElseRule,
        code = indoc! {r#"
            <?php

            if ($x) {
                return;
            } else {}
        "#},
        fixed = indoc! {r#"
            <?php

            if ($x) {
                return;
            }
        "#}
    }
}
