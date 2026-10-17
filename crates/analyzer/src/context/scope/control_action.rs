use mago_span::HasSpan;
use mago_syntax::cst::Access;
use mago_syntax::cst::ArrayAccess;
use mago_syntax::cst::ArrowFunction;
use mago_syntax::cst::Binary;
use mago_syntax::cst::BinaryOperator;
use mago_syntax::cst::Call;
use mago_syntax::cst::Clone;
use mago_syntax::cst::Closure;
use mago_syntax::cst::Conditional;
use mago_syntax::cst::Construct;
use mago_syntax::cst::Expression;
use mago_syntax::cst::ForBody;
use mago_syntax::cst::ForeachBody;
use mago_syntax::cst::Instantiation;
use mago_syntax::cst::Literal;
use mago_syntax::cst::Statement;
use mago_syntax::cst::Switch;
use mago_syntax::cst::Throw;
use mago_syntax::cst::WhileBody;
use mago_syntax::utils::unparenthesized_expression;
use mago_syntax::walker::Walker;

use crate::artifacts::AnalysisArtifacts;
use crate::expression::binary::utils::is_always_identical_to;
use mago_codex::ttype::union::TUnion;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug)]
#[repr(u8)]
pub enum ControlAction {
    End = 0b000_0001,
    Break = 0b000_0010,
    BreakImmediateLoop = 0b000_0100,
    Continue = 0b000_1000,
    LeaveSwitch = 0b001_0000,
    None = 0b010_0000,
    Return = 0b100_0000,
    /// A numbered break or continue that exits the switch whose case is being analyzed.
    BreakTargetSwitch = 0b1000_0000,
}

/// A compact bitfield set for ControlAction values.
/// Uses a single u8 instead of HashSet for better performance.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ControlActionSet(u8);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug)]
#[repr(u8)]
pub enum BreakType {
    Switch = 0b01,
    Loop = 0b10,
}

impl ControlAction {
    pub const ALL: [ControlAction; 8] = [
        ControlAction::End,
        ControlAction::Break,
        ControlAction::BreakImmediateLoop,
        ControlAction::Continue,
        ControlAction::LeaveSwitch,
        ControlAction::None,
        ControlAction::Return,
        ControlAction::BreakTargetSwitch,
    ];

    pub fn from_statements(
        statements: Vec<&Statement>,
        break_type: Vec<BreakType>,
        artifacts: Option<&AnalysisArtifacts>,
        return_is_exit: bool,
    ) -> ControlActionSet {
        Self::from_statements_with_switch_target(statements, break_type, artifacts, return_is_exit, None)
    }

    /// Analyzes a case body while preserving the switch that owns the case as a transfer target.
    pub(crate) fn from_switch_case_statements(
        statements: Vec<&Statement>,
        break_type: Vec<BreakType>,
        artifacts: Option<&AnalysisArtifacts>,
        return_is_exit: bool,
    ) -> ControlActionSet {
        let target_switch_index = break_type.len().checked_sub(1);

        Self::from_statements_with_switch_target(statements, break_type, artifacts, return_is_exit, target_switch_index)
    }

