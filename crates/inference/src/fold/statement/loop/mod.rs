use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::literal::LiteralKind;
use mago_hir::ir::statement::Statement;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::atom::payload::array::ArrayAtom;
use mago_oracle::ty::atom::payload::array::KnownItem;
use mago_oracle::ty::atom::payload::scalar::int::IntAtom;

use crate::error::InferenceResult;
use crate::flow::ControlFlow;
use crate::flow::Flow;
use crate::fold::Environment;
use crate::fold::InferenceFolder;
use crate::fold::LoopFrame;
use crate::fold::statement::r#loop::assignment_map::assignment_depth;

pub(crate) mod assignment_map;
pub(crate) mod do_while;
pub(crate) mod r#for;
pub(crate) mod foreach;
pub(crate) mod r#while;

/// Re-folding the body more than this many times is treated as enough to reach
/// the fixed point; deeper assignment chains widen rather than spin.
const ITERATION_LIMIT: usize = 8;

/// The typed pieces a loop body's analysis produces, handed back to the
/// construct-specific shell so it can rebuild its own node (`While`, `For`, …).
pub(crate) struct LoopOutcome<'arena> {
    pub conditions: &'arena [Expression<'arena, SymbolId, Flow, Type<'arena>>],
    pub increments: &'arena [Expression<'arena, SymbolId, Flow, Type<'arena>>],
    pub body: &'arena Statement<'arena, SymbolId, Flow, Type<'arena>>,
    pub exit: ControlFlow,
    pub reachable: bool,
}

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    /// The shared loop engine. `conditions` are the pre-conditions evaluated each
    /// iteration (empty for `foreach` and `for(;;)`); for a `do`/`while` they are
    /// applied after the body. `increments` run at the end of each iteration.
    /// `always_enters` means the body runs at least once; `infinite` means there
    /// is no condition that can end it (`while(true)`, `for(;;)`, `do {} while(true)`).
    ///
    /// It converges the loop-head environment by re-folding the body up to its
    /// assignment depth (widening monotone integers), folds it one final time to
    /// produce the typed tree, then computes the environment and control flow that
    /// reach the code after the loop.
    pub(crate) fn analyze_loop(
        &mut self,
        conditions: &[&'source Expression<'source, SymbolId, S, E>],
        increments: &[&'source Expression<'source, SymbolId, S, E>],
        body: &'source Statement<'source, SymbolId, S, E>,
        is_do: bool,
        always_enters: bool,
        infinite: bool,
    ) -> InferenceResult<LoopOutcome<'arena>> {
        let entry_reachable = self.reachable;
        let entry = self.environment.clone();

        // A body with no assignment chains stabilizes in a single fold; otherwise
        // re-fold to a fixed point, bounded so widening always terminates it.
        let cap =
            if assignment_depth(conditions, increments, body, ITERATION_LIMIT) == 0 { 1 } else { ITERATION_LIMIT };
        let head = self.converge_head(&entry, conditions, increments, body, is_do, cap)?;

        // Producing pass: fold against the converged head and keep the typed tree.
        self.loops.push(LoopFrame::default());
        self.environment = head.clone();
        self.reachable = entry_reachable;

        let mut typed_conditions = Vec::new_in(self.arena);
        if !is_do {
            self.apply_conditions(conditions, &mut typed_conditions)?;
        }

        let typed_body = self.infer_statement(body)?;
        let body_exit = typed_body.meta.exit;
        let body_falls_through = matches!(body_exit, ControlFlow::Fallthrough);
        let body_end = self.environment.clone();

        if is_do {
            self.environment = body_end.clone();
            self.apply_conditions(conditions, &mut typed_conditions)?;
        }

        let mut typed_increments = Vec::new_in(self.arena);
        for increment in increments {
            let typed = self.infer_expression(increment)?;
            typed_increments.push(typed);
        }
        let body_end_after_increment = self.environment.clone();

        let frame = self.loops.pop().unwrap_or_default();
        let has_break = frame.break_environment.is_some();
        let has_continue = frame.continue_environment.is_some();

        let completes_via_condition = !infinite;
        let definite = always_enters && !has_break && !has_continue;

        let mut exit_environment: Option<Environment<'source, 'arena, A>> = None;
        if completes_via_condition {
            let base = if definite && body_falls_through { body_end_after_increment } else { head.clone() };
            let normal = if conditions.is_empty() { base } else { self.narrow_on_exit(base, conditions)? };
            exit_environment = Environment::merge_options(exit_environment, Some(normal), &mut self.ty);
        }

        if conditions.is_empty() && !always_enters && !infinite {
            exit_environment = Environment::merge_options(exit_environment, Some(entry.clone()), &mut self.ty);
        }

        exit_environment = Environment::merge_options(exit_environment, frame.break_environment, &mut self.ty);

        let exit = loop_exit(body_exit, completes_via_condition, has_break, always_enters);
        let reachable_after = entry_reachable && matches!(exit, ControlFlow::Fallthrough);

        let mut environment = exit_environment.unwrap_or(head);
        if definite {
            self.mark_environment_keys_definite(&mut environment);
        }

        self.environment = environment;
        self.reachable = reachable_after;

        Ok(LoopOutcome {
            conditions: typed_conditions.leak(),
            increments: typed_increments.leak(),
            body: self.arena.alloc(typed_body),
            exit,
            reachable: entry_reachable,
        })
    }

    /// Re-folds the body to a fixed point, returning the converged loop-head
    /// environment. Each pass starts from `entry ∪ back-edge`, widening monotone
    /// integers so a counter does not spin; it stops as soon as the head stops
    /// changing or `cap` passes are reached.
    fn converge_head(
        &mut self,
        entry: &Environment<'source, 'arena, A>,
        conditions: &[&'source Expression<'source, SymbolId, S, E>],
        increments: &[&'source Expression<'source, SymbolId, S, E>],
        body: &'source Statement<'source, SymbolId, S, E>,
        is_do: bool,
        cap: usize,
    ) -> InferenceResult<Environment<'source, 'arena, A>> {
        let mut head = entry.clone();
        for _ in 0..cap {
            let next = self.iterate_once(entry, &head, conditions, increments, body, is_do)?;
            let stable = head.equals(&next);
            head = next;
            if stable {
                break;
            }
        }

        Ok(head)
    }

    fn iterate_once(
        &mut self,
        entry: &Environment<'source, 'arena, A>,
        head: &Environment<'source, 'arena, A>,
        conditions: &[&'source Expression<'source, SymbolId, S, E>],
        increments: &[&'source Expression<'source, SymbolId, S, E>],
        body: &'source Statement<'source, SymbolId, S, E>,
        is_do: bool,
    ) -> InferenceResult<Environment<'source, 'arena, A>> {
        self.loops.push(LoopFrame::default());
        self.environment.clone_from(head);
        self.reachable = true;

        if !is_do {
            self.narrow_into_body(conditions)?;
        }

        let typed_body = self.infer_statement(body)?;
        let fallthrough = matches!(typed_body.meta.exit, ControlFlow::Fallthrough).then(|| self.environment.clone());

        let frame = self.loops.pop().unwrap_or_default();
        let mut back = Environment::merge_options(fallthrough, frame.continue_environment, &mut self.ty);

        if is_do && let Some(back_environment) = back.take() {
            self.environment = back_environment;
            self.narrow_into_body(conditions)?;
            back = Some(self.environment.clone());
        }

        if let Some(back_environment) = back.take() {
            self.environment = back_environment;
            for increment in increments {
                self.infer_expression(increment)?;
            }

            back = Some(self.environment.clone());
        }

        let next = match back {
            Some(back_environment) => entry.clone().union(back_environment, &mut self.ty),
            None => entry.clone(),
        };

        Ok(self.widen_environment(head, next))
    }

    /// Applies each pre-condition's truthy narrowing to the current environment,
    /// discarding the typed nodes (used by the convergence passes).
    fn narrow_into_body(&mut self, conditions: &[&'source Expression<'source, SymbolId, S, E>]) -> InferenceResult<()> {
        for condition in conditions {
            let (_typed, when_true, _when_false) = self.analyze_condition(condition)?;
            match when_true {
                Some(environment) => self.environment = environment,
                None => self.reachable = false,
            }
        }

        Ok(())
    }

    /// As [`Self::narrow_into_body`] but keeps the typed condition nodes (the
    /// producing pass).
    fn apply_conditions(
        &mut self,
        conditions: &[&'source Expression<'source, SymbolId, S, E>],
        typed: &mut Vec<'arena, Expression<'arena, SymbolId, Flow, Type<'arena>>, A>,
    ) -> InferenceResult<()> {
        for condition in conditions {
            let (typed_condition, when_true, _when_false) = self.analyze_condition(condition)?;
            match when_true {
                Some(environment) => self.environment = environment,
                None => self.reachable = false,
            }

            typed.push(typed_condition);
        }

        Ok(())
    }

    /// The environment on the path that exits the loop because the condition went
    /// false: `base` narrowed by the negated condition. Only narrows for a single
    /// condition (the precise case for `while`/`for`); multiple conditions are
    /// left un-narrowed.
    fn narrow_on_exit(
        &mut self,
        base: Environment<'source, 'arena, A>,
        conditions: &[&'source Expression<'source, SymbolId, S, E>],
    ) -> InferenceResult<Environment<'source, 'arena, A>> {
        let [condition] = conditions else {
            return Ok(base);
        };

        self.environment.clone_from(&base);
        let (_typed, _when_true, when_false) = self.analyze_condition(condition)?;

        Ok(when_false.unwrap_or(base))
    }

    fn widen_environment(
        &mut self,
        previous: &Environment<'source, 'arena, A>,
        mut next: Environment<'source, 'arena, A>,
    ) -> Environment<'source, 'arena, A> {
        for (place, current) in next.entries_mut() {
            if let Some(prior) = previous.lookup(place)
                && let Some(widened) = widen_integer(&mut self.ty, prior, *current)
            {
                *current = widened;
            }
        }

        next
    }

    fn mark_environment_keys_definite(&mut self, environment: &mut Environment<'source, 'arena, A>) {
        for (_, ty) in environment.entries_mut() {
            if let Some(definite) = mark_keys_definite(&mut self.ty, *ty) {
                *ty = definite;
            }
        }
    }
}

