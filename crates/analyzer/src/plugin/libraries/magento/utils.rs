//! Shared utilities for Magento analyzer hooks.

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_syntax::ast::MethodCall;
use mago_syntax::ast::class_like::member::ClassLikeMemberSelector;

use crate::plugin::context::HookContext;

/// Extracts the method name from a method call if it's a static identifier.
///
/// Returns `None` for dynamic method calls (e.g. `$obj->$method()`).
pub fn get_method_name<'a>(call: &'a MethodCall<'a>) -> Option<&'a str> {
    match &call.method {
        ClassLikeMemberSelector::Identifier(id) => Some(id.value),
        _ => None,
    }
}

/// Returns the class names of the receiver object's type.
///
/// Iterates over all atomic types in the receiver's union type and collects
/// named object class names.
pub fn get_receiver_class_names(call: &MethodCall<'_>, context: &HookContext<'_, '_>) -> Vec<String> {
    let Some(obj_type) = context.get_expression_type(call.object) else {
        return Vec::new();
    };

    let mut names = Vec::new();
    for atomic in obj_type.types.as_ref() {
        match atomic {
            TAtomic::Object(TObject::Named(named)) => {
                names.push(named.name.as_str().to_string());
            }
            TAtomic::Object(TObject::Enum(e)) => {
                names.push(e.name.as_str().to_string());
            }
            _ => {}
        }
    }

    names
}
