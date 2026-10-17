use mago_span::HasSpan;
use mago_syntax::cst::Construct;
use mago_syntax::cst::Expression;
use mago_syntax::cst::ForBody;
use mago_syntax::cst::ForeachBody;
use mago_syntax::cst::Literal;
use mago_syntax::cst::Statement;
use mago_syntax::cst::WhileBody;

use crate::artifacts::AnalysisArtifacts;

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
    /// A break or continue transfer whose switch target has not been resolved yet.
    BreakTargetSwitch = 0b1000_0000,
}

/// A compact bitfield set for ControlAction values.
/// Switch targets are stored separately because nested transfers need to retain their target index.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ControlActionSet {
    actions: u8,
    switch_targets: Vec<usize>,
}

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
        Self::from_statements_with_switch_target(statements, break_type, artifacts, return_is_exit)
    }

    /// Analyzes a case body while preserving the switch that owns the case as a transfer target.
    pub(crate) fn from_switch_case_statements(
        statements: Vec<&Statement>,
        break_type: Vec<BreakType>,
        artifacts: Option<&AnalysisArtifacts>,
        return_is_exit: bool,
    ) -> ControlActionSet {
        let analyzed_switch_index = break_type.len().checked_sub(1);

        let mut actions = Self::from_statements_with_switch_target(statements, break_type, artifacts, return_is_exit);
        if let Some(analyzed_switch_index) = analyzed_switch_index {
            actions.resolve_switch_targets(analyzed_switch_index);
        }

        actions
    }

    fn from_statements_with_switch_target(
        statements: Vec<&Statement>,
        break_type: Vec<BreakType>,
        artifacts: Option<&AnalysisArtifacts>,
        return_is_exit: bool,
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
                    );
                    loop_actions.retain(|action| !action.is_none());

                    control_actions.extend(&loop_actions);

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
                    );

                    if !block_actions.contains(ControlAction::None) {
                        control_actions.extend(&block_actions);
                        control_actions.retain(|action| action != ControlAction::None);

                        return control_actions;
                    }

                    block_actions.retain(|action| action != ControlAction::None);
                    control_actions.extend(&block_actions);
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
                                Some(BreakType::Switch) => control_actions.insert_switch_target(target),
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
                                    Some(BreakType::Switch) => control_actions.insert_switch_target(target),
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
                                    control_actions.insert_switch_target(target);
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
                    let switch_target_index = break_type.len();

                    // Resolve cases backwards so each case's fallthrough inherits the next case's possible actions.
                    for case in cases.iter().rev() {
                        let mut case_break_type = break_type.clone();
                        case_break_type.push(BreakType::Switch);

                        let case_actions = ControlAction::from_statements_with_switch_target(
                            case.statements().iter().collect(),
                            case_break_type,
                            artifacts,
                            return_is_exit,
                        );

                        let mut resolved_case_actions = ControlActionSet::new();
                        for action in case_actions.iter() {
                            match action {
                                ControlAction::None => resolved_case_actions.extend(&fallthrough_actions),
                                ControlAction::LeaveSwitch => resolved_case_actions.insert(ControlAction::None),
                                ControlAction::BreakTargetSwitch => {
                                    for target in case_actions.switch_targets() {
                                        if target == switch_target_index {
                                            resolved_case_actions.insert(ControlAction::None);
                                        } else {
                                            resolved_case_actions.insert_switch_target(target);
                                        }
                                    }
                                }
                                _ => resolved_case_actions.insert(action),
                            }
                        }

                        all_case_actions.extend(&resolved_case_actions);
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
                    control_actions.extend(&all_case_actions);

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
                        );

                        all_leave = all_leave && !elseif_control_actions.contains(ControlAction::None);

                        all_elseif_actions.extend(&elseif_control_actions);
                    }

                    control_actions.extend(&if_statement_actions);
                    control_actions.extend(&else_statement_actions);
                    control_actions.extend(&all_elseif_actions);

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
                    );

                    let mut try_catch_actions = try_statement_actions;
                    for catch in &try_catch.catch_clauses {
                        let catch_actions = ControlAction::from_statements_with_switch_target(
                            catch.block.statements.iter().collect::<Vec<_>>(),
                            break_type.clone(),
                            artifacts,
                            return_is_exit,
                        );
                        try_catch_actions.extend(&catch_actions);
                    }

                    if let Some(finally_clause) = try_catch.finally_clause.as_ref()
                        && !finally_clause.block.statements.is_empty()
                    {
                        let mut finally_statement_actions = ControlAction::from_statements_with_switch_target(
                            finally_clause.block.statements.iter().collect::<Vec<_>>(),
                            break_type.clone(),
                            artifacts,
                            return_is_exit,
                        );

                        let finally_can_complete_normally = finally_statement_actions.contains(ControlAction::None);
                        let try_catch_can_complete_normally = try_catch_actions.contains(ControlAction::None);

                        finally_statement_actions.retain(|action| !action.is_none());
                        control_actions.extend(&finally_statement_actions);

                        // An abrupt finally replaces pending actions; a normal finally preserves them.
                        if finally_can_complete_normally {
                            control_actions.extend(&try_catch_actions);
                        }

                        if !finally_can_complete_normally || !try_catch_can_complete_normally {
                            return control_actions;
                        }

                        control_actions.retain(|action| !action.is_none());
                        continue;
                    }

                    let try_catch_can_complete_normally = try_catch_actions.contains(ControlAction::None);
                    control_actions.extend(&try_catch_actions);
                    control_actions.retain(|action| !action.is_none());

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
        matches!(self, ControlAction::BreakImmediateLoop)
    }

    #[inline]
    pub const fn is_break_or_continue(self) -> bool {
        matches!(self, ControlAction::Break | ControlAction::Continue)
    }
}

