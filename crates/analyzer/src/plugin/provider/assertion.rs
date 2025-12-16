//! Assertion providers for function and method calls.
//!
//! These providers allow plugins to specify additional type assertions that
//! should be applied after a function or method call.

use std::collections::BTreeMap;

use mago_algebra::assertion_set::Conjunction;
use mago_atom::Atom;
use mago_codex::assertion::Assertion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::function::FunctionTarget;
use crate::plugin::provider::method::MethodTarget;

/// Assertions to apply after an invocation.
///
/// Contains maps from variable names to assertion sets for:
/// - `immediate`: assertions that apply unconditionally after the call
/// - `if_true`: assertions that hold when the call returns truthy
/// - `if_false`: assertions that hold when the call returns falsy
#[derive(Debug, Clone, Default)]
pub struct InvocationAssertions {
    /// Assertions that apply unconditionally after the invocation.
    ///
    /// Keys are variable names (e.g., "$x"), values are assertion sets.
    pub type_assertions: BTreeMap<Atom, Conjunction<Assertion>>,

    /// Assertions that hold when the invocation returns truthy.
    ///
    /// Keys are variable names (e.g., "$x"), values are assertion sets.
    pub if_true: BTreeMap<Atom, Conjunction<Assertion>>,

    /// Assertions that hold when the invocation returns falsy.
    ///
    /// Keys are variable names (e.g., "$x"), values are assertion sets.
    pub if_false: BTreeMap<Atom, Conjunction<Assertion>>,
}

impl InvocationAssertions {
    /// Create new empty assertions.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if there are any assertions.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.type_assertions.is_empty() && self.if_true.is_empty() && self.if_false.is_empty()
    }

    /// Add an immediate assertion for a variable.
    pub fn add_immediate(&mut self, variable: Atom, assertions: Conjunction<Assertion>) {
        self.type_assertions.insert(variable, assertions);
    }

    /// Add an if-true assertion for a variable.
    pub fn add_if_true(&mut self, variable: Atom, assertions: Conjunction<Assertion>) {
        self.if_true.insert(variable, assertions);
    }

    /// Add an if-false assertion for a variable.
    pub fn add_if_false(&mut self, variable: Atom, assertions: Conjunction<Assertion>) {
        self.if_false.insert(variable, assertions);
    }
}

/// Provider for getting additional assertions from function calls.
///
/// This allows plugins to specify type narrowing for calls like:
/// - `assert($x instanceof Foo)` - narrows `$x` to `Foo` after the call
/// - `Assert::assertIsString($x)` - narrows `$x` to `string` after the call
pub trait FunctionAssertionProvider: Provider {
    /// The functions this provider handles.
    fn targets() -> FunctionTarget
    where
        Self: Sized;

    /// Get assertions for the invocation.
    ///
    /// Returns `Some(assertions)` if this provider has assertions to add,
    /// `None` otherwise.
    fn get_assertions(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<InvocationAssertions>;
}

/// Provider for getting additional assertions from method calls.
///
/// This allows plugins to specify type narrowing for method calls like:
/// - `$validator->isString($x)` - narrows `$x` to `string` if returns true
/// - `Assert::assertInstanceOf(Foo::class, $x)` - narrows `$x` to `Foo`
pub trait MethodAssertionProvider: Provider {
    /// The methods this provider handles.
    fn targets() -> &'static [MethodTarget]
    where
        Self: Sized;

    /// Get assertions for the method invocation.
    ///
    /// Returns `Some(assertions)` if this provider has assertions to add,
    /// `None` otherwise.
    fn get_assertions(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        class_name: &str,
        method_name: &str,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<InvocationAssertions>;
}
