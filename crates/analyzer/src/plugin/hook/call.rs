//! Call hooks for function and method call events.

use mago_syntax::ast::FunctionCall;
use mago_syntax::ast::MethodCall;
use mago_syntax::ast::NullSafeMethodCall;
use mago_syntax::ast::StaticMethodCall;

use crate::plugin::context::HookContext;
use crate::plugin::hook::ExpressionHookResult;
use crate::plugin::hook::HookResult;
use crate::plugin::provider::Provider;

/// Hook trait for intercepting function call analysis.
///
/// This hook receives the real AST function call node and full mutable context,
/// allowing hooks to inspect calls, report issues, modify analysis state,
/// and optionally skip analysis with a custom return type.
pub trait FunctionCallHook: Provider {
    /// Called before a function call is analyzed.
    ///
    /// Return `ExpressionHookResult::Continue` to proceed with normal analysis,
    /// `ExpressionHookResult::Skip` to skip analysis (type will be `mixed`), or
    /// `ExpressionHookResult::SkipWithType(ty)` to skip with a custom return type.
    fn before_function_call(
        &self,
        _call: &FunctionCall<'_>,
        _context: &mut HookContext<'_, '_>,
    ) -> HookResult<ExpressionHookResult> {
        Ok(ExpressionHookResult::Continue)
    }

    /// Called after a function call has been analyzed.
    fn after_function_call(&self, _call: &FunctionCall<'_>, _context: &mut HookContext<'_, '_>) -> HookResult<()> {
        Ok(())
    }
}

/// Hook trait for intercepting method call analysis.
///
/// This hook receives the real AST method call node and full mutable context,
/// allowing hooks to inspect calls, report issues, modify analysis state,
/// and optionally skip analysis with a custom return type.
pub trait MethodCallHook: Provider {
    /// Called before a method call is analyzed.
    ///
    /// Return `ExpressionHookResult::Continue` to proceed with normal analysis,
    /// `ExpressionHookResult::Skip` to skip analysis (type will be `mixed`), or
    /// `ExpressionHookResult::SkipWithType(ty)` to skip with a custom return type.
    fn before_method_call(
        &self,
        _call: &MethodCall<'_>,
        _context: &mut HookContext<'_, '_>,
    ) -> HookResult<ExpressionHookResult> {
        Ok(ExpressionHookResult::Continue)
    }

    /// Called after a method call has been analyzed.
    fn after_method_call(&self, _call: &MethodCall<'_>, _context: &mut HookContext<'_, '_>) -> HookResult<()> {
        Ok(())
    }
}

/// Hook trait for intercepting static method call analysis.
pub trait StaticMethodCallHook: Provider {
    /// Called before a static method call is analyzed.
    fn before_static_method_call(
        &self,
        _call: &StaticMethodCall<'_>,
        _context: &mut HookContext<'_, '_>,
    ) -> HookResult<ExpressionHookResult> {
        Ok(ExpressionHookResult::Continue)
    }

    /// Called after a static method call has been analyzed.
    fn after_static_method_call(
        &self,
        _call: &StaticMethodCall<'_>,
        _context: &mut HookContext<'_, '_>,
    ) -> HookResult<()> {
        Ok(())
    }
}

/// Hook trait for intercepting nullsafe method call analysis.
pub trait NullSafeMethodCallHook: Provider {
    /// Called before a nullsafe method call is analyzed.
    fn before_nullsafe_method_call(
        &self,
        _call: &NullSafeMethodCall<'_>,
        _context: &mut HookContext<'_, '_>,
    ) -> HookResult<ExpressionHookResult> {
        Ok(ExpressionHookResult::Continue)
    }

    /// Called after a nullsafe method call has been analyzed.
    fn after_nullsafe_method_call(
        &self,
        _call: &NullSafeMethodCall<'_>,
        _context: &mut HookContext<'_, '_>,
    ) -> HookResult<()> {
        Ok(())
    }
}
