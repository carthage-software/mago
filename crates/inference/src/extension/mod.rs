//! Extension points for steering inference from outside the crate.
//!
//! There are exactly two: [`ExtensionInference`] supplies a better type for an
//! already-typed expression (framework container lookups, alternate platform
//! semantics), and [`ExtensionAssertion`] extracts the facts an expression
//! establishes about variables (so narrowing can use knowledge the pure core
//! does not have, e.g. `is_string($x)`). Both are opt-in: with no extensions
//! enabled the core stays pure and pays nothing (the registry is skipped).

use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_hir::ir::expression::Expression;
use mago_oracle::assertion::Assertion;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_oracle::var::Var;

use crate::flow::Flow;

pub mod assertion;
pub mod context;
pub mod inference;
pub mod semantics;

pub use context::ExtensionContext;

/// When an assertion holds relative to the truth value of the expression it was
/// extracted from — mirroring `@assert` / `@assert-if-true` / `@assert-if-false`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssertionTiming {
    /// Holds unconditionally once the expression has been evaluated.
    Always,
    /// Holds on the branch where the expression is truthy.
    WhenTrue,
    /// Holds on the branch where the expression is falsy.
    WhenFalse,
}

/// Where an [`ExtensionAssertion`] deposits the facts it extracts. A thin handle
/// over the inference scratch buffer — concrete, so `push` inlines.
pub struct AssertionSink<'sink, 'source, 'arena, A: Arena> {
    entries: &'sink mut Vec<'source, (Var<'arena>, Assertion<'arena>, AssertionTiming), A>,
}

impl<'sink, 'source, 'arena, A: Arena> AssertionSink<'sink, 'source, 'arena, A> {
    pub(crate) fn new(entries: &'sink mut Vec<'source, (Var<'arena>, Assertion<'arena>, AssertionTiming), A>) -> Self {
        Self { entries }
    }

    pub fn push(&mut self, variable: Var<'arena>, assertion: Assertion<'arena>, timing: AssertionTiming) {
        self.entries.push((variable, assertion, timing));
    }
}

/// Supplies a better type for an expression after it has been inferred.
///
/// Runs once per expression with the children already typed; returning
/// `Some(type)` replaces the expression's inferred type, `None` leaves it.
pub trait ExtensionInference<A: Arena>: Send + Sync {
    fn infer<'arena>(
        &self,
        context: &mut ExtensionContext<'_, '_, 'arena, A>,
        expression: &Expression<'arena, SymbolId, Flow, Type<'arena>>,
    ) -> Option<Type<'arena>>;
}

/// Extracts the assertions an expression establishes about variables.
///
/// Runs once per expression with the children already typed; each assertion is
/// tagged with the [`AssertionTiming`] under which it holds.
pub trait ExtensionAssertion<A: Arena>: Send + Sync {
    fn assertions<'ctx, 'source, 'arena>(
        &self,
        context: &mut ExtensionContext<'ctx, 'source, 'arena, A>,
        expression: &Expression<'arena, SymbolId, Flow, Type<'arena>>,
        out: &mut AssertionSink<'ctx, 'source, 'arena, A>,
    );
}

/// The extensions enabled for an inference run. The two slices are the only
/// `dyn` in the design — a heterogeneous, caller-assembled registry — and are
/// consulted only when non-empty.
pub struct Extensions<'ext, A: Arena> {
    pub inference: &'ext [&'ext dyn ExtensionInference<A>],
    pub assertion: &'ext [&'ext dyn ExtensionAssertion<A>],
}

impl<'ext, A: Arena> Extensions<'ext, A> {
    #[must_use]
    pub fn new(
        inference: &'ext [&'ext dyn ExtensionInference<A>],
        assertion: &'ext [&'ext dyn ExtensionAssertion<A>],
    ) -> Self {
        Self { inference, assertion }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inference.is_empty() && self.assertion.is_empty()
    }
}

impl<A: Arena> std::fmt::Debug for Extensions<'_, A> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Extensions")
            .field("inference", &self.inference.len())
            .field("assertion", &self.assertion.len())
            .finish()
    }
}

impl<A: Arena> Clone for Extensions<'_, A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: Arena> Copy for Extensions<'_, A> {}

impl<A: Arena> Default for Extensions<'_, A> {
    fn default() -> Self {
        Self { inference: &[], assertion: &[] }
    }
}
