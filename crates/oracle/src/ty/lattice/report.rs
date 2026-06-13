use mago_flags::U8Flags;

use crate::ty::atom::Atom;
use crate::ty::template::Bound;
use crate::ty::template::TemplateKey;
use crate::ty::Type;

/// Diagnostic output from the lattice operations.
///
/// Produced by [`refines`](crate::ty::lattice::refines),
/// [`generalizes`](crate::ty::lattice::generalizes),
/// [`overlaps`](crate::ty::lattice::overlaps), the meet, and the subtraction.
///
/// Operations return `bool` / [`Type`]; this struct carries the *why*.
/// Callers pass `&mut LatticeReport` and read the fields after the call.
///
/// Empty `bounds` (the default) is allocation-free; the allocation is paid
/// only when a rule pushes a bound.
#[derive(Debug, Clone, Default)]
pub struct LatticeReport<'arena> {
    /// The set of coercion patterns the operation observed. Multiple flags
    /// may be set in a single call (e.g. a nested-mixed input that also
    /// triggered a true-union narrowing).
    pub causes: U8Flags<CoercionCause>,
    /// The smallest *type* that, substituted for the input, would have made
    /// the comparison succeed cleanly (no coercion). `None` when no rule
    /// could compute one. Use this when reporting on the whole union.
    pub replacement: Option<Type<'arena>>,
    /// The single problematic *atom* in the input, when only one atom of a
    /// wider union was at fault. The reconciler swaps just this atom rather
    /// than rebuilding the entire union. `None` when the issue is at the
    /// union level (or no replacement is computable).
    pub replacement_atom: Option<Atom<'arena>>,
    /// Bounds on free template parameters that surfaced during the
    /// comparison itself (distinct from a separate inference pass through
    /// the standin walk). Each entry tags which template the bound applies
    /// to and whether it is a Lower / Upper / Equality constraint. Empty in
    /// the common case.
    pub bounds: Vec<(TemplateKey<'arena>, Bound<'arena>)>,
}

impl<'arena> LatticeReport<'arena> {
    /// A fresh report with no causes, no replacements, and no bounds.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a coercion cause without disturbing other fields.
    #[inline]
    pub fn add_cause(&mut self, cause: CoercionCause) {
        self.causes.set(cause);
    }

    /// Record a union-level replacement; subsequent calls overwrite.
    #[inline]
    pub const fn set_replacement(&mut self, ty: Type<'arena>) {
        self.replacement = Some(ty);
    }

    /// Record an atom-level replacement; subsequent calls overwrite.
    #[inline]
    pub const fn set_replacement_atom(&mut self, atom: Atom<'arena>) {
        self.replacement_atom = Some(atom);
    }

    /// Record a bound surfaced for `key`.
    #[inline]
    pub fn push_bound(&mut self, key: TemplateKey<'arena>, bound: Bound<'arena>) {
        self.bounds.push((key, bound));
    }

    /// `true` iff at least one coercion cause was recorded.
    #[inline]
    #[must_use]
    pub const fn coerced(&self) -> bool {
        !self.causes.is_empty()
    }
}

/// One coercion pattern the lattice observed during a comparison, carried
/// as a `U8Flags<CoercionCause>` on [`LatticeReport::causes`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum CoercionCause {
    /// The input contained a `mixed` somewhere that the container
    /// constrained: e.g. `array<string, mixed>` flowing into
    /// `array<string, int>`. Distinct from a top-level `mixed` because the
    /// programmer's mental model is "I have an array of values" rather
    /// than "I have a top-typed thing".
    NestedMixed = 1 << 0,
    /// The input was a generic parameter whose constraint is `mixed`
    /// (`@template T` with no `of` clause, or `@template T of mixed`).
    /// Distinct from [`NestedMixed`](Self::NestedMixed): the fix
    /// suggestion is "tighten the template constraint", not "stop shoving
    /// `mixed` into the container".
    FromAsMixed = 1 << 1,
    /// A "true union" atom kind (`mixed`, `array_key`, `bool`,
    /// `object_any`, `scalar`, `numeric`) was narrowed to one of its
    /// concrete subforms by the container. The standard PHP pattern: the
    /// input *could* be the right thing at runtime, but the type system
    /// can't prove it.
    TrueUnionNarrow = 1 << 2,
    /// PHP's runtime would coerce the input to fit (e.g. `int -> float`).
    /// Distinct from the other causes because the coercion is silent at
    /// runtime: the programmer may not realise it happened.
    PhpRuntimeCoerce = 1 << 3,
    /// A literal-shaped value was accepted where its general form was
    /// expected, or vice versa. Reserved for rules that promote
    /// `Literal(5)` into `int` or accept `int` into `literal-int` slots.
    LiteralPromoted = 1 << 4,
    /// A generic position was filled with its declared default rather than
    /// an explicit type-argument. Variance checks must skip the reverse
    /// direction for default-filled positions.
    TemplateDefault = 1 << 5,
    /// `object` (the unspecified-class atom) was accepted where a concrete
    /// class was expected. Unsound in general; the consumer may want to
    /// warn.
    ObjectAnyDown = 1 << 6,
}

impl From<CoercionCause> for u8 {
    fn from(cause: CoercionCause) -> Self {
        cause as u8
    }
}
