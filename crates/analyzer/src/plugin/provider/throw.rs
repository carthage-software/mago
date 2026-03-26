//! Throw type providers for expressions, functions, and methods.
//!
//! These providers allow plugins to specify what exceptions an expression,
//! function call, or method call can throw.

use foldhash::HashSet;

use mago_atom::Atom;
use mago_syntax::ast::Expression;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::function::FunctionTarget;
use crate::plugin::provider::method::MethodTarget;

/// Provider for getting thrown exception class names from any expression.
///
/// This is the most general form of throw type provider, allowing plugins
/// to specify exceptions for any expression type (property access, array access, etc.).
pub trait ExpressionThrowTypeProvider: Provider {
    /// Get the exception class names that an expression can throw.
    ///
    /// Returns a set of fully qualified exception class names.
    /// Returns an empty set if the expression doesn't throw.
    fn get_thrown_exceptions(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        expression: &Expression<'_>,
    ) -> HashSet<Atom>;
}

/// Provider for getting thrown exception class names from function calls.
pub trait FunctionThrowTypeProvider: Provider {
    /// The functions this provider handles.
    fn targets() -> FunctionTarget
    where
        Self: Sized;

    /// Get the exception class names that a function invocation can throw.
    ///
    /// Returns a set of fully qualified exception class names.
    /// Returns an empty set if the function doesn't throw.
    fn get_thrown_exceptions(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> HashSet<Atom>;
}

/// Provider for getting thrown exception class names from method calls.
pub trait MethodThrowTypeProvider: Provider {
    /// The methods this provider handles.
    fn targets() -> &'static [MethodTarget]
    where
        Self: Sized;

    /// Get the exception class names that a method invocation can throw.
    ///
    /// Returns a set of fully qualified exception class names.
    /// Returns an empty set if the method doesn't throw.
    fn get_thrown_exceptions(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        class_name: &str,
        method_name: &str,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> HashSet<Atom>;
}
