//! Standin replacement: template inference at call sites.
//!
//! The operation walks a *parameter type* alongside an *argument
//! type* in lockstep. Wherever the parameter mentions a template
//! parameter `T`, a bound on `T` is recorded against the corresponding
//! sub-type of the argument; the parameter slot itself is replaced by
//! `T`'s constraint (the loosest type a value at that position can
//! still inhabit). The caller threads the same [`TemplateState`] across
//! every parameter of a call site, then runs bound reconciliation to
//! materialise each `T`'s witness.
//!
//! # Public API
//!
//! ```ignore
//! let mut state = TemplateState::new();
//! let options = StandinOptions::default();
//! let refined = standin(parameter, argument, &symbols, &mut state, &options, &mut builder);
//! ```
//!
//! Repeat for each call-site argument, then run reconciliation on
//! `state.bounds_for(...)`.
//!
//! # Scope
//!
//! - `GenericParameter T` (anywhere in the parameter tree): record a
//!   bound and emit `T`'s constraint.
//! - Same-class generic objects: walk type arguments by position, with
//!   the declared variance for each parameter (covariant ⇒
//!   lower bound, contravariant ⇒ upper, invariant ⇒ equality).
//!   Descendant arguments project through
//!   [`SymbolTable::inherited_template_argument`].
//! - `Reference(C, [τ_i])` (an un-expanded `C<…>` symbol): co-traversed
//!   identically to a generic object, so a template hidden behind a
//!   not-yet-resolved symbol is still bound.
//! - `class-string<T>` (bounded / generic class-like-string): walk the
//!   constraint against the class an argument class-string names, so
//!   `class-string<T>` vs `Foo::class` binds `T = Foo`.
//! - `object{k: T, …}` (object shape): walk each property against the
//!   argument property of the same name, covariantly.
//! - `List(τ)` against `List(σ)` or `Iterable(_, σ)`: covariant walk.
//! - `Iterable(τ_K, τ_V)` against another iterable or a list: covariant
//!   walks on key and value.
//! - `Keyed(τ_K, τ_V, {k → τ})` against keyed-array or iterable
//!   arguments: covariant walks on key, value, and matching known
//!   items.
//! - `Callable(Sig)` against a callable argument: contravariant walks
//!   on parameter types, covariant walk on the return type.
//! - Distribution over union: each parameter atom inspects every
//!   argument atom that could contribute (literal-equal atoms are
//!   filtered once that case is observed).
//!
//! `Conditional` and `Derived` (`key-of<T>`, `T[K]`, …) parameter shapes
//! are *not* co-traversed: there is no sound inverse that recovers a
//! unique `T` from a concrete argument (which `T` has `key-of<T> = 'a'`?),
//! so inferring across them would guess. `Alias` references are resolved
//! to their structural body by expansion before inference runs, so they
//! never reach the walk as aliases.

use std::collections::BTreeMap;

use mago_allocator::Arena;
use mago_span::Span;

use crate::path::Path;
use crate::symbol::SymbolTable;
use crate::symbol::part::generic::Variance;
use crate::ty::Type;
use crate::ty::atom::Atom;
use crate::ty::atom::kind::AtomKind;
use crate::ty::atom::payload::array::ArrayAtom;
use crate::ty::atom::payload::array::KnownItem;
use crate::ty::atom::payload::array::ListAtom;
use crate::ty::atom::payload::callable::CallableAtom;
use crate::ty::atom::payload::callable::Parameter;
use crate::ty::atom::payload::callable::Signature;
use crate::ty::atom::payload::generic_parameter::DefiningEntity;
use crate::ty::atom::payload::generic_parameter::GenericParameterAtom;
use crate::ty::atom::payload::iterable::IterableAtom;
use crate::ty::atom::payload::object::named::ObjectAtom;
use crate::ty::atom::payload::object::shape::KnownProperty;
use crate::ty::atom::payload::object::shape::ObjectShapeAtom;
use crate::ty::atom::payload::reference::SymbolReferenceAtom;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeStringAtom;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeStringSpecifier;
use crate::ty::builder::TypeBuilder;
use crate::ty::template::Bound;
use crate::ty::template::BoundKind;
use crate::ty::template::StandinOptions;
use crate::ty::template::TemplateKey;
use crate::ty::well_known;

impl StandinOptions {
    #[must_use]
    #[inline]
    pub const fn with_argument_offset(mut self, offset: u32) -> Self {
        self.argument_offset = offset;
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_default_variance(mut self, variance: Variance) -> Self {
        self.default_variance = variance;
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_max_depth(mut self, depth: u32) -> Self {
        self.max_depth = depth;
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }
}

/// Definition of a template parameter as it exists in the inference scope.
///
/// Distinct from any bound inferred for it. The analyzer needs to ask
/// "does this template exist in scope" before it asks "what was
/// inferred", because a template never bound is indistinguishable from
/// one that doesn't exist if you only carry bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GenericTemplate<'arena> {
    pub defining_entity: DefiningEntity<'arena>,
    pub constraint: Type<'arena>,
}

/// Definitions, bounds, and anti-bounds collected across one or more standin walks.
///
/// The same [`TemplateState`] is threaded through every parameter of a
/// call so reconciliation sees the full set per template parameter.
///
/// Three parallel maps:
///
/// - `template_types` records *definitions* - the templates the inference
///   scope knows about, with their constraints.
/// - `bounds` records *inferred bounds* - what the standin walk
///   discovered for each template.
/// - `anti_bounds` records types each template *cannot* be - set by the
///   reconciler when one branch of a conditional rules out a possibility
///   the other branch must remember.
///
/// The walk auto-declares every template it encounters; consumers can
/// also call [`TemplateState::declare`] explicitly for templates that
/// don't appear in any walked parameter type.
#[derive(Debug, Default, Clone)]
pub struct TemplateState<'arena> {
    template_types: BTreeMap<TemplateKey<'arena>, GenericTemplate<'arena>>,
    bounds: BTreeMap<TemplateKey<'arena>, Vec<Bound<'arena>>>,
    anti_bounds: BTreeMap<TemplateKey<'arena>, Vec<Type<'arena>>>,
}

