//! Lattice join (least upper bound) of atom multisets.
//!
//! [`compute`] takes a slice of [`Atom`]s and returns the canonical
//! multiset that the corresponding union should hold. The pass is purely
//! structural: it inspects atom identity and kind tags only, never
//! consults the lattice machinery, and so can run without any
//! subtype-driven information.
//!
//! In type-lattice terms, `compute(atoms)` is the least upper bound
//! (join, ⊔) of the atom multiset under the subtype order. Pairs with
//! [`crate::ty::meet`] (greatest lower bound, ⊓).
//!
//! # Why join is separate from union construction
//!
//! The join preserves the subtype order. For any unions `A`, `B`:
//!
//! ```text
//! A ≤ B  ⟺  compute(A) ≤ B  ⟺  compute(A) ≤ compute(B)  ⟺  A ≤ compute(B)
//! ```
//!
//! That property is what lets the builder store unions in whatever shape
//! the caller hands in (sorted + deduplicated, but not otherwise canonical),
//! and the lattice answer refinement questions correctly on either side.
//! Calling [`compute`] is therefore an optional optimization for size and
//! readability, never a precondition for soundness.
//!
//! # What this pass does
//!
//! - Drops `never` when any non-`never` atom exists; collapses an
//!   all-`never` multiset to `[never]`. Replaces `void` with `null` in
//!   any union of length > 1 (PHP semantics: callers see `null` from a
//!   `void` function); preserves `void` when it is the only atom.
//! - Lets vanilla `mixed` absorb every other atom.
//! - Merges `true ∨ false → bool`; lets `bool` absorb `true` / `false`.
//! - Lets `resource` absorb `open-resource` / `closed-resource`; merges
//!   `open-resource ∨ closed-resource → resource` when neither is dominated.
//! - Lets a same-kind dominator (`int`, `float`, `string`, `resource`,
//!   `callable`) absorb every other atom of its kind.
//! - Lets `object` absorb the entire object family (named objects, enums,
//!   shapes, has-method, has-property).
//!
//! Family-specific payload merges (range merging, string-axis merging,
//! list / keyed-array element-type unions, scalar synthesis, mixed
//! constraint joining, literal-count and shape-count thresholds) live
//! in the family submodules and are gated per [`JoinOptions`] toggle.

mod family;

use mago_allocator::Arena;

use crate::ty::atom::Atom;
use crate::ty::atom::kind::AtomKind;
use crate::ty::builder::TypeBuilder;
use crate::ty::lattice::CoercionCause;
use crate::ty::lattice::LatticeOptions;
use crate::ty::lattice::LatticeReport;
use crate::ty::lattice::atom_refines;
use crate::ty::well_known::BOOL;
use crate::ty::well_known::CALLABLE;
use crate::ty::well_known::CLOSED_RESOURCE;
use crate::ty::well_known::FALSE;
use crate::ty::well_known::FLOAT;
use crate::ty::well_known::INT;
use crate::ty::well_known::MIXED;
use crate::ty::well_known::NEVER;
use crate::ty::well_known::NULL;
use crate::ty::well_known::OBJECT;
use crate::ty::well_known::OPEN_RESOURCE;
use crate::ty::well_known::RESOURCE;
use crate::ty::well_known::STRING;
use crate::ty::well_known::TRUE;
use crate::ty::well_known::VOID;
use crate::world::NullWorld;

/// Compute the join (least upper bound) of a slice of atoms with the
/// canonical preset.
///
/// [`JoinOptions::default`] enables payload-level merges (range merging,
/// string-axis merging, scalar synthesis, list and keyed-array
/// element-type unions), subtype-driven absorption, and the standard
/// 128 / 32 literal/shape thresholds. Use [`compute_with`] with
/// [`JoinOptions::structural`] for sort + dedup only.
///
/// Returns a freshly-allocated, sorted, deduplicated [`Vec`]. Empty
/// input collapses to `[NEVER]` so callers always receive a non-empty
/// multiset suitable for [`Type`] construction.
///
/// [`Type`]: crate::Type
#[inline]
#[must_use]
pub fn compute<'arena, S, A>(atoms: &[Atom<'arena>], builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Vec<Atom<'arena>>
where
    S: Arena,
    A: Arena,
{
    compute_with(atoms, &JoinOptions::default(), builder)
}