    fn from_statements_with_switch_target(
        statements: Vec<&Statement>,
        break_type: Vec<BreakType>,
        artifacts: Option<&AnalysisArtifacts>,
        return_is_exit: bool,
        // Keeps transfers aimed at the enclosing case's switch from being consumed by nested switches.
        target_switch_index: Option<usize>,
    ) -> ControlActionSet {
        let statements_len = statements.len();
        if 0 == statements_len {
            return ControlActionSet::from_single(ControlAction::None);
        }

        if 1 == statements_len
            && let Some(Statement::Block(block)) = statements.first()
        {
            return ControlAction::from_statements_with_switch_target(
                block.statements.iter().collect::<Vec<_>>(),
                break_type,
                artifacts,
                return_is_exit,
                target_switch_index,
            );
        }

        let mut control_actions = ControlActionSet::new();
        for statement in statements {
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
                    let has_enclosing_loop_break = control_actions.contains(ControlAction::BreakImmediateLoop);

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
                        #[allow(clippy::unreachable)]
                        _ => unreachable!(),
                    };

                    let mut loop_break_types = break_type.clone();
                    loop_break_types.push(BreakType::Loop);

                    let mut loop_actions = ControlAction::from_statements_with_switch_target(
                        inner_statements,
                        loop_break_types,
                        artifacts,
                        return_is_exit,
                        target_switch_index,
                    );
                    loop_actions.retain(|action| !action.is_none());

                    control_actions.extend(loop_actions);

                    if !control_actions
                        .iter()
                        .any(|a| matches!(a, ControlAction::Break) || a.is_break_immediate_loop() || a.is_exit_path())
                    {
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
                            for condition in &for_loop.conditions {
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

                    if has_enclosing_loop_break {
                        control_actions.insert(ControlAction::BreakImmediateLoop);
                    }
                }
                Statement::Block(block) => {
                    let mut block_actions = ControlAction::from_statements_with_switch_target(
                        block.statements.iter().collect::<Vec<_>>(),
                        break_type.clone(),
                        artifacts,
                        return_is_exit,
                        target_switch_index,
                    );

                    if !block_actions.contains(ControlAction::None) {
                        control_actions.extend(block_actions);
                        control_actions.retain(|action| action != ControlAction::None);

                        return control_actions;
                    }

                    block_actions.retain(|action| action != ControlAction::None);
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
                            let target = break_type_len - count;
                            // A continue targeting the analyzed switch exits the case instead of continuing locally.
                            match break_type.get(target) {
                                Some(BreakType::Switch) if target_switch_index == Some(target) => {
                                    control_actions.insert(ControlAction::BreakTargetSwitch);
                                }
                                Some(BreakType::Switch)
                                    if target_switch_index.is_some_and(|switch_index| target < switch_index) =>
                                {
                                    control_actions.insert(ControlAction::Break);
                                }
                                Some(BreakType::Switch) => {
                                    control_actions.insert(ControlAction::LeaveSwitch);
                                }
                                _ if count > 1 => {
                                    control_actions.insert(ControlAction::Continue);
                                }
                                _ => {}
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
                            if count > 1 {
                                let target = break_type_len - count;
                                // Preserve transfers to the analyzed switch so its caller can resolve the exit.
                                match break_type.get(target) {
                                    Some(BreakType::Switch) if target_switch_index == Some(target) => {
                                        control_actions.insert(ControlAction::BreakTargetSwitch)
                                    }
                                    Some(BreakType::Switch)
                                        if target_switch_index.is_some_and(|switch_index| target < switch_index) =>
                                    {
                                        control_actions.insert(ControlAction::Break)
                                    }
                                    Some(BreakType::Switch) => control_actions.insert(ControlAction::LeaveSwitch),
                                    Some(BreakType::Loop) => {
                                        let target_is_immediate_loop =
                                            !break_type[target + 1..].contains(&BreakType::Loop);

                                        control_actions.insert(if target_is_immediate_loop {
                                            ControlAction::BreakImmediateLoop
                                        } else {
                                            ControlAction::Break
                                        });
                                    }
                                    None => control_actions.insert(ControlAction::Break),
                                }
                            } else if let Some(b) = break_type.get(break_type_len - count) {
                                if b.is_switch() {
                                    let target = break_type_len - count;
                                    if target_switch_index == Some(target) {
                                        control_actions.insert(ControlAction::BreakTargetSwitch);
                                    } else if target_switch_index.is_some_and(|switch_index| target < switch_index) {
                                        control_actions.insert(ControlAction::Break);
                                    } else {
                                        control_actions.insert(ControlAction::LeaveSwitch);
                                    }
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
                    let mut all_case_actions = ControlActionSet::new();
                    let cases = switch.body.cases();
                    let has_default_case = cases.iter().any(|case| case.is_default());
                    let mut fallthrough_actions = ControlActionSet::from_single(ControlAction::None);
                    let is_local_switch_target = target_switch_index.is_none();
                    let switch_target_index = target_switch_index.or(Some(break_type.len()));

                    // Resolve cases backwards so each case's fallthrough inherits the next case's possible actions.
                    for case in cases.iter().rev() {
                        let mut case_break_type = break_type.clone();
                        case_break_type.push(BreakType::Switch);

                        let case_actions = ControlAction::from_statements_with_switch_target(
                            case.statements().iter().collect(),
                            case_break_type,
                            artifacts,
                            return_is_exit,
                            switch_target_index,
                        );

                        let mut resolved_case_actions = ControlActionSet::new();
                        for action in case_actions.iter() {
                            match action {
                                ControlAction::None => resolved_case_actions.extend(fallthrough_actions),
                                ControlAction::LeaveSwitch => resolved_case_actions.insert(ControlAction::None),
                                ControlAction::BreakTargetSwitch if is_local_switch_target => {
                                    resolved_case_actions.insert(ControlAction::None)
                                }
                                _ => resolved_case_actions.insert(action),
                            }
                        }

                        all_case_actions.extend(resolved_case_actions);
                        fallthrough_actions = resolved_case_actions;
                    }

                    if !has_default_case
                        && !artifacts.is_some_and(|artifacts| {
                            artifacts.fully_matched_switch_offsets.contains(&switch.span().start.offset)
                        })
                    {
                        all_case_actions.insert(ControlAction::None);
                    }

                    let can_complete_normally = all_case_actions.contains(ControlAction::None);
                    all_case_actions.retain(|action| !action.is_none());
                    control_actions.extend(all_case_actions);

                    if !can_complete_normally {
                        return control_actions;
                    }
                }
                Statement::If(if_statement) => {
                    let if_statement_actions = ControlAction::from_statements_with_switch_target(
                        if_statement.body.statements().iter().collect(),
                        break_type.clone(),
                        artifacts,
                        return_is_exit,
                        target_switch_index,
                    );

                    let mut all_leave = !if_statement_actions.contains(ControlAction::None);

                    let mut has_else_actions = false;
                    let else_statement_actions = match if_statement.body.else_statements() {
                        Some(statements) => {
                            has_else_actions = true;

                            let else_statement_actions = ControlAction::from_statements_with_switch_target(
                                statements.iter().collect(),
                                break_type.clone(),
                                artifacts,
                                return_is_exit,
                                target_switch_index,
                            );

                            all_leave = all_leave && !else_statement_actions.contains(ControlAction::None);

                            else_statement_actions
                        }
                        None => ControlActionSet::new(),
                    };

                    all_leave = all_leave && has_else_actions && !else_statement_actions.contains(ControlAction::None);

                    let mut all_elseif_actions = ControlActionSet::new();
                    for else_if_statements in if_statement.body.else_if_statements() {
                        let elseif_control_actions = ControlAction::from_statements_with_switch_target(
                            else_if_statements.iter().collect(),
                            break_type.clone(),
                            artifacts,
                            return_is_exit,
                            target_switch_index,
                        );

                        all_leave = all_leave && !elseif_control_actions.contains(ControlAction::None);

                        all_elseif_actions.extend(elseif_control_actions);
                    }

                    control_actions.extend(if_statement_actions);
                    control_actions.extend(else_statement_actions);
                    control_actions.extend(all_elseif_actions);

                    if all_leave {
                        return control_actions;
                    }

                    control_actions.retain(|action| action != ControlAction::None);
                }
                Statement::Try(try_catch) => {
                    let try_statement_actions = ControlAction::from_statements_with_switch_target(
                        try_catch.block.statements.iter().collect::<Vec<_>>(),
                        break_type.clone(),
                        artifacts,
                        return_is_exit,
                        target_switch_index,
                    );

                    let mut try_catch_actions = try_statement_actions;
                    // Catch paths are possible only if the try body can throw; dead catches must not affect flow.
                    if try_block_may_throw(try_catch.block.statements.as_slice(), artifacts) {
                        for catch in &try_catch.catch_clauses {
                            try_catch_actions.extend(ControlAction::from_statements_with_switch_target(
                                catch.block.statements.iter().collect::<Vec<_>>(),
                                break_type.clone(),
                                artifacts,
                                return_is_exit,
                                target_switch_index,
                            ));
                        }
                    }

                    if let Some(finally_clause) = try_catch.finally_clause.as_ref()
                        && !finally_clause.block.statements.is_empty()
                    {
                        let mut finally_statement_actions = ControlAction::from_statements_with_switch_target(
                            finally_clause.block.statements.iter().collect::<Vec<_>>(),
                            break_type.clone(),
                            artifacts,
                            return_is_exit,
                            target_switch_index,
                        );

                        let finally_can_complete_normally = finally_statement_actions.contains(ControlAction::None);
                        let try_catch_can_complete_normally = try_catch_actions.contains(ControlAction::None);

                        finally_statement_actions.retain(|action| !action.is_none());
                        control_actions.extend(finally_statement_actions);

                        // An abrupt finally replaces pending try/catch exits; a normally completing finally preserves them.
                        if finally_can_complete_normally {
                            control_actions.extend(try_catch_actions);
                        }

                        if !finally_can_complete_normally || !try_catch_can_complete_normally {
                            return control_actions;
                        }

                        control_actions.retain(|action| !action.is_none());
                        continue;
                    }

                    let try_catch_can_complete_normally = try_catch_actions.contains(ControlAction::None);
                    control_actions.extend(try_catch_actions);
                    control_actions.retain(|c| !c.is_none());

                    if !try_catch_can_complete_normally {
                        return control_actions;
                    }
                }
                _ => {}
            }
        }

        control_actions.insert(ControlAction::None);

        control_actions
    }

    #[inline]
    pub const fn is_none(self) -> bool {
        matches!(self, ControlAction::None)
    }

    #[inline]
    pub const fn is_exit_path(self) -> bool {
        matches!(self, ControlAction::End | ControlAction::Return)
    }

    #[inline]
    pub const fn is_break_immediate_loop(self) -> bool {
        matches!(self, ControlAction::BreakImmediateLoop)
    }

    #[inline]
    pub const fn is_leave_switch(self) -> bool {
        matches!(self, ControlAction::LeaveSwitch)
    }

    #[inline]
    pub const fn is_break_or_continue(self) -> bool {
        matches!(self, ControlAction::Break | ControlAction::Continue)
    }
}

// Conservatively skip catch-flow analysis for simple non-throwing statements; treat other statements as throwable.
pub(crate) fn try_block_may_throw(statements: &[Statement<'_>], artifacts: Option<&AnalysisArtifacts>) -> bool {
    for statement in statements {
        match statement {
            Statement::Noop(_)
            | Statement::Label(_)
            | Statement::Function(_)
            | Statement::Class(_)
            | Statement::Interface(_)
            | Statement::Trait(_)
            | Statement::Enum(_) => {}
            Statement::Block(block) => {
                if try_block_may_throw(block.statements.as_slice(), artifacts) {
                    return true;
                }
            }
            Statement::Expression(expression)
                if matches!(
                    unparenthesized_expression(expression.expression),
                    Expression::Assignment(assignment)
                        if assignment.operator.is_assign()
                            && matches!(unparenthesized_expression(assignment.lhs), Expression::Variable(_))
                            && matches!(
                                unparenthesized_expression(assignment.rhs),
                                Expression::Literal(_)
                                    | Expression::Variable(_)
                                    | Expression::Closure(_)
                                    | Expression::ArrowFunction(_)
                            )
                ) => {}
            Statement::Return(return_statement) if return_statement.value.is_none() => return false,
            Statement::If(if_statement) => {
                if if_statement_may_throw(if_statement, artifacts) {
                    return true;
                }
            }
            Statement::Switch(switch) => {
                if switch_statement_may_throw(switch, artifacts) {
                    return true;
                }
            }
            _ => return true,
        }
    }

    false
}

fn switch_statement_may_throw(switch: &Switch<'_>, artifacts: Option<&AnalysisArtifacts>) -> bool {
    if expression_may_throw(switch.expression, artifacts) {
        return true;
    }

    let switch_type = artifacts.and_then(|artifacts| artifacts.get_rc_expression_type(switch.expression));
    let mut can_evaluate_later_cases = true;
    let cases = switch.body.cases();
    let mut direct_entries = vec![false; cases.len()];
    let mut default_index = None;

    for (index, case) in cases.iter().enumerate() {
        if case.is_default() {
            default_index = Some(index);
            continue;
        }

        if let Some(condition) = case.expression() {
            if can_evaluate_later_cases && expression_may_throw(condition, artifacts) {
                return true;
            }

            let condition_type = artifacts.and_then(|artifacts| artifacts.get_rc_expression_type(condition));
            let matchability = switch_type
                .zip(condition_type)
                .and_then(|(switch_type, condition_type)| switch_case_matchability(switch_type, condition_type));
            direct_entries[index] = can_evaluate_later_cases && matchability != Some(false);

            if can_evaluate_later_cases && matchability == Some(true) {
                can_evaluate_later_cases = false;
            }
        }
    }

    if let Some(default_index) = default_index {
        direct_entries[default_index] = can_evaluate_later_cases;
    }

    let mut can_fall_through = false;
    for (index, case) in cases.iter().enumerate() {
        let direct_entry_possible = direct_entries[index];
        let case_is_reachable = direct_entry_possible || can_fall_through;
        if case_is_reachable && try_block_may_throw(case.statements(), artifacts) {
            return true;
        }

        let actions = ControlAction::from_switch_case_statements(
            case.statements().iter().collect(),
            vec![BreakType::Switch],
            artifacts,
            true,
        );
        can_fall_through = case_is_reachable && actions.contains(ControlAction::None);
    }

    false
}

fn switch_case_matchability(switch_type: &TUnion, condition_type: &TUnion) -> Option<bool> {
    if is_always_identical_to(switch_type, condition_type) {
        return Some(true);
    }

    if let (Some(switch_value), Some(condition_value)) =
        (switch_type.get_single_literal_int_value(), condition_type.get_single_literal_int_value())
    {
        return Some(switch_value == condition_value);
    }

    if let (Some(switch_value), Some(condition_value)) =
        (switch_type.get_single_literal_float_value(), condition_type.get_single_literal_float_value())
    {
        return Some(switch_value == condition_value);
    }

    if switch_type.is_true() {
        if condition_type.is_always_truthy() {
            return Some(true);
        }
        if condition_type.is_always_falsy() {
            return Some(false);
        }
    } else if switch_type.is_false() {
        if condition_type.is_always_falsy() {
            return Some(true);
        }
        if condition_type.is_always_truthy() {
            return Some(false);
        }
    }

    None
}

fn if_statement_may_throw(if_statement: &mago_syntax::cst::If<'_>, artifacts: Option<&AnalysisArtifacts>) -> bool {
    if expression_may_throw(if_statement.condition, artifacts) {
        return true;
    }

    let first_condition_type = artifacts.and_then(|artifacts| artifacts.get_rc_expression_type(if_statement.condition));
    let mut can_reach_next_branch = true;

    if !first_condition_type.is_some_and(|condition_type| condition_type.is_always_falsy())
        && try_block_may_throw(if_statement.body.statements(), artifacts)
    {
        return true;
    }

    if first_condition_type.is_some_and(|condition_type| condition_type.is_always_truthy()) {
        can_reach_next_branch = false;
    }

    for (condition, statements) in if_statement.body.else_if_clauses() {
        if !can_reach_next_branch {
            break;
        }

        if expression_may_throw(condition, artifacts) {
            return true;
        }

        let condition_type = artifacts.and_then(|artifacts| artifacts.get_rc_expression_type(condition));
        if !condition_type.is_some_and(|condition_type| condition_type.is_always_falsy())
            && try_block_may_throw(statements, artifacts)
        {
            return true;
        }

        if condition_type.is_some_and(|condition_type| condition_type.is_always_truthy()) {
            can_reach_next_branch = false;
        }
    }

    can_reach_next_branch
        && if_statement.body.else_statements().is_some_and(|statements| try_block_may_throw(statements, artifacts))
}

struct ThrowingExpressionScanContext<'context> {
    artifacts: Option<&'context AnalysisArtifacts>,
    may_throw: bool,
}

struct ThrowingExpressionScanner;

impl<'ast, 'arena, 'context> Walker<'ast, 'arena, ThrowingExpressionScanContext<'context>>
    for ThrowingExpressionScanner
{
    fn walk_binary(&self, binary: &'ast Binary<'arena>, context: &mut ThrowingExpressionScanContext<'context>) {
        self.walk_expression(binary.lhs, context);

        if context.may_throw {
            return;
        }

        let lhs_type = context.artifacts.and_then(|artifacts| artifacts.get_rc_expression_type(binary.lhs));
        let rhs_is_reachable = match binary.operator {
            BinaryOperator::And(_) | BinaryOperator::LowAnd(_) => {
                !lhs_type.is_some_and(|lhs_type| lhs_type.is_always_falsy())
            }
            BinaryOperator::Or(_) | BinaryOperator::LowOr(_) => {
                !lhs_type.is_some_and(|lhs_type| lhs_type.is_always_truthy())
            }
            BinaryOperator::NullCoalesce(_) => {
                context.artifacts.is_some_and(|artifacts| {
                    crate::callables::static_local_expression_may_be_nullish(
                        binary.lhs,
                        lhs_type.map(|type_union| type_union.as_ref()),
                        artifacts,
                    )
                }) || lhs_type.is_none_or(|lhs_type| {
                    lhs_type.has_nullish() || lhs_type.possibly_undefined() || lhs_type.possibly_undefined_from_try()
                })
            }
            _ => true,
        };

        if rhs_is_reachable {
            self.walk_expression(binary.rhs, context);
        }
    }

    fn walk_conditional(
        &self,
        conditional: &'ast Conditional<'arena>,
        context: &mut ThrowingExpressionScanContext<'context>,
    ) {
        self.walk_expression(conditional.condition, context);

        if context.may_throw {
            return;
        }

        let condition_type =
            context.artifacts.and_then(|artifacts| artifacts.get_rc_expression_type(conditional.condition));
        if !condition_type.is_some_and(|condition_type| condition_type.is_always_falsy())
            && let Some(then) = conditional.then
        {
            self.walk_expression(then, context);
        }

        if !context.may_throw && !condition_type.is_some_and(|condition_type| condition_type.is_always_truthy()) {
            self.walk_expression(conditional.r#else, context);
        }
    }

    fn walk_call(&self, _: &'ast Call<'arena>, context: &mut ThrowingExpressionScanContext<'context>) {
        context.may_throw = true;
    }

    fn walk_access(&self, _: &'ast Access<'arena>, context: &mut ThrowingExpressionScanContext<'context>) {
        context.may_throw = true;
    }

    fn walk_array_access(&self, _: &'ast ArrayAccess<'arena>, context: &mut ThrowingExpressionScanContext<'context>) {
        context.may_throw = true;
    }

    fn walk_clone(&self, _: &'ast Clone<'arena>, context: &mut ThrowingExpressionScanContext<'context>) {
        context.may_throw = true;
    }

    fn walk_instantiation(
        &self,
        _: &'ast Instantiation<'arena>,
        context: &mut ThrowingExpressionScanContext<'context>,
    ) {
        context.may_throw = true;
    }

    fn walk_throw(&self, _: &'ast Throw<'arena>, context: &mut ThrowingExpressionScanContext<'context>) {
        context.may_throw = true;
    }

    fn walk_closure(&self, _: &'ast Closure<'arena>, _: &mut ThrowingExpressionScanContext<'context>) {}

    fn walk_arrow_function(&self, _: &'ast ArrowFunction<'arena>, _: &mut ThrowingExpressionScanContext<'context>) {}
}

pub(crate) fn expression_may_throw(expression: &Expression<'_>, artifacts: Option<&AnalysisArtifacts>) -> bool {
    let mut context = ThrowingExpressionScanContext { artifacts, may_throw: false };
    ThrowingExpressionScanner.walk_expression(expression, &mut context);
    context.may_throw
}

impl BreakType {
    #[inline]
    pub const fn is_switch(self) -> bool {
        matches!(self, BreakType::Switch)
    }
}

impl ControlActionSet {
    pub const EMPTY: Self = Self(0);

    #[inline]
    pub const fn new() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn from_single(action: ControlAction) -> Self {
        Self(action as u8)
    }

    #[inline]
    pub fn insert(&mut self, action: ControlAction) {
        self.0 |= action as u8;
    }

    #[inline]
    pub const fn contains(&self, action: ControlAction) -> bool {
        (self.0 & action as u8) != 0
    }

    #[inline]
    pub fn extend(&mut self, other: Self) {
        self.0 |= other.0;
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Returns the number of actions in the set
    #[inline]
    pub const fn len(&self) -> u32 {
        self.0.count_ones()
    }

    /// Retain only actions that satisfy the predicate
    #[inline]
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(ControlAction) -> bool,
    {
        for action in ControlAction::ALL {
            if self.contains(action) && !f(action) {
                self.0 &= !(action as u8);
            }
        }
    }

    /// Iterator over all actions in the set
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = ControlAction> + '_ {
        ControlAction::ALL.into_iter().filter(|&action| self.contains(action))
    }
}

impl FromIterator<ControlAction> for ControlActionSet {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = ControlAction>,
    {
        let mut set = Self::new();
        for action in iter {
            set.insert(action);
        }
        set
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