/// Widens a monotonically-growing integer toward `±∞` so a loop counter does not
/// produce a fresh literal each pass. Opens the upper bound when the new type's
/// maximum grew, the lower bound when its minimum shrank. `None` when neither type
/// is a pure integer or nothing changed.
fn widen_integer<'arena, A>(
    builder: &mut TypeBuilder<'_, 'arena, A, A>,
    previous: Type<'arena>,
    next: Type<'arena>,
) -> Option<Type<'arena>>
where
    A: Arena,
{
    let (previous_lower, previous_upper) = integer_bounds(previous)?;
    let (next_lower, next_upper) = integer_bounds(next)?;

    let upper = match (previous_upper, next_upper) {
        (Some(previous), Some(next)) if next > previous => None,
        (Some(_), None) => None,
        _ => next_upper,
    };

    let lower = match (previous_lower, next_lower) {
        (Some(previous), Some(next)) if next < previous => None,
        (Some(_), None) => None,
        _ => next_lower,
    };

    if (lower, upper) == (next_lower, next_upper) {
        return None;
    }

    Some(builder.int_range_type(lower, upper))
}

/// After a loop that always enters, keys an array gained inside the body are
/// guaranteed present, so their optional flag is cleared. Returns `None` when
/// nothing changed.
fn mark_keys_definite<'arena, A>(builder: &mut TypeBuilder<'_, 'arena, A, A>, ty: Type<'arena>) -> Option<Type<'arena>>
where
    A: Arena,
{
    let mut atoms = builder.scratch_vec::<Atom<'arena>>();
    let mut changed = false;
    for atom in ty.atoms {
        match atom {
            Atom::Array(array) => match definite_array(builder, array) {
                Some(definite) => {
                    changed = true;
                    let definite = builder.array(definite);
                    atoms.push(definite);
                }
                None => atoms.push(*atom),
            },
            other => atoms.push(*other),
        }
    }

    changed.then(|| builder.union_of(&atoms))
}