/// Compute the join with caller-controlled extended rules per
/// [`JoinOptions`].
///
/// The structural pass runs first; each extended rule fires after,
/// gated on its own option, so the analyzer can pick the simplification
/// aggressiveness per call site.
#[inline]
#[must_use]
pub fn compute_with<'arena, S, A>(
    atoms: &[Atom<'arena>],
    options: &JoinOptions,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Vec<Atom<'arena>>
where
    S: Arena,
    A: Arena,
{
    if atoms.is_empty() {
        return vec![NEVER];
    }

    if atoms.iter().any(|atom| atom.kind() == AtomKind::Mixed)
        && let Some(mixed_result) = family::mixed::apply_mixed_constraint_join(atoms)
    {
        return vec![mixed_result];
    }

    let mut output: Vec<Atom<'arena>> =
        if options.merge_string_axes && atoms.iter().filter(|atom| atom.kind() == AtomKind::String).count() >= 2 {
            family::string::apply_string_axis_merge_in_order(atoms, builder)
        } else {
            atoms.to_vec()
        };
    output.sort_unstable();
    output.dedup();
    canonicalize(&mut output);

    if options.overwrite_empty_array {
        family::array::apply_overwrite_empty_array(&mut output);
    }
    if let Some(threshold) = options.int_literal_collapse_threshold {
        family::int::apply_int_literal_collapse(&mut output, threshold);
    }
    if let Some(threshold) = options.string_literal_collapse_threshold {
        family::string::apply_string_literal_collapse(&mut output, threshold);
    }
    if let Some(threshold) = options.float_literal_collapse_threshold {
        family::float::apply_float_literal_collapse(&mut output, threshold);
    }
    if let Some(threshold) = options.array_shape_collapse_threshold {
        family::array::apply_array_shape_collapse(&mut output, threshold);
    }
    if options.merge_int_ranges {
        family::int::apply_merge_int_ranges(&mut output, builder);
    }
    if options.absorb_refinements {
        apply_subtype_absorption(&mut output, builder);
    }
    if options.synthesise_scalar {
        family::scalar::apply_scalar_synthesis(&mut output);
    }
    if options.merge_array_shapes {
        family::array::apply_merge_array_shapes(&mut output, builder);
    }
    if options.merge_list_element_types {
        family::list::apply_merge_list_element_types(&mut output, builder);
        family::list::apply_merge_sealed_lists(&mut output, builder);
    }
    if options.merge_keyed_array_params {
        family::array::apply_merge_keyed_array_params(&mut output, builder);
    }
    if options.rewrite_int_keyed_to_list {
        family::array::apply_rewrite_int_keyed_to_list(&mut output, builder);
    }

    output.sort_unstable();
    output.dedup();
    output
}

/// Caller-controlled toggles for [`compute_with`].
///
/// [`Default`] returns the canonical preset: every payload-level
/// merge rule on, with the standard 128-literal / 32-array
/// thresholds, so a plain [`compute`] call gives the lattice-canonical
/// form. Use [`JoinOptions::structural`] when you want a single rule
/// in isolation (typical for option-coverage tests) or to skip the
/// payload-level work for callers that only need sort + dedup +
/// same-kind dominator.
#[derive(Debug, Clone, Copy)]
pub struct JoinOptions {
    /// Merge adjacent integer literals and ranges into wider ranges
    /// (e.g. `0 | 1 | 2` → `int<0, 2>`). Touches Int-kind atoms only.
    pub merge_int_ranges: bool,
    /// When the union contains more than this many distinct integer
    /// literals, drop them and add the general `int` form. `None`
    /// disables; `Some(0)` always collapses if any literals are present.
    pub int_literal_collapse_threshold: Option<u16>,
    /// When the union contains more than this many distinct `string`
    /// literals, drop them and add the general `string` form.
    pub string_literal_collapse_threshold: Option<u16>,
    /// When the union contains more than this many distinct `float`
    /// literals, drop them and add the general `float` form.
    pub float_literal_collapse_threshold: Option<u16>,
    /// When the union contains more than this many array shapes (keyed
    /// or list), collapse them to the general `array` form.
    pub array_shape_collapse_threshold: Option<u16>,
    /// Detect keyed-array shapes whose keys are `0..n-1` integers and
    /// rewrite them as `list` shapes.
    pub rewrite_int_keyed_to_list: bool,
    /// Merge multiple keyed-array shapes that share at least one key
    /// into a single shape with per-key value unions.
    pub merge_array_shapes: bool,
    /// Drop `EMPTY_ARRAY` from the union when another `Array` or `List`
    /// atom is present.
    pub overwrite_empty_array: bool,
    /// Apply subtype-driven absorption (refined int ranges, refined
    /// string axes, family hierarchy: numeric/scalar/array-key).
    pub absorb_refinements: bool,
    /// Merge same-kind strings via the AND-of-flags algebra (e.g.
    /// `lower | upper → string`, `non_empty | lit("") → string`,
    /// `truthy | lit("0") → string`). Compatible literals are absorbed
    /// into the merged refined form; incompatible literals stay separate.
    pub merge_string_axes: bool,
    /// Collapse `int | string | float | bool` to `scalar` once all four
    /// general primitives are present in the union.
    pub synthesise_scalar: bool,
    /// Merge multiple unsealed lists with the same non-empty flag into
    /// a single list whose element type is the union of theirs (e.g.
    /// `list<int> | list<string> → list<int|string>`).
    pub merge_list_element_types: bool,
    /// Same merge for unsealed keyed arrays (`array<K1, V1> | array<K2, V2>
    /// → array<K1|K2, V1|V2>`).
    pub merge_keyed_array_params: bool,
}

impl Default for JoinOptions {
    /// The canonical preset: every payload-level merge / absorption rule
    /// enabled, standard literal thresholds (128 ints / 128 strings /
    /// 128 floats / 32 array shapes), `overwrite_empty_array` and
    /// `rewrite_int_keyed_to_list` left off (they change the *shape* of
    /// the output, not just collapse equivalent forms, so they remain
    /// opt-in).
    #[inline]
    fn default() -> Self {
        Self {
            merge_int_ranges: true,
            int_literal_collapse_threshold: Some(128),
            string_literal_collapse_threshold: Some(128),
            float_literal_collapse_threshold: Some(128),
            array_shape_collapse_threshold: Some(32),
            rewrite_int_keyed_to_list: false,
            merge_array_shapes: true,
            overwrite_empty_array: false,
            absorb_refinements: true,
            merge_string_axes: true,
            synthesise_scalar: true,
            merge_list_element_types: true,
            merge_keyed_array_params: true,
        }
    }
}

impl JoinOptions {
    /// All payload-level rules off, all thresholds disabled. The
    /// resulting [`compute_with`] call performs only the structural
    /// canonicalisation (sort, dedup, same-kind dominator absorption,
    /// `void | null → null`, `true | false → bool`). Useful for testing
    /// a single rule in isolation.
    #[inline]
    #[must_use]
    pub const fn structural() -> Self {
        Self {
            merge_int_ranges: false,
            int_literal_collapse_threshold: None,
            string_literal_collapse_threshold: None,
            float_literal_collapse_threshold: None,
            array_shape_collapse_threshold: None,
            rewrite_int_keyed_to_list: false,
            merge_array_shapes: false,
            overwrite_empty_array: false,
            absorb_refinements: false,
            merge_string_axes: false,
            synthesise_scalar: false,
            merge_list_element_types: false,
            merge_keyed_array_params: false,
        }
    }

    #[must_use]
    #[inline]
    pub const fn with_merge_int_ranges(mut self, on: bool) -> Self {
        self.merge_int_ranges = on;
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_int_literal_collapse_threshold(mut self, threshold: u16) -> Self {
        self.int_literal_collapse_threshold = Some(threshold);
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_string_literal_collapse_threshold(mut self, threshold: u16) -> Self {
        self.string_literal_collapse_threshold = Some(threshold);
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_float_literal_collapse_threshold(mut self, threshold: u16) -> Self {
        self.float_literal_collapse_threshold = Some(threshold);
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_array_shape_collapse_threshold(mut self, threshold: u16) -> Self {
        self.array_shape_collapse_threshold = Some(threshold);
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_rewrite_int_keyed_to_list(mut self, on: bool) -> Self {
        self.rewrite_int_keyed_to_list = on;
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_merge_array_shapes(mut self, on: bool) -> Self {
        self.merge_array_shapes = on;
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_overwrite_empty_array(mut self, on: bool) -> Self {
        self.overwrite_empty_array = on;
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_absorb_refinements(mut self, on: bool) -> Self {
        self.absorb_refinements = on;
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_merge_string_axes(mut self, on: bool) -> Self {
        self.merge_string_axes = on;
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_synthesise_scalar(mut self, on: bool) -> Self {
        self.synthesise_scalar = on;
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_merge_list_element_types(mut self, on: bool) -> Self {
        self.merge_list_element_types = on;
        self
    }

    #[must_use]
    #[inline]
    pub const fn with_merge_keyed_array_params(mut self, on: bool) -> Self {
        self.merge_keyed_array_params = on;
        self
    }
}

/// Drop any atom absorbed by another structurally-larger atom in the
/// same multiset (`a <: b` ⇒ drop `a`). Uses the lattice's atom
/// refinement check with [`NullWorld`], so only purely-structural rules
/// fire. Coercion-driven refinements (e.g. `int <: float` via PHP's
/// runtime int-to-float coercion) do **not** drive absorption: keeping
/// `int|float` distinct preserves the information that the value was
/// originally typed as `int`.
#[inline]
fn apply_subtype_absorption<'arena, S, A>(atoms: &mut Vec<Atom<'arena>>, builder: &mut TypeBuilder<'_, 'arena, S, A>)
where
    S: Arena,
    A: Arena,
{
    if atoms.len() < 2 {
        return;
    }

    let world = NullWorld;
    let options = LatticeOptions::default();
    let mut absorbed = vec![false; atoms.len()];

    for input_index in 0..atoms.len() {
        if absorbed[input_index] {
            continue;
        }

        for container_index in 0..atoms.len() {
            if input_index == container_index || absorbed[container_index] {
                continue;
            }

            let mut report = LatticeReport::new();
            if atom_refines(atoms[input_index], atoms[container_index], &world, options, &mut report, builder)
                && !report.causes.contains(CoercionCause::PhpRuntimeCoerce)
            {
                absorbed[input_index] = true;
                break;
            }
        }
    }

    let mut index = 0;
    atoms.retain(|_| {
        let keep = !absorbed[index];
        index += 1;
        keep
    });
}

/// Apply the structural canonicalization rules. `atoms` must be sorted
/// and deduplicated on entry; sorted order is preserved on exit.
#[inline]
fn canonicalize(atoms: &mut Vec<Atom<'_>>) {
    if atoms.contains(&MIXED) {
        atoms.clear();
        atoms.push(MIXED);
        return;
    }

    if atoms.contains(&NEVER) && atoms.iter().any(|atom| *atom != NEVER) {
        atoms.retain(|atom| *atom != NEVER);
    }

    if atoms.contains(&VOID) && atoms.len() > 1 {
        atoms.retain(|atom| *atom != VOID);
        if !atoms.contains(&NULL) {
            let position = atoms.binary_search(&NULL).unwrap_or_else(|insertion| insertion);
            atoms.insert(position, NULL);
        }
    }

    let has_bool = atoms.contains(&BOOL);
    let has_true = atoms.contains(&TRUE);
    let has_false = atoms.contains(&FALSE);

    if has_bool {
        atoms.retain(|atom| *atom != TRUE && *atom != FALSE);
    } else if has_true && has_false {
        atoms.retain(|atom| *atom != TRUE && *atom != FALSE);
        let position = atoms.binary_search(&BOOL).unwrap_or_else(|insertion| insertion);
        atoms.insert(position, BOOL);
    }

    let has_open_resource = atoms.contains(&OPEN_RESOURCE);
    let has_closed_resource = atoms.contains(&CLOSED_RESOURCE);
    let has_resource = atoms.contains(&RESOURCE);
    if has_open_resource && has_closed_resource && !has_resource {
        atoms.retain(|atom| *atom != OPEN_RESOURCE && *atom != CLOSED_RESOURCE);
        let position = atoms.binary_search(&RESOURCE).unwrap_or_else(|insertion| insertion);
        atoms.insert(position, RESOURCE);
    }

    apply_same_kind_dominator(atoms, INT);
    apply_same_kind_dominator(atoms, FLOAT);
    apply_same_kind_dominator(atoms, STRING);
    apply_same_kind_dominator(atoms, RESOURCE);
    apply_same_kind_dominator(atoms, CALLABLE);

    if atoms.contains(&OBJECT) {
        atoms.retain(|atom| *atom == OBJECT || !is_object_family_kind(atom.kind()));
    }
}

/// If `dominator` is in `atoms`, drop every other atom of the same
/// kind (the dominator is the unrefined / top-of-its-family form).
#[inline]
fn apply_same_kind_dominator<'arena>(atoms: &mut Vec<Atom<'arena>>, dominator: Atom<'arena>) {
    if !atoms.contains(&dominator) {
        return;
    }

    let kind = dominator.kind();
    atoms.retain(|atom| *atom == dominator || atom.kind() != kind);
}

/// `true` for the kinds that all sit under `ObjectAny` and are absorbed by
/// it: named objects, enums (including specific cases), object shapes,
/// has-method / has-property narrowings.
#[inline]
const fn is_object_family_kind(kind: AtomKind) -> bool {
    matches!(
        kind,
        AtomKind::Object | AtomKind::Enum | AtomKind::ObjectShape | AtomKind::HasMethod | AtomKind::HasProperty
    )
}
