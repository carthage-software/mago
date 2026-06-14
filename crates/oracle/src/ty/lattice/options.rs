use mago_flags::U16Flags;

use crate::ty::flags::FlowFlag;

/// Caller-controlled options for the lattice operations
/// ([`refines`](crate::ty::lattice::refines),
/// [`generalizes`](crate::ty::lattice::generalizes),
/// [`overlaps`](crate::ty::lattice::overlaps)).
///
/// Each field tweaks the algorithm without changing its meaning at the type
/// level. Defaults are all `false`. Use [`LatticeOptions::default`] for the
/// common case and chain `with_*` builders for any flags you need; or derive
/// options from a value's flow flags via [`LatticeOptions::of_flags`] /
/// [`LatticeOptions::assertion_of_flags`].
///
/// `LatticeOptions` is `Copy` and small enough to pass by value; the
/// operations take it that way.
#[derive(Debug, Clone, Copy, Default)]
pub struct LatticeOptions {
    /// Skip `null` atoms in the input union when refining. Used by
    /// nullsafe-aware analyzers: a `?int` argument can be passed to an
    /// `int` parameter under this flag without a "null leak" diagnostic.
    pub ignore_null: bool,
    /// Skip the `false` atom in the input union when refining. Used by
    /// `int|false` style return values that the caller has narrowed away
    /// from `false`.
    pub ignore_false: bool,
    /// The refinement is being checked inside a runtime assertion (e.g.
    /// `assert($x instanceof Foo)`). Some rules become more permissive in
    /// this mode.
    pub inside_assertion: bool,
    /// Treat an unpinned generic type argument (a raw `Foo` where the
    /// declaration has parameters, e.g. `Foo` against `Foo<int>`) as
    /// compatible with any pinned argument, recording
    /// [`CoercionCause::TemplateDefault`](crate::ty::lattice::CoercionCause::TemplateDefault).
    /// This is the gradual-typing posture the checker uses for raw generics.
    ///
    /// It is **off by default** because the leniency is unsound as a lattice
    /// relation: under an invariant parameter it would let `meet` keep
    /// `Foo<int>` as a lower bound of `Foo<mixed>` even though `Foo<int>` is
    /// not a `Foo<mixed>`, breaking the meet/subtract laws. The lattice
    /// operations and their property tests run with it off (strict, sound);
    /// assignment-compatibility checks opt in.
    pub template_default_coercion: bool,
}

impl LatticeOptions {
    /// Derive options from a value's flow flags: `ignore_null` mirrors
    /// [`FlowFlag::IgnoreNullableIssues`] and `ignore_false` mirrors
    /// [`FlowFlag::IgnoreFalsableIssues`]. `inside_assertion` stays
    /// `false`.
    #[inline]
    #[must_use]
    pub fn of_flags(flags: U16Flags<FlowFlag>) -> Self {
        Self {
            ignore_null: flags.contains(FlowFlag::IgnoreNullableIssues),
            ignore_false: flags.contains(FlowFlag::IgnoreFalsableIssues),
            inside_assertion: false,
            template_default_coercion: false,
        }
    }

    /// Same as [`of_flags`](Self::of_flags), but with `inside_assertion`
    /// set.
    #[inline]
    #[must_use]
    pub fn assertion_of_flags(flags: U16Flags<FlowFlag>) -> Self {
        Self::of_flags(flags).inside_assertion()
    }

    /// Set [`ignore_null`](Self::ignore_null) to `true`.
    #[must_use]
    #[inline]
    pub const fn with_ignore_null(mut self) -> Self {
        self.ignore_null = true;
        self
    }

    /// Set [`ignore_false`](Self::ignore_false) to `true`.
    #[must_use]
    #[inline]
    pub const fn with_ignore_false(mut self) -> Self {
        self.ignore_false = true;
        self
    }

    /// Set [`inside_assertion`](Self::inside_assertion) to `true`.
    #[must_use]
    #[inline]
    pub const fn inside_assertion(mut self) -> Self {
        self.inside_assertion = true;
        self
    }

    /// Set [`template_default_coercion`](Self::template_default_coercion) to
    /// `true`.
    #[must_use]
    #[inline]
    pub const fn with_template_default_coercion(mut self) -> Self {
        self.template_default_coercion = true;
        self
    }
}
