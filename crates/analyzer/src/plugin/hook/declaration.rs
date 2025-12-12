//! Declaration hooks for class and function analysis events.

use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_syntax::ast::*;

use crate::plugin::context::HookContext;
use crate::plugin::hook::HookResult;
use crate::plugin::provider::Provider;

/// Hook trait for intercepting class declaration analysis.
///
/// This hook receives the real AST class node, full class metadata,
/// and mutable context, allowing hooks to inspect classes, report issues,
/// and modify analysis state.
pub trait ClassDeclarationHook: Provider {
    /// Called when entering a class declaration.
    fn on_enter_class<'ast, 'arena>(
        &self,
        _class: &'ast Class<'arena>,
        _metadata: &ClassLikeMetadata,
        _context: &mut HookContext<'_, '_>,
    ) -> HookResult<()> {
        Ok(())
    }

    /// Called when leaving a class declaration.
    fn on_leave_class<'ast, 'arena>(
        &self,
        _class: &'ast Class<'arena>,
        _metadata: &ClassLikeMetadata,
        _context: &mut HookContext<'_, '_>,
    ) -> HookResult<()> {
        Ok(())
    }
}

/// Hook trait for intercepting interface declaration analysis.
pub trait InterfaceDeclarationHook: Provider {
    /// Called when entering an interface declaration.
    fn on_enter_interface<'ast, 'arena>(
        &self,
        _interface: &'ast Interface<'arena>,
        _metadata: &ClassLikeMetadata,
        _context: &mut HookContext<'_, '_>,
    ) -> HookResult<()> {
        Ok(())
    }

    /// Called when leaving an interface declaration.
    fn on_leave_interface<'ast, 'arena>(
        &self,
        _interface: &'ast Interface<'arena>,
        _metadata: &ClassLikeMetadata,
        _context: &mut HookContext<'_, '_>,
    ) -> HookResult<()> {
        Ok(())
    }
}

/// Hook trait for intercepting trait declaration analysis.
pub trait TraitDeclarationHook: Provider {
    /// Called when entering a trait declaration.
    fn on_enter_trait<'ast, 'arena>(
        &self,
        _trait_: &'ast Trait<'arena>,
        _metadata: &ClassLikeMetadata,
        _context: &mut HookContext<'_, '_>,
    ) -> HookResult<()> {
        Ok(())
    }

    /// Called when leaving a trait declaration.
    fn on_leave_trait<'ast, 'arena>(
        &self,
        _trait_: &'ast Trait<'arena>,
        _metadata: &ClassLikeMetadata,
        _context: &mut HookContext<'_, '_>,
    ) -> HookResult<()> {
        Ok(())
    }
}

/// Hook trait for intercepting enum declaration analysis.
pub trait EnumDeclarationHook: Provider {
    /// Called when entering an enum declaration.
    fn on_enter_enum<'ast, 'arena>(
        &self,
        _enum_: &'ast Enum<'arena>,
        _metadata: &ClassLikeMetadata,
        _context: &mut HookContext<'_, '_>,
    ) -> HookResult<()> {
        Ok(())
    }

    /// Called when leaving an enum declaration.
    fn on_leave_enum<'ast, 'arena>(
        &self,
        _enum_: &'ast Enum<'arena>,
        _metadata: &ClassLikeMetadata,
        _context: &mut HookContext<'_, '_>,
    ) -> HookResult<()> {
        Ok(())
    }
}

/// Hook trait for intercepting function declaration analysis.
///
/// This hook receives the real AST function node, full function metadata,
/// and mutable context, allowing hooks to inspect functions, report issues,
/// and modify analysis state.
pub trait FunctionDeclarationHook: Provider {
    /// Called when entering a function declaration.
    fn on_enter_function<'ast, 'arena>(
        &self,
        _function: &'ast Function<'arena>,
        _metadata: &FunctionLikeMetadata,
        _context: &mut HookContext<'_, '_>,
    ) -> HookResult<()> {
        Ok(())
    }

    /// Called when leaving a function declaration.
    fn on_leave_function<'ast, 'arena>(
        &self,
        _function: &'ast Function<'arena>,
        _metadata: &FunctionLikeMetadata,
        _context: &mut HookContext<'_, '_>,
    ) -> HookResult<()> {
        Ok(())
    }
}
