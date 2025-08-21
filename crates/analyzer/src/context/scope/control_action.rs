use ahash::HashSet;

use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::artifacts::AnalysisArtifacts;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug)]
#[repr(u8)]
pub enum ControlAction {
    End,
    Break,
    BreakImmediateLoop,
    Continue,
    LeaveSwitch,
    None,
    Return,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug)]
#[repr(u8)]
pub enum BreakType {
    Switch,
    Loop,
}

impl ControlAction {
    pub fn from_statements(
        statements: Vec<&Statement>,
        break_type: Vec<BreakType>,
        artifacts: Option<&AnalysisArtifacts>,
        return_is_exit: bool,
    ) -> HashSet<ControlAction> {
        let statements_len = statements.len();
        if 0 == statements_len {
            return HashSet::from_iter([ControlAction::None]);
        }

        if 1 == statements_len
            && let Some(Statement::Block(block)) = statements.first()
        {
            return ControlAction::from_statements(
                block.statements.iter().collect::<Vec<_>>(),
                break_type,
                artifacts,
                return_is_exit,
            );
        }

        let mut control_actions = HashSet::default();
        'statements_loop: for statement in statements {
            match statement {
                _ if is_return_or_throw_or_exit(statement) => {
                    if let Statement::Return(return_statement) = statement
                        && !return_is_exit
                    {
                        if let (Some(artifacts), Some(expression)) = (artifacts, &return_statement.value)
                            && let Some(return_type) = artifacts.get_expression_type(&expression)
                            && return_type.is_never()
                        {
                            control_actions.insert(ControlAction::End);

                            return control_actions;
                        }

                        control_actions.insert(ControlAction::Return);

                        return control_actions;
                    }

                    control_actions.insert(ControlAction::End);

                    return control_actions;
                }
                _ if statement.is_loop() => {
                    let (inner_statements, condition) = match statement {
                        Statement::For(for_loop) => (
                            match &for_loop.body {
                                ForBody::Statement(statement) => vec![*statement],
                                ForBody::ColonDelimited(for_colon_delimited_body) => {
                                    for_colon_delimited_body.statements.iter().collect::<Vec<_>>()
                                }
                            },
                            None,
                        ),
                        Statement::Foreach(foreach_loop) => (
                            match &foreach_loop.body {
                                ForeachBody::Statement(statement) => vec![*statement],
                                ForeachBody::ColonDelimited(foreach_colon_delimited_body) => {
                                    foreach_colon_delimited_body.statements.iter().collect::<Vec<_>>()
                                }
                            },
                            None,
                        ),
                        Statement::DoWhile(do_while) => (vec![do_while.statement], Some(do_while.condition)),
                        Statement::While(while_loop) => (
                            match &while_loop.body {
                                WhileBody::Statement(statement) => vec![*statement],
                                WhileBody::ColonDelimited(while_colon_delimited_body) => {
                                    while_colon_delimited_body.statements.iter().collect::<Vec<_>>()
                                }
                            },
                            Some(while_loop.condition),
                        ),
                        _ => unreachable!(),
                    };

                    let mut loop_break_types = break_type.clone();
                    loop_break_types.push(BreakType::Loop);

                    let mut loop_actions =
                        ControlAction::from_statements(inner_statements, loop_break_types, artifacts, return_is_exit);
                    loop_actions.retain(|action| !action.is_none());

                    control_actions.extend(loop_actions);

                    if !control_actions.iter().any(|a| a.is_break_immediate_loop() || a.is_exit_path()) {
                        if let (Some(artifacts), Some(condition)) = (artifacts, condition)
                            && let Some(condition_type) = artifacts.get_expression_type(condition)
                            && condition_type.is_always_truthy()
                        {
                            return control_actions;
                        }

                        if let Some(artifacts) = artifacts
                            && let Statement::For(for_loop) = statement
                        {
                            let mut is_infinite_loop = true;
                            for condition in for_loop.conditions.iter() {
                                let Some(condition_type) = artifacts.get_expression_type(condition) else {
                                    is_infinite_loop = false;

                                    continue;
                                };

                                if !condition_type.is_always_truthy() {
                                    is_infinite_loop = false;
                                }
                            }

                            if is_infinite_loop {
                                return control_actions;
                            }
                        }
                    }

                    control_actions.retain(|a| !a.is_break_immediate_loop());
                }
                Statement::Block(block) => {
                    let mut block_actions = ControlAction::from_statements(
                        block.statements.iter().collect::<Vec<_>>(),
                        break_type.clone(),
                        artifacts,
                        return_is_exit,
                    );

                    if !block_actions.contains(&ControlAction::None) {
                        control_actions.extend(block_actions);
                        control_actions.retain(|action| *action != ControlAction::None);

                        return control_actions;
                    }

                    block_actions.retain(|action| *action != ControlAction::None);
                    control_actions.extend(block_actions);
                }
                Statement::Expression(statement_expression) => {
                    if let Some(artifacts) = artifacts
                        && let Some(expression_type) = artifacts.get_expression_type(&statement_expression.expression)
                        && expression_type.is_never()
                    {
                        control_actions.insert(ControlAction::End);

                        return control_actions;
                    }
                }
                Statement::Continue(continue_statement) => {
                    let break_type_len = break_type.len();
                    if break_type_len != 0 {
                        let maybe_count = match continue_statement.level.as_ref() {
                            None => Some(1),
                            Some(Expression::Literal(Literal::Integer(lit_int))) => {
                                lit_int.value.as_ref().map(|&value| value as usize)
                            }
                            _ => None,
                        };

                        if let Some(count) = maybe_count
                            && break_type_len >= count
                        {
                            if matches!(break_type.get(break_type_len - count), Some(BreakType::Switch)) {
                                control_actions.insert(ControlAction::LeaveSwitch);
                            }

                            return control_actions;
                        }
                    }

                    control_actions.insert(ControlAction::Continue);

                    return control_actions;
                }
                Statement::Break(continue_statement) => {
                    let break_type_len = break_type.len();
                    if break_type_len != 0 {
                        let maybe_count = match continue_statement.level.as_ref() {
                            None => Some(1),
                            Some(Expression::Literal(Literal::Integer(lit_int))) => {
                                lit_int.value.as_ref().map(|&value| value as usize)
                            }
                            _ => None,
                        };

                        if let Some(count) = maybe_count
                            && break_type_len >= count
                        {
                            if let Some(b) = break_type.get(break_type_len - count) {
                                if b.is_switch() {
                                    control_actions.insert(ControlAction::LeaveSwitch);
                                } else {
                                    control_actions.insert(ControlAction::BreakImmediateLoop);
                                }
                            }

                            return control_actions;
                        }
                    }

                    control_actions.insert(ControlAction::Break);

                    return control_actions;
                }
                Statement::Switch(switch) => {
                    let mut has_ended = false;
                    let mut has_non_breaking_default = false;
                    let mut has_default_terminator = false;
                    let mut all_case_actions = vec![];

                    for case in switch.body.cases().iter().rev() {
                        let mut case_break_type = break_type.clone();
                        case_break_type.push(BreakType::Switch);

                        let case_actions = ControlAction::from_statements(
                            case.statements().iter().collect(),
                            case_break_type,
                            artifacts,
                            return_is_exit,
                        );

                        if case_actions.iter().any(|c| c.is_break_or_continue() || c.is_leave_switch()) {
                            continue 'statements_loop;
                        }

                        if case.is_default() {
                            has_non_breaking_default = true;
                        }

                        let case_does_end = case_actions.iter().any(|c| c.is_exit_path());
                        if case_does_end {
                            has_ended = true;
                        }

                        all_case_actions.extend(case_actions);

                        if !case_does_end && !has_ended {
                            continue 'statements_loop;
                        }

                        if has_non_breaking_default && case_does_end {
                            has_default_terminator = true;
                        }
                    }

                    all_case_actions.retain(|c| !c.is_none());

                    control_actions.extend(all_case_actions);

                    if has_default_terminator
                        || artifacts.is_some_and(|artifacts| {
                            artifacts.fully_matched_switch_offsets.contains(&switch.span().start.offset)
                        })
                    {
                        return control_actions;
                    }
                }
                Statement::If(if_statement) => {
                    let if_statement_actions = ControlAction::from_statements(
                        if_statement.body.statements().iter().collect(),
                        break_type.clone(),
                        artifacts,
                        return_is_exit,
                    );

                    let mut all_leave = if_statement_actions.iter().all(|c| !c.is_none());

                    let mut has_else_actions = false;
                    let else_statement_actions = match if_statement.body.else_statements() {
                        Some(statements) => {
                            has_else_actions = true;

                            let else_statement_actions = ControlAction::from_statements(
                                statements.iter().collect(),
                                break_type.clone(),
                                artifacts,
                                return_is_exit,
                            );

                            all_leave = all_leave && else_statement_actions.iter().all(|c| !c.is_none());

                            else_statement_actions
                        }
                        None => HashSet::default(),
                    };

                    all_leave = all_leave && has_else_actions && else_statement_actions.iter().all(|c| !c.is_none());

                    let mut all_elseif_actions = vec![];
                    for else_if_statements in if_statement.body.else_if_statements() {
                        let elseif_control_actions = ControlAction::from_statements(
                            else_if_statements.iter().collect(),
                            break_type.clone(),
                            artifacts,
                            return_is_exit,
                        );

                        all_leave = all_leave && elseif_control_actions.iter().all(|c| !c.is_none());

                        all_elseif_actions.extend(elseif_control_actions);
                    }

                    control_actions.extend(if_statement_actions);
                    control_actions.extend(else_statement_actions);
                    control_actions.extend(all_elseif_actions);

                    if all_leave {
                        return control_actions;
                    }

                    control_actions.retain(|action| *action != ControlAction::None);
                }
                Statement::Try(try_catch) => {
                    let try_statement_actions = ControlAction::from_statements(
                        try_catch.block.statements.iter().collect::<Vec<_>>(),
                        break_type.clone(),
                        artifacts,
                        return_is_exit,
                    );

                    let try_leaves = try_statement_actions.iter().all(|c| !c.is_none());

                    let mut all_catch_actions = vec![];
                    if !try_catch.catch_clauses.is_empty() {
                        let mut all_catches_leave = try_leaves;
                        for catch in try_catch.catch_clauses.iter() {
                            let catch_actions = ControlAction::from_statements(
                                catch.block.statements.iter().collect::<Vec<_>>(),
                                break_type.clone(),
                                artifacts,
                                return_is_exit,
                            );

                            all_catches_leave = all_catches_leave && catch_actions.iter().all(|c| !c.is_none());

                            if !all_catches_leave {
                                control_actions.extend(catch_actions);
                            } else {
                                all_catch_actions.extend(catch_actions);
                            }
                        }

                        if all_catches_leave && !try_statement_actions.iter().all(|c| c.is_none()) {
                            control_actions.extend(try_statement_actions);
                            control_actions.extend(all_catch_actions);

                            return control_actions;
                        }
                    } else if try_leaves {
                        control_actions.extend(try_statement_actions);

                        return control_actions;
                    }

                    if let Some(finally_clause) = try_catch.finally_clause.as_ref()
                        && !finally_clause.block.statements.is_empty()
                    {
                        let finally_statement_actions = ControlAction::from_statements(
                            finally_clause.block.statements.iter().collect::<Vec<_>>(),
                            break_type.clone(),
                            artifacts,
                            return_is_exit,
                        );

                        if !finally_statement_actions.iter().any(|c| c.is_none()) {
                            control_actions.retain(|c| !c.is_none());
                            control_actions.extend(finally_statement_actions);

                            return control_actions;
                        }
                    }

                    control_actions.extend(try_statement_actions);
                    control_actions.retain(|c| !c.is_none());
                }
                _ => {}
            };
        }