impl<'arena> TemplateState<'arena> {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// All bounds recorded for `(defining_entity, name)`, in insertion
    /// order. Empty when no bound has been collected.
    #[inline]
    pub fn bounds_for(&self, key: TemplateKey<'arena>) -> &[Bound<'arena>] {
        self.bounds.get(&key).map_or(&[][..], Vec::as_slice)
    }

    /// Iterate over every recorded `(key, bounds)` pair.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&TemplateKey<'arena>, &[Bound<'arena>])> {
        self.bounds.iter().map(|(key, bounds)| (key, bounds.as_slice()))
    }

    /// Register a template parameter as existing in scope. Idempotent on
    /// the same `(key, constraint)` pair; subsequent calls with a
    /// different constraint overwrite (the latest declaration wins, as
    /// inner scopes shadow outer ones).
    #[inline]
    pub fn declare(&mut self, key: TemplateKey<'arena>, constraint: Type<'arena>) {
        self.template_types.insert(key, GenericTemplate { defining_entity: key.defining_entity, constraint });
    }

    /// Declaration recorded for `key`, or `None` when the template has
    /// never been declared (or auto-declared by a walk).
    #[inline]
    #[must_use]
    pub fn declaration(&self, key: TemplateKey<'arena>) -> Option<&GenericTemplate<'arena>> {
        self.template_types.get(&key)
    }

    /// `true` iff `key` has a declaration recorded. The analyzer uses
    /// this to distinguish "no bound was inferred for an in-scope
    /// template" from "this template doesn't exist in this scope".
    #[inline]
    #[must_use]
    pub fn is_declared(&self, key: TemplateKey<'arena>) -> bool {
        self.template_types.contains_key(&key)
    }

    /// Iterate over every recorded `(key, declaration)` pair.
    #[inline]
    pub fn declarations(&self) -> impl Iterator<Item = (&TemplateKey<'arena>, &GenericTemplate<'arena>)> {
        self.template_types.iter()
    }

    /// Iterate every `(key, bounds)` pair whose defining entity matches
    /// `entity`. The analyzer uses this to walk a single class's
    /// inferences without touching unrelated scopes.
    #[inline]
    pub fn bounds_in_scope(
        &self,
        entity: DefiningEntity<'arena>,
    ) -> impl Iterator<Item = (&TemplateKey<'arena>, &[Bound<'arena>])> {
        self.bounds
            .iter()
            .filter(move |(key, _)| key.defining_entity == entity)
            .map(|(key, bounds)| (key, bounds.as_slice()))
    }

    /// Iterate every `(key, declaration)` pair whose defining entity
    /// matches `entity`.
    #[inline]
    pub fn declarations_in_scope(
        &self,
        entity: DefiningEntity<'arena>,
    ) -> impl Iterator<Item = (&TemplateKey<'arena>, &GenericTemplate<'arena>)> {
        self.template_types.iter().filter(move |(key, _)| key.defining_entity == entity)
    }

    /// Record a type that `key`'s template may **not** be. Used by the
    /// reconciler when a conditional branch rules out a possibility the
    /// other branch must remember.
    #[inline]
    pub fn add_anti_bound(&mut self, key: TemplateKey<'arena>, ty: Type<'arena>) {
        self.anti_bounds.entry(key).or_default().push(ty);
    }

    /// Types `key`'s template is forbidden from being. Empty when no
    /// anti-bounds have been recorded.
    #[inline]
    pub fn anti_bounds_for(&self, key: TemplateKey<'arena>) -> &[Type<'arena>] {
        self.anti_bounds.get(&key).map_or(&[][..], Vec::as_slice)
    }

    /// Iterate every `(key, anti-bounds)` pair whose defining entity
    /// matches `entity`.
    #[inline]
    pub fn anti_bounds_in_scope(
        &self,
        entity: DefiningEntity<'arena>,
    ) -> impl Iterator<Item = (&TemplateKey<'arena>, &[Type<'arena>])> {
        self.anti_bounds
            .iter()
            .filter(move |(key, _)| key.defining_entity == entity)
            .map(|(key, types)| (key, types.as_slice()))
    }

    /// Re-key every declaration, bound, and anti-bound from `from` so
    /// it appears under `to`. Used when a class extends another and
    /// the parent's inferences must propagate up under the child's
    /// entity. Bounds and anti-bounds append (the destination's
    /// existing list grows); declarations overwrite (latest scope wins).
    #[inline]
    pub fn merge_scope(&mut self, from: DefiningEntity<'arena>, to: DefiningEntity<'arena>) {
        if from == to {
            return;
        }

        let moved_declarations: Vec<(TemplateKey<'arena>, GenericTemplate<'arena>)> = self
            .template_types
            .iter()
            .filter(|(key, _)| key.defining_entity == from)
            .map(|(key, declaration)| (*key, *declaration))
            .collect();
        for (key, declaration) in moved_declarations {
            self.template_types.remove(&key);
            let new_key = TemplateKey { defining_entity: to, name: key.name };
            self.template_types.insert(new_key, GenericTemplate { defining_entity: to, ..declaration });
        }

        let moved_bounds: Vec<(TemplateKey<'arena>, Vec<Bound<'arena>>)> = self
            .bounds
            .iter()
            .filter(|(key, _)| key.defining_entity == from)
            .map(|(key, bounds)| (*key, bounds.clone()))
            .collect();
        for (key, bounds) in moved_bounds {
            self.bounds.remove(&key);
            let new_key = TemplateKey { defining_entity: to, name: key.name };
            self.bounds.entry(new_key).or_default().extend(bounds);
        }

        let moved_anti_bounds: Vec<(TemplateKey<'arena>, Vec<Type<'arena>>)> = self
            .anti_bounds
            .iter()
            .filter(|(key, _)| key.defining_entity == from)
            .map(|(key, anti_bounds)| (*key, anti_bounds.clone()))
            .collect();
        for (key, anti_bounds) in moved_anti_bounds {
            self.anti_bounds.remove(&key);
            let new_key = TemplateKey { defining_entity: to, name: key.name };
            self.anti_bounds.entry(new_key).or_default().extend(anti_bounds);
        }
    }

    /// Consume `self` and return a read-only [`TemplateResult`].
    ///
    /// Once inference for a call site is complete, the analyzer freezes
    /// the state so later substitution passes cannot accidentally mutate
    /// it. The split is type-state: callers who need to
    /// keep adding bounds must keep the [`TemplateState`] handle; once
    /// substitution starts they take a [`TemplateResult`] and the type
    /// system rules out any further accumulation.
    #[inline]
    #[must_use]
    pub fn freeze(self) -> TemplateResult<'arena> {
        TemplateResult { template_types: self.template_types, bounds: self.bounds, anti_bounds: self.anti_bounds }
    }

    #[inline]
    fn record(&mut self, key: TemplateKey<'arena>, bound: Bound<'arena>) {
        self.bounds.entry(key).or_default().push(bound);
    }
}