fn definite_array<'arena, A>(
    builder: &mut TypeBuilder<'_, 'arena, A, A>,
    array: &ArrayAtom<'arena>,
) -> Option<ArrayAtom<'arena>>
where
    A: Arena,
{
    let known_items = array.known_items?;
    if known_items.iter().all(|item| !item.optional) {
        return None;
    }

    let mut items = builder.scratch_vec::<KnownItem<'arena>>();
    for item in known_items {
        items.push(KnownItem { key: item.key, value: item.value, optional: false });
    }

    let known_items = builder.known_items(&items);
    Some(ArrayAtom {
        key_param: array.key_param,
        value_param: array.value_param,
        known_items: Some(known_items),
        flags: array.flags,
    })
}

/// The combined `(lower, upper)` bound of a type when every atom is an integer
/// (`None` for an open side). `None` when any atom is not an integer, so only a
/// pure-integer type is ever widened.
fn integer_bounds(ty: Type<'_>) -> Option<(Option<i64>, Option<i64>)> {
    if ty.atoms.is_empty() {
        return None;
    }

    let mut lower = Some(i64::MAX);
    let mut upper = Some(i64::MIN);
    for atom in ty.atoms {
        let (atom_lower, atom_upper) = match atom {
            Atom::Int(IntAtom::Literal(value)) => (Some(*value), Some(*value)),
            Atom::Int(IntAtom::Range(range)) => (range.lower(), range.upper()),
            Atom::Int(IntAtom::Unspecified | IntAtom::UnspecifiedLiteral) => (None, None),
            _ => return None,
        };

        lower = match (lower, atom_lower) {
            (Some(current), Some(value)) => Some(current.min(value)),
            _ => None,
        };
        upper = match (upper, atom_upper) {
            (Some(current), Some(value)) => Some(current.max(value)),
            _ => None,
        };
    }

    Some((lower, upper))
}

