use mago_allocator::Arena;
use mago_allocator::collections::HashMap;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::well_known::TYPE_MIXED;
use mago_oracle::var::Var;

/// The local-variable type environment threaded through inference: a map from
/// each variable to its current inferred type, allocated on the scratch arena.
#[derive(Debug)]
pub(crate) struct Environment<'source, 'arena, A: Arena> {
    arena: &'source A,
    variables: HashMap<'source, Var<'arena>, Type<'arena>, A>,
}

impl<A: Arena> Clone for Environment<'_, '_, A> {
    fn clone(&self) -> Self {
        Self { arena: self.arena, variables: self.variables.clone() }
    }

    fn clone_from(&mut self, source: &Self) {
        self.arena = source.arena;
        self.variables.clone_from(&source.variables);
    }
}

impl<'source, 'arena, A: Arena> Environment<'source, 'arena, A> {
    pub(crate) fn new_in(arena: &'source A) -> Self {
        Self { arena, variables: HashMap::new_in(arena) }
    }

    /// The current type of `variable`, or `mixed` if it has never been written —
    /// an unknown or undefined local reads as `mixed`.
    pub(crate) fn get(&self, variable: Var<'arena>) -> Type<'arena> {
        self.variables.get(&variable).copied().unwrap_or(TYPE_MIXED)
    }

    pub(crate) fn set(&mut self, variable: Var<'arena>, ty: Type<'arena>) {
        self.variables.insert(variable, ty);
    }

    /// Forgets `variable`, so a later read sees it as undefined (`mixed`).
    pub(crate) fn unset(&mut self, variable: Var<'arena>) {
        self.variables.remove(&variable);
    }

    /// Merges a conditionally-taken path back in: keeps only the variables that
    /// existed in `before` (so a variable introduced only on the conditional path,
    /// or by scoped narrowing, does not leak as definite), unioning each with its
    /// current value. Used after a short-circuit operand.
    pub(crate) fn merge_with(&mut self, before: Self, builder: &mut TypeBuilder<'source, 'arena, A, A>) {
        let mut merged = Self::new_in(self.arena);
        for (variable, before_type) in &before.variables {
            let value = match self.variables.get(variable).copied() {
                Some(current) => union_types(builder, *before_type, current),
                None => *before_type,
            };

            merged.variables.insert(*variable, value);
        }

        *self = merged;
    }

    /// Unions `self` with `other` over the union of their keys (a variable present
    /// in only one keeps its type). Used to join the two truth-paths of a
    /// short-circuit operand and the reachable branches of an `if`/`match`.
    pub(crate) fn union(mut self, other: Self, builder: &mut TypeBuilder<'source, 'arena, A, A>) -> Self {
        for (variable, right_type) in &other.variables {
            let value = match self.variables.get(variable).copied() {
                Some(left_type) => union_types(builder, left_type, *right_type),
                None => *right_type,
            };

            self.variables.insert(*variable, value);
        }

        self
    }

    /// Joins two optional environments, where `None` is an unreachable path: two
    /// reachable paths are [`Self::union`]ed, and a single reachable path passes
    /// through.
    pub(crate) fn merge_options(
        left: Option<Self>,
        right: Option<Self>,
        builder: &mut TypeBuilder<'source, 'arena, A, A>,
    ) -> Option<Self> {
        match (left, right) {
            (None, None) => None,
            (Some(environment), None) | (None, Some(environment)) => Some(environment),
            (Some(left), Some(right)) => Some(left.union(right, builder)),
        }
    }
}

/// The union of two types, deduplicated on the builder's scratch arena.
pub(crate) fn union_types<'source, 'arena, A>(
    builder: &mut TypeBuilder<'source, 'arena, A, A>,
    left: Type<'arena>,
    right: Type<'arena>,
) -> Type<'arena>
where
    A: Arena,
{
    let mut atoms = builder.scratch_vec::<Atom<'arena>>();
    atoms.extend_from_slice(left.atoms);
    atoms.extend_from_slice(right.atoms);

    builder.union_of(&atoms)
}