/// Read-only view of template definitions, bounds, and anti-bounds.
///
/// Produced by [`TemplateState::freeze`]. Substitution and reconciliation
/// take this; mutation is rejected at compile time because no
/// `&mut self` methods exist.
#[derive(Debug, Clone, Default)]
pub struct TemplateResult<'arena> {
    template_types: BTreeMap<TemplateKey<'arena>, GenericTemplate<'arena>>,
    bounds: BTreeMap<TemplateKey<'arena>, Vec<Bound<'arena>>>,
    anti_bounds: BTreeMap<TemplateKey<'arena>, Vec<Type<'arena>>>,
}

impl<'arena> TemplateResult<'arena> {
    #[inline]
    pub fn bounds_for(&self, key: TemplateKey<'arena>) -> &[Bound<'arena>] {
        self.bounds.get(&key).map_or(&[][..], Vec::as_slice)
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&TemplateKey<'arena>, &[Bound<'arena>])> {
        self.bounds.iter().map(|(key, bounds)| (key, bounds.as_slice()))
    }

    #[inline]
    #[must_use]
    pub fn declaration(&self, key: TemplateKey<'arena>) -> Option<&GenericTemplate<'arena>> {
        self.template_types.get(&key)
    }

    #[inline]
    #[must_use]
    pub fn is_declared(&self, key: TemplateKey<'arena>) -> bool {
        self.template_types.contains_key(&key)
    }

    #[inline]
    pub fn declarations(&self) -> impl Iterator<Item = (&TemplateKey<'arena>, &GenericTemplate<'arena>)> {
        self.template_types.iter()
    }

    #[inline]
    pub fn bounds_in_scope(
        &self,
        entity: DefiningEntity<'arena>,
    ) -> impl Iterator<Item = (&TemplateKey<'arena>, &[Bound<'arena>])> {
        self.bounds
            .iter()
            .filter(move |(key, _)| key.defining_entity == entity)
            .map(|(key, bounds)| (key, bounds.as_slice()))
    }

    #[inline]
    pub fn declarations_in_scope(
        &self,
        entity: DefiningEntity<'arena>,
    ) -> impl Iterator<Item = (&TemplateKey<'arena>, &GenericTemplate<'arena>)> {
        self.template_types.iter().filter(move |(key, _)| key.defining_entity == entity)
    }

    #[inline]
    pub fn anti_bounds_for(&self, key: TemplateKey<'arena>) -> &[Type<'arena>] {
        self.anti_bounds.get(&key).map_or(&[][..], Vec::as_slice)
    }

    #[inline]
    pub fn anti_bounds_in_scope(
        &self,
        entity: DefiningEntity<'arena>,
    ) -> impl Iterator<Item = (&TemplateKey<'arena>, &[Type<'arena>])> {
        self.anti_bounds
            .iter()
            .filter(move |(key, _)| key.defining_entity == entity)
            .map(|(key, types)| (key, types.as_slice()))
    }
}

/// Walk `parameter` and `argument` in lockstep; record bounds against any
/// template parameters mentioned in `parameter`.
///
/// Returns the refined parameter type - the standin - which mentions no
/// template parameter from `parameter`'s defining entity.
///
/// `state` accumulates bounds; reuse one across every parameter of a
/// call site so reconciliation sees the full set.
#[inline]
pub fn standin<'arena, S, A>(
    parameter: Type<'arena>,
    argument: Type<'arena>,
    symbols: &SymbolTable<'arena, A>,
    state: &mut TemplateState<'arena>,
    options: &StandinOptions,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    walk_type(parameter, argument, options.default_variance, 0, None, symbols, state, options, builder)
}