/// Whether a loop condition is the literal `true` or a positive integer literal,
/// the syntactic forms that make a `while`/`do-while` unconditionally infinite.
pub(crate) fn is_truthy_literal<I, S, E>(condition: &Expression<'_, I, S, E>) -> bool {
    let ExpressionKind::Literal(literal) = &condition.kind else {
        return false;
    };

    match literal.kind {
        LiteralKind::True => true,
        LiteralKind::Integer(integer) => integer.value.is_none_or(|value| value > 0),
        _ => false,
    }
}

/// The control flow that leaves the loop statement. A loop that can complete (its
/// condition can go false) or that is broken out of falls through; a multi-level
/// `break`/`continue` from the body propagates one level out; an infinite loop
/// that is never broken never falls through.
fn loop_exit(
    body_exit: ControlFlow,
    completes_via_condition: bool,
    has_break: bool,
    always_enters: bool,
) -> ControlFlow {
    if completes_via_condition || has_break {
        return ControlFlow::Fallthrough;
    }

    match body_exit {
        ControlFlow::Break(level) if level >= 2 => ControlFlow::Break(level - 1),
        ControlFlow::Continue(level) if level >= 2 => ControlFlow::Continue(level - 1),
        ControlFlow::Return if always_enters => ControlFlow::Return,
        _ => ControlFlow::Diverge,
    }
}