        control_actions.insert(ControlAction::None);

        control_actions
    }

    #[inline]
    pub const fn is_none(&self) -> bool {
        matches!(self, ControlAction::None)
    }

    #[inline]
    pub const fn is_exit_path(&self) -> bool {
        matches!(self, ControlAction::End | ControlAction::Return)
    }

    #[inline]
    pub const fn is_break_immediate_loop(&self) -> bool {
        matches!(self, ControlAction::BreakImmediateLoop)
    }

    #[inline]
    pub const fn is_leave_switch(&self) -> bool {
        matches!(self, ControlAction::BreakImmediateLoop)
    }

    #[inline]
    pub const fn is_break_or_continue(&self) -> bool {
        matches!(self, ControlAction::Break | ControlAction::Continue)
    }
}

impl BreakType {
    #[inline]
    pub const fn is_switch(&self) -> bool {
        matches!(self, BreakType::Switch)
    }
}

#[inline]
const fn is_return_or_throw_or_exit(statement: &Statement) -> bool {
    match statement {
        Statement::Return(_) => true,
        Statement::Expression(expression) => {
            matches!(
                *expression.expression,
                Expression::Throw(_) | Expression::Construct(Construct::Die(_) | Construct::Exit(_))
            )
        }
        _ => false,
    }
}