#[inline]
fn walk_type<'arena, S, A>(
    parameter: Type<'arena>,
    argument: Type<'arena>,
    variance: Variance,
    depth: u32,
    introducing_class: Option<Path<'arena>>,
    symbols: &SymbolTable<'arena, A>,
    state: &mut TemplateState<'arena>,
    options: &StandinOptions,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    if parameter == argument {
        return parameter;
    }
    if depth > options.max_depth {
        return collapse_to_constraints(parameter, builder);
    }

    let mut new_atoms: Vec<Atom<'arena>> = Vec::with_capacity(parameter.atoms.len());
    let mut changed = false;

    for &parameter_atom in parameter.atoms {
        let projected = project_argument(parameter_atom, argument);
        match walk_atom(parameter_atom, projected, variance, depth, introducing_class, symbols, state, options, builder)
        {
            Walk::Unchanged => new_atoms.push(parameter_atom),
            Walk::Single(atom) => {
                changed = true;
                new_atoms.push(atom);
            }
            Walk::Many(atoms) => {
                changed = true;
                new_atoms.extend(atoms);
            }
        }
    }

    if !changed {
        return parameter;
    }

    builder.union_of(&new_atoms)
}

/// Past the iteration-depth cutoff: replace any template parameter atom
/// in `parameter` with its constraint, leaving other atoms untouched.
/// No bound is recorded - the walk terminated.
#[inline]
fn collapse_to_constraints<'arena, S, A>(
    parameter: Type<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    let mut new_atoms: Vec<Atom<'arena>> = Vec::with_capacity(parameter.atoms.len());
    let mut changed = false;
    for &parameter_atom in parameter.atoms {
        if let Atom::GenericParameter(payload) = parameter_atom {
            new_atoms.extend_from_slice(payload.constraint.atoms);
            changed = true;
        } else {
            new_atoms.push(parameter_atom);
        }
    }
    if !changed {
        return parameter;
    }

    builder.union_of(&new_atoms)
}

enum Walk<'arena> {
    Unchanged,
    Single(Atom<'arena>),
    Many(Vec<Atom<'arena>>),
}

#[inline]
fn walk_atom<'arena, S, A>(
    parameter: Atom<'arena>,
    argument: Type<'arena>,
    variance: Variance,
    depth: u32,
    introducing_class: Option<Path<'arena>>,
    symbols: &SymbolTable<'arena, A>,
    state: &mut TemplateState<'arena>,
    options: &StandinOptions,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Walk<'arena>
where
    S: Arena,
    A: Arena,
{
    match parameter.kind() {
        AtomKind::GenericParameter => {
            walk_generic_parameter(parameter, argument, variance, depth, introducing_class, state, options)
        }
        AtomKind::Object => {
            walk_object(parameter, argument, introducing_class, depth, symbols, state, options, builder)
        }
        AtomKind::Reference => {
            walk_reference(parameter, argument, introducing_class, depth, symbols, state, options, builder)
        }
        AtomKind::ClassLikeString => {
            walk_class_like_string(parameter, argument, depth, introducing_class, symbols, state, options, builder)
        }
        AtomKind::ObjectShape => {
            walk_object_shape(parameter, argument, depth, introducing_class, symbols, state, options, builder)
        }
        AtomKind::List => walk_list(parameter, argument, depth, introducing_class, symbols, state, options, builder),
        AtomKind::Array => {
            walk_keyed_array(parameter, argument, depth, introducing_class, symbols, state, options, builder)
        }
        AtomKind::Iterable => {
            walk_iterable(parameter, argument, depth, introducing_class, symbols, state, options, builder)
        }
        AtomKind::Callable => {
            walk_callable(parameter, argument, depth, introducing_class, symbols, state, options, builder)
        }
        _ => Walk::Unchanged,
    }
}

/// `T` against argument `ρ`: record a bound on `T` decorated with the
/// current variance, then emit `T`'s constraint as the refined type.
/// When the constraint is unbounded (`mixed`), the standin keeps the
/// loosest possible witness.
#[inline]
fn walk_generic_parameter<'arena>(
    parameter: Atom<'arena>,
    argument: Type<'arena>,
    variance: Variance,
    depth: u32,
    introducing_class: Option<Path<'arena>>,
    state: &mut TemplateState<'arena>,
    options: &StandinOptions,
) -> Walk<'arena> {
    let Atom::GenericParameter(payload) = parameter else {
        return Walk::Unchanged;
    };

    let key = TemplateKey { defining_entity: payload.defining_entity, name: payload.name };
    if !state.is_declared(key) {
        state.declare(key, payload.constraint);
    }
    let kind = match variance {
        Variance::Covariant => BoundKind::Lower,
        Variance::Contravariant => BoundKind::Upper,
        Variance::Invariant => BoundKind::Equality,
    };
    state.record(
        key,
        Bound {
            kind,
            ty: argument,
            argument_offset: options.argument_offset,
            depth,
            equality_bound_classlike: introducing_class,
            span: options.span,
        },
    );

    match payload.constraint.atoms {
        [single] => Walk::Single(*single),
        atoms => Walk::Many(atoms.to_vec()),
    }
}

/// `Object(C, [τ_i])` against an argument that resolves to a class in
/// `C`'s closure. Same-class arguments walk by position; descendant
/// arguments (`D <: C`) project through
/// [`SymbolTable::inherited_template_argument`] and then substitute `D`'s
/// actual type arguments to recover the type `D` passes for `C`'s
/// `i`-th slot. The variance comes from `C`'s declaration, not `D`'s.
#[inline]
fn walk_object<'arena, S, A>(
    parameter: Atom<'arena>,
    argument: Type<'arena>,
    introducing_class: Option<Path<'arena>>,
    depth: u32,
    symbols: &SymbolTable<'arena, A>,
    state: &mut TemplateState<'arena>,
    options: &StandinOptions,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Walk<'arena>