impl BreakType {
    #[inline]
    pub const fn is_switch(self) -> bool {
        matches!(self, BreakType::Switch)
    }
}

impl ControlActionSet {
    pub const EMPTY: Self = Self { actions: 0, switch_targets: Vec::new() };

    #[inline]
    pub const fn new() -> Self {
        Self { actions: 0, switch_targets: Vec::new() }
    }

    #[inline]
    pub const fn from_single(action: ControlAction) -> Self {
        Self { actions: action as u8, switch_targets: Vec::new() }
    }

    #[inline]
    pub fn insert(&mut self, action: ControlAction) {
        self.actions |= action as u8;
    }

    #[inline]
    pub fn insert_switch_target(&mut self, target: usize) {
        if let Err(index) = self.switch_targets.binary_search(&target) {
            self.switch_targets.insert(index, target);
        }
        self.actions |= ControlAction::BreakTargetSwitch as u8;
    }

    #[inline]
    pub const fn contains(&self, action: ControlAction) -> bool {
        (self.actions & action as u8) != 0
    }

    #[inline]
    pub fn extend(&mut self, other: &Self) {
        self.actions |= other.actions;
        for target in &other.switch_targets {
            self.insert_switch_target(*target);
        }
    }

    #[inline]
    pub fn switch_targets(&self) -> impl Iterator<Item = usize> + '_ {
        self.switch_targets.iter().copied()
    }

    pub fn resolve_switch_targets(&mut self, switch_index: usize) {
        let targets = std::mem::take(&mut self.switch_targets);
        self.actions &= !(ControlAction::BreakTargetSwitch as u8);

        for target in targets {
            debug_assert!(target <= switch_index, "A nested switch target should be consumed by its switch");
            if target == switch_index {
                self.insert(ControlAction::LeaveSwitch);
            } else if target < switch_index {
                self.insert(ControlAction::Break);
            } else {
                self.insert_switch_target(target);
            }
        }
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.actions == 0
    }

    /// Returns the number of actions in the set
    #[inline]
    pub const fn len(&self) -> u32 {
        self.actions.count_ones()
    }

    /// Retain only actions that satisfy the predicate
    #[inline]
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(ControlAction) -> bool,
    {
        for action in ControlAction::ALL {
            if self.contains(action) && !f(action) {
                self.actions &= !(action as u8);
                if action == ControlAction::BreakTargetSwitch {
                    self.switch_targets.clear();
                }
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
