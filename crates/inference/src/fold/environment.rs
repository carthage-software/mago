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

    /// The narrowed type recorded for a place, or `None` when it has not been
    /// narrowed. Unlike [`Self::get`], absence is reported rather than defaulted
    /// to `mixed`, so a derived place (an array element) can fall back to the
    /// type computed from its container's shape.
    pub(crate) fn lookup(&self, place: Var<'arena>) -> Option<Type<'arena>> {
        self.variables.get(&place).copied()
    }

    /// Drops every derived place rooted at `root` (e.g. `$a[0]`, `$a['k']`), whose
    /// narrowing a write to `root` invalidates. The root's own entry is left as is,
    /// since the write replaces it directly. A variable name never contains `[` or
    /// `-`, so the byte after the root is an unambiguous boundary.
    pub(crate) fn invalidate_rooted_in(&mut self, root: Var<'arena>) {
        let root_bytes = root.as_bytes();
        self.variables.retain(|place, _| {
            let bytes = place.as_bytes();
            !(bytes.len() > root_bytes.len()
                && bytes.starts_with(root_bytes)
                && matches!(bytes[root_bytes.len()], b'[' | b'-'))
        });
    }

    /// Forgets `variable`, so a later read sees it as undefined (`mixed`).
    pub(crate) fn unset(&mut self, variable: Var<'arena>) {
        self.variables.remove(&variable);
    }

    /// Whether two environments bind exactly the same places to the same types.
    /// Types are hash-consed, so the per-entry comparison is cheap; this is the
    /// fixed-point test a loop uses to decide its head has stabilized.
    pub(crate) fn equals(&self, other: &Self) -> bool {
        self.variables.len() == other.variables.len()
            && self.variables.iter().all(|(place, ty)| other.variables.get(place) == Some(ty))
    }

    /// Each place paired with a mutable handle to its type, for rewriting types in
    /// place without reallocating the map (loop widening / definite-key marking).
    pub(crate) fn entries_mut(&mut self) -> impl Iterator<Item = (Var<'arena>, &mut Type<'arena>)> {
        self.variables.iter_mut().map(|(place, ty)| (*place, ty))
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
        // A derived place (an array element, later a property) is narrowed only where
        // it is explicitly recorded; absent on a path it stands for the wider
        // container-shaped type. So a derived place survives the join only when both
        // paths carry it — otherwise it is dropped and reads fall back to the shape.
        // Plain variables are always tracked, so they keep the one-sided behavior.
        self.variables.retain(|place, _| is_variable_place(*place) || other.variables.contains_key(place));

        for (place, right_type) in &other.variables {
            match self.variables.get(place).copied() {
                Some(left_type) => {
                    self.variables.insert(*place, union_types(builder, left_type, *right_type));
                }
                None if is_variable_place(*place) => {
                    self.variables.insert(*place, *right_type);
                }
                None => {}
            }
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

/// A place key is a plain variable when it names nothing derived: variable names
/// contain neither `[` (an array element) nor `-` (a property arrow), so either
/// byte marks a derived place that is only valid where it has been narrowed.
fn is_variable_place(place: Var<'_>) -> bool {
    !place.as_bytes().iter().any(|byte| matches!(byte, b'[' | b'-'))
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