where
    S: Arena,
    A: Arena,
{
    let Atom::Object(parameter_object) = parameter else {
        return Walk::Unchanged;
    };
    let Some(parameter_arguments) = parameter_object.type_arguments else {
        return Walk::Unchanged;
    };

    match refine_named_generic_arguments(
        parameter_object.name,
        parameter_arguments,
        argument,
        introducing_class,
        depth,
        symbols,
        state,
        options,
        builder,
    ) {
        Some(new_arguments) => {
            let new_argument_slice = builder.types(&new_arguments);
            Walk::Single(builder.object(ObjectAtom { type_arguments: Some(new_argument_slice), ..*parameter_object }))
        }
        None => Walk::Unchanged,
    }
}

/// `Reference(C, [τ_i])` (an unresolved `C<…>` symbol) co-traverses
/// exactly like [`walk_object`]: a `Foo<T>` parameter against a `Foo<int>`
/// argument binds `T` even when `Foo` has not yet been expanded to a
/// structural object. Rebuilds a `Reference` so the un-expanded form is
/// preserved.
#[inline]
fn walk_reference<'arena, S, A>(
    parameter: Atom<'arena>,
    argument: Type<'arena>,
    introducing_class: Option<Path<'arena>>,
    depth: u32,
    symbols: &SymbolTable<'arena, A>,
    state: &mut TemplateState<'arena>,
    options: &StandinOptions,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Walk<'arena>
where
    S: Arena,
    A: Arena,
{
    let Atom::Reference(parameter_reference) = parameter else {
        return Walk::Unchanged;
    };
    let Some(parameter_arguments) = parameter_reference.type_arguments else {
        return Walk::Unchanged;
    };

    match refine_named_generic_arguments(
        parameter_reference.name,
        parameter_arguments,
        argument,
        introducing_class,
        depth,
        symbols,
        state,
        options,
        builder,
    ) {
        Some(new_arguments) => {
            let new_argument_slice = builder.types(&new_arguments);
            Walk::Single(builder.reference(SymbolReferenceAtom {
                name: parameter_reference.name,
                type_arguments: Some(new_argument_slice),
            }))
        }
        None => Walk::Unchanged,
    }
}

/// A name plus optional type-argument list - the shape `Object` and
/// `Reference` atoms share. The standin co-traverses both identically.
#[derive(Clone, Copy)]
struct NamedGenericView<'arena> {
    name: Path<'arena>,
    type_arguments: Option<&'arena [Type<'arena>]>,
}

#[inline]
fn named_generic_view(atom: Atom<'_>) -> Option<NamedGenericView<'_>> {
    match atom {
        Atom::Object(payload) => Some(NamedGenericView { name: payload.name, type_arguments: payload.type_arguments }),
        Atom::Reference(payload) => {
            Some(NamedGenericView { name: payload.name, type_arguments: payload.type_arguments })
        }
        _ => None,
    }
}

/// Co-traverse a named generic container's type arguments against `argument`,
/// recording bounds per position. Shared by [`walk_object`] and
/// [`walk_reference`]. Returns the refined argument list, or `None` when no
/// argument atom names `container_name` (or a descendant) or nothing changed.
#[inline]
fn refine_named_generic_arguments<'arena, S, A>(
    container_name: Path<'arena>,
    parameter_arguments: &'arena [Type<'arena>],
    argument: Type<'arena>,
    introducing_class: Option<Path<'arena>>,
    depth: u32,
    symbols: &SymbolTable<'arena, A>,
    state: &mut TemplateState<'arena>,
    options: &StandinOptions,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Vec<Type<'arena>>>
where
    S: Arena,
    A: Arena,
{
    let argument_view = argument.atoms.iter().copied().find_map(|atom| {
        named_generic_view(atom)
            .filter(|view| view.name == container_name || symbols.descends_from(view.name.id, container_name.id))
    })?;

    let mut new_arguments: Vec<Type<'arena>> = Vec::with_capacity(parameter_arguments.len());
    let mut changed = false;
    for (position, &parameter_argument) in parameter_arguments.iter().enumerate() {
        let projected = projected_named_argument(container_name, argument_view, position, symbols, builder);
        let position_variance = symbols
            .template_parameter_at(container_name.id, position)
            .map_or(Variance::Invariant, |template_parameter| template_parameter.variance);
        let nested_introducing_class = match position_variance {
            Variance::Invariant => Some(container_name),
            Variance::Covariant | Variance::Contravariant => introducing_class,
        };
        let refined = match projected {
            Some(argument_type) => walk_type(
                parameter_argument,
                argument_type,
                position_variance,
                depth + 1,
                nested_introducing_class,
                symbols,
                state,
                options,
                builder,
            ),
            None => parameter_argument,
        };
        if refined != parameter_argument {
            changed = true;
        }
        new_arguments.push(refined);
    }

    if changed { Some(new_arguments) } else { None }
}

/// Pick the type the argument passes to `container_class`'s
/// `position`-th type parameter. For same-class arguments that's just
/// `argument.type_arguments[position]`. For descendant arguments it
/// goes through [`SymbolTable::inherited_template_argument`] and substitutes
/// the argument's own template arguments into the inherited expression.
#[inline]
fn projected_named_argument<'arena, S, A>(
    container_class: Path<'arena>,
    argument_view: NamedGenericView<'arena>,
    position: usize,
    symbols: &SymbolTable<'arena, A>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Type<'arena>>
where
    S: Arena,
    A: Arena,
{
    if argument_view.name == container_class {
        let type_arguments = argument_view.type_arguments?;
        return type_arguments.get(position).copied();
    }

    let inherited = symbols.inherited_template_argument(argument_view.name.id, container_class.id, position)?;

    let actual_arguments: &[Type<'arena>] = argument_view.type_arguments.unwrap_or_default();
    let argument_entity = DefiningEntity::ClassLike(argument_view.name);

    Some(crate::ty::template::substitute(
        inherited,
        &|payload: &GenericParameterAtom<'arena>| -> Option<Type<'arena>> {
            if payload.defining_entity != argument_entity {
                return None;
            }
            let parameter_position = symbols.template_parameter_index(argument_view.name.id, payload.name)?;
            actual_arguments.get(parameter_position).copied()
        },
        builder,
    ))
}

/// `class-string<T>` (a bounded or generic class-like-string whose constraint
/// mentions a template) against a class-string argument: walk the parameter's
/// constraint against the class the argument names, so `class-string<T>` vs
/// `Foo::class` binds `T` to `Foo`. The match is covariant
/// (`class-string<X> <: class-string<Y>` when `X <: Y`).
#[inline]
fn walk_class_like_string<'arena, S, A>(
    parameter: Atom<'arena>,
    argument: Type<'arena>,
    depth: u32,
    introducing_class: Option<Path<'arena>>,
    symbols: &SymbolTable<'arena, A>,
    state: &mut TemplateState<'arena>,
    options: &StandinOptions,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Walk<'arena>
where
    S: Arena,
    A: Arena,
{
    let Atom::ClassLikeString(parameter_class_string) = parameter else {
        return Walk::Unchanged;
    };
    let parameter_constraint = match parameter_class_string.specifier {
        ClassLikeStringSpecifier::OfType { constraint } | ClassLikeStringSpecifier::Generic { constraint } => {
            constraint
        }
        ClassLikeStringSpecifier::Literal { .. } | ClassLikeStringSpecifier::Any => return Walk::Unchanged,
    };

    let argument_subject = argument.atoms.iter().copied().find_map(|atom| match atom {
        Atom::ClassLikeString(argument_class_string) => class_like_string_subject(argument_class_string, builder),
        _ => None,
    });
    let Some(argument_subject) = argument_subject else {
        return Walk::Unchanged;
    };

    let refined = walk_type(
        parameter_constraint,
        argument_subject,
        Variance::Covariant,
        depth + 1,
        introducing_class,
        symbols,
        state,
        options,
        builder,
    );
    if refined == parameter_constraint {
        return Walk::Unchanged;
    }

    let new_specifier = match parameter_class_string.specifier {
        ClassLikeStringSpecifier::OfType { .. } => ClassLikeStringSpecifier::OfType { constraint: refined },
        ClassLikeStringSpecifier::Generic { .. } => ClassLikeStringSpecifier::Generic { constraint: refined },
        other => other,
    };

    Walk::Single(builder.class_like_string(ClassLikeStringAtom { specifier: new_specifier, ..*parameter_class_string }))
}

/// The instance type a class-string argument names: a literal `Foo::class`
/// names `Foo`; a bounded or generic class-string carries its constraint
/// directly; a bare `class-string` names no specific class.
#[inline]
fn class_like_string_subject<'arena, S, A>(
    atom: &ClassLikeStringAtom<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Type<'arena>>
where
    S: Arena,
    A: Arena,
{
    match atom.specifier {
        ClassLikeStringSpecifier::Literal { value } => {
            let object = builder.object_named(value.as_bytes());

            Some(builder.union_of(&[object]))
        }
        ClassLikeStringSpecifier::OfType { constraint } | ClassLikeStringSpecifier::Generic { constraint } => {
            Some(constraint)
        }
        ClassLikeStringSpecifier::Any => None,
    }
}

/// `object{k: T, …}` against `object{k: σ, …}`: walk each parameter property
/// against the argument property of the same name, covariantly - mirroring
/// keyed-array known-item co-traversal.
#[inline]
fn walk_object_shape<'arena, S, A>(
    parameter: Atom<'arena>,
    argument: Type<'arena>,
    depth: u32,
    introducing_class: Option<Path<'arena>>,
    symbols: &SymbolTable<'arena, A>,
    state: &mut TemplateState<'arena>,
    options: &StandinOptions,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Walk<'arena>
where
    S: Arena,
    A: Arena,
{
    let Atom::ObjectShape(parameter_shape) = parameter else {
        return Walk::Unchanged;
    };
    let Some(parameter_properties) = parameter_shape.known_properties else {
        return Walk::Unchanged;
    };

    let argument_properties = argument.atoms.iter().copied().find_map(|atom| match atom {
        Atom::ObjectShape(argument_shape) => Some(argument_shape.known_properties.unwrap_or(&[])),
        _ => None,
    });
    let Some(argument_properties) = argument_properties else {
        return Walk::Unchanged;
    };

    let mut new_entries: Vec<KnownProperty<'arena>> = Vec::with_capacity(parameter_properties.len());
    let mut changed = false;
    for entry in parameter_properties {
        let argument_value =
            argument_properties.iter().find(|candidate| candidate.name == entry.name).map(|candidate| candidate.value);
        let refined = match argument_value {
            Some(argument_value) => walk_type(
                entry.value,
                argument_value,
                Variance::Covariant,
                depth + 1,
                introducing_class,
                symbols,
                state,
                options,
                builder,
            ),
            None => entry.value,
        };
        if refined != entry.value {
            changed = true;
        }
        new_entries.push(KnownProperty { value: refined, ..*entry });
    }

    if !changed {
        return Walk::Unchanged;
    }

    let known_properties = builder.known_properties(&new_entries);
    Walk::Single(builder.object_shape(ObjectShapeAtom { known_properties: Some(known_properties), ..*parameter_shape }))
}

/// `List(τ)` against `List(σ)` or `Iterable(_, σ)`: walk τ vs σ
/// covariantly. The element type's variance is treated as covariant
/// for inference: covariant positions accumulate lower bounds.
#[inline]
fn walk_list<'arena, S, A>(
    parameter: Atom<'arena>,
    argument: Type<'arena>,
    depth: u32,
    introducing_class: Option<Path<'arena>>,
    symbols: &SymbolTable<'arena, A>,
    state: &mut TemplateState<'arena>,
    options: &StandinOptions,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Walk<'arena>
where
    S: Arena,
    A: Arena,
{
    let Atom::List(parameter_list) = parameter else {
        return Walk::Unchanged;
    };

    let argument_element_type = argument.atoms.iter().copied().find_map(|atom| match atom {
        Atom::List(argument_list) => Some(argument_list.element_type),
        Atom::Iterable(argument_iterable) => Some(argument_iterable.value_type),
        _ => None,
    });

    let Some(argument_element_type) = argument_element_type else {
        return Walk::Unchanged;
    };

    let refined = walk_type(
        parameter_list.element_type,
        argument_element_type,
        Variance::Covariant,
        depth + 1,
        introducing_class,
        symbols,
        state,
        options,
        builder,
    );
    if refined == parameter_list.element_type {
        return Walk::Unchanged;
    }

    Walk::Single(builder.list(ListAtom { element_type: refined, ..*parameter_list }))
}

/// `Iterable(τ_K, τ_V)` against `Iterable(σ_K, σ_V)` or `List(σ)`
/// (which exposes `int` keys and `σ` values).
#[inline]
fn walk_iterable<'arena, S, A>(
    parameter: Atom<'arena>,
    argument: Type<'arena>,
    depth: u32,
    introducing_class: Option<Path<'arena>>,
    symbols: &SymbolTable<'arena, A>,
    state: &mut TemplateState<'arena>,
    options: &StandinOptions,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Walk<'arena>
where
    S: Arena,
    A: Arena,
{
    let Atom::Iterable(parameter_iterable) = parameter else {
        return Walk::Unchanged;
    };

    let pair = argument.atoms.iter().copied().find_map(|atom| match atom {
        Atom::Iterable(argument_iterable) => Some((argument_iterable.key_type, argument_iterable.value_type)),
        Atom::List(argument_list) => Some((well_known::TYPE_INT, argument_list.element_type)),
        _ => None,
    });

    let Some((argument_key, argument_value)) = pair else {
        return Walk::Unchanged;
    };

    let new_key = walk_type(
        parameter_iterable.key_type,
        argument_key,
        Variance::Covariant,
        depth + 1,
        introducing_class,
        symbols,
        state,
        options,
        builder,
    );
    let new_value = walk_type(
        parameter_iterable.value_type,
        argument_value,
        Variance::Covariant,
        depth + 1,
        introducing_class,
        symbols,
        state,
        options,
        builder,
    );
    if new_key == parameter_iterable.key_type && new_value == parameter_iterable.value_type {
        return Walk::Unchanged;
    }

    Walk::Single(builder.iterable(IterableAtom { key_type: new_key, value_type: new_value }))
}

/// `Keyed(τ_K, τ_V, {k → τ})` against a keyed-array argument: walk
/// `τ_K` against the argument's key parameter (covariantly), `τ_V`
/// against the value parameter, and each known item against the
/// argument's matching known item. Iterable arguments contribute their
/// key/value to `τ_K` / `τ_V` only - known-item entries don't have a
/// corresponding projection.
#[inline]
fn walk_keyed_array<'arena, S, A>(
    parameter: Atom<'arena>,
    argument: Type<'arena>,
    depth: u32,
    introducing_class: Option<Path<'arena>>,
    symbols: &SymbolTable<'arena, A>,
    state: &mut TemplateState<'arena>,
    options: &StandinOptions,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Walk<'arena>
where
    S: Arena,
    A: Arena,
{
    let Atom::Array(parameter_array) = parameter else {
        return Walk::Unchanged;
    };

    let argument_projection = argument.atoms.iter().copied().find_map(|atom| match atom {
        Atom::Array(argument_array) => Some(KeyedProjection {
            key: argument_array.key_param,
            value: argument_array.value_param,
            known_items: argument_array.known_items,
        }),
        Atom::Iterable(argument_iterable) => Some(KeyedProjection {
            key: Some(argument_iterable.key_type),
            value: Some(argument_iterable.value_type),
            known_items: None,
        }),
        _ => None,
    });
    let Some(argument_projection) = argument_projection else {
        return Walk::Unchanged;
    };

    let new_key = match (parameter_array.key_param, argument_projection.key) {
        (Some(parameter_key), Some(argument_key)) => Some(walk_type(
            parameter_key,
            argument_key,
            Variance::Covariant,
            depth + 1,
            introducing_class,
            symbols,
            state,
            options,
            builder,
        )),
        _ => parameter_array.key_param,
    };
    let new_value = match (parameter_array.value_param, argument_projection.value) {
        (Some(parameter_value), Some(argument_value)) => Some(walk_type(
            parameter_value,
            argument_value,
            Variance::Covariant,
            depth + 1,
            introducing_class,
            symbols,
            state,
            options,
            builder,
        )),
        _ => parameter_array.value_param,
    };

    let new_known_items = parameter_array.known_items.map(|parameter_entries| {
        let argument_entries: &[KnownItem<'arena>] = argument_projection.known_items.unwrap_or(&[]);
        let mut new_entries: Vec<KnownItem<'arena>> = Vec::with_capacity(parameter_entries.len());
        let mut changed_inner = false;
        for entry in parameter_entries {
            let argument_value = argument_entries
                .iter()
                .find(|argument_entry| argument_entry.key == entry.key)
                .map(|argument_entry| argument_entry.value);
            let refined_value = match argument_value {
                Some(argument_value) => walk_type(
                    entry.value,
                    argument_value,
                    Variance::Covariant,
                    depth + 1,
                    introducing_class,
                    symbols,
                    state,
                    options,
                    builder,
                ),
                None => entry.value,
            };
            if refined_value != entry.value {
                changed_inner = true;
            }
            new_entries.push(KnownItem { value: refined_value, ..*entry });
        }
        if changed_inner { (builder.known_items(&new_entries), true) } else { (parameter_entries, false) }
    });

    let key_changed = new_key != parameter_array.key_param;
    let value_changed = new_value != parameter_array.value_param;
    let known_changed = new_known_items.is_some_and(|(_, changed_inner)| changed_inner);
    if !key_changed && !value_changed && !known_changed {
        return Walk::Unchanged;
    }

    Walk::Single(builder.array(ArrayAtom {
        key_param: new_key,
        value_param: new_value,
        known_items: new_known_items.map(|(entries, _)| entries),
        ..*parameter_array
    }))
}

struct KeyedProjection<'arena> {
    key: Option<Type<'arena>>,
    value: Option<Type<'arena>>,
    known_items: Option<&'arena [KnownItem<'arena>]>,
}

/// `Callable(Sig(s_p))` against a callable argument: walk parameter
/// types pointwise contravariantly and the return type covariantly.
/// Aliases and `Any` callables pass through.
#[inline]
fn walk_callable<'arena, S, A>(
    parameter: Atom<'arena>,
    argument: Type<'arena>,
    depth: u32,
    introducing_class: Option<Path<'arena>>,
    symbols: &SymbolTable<'arena, A>,
    state: &mut TemplateState<'arena>,
    options: &StandinOptions,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Walk<'arena>
where
    S: Arena,
    A: Arena,
{
    let Atom::Callable(parameter_callable) = parameter else {
        return Walk::Unchanged;
    };
    let (CallableAtom::Signature(parameter_signature) | CallableAtom::Closure(parameter_signature)) =
        parameter_callable
    else {
        return Walk::Unchanged;
    };

    let argument_signature = argument.atoms.iter().copied().find_map(|atom| match atom {
        Atom::Callable(CallableAtom::Signature(signature) | CallableAtom::Closure(signature)) => Some(signature),
        _ => None,
    });
    let Some(argument_signature) = argument_signature else {
        return Walk::Unchanged;
    };

    let new_return = walk_type(
        parameter_signature.return_type,
        argument_signature.return_type,
        Variance::Covariant,
        depth + 1,
        introducing_class,
        symbols,
        state,
        options,
        builder,
    );

    let new_parameter_list = parameter_signature.parameters.map(|parameter_entries| {
        let argument_entries: &[Parameter<'arena>] = argument_signature.parameters.unwrap_or(&[]);
        let mut new_entries: Vec<Parameter<'arena>> = Vec::with_capacity(parameter_entries.len());
        let mut changed_inner = false;
        for (position, parameter_entry) in parameter_entries.iter().enumerate() {
            let argument_type = argument_entries.get(position).map(|argument_entry| argument_entry.r#type);
            let refined = match argument_type {
                Some(argument_type) => walk_type(
                    parameter_entry.r#type,
                    argument_type,
                    Variance::Contravariant,
                    depth + 1,
                    introducing_class,
                    symbols,
                    state,
                    options,
                    builder,
                ),
                None => parameter_entry.r#type,
            };
            if refined != parameter_entry.r#type {
                changed_inner = true;
            }
            new_entries.push(Parameter { r#type: refined, ..*parameter_entry });
        }
        if changed_inner { (builder.parameters(&new_entries), true) } else { (parameter_entries, false) }
    });

    let return_changed = new_return != parameter_signature.return_type;
    let parameters_changed = new_parameter_list.is_some_and(|(_, changed_inner)| changed_inner);
    if !return_changed && !parameters_changed {
        return Walk::Unchanged;
    }

    let new_signature = builder.signature(Signature {
        return_type: new_return,
        parameters: new_parameter_list.map(|(entries, _)| entries).or(parameter_signature.parameters),
        ..*parameter_signature
    });
    let new_callable = match parameter_callable {
        CallableAtom::Signature(_) => CallableAtom::Signature(new_signature),
        CallableAtom::Closure(_) => CallableAtom::Closure(new_signature),
        _ => return Walk::Unchanged,
    };

    Walk::Single(Atom::Callable(new_callable))
}

/// Pass the entire argument through to the next walk. Refinements like
/// "pick the Object atom whose name matches" happen inside each
/// per-atom handler, not here.
#[inline]
const fn project_argument<'arena>(_parameter: Atom<'arena>, argument: Type<'arena>) -> Type<'arena> {
    argument
}
