//! Expression hook for Laravel Eloquent model property access.
//!
//! This hook intercepts property access expressions on Eloquent Model
//! subclasses and provides precise types for virtual properties that
//! are not declared in PHP code but exist at runtime:
//!
//! - **Relationship properties:** `$user->posts` → `Collection<Post>`
//! - **Relationship count properties:** `$user->posts_count` → `int`
//! - **Legacy accessors:** `$user->full_name` → return type of `getFullNameAttribute()`
//! - **Modern accessors:** `$user->full_name` → first generic arg of `Attribute<string>`
//! - **Cast properties:** `$user->status` → type mapped from `$casts['status']`
//! - **Attribute defaults:** `$user->name` → type inferred from `$attributes['name']`
//! - **Column name properties:** `$user->email` → `mixed` (from `$fillable`/`$guarded`/`$hidden`)
//!
//! The hook returns `ExpressionHookResult::SkipWithType(ty)` when it can
//! resolve a virtual property, completely replacing the normal analysis
//! (which would report `NonExistentProperty` — suppressed by Phase 1's
//! issue filter — and assign `mixed`).
//!
//! **Priority order** (matching phpantom):
//! 1. Casts (highest priority)
//! 2. Attribute defaults (skipped if already in casts)
//! 3. Column names from `$fillable`/`$guarded`/`$hidden` (skipped if in casts or defaults)
//! 4. Accessor properties (legacy and modern)
//! 5. Relationship properties
//! 6. Relationship count properties
//!
//! Derived from phpantom_lsp's `LaravelModelProvider::provide()`.

use mago_atom::Atom;
use mago_atom::atom;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::generic::TGenericParameter;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::union::TUnion;
use mago_syntax::ast::Access;
use mago_syntax::ast::ClassLikeMemberSelector;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Variable;

use crate::plugin::context::HookContext;
use crate::plugin::hook::ExpressionHook;
use crate::plugin::hook::ExpressionHookResult;
use crate::plugin::hook::HookResult;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;

use super::accessors::resolve_accessor_property_type;
use super::casts::is_column_name_property;
use super::casts::resolve_attribute_default_type;
use super::casts::resolve_cast_property_type;
use super::relationships::resolve_count_property_type;
use super::relationships::resolve_relationship_property_type;
use super::utils::is_eloquent_model_parent;

static META: ProviderMeta = ProviderMeta::new(
    "laravel-model-property",
    "Laravel Model Property",
    "Provides precise types for Eloquent model virtual properties (relationships, accessors, casts)",
);

/// Expression hook that intercepts property access on Eloquent Model
/// subclasses and returns precise types for virtual properties.
pub struct LaravelModelPropertyHook;

impl Provider for LaravelModelPropertyHook {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl ExpressionHook for LaravelModelPropertyHook {
    fn before_expression(
        &self,
        expr: &Expression<'_>,
        context: &mut HookContext<'_, '_>,
    ) -> HookResult<ExpressionHookResult> {
        // We only care about property access expressions: $obj->prop
        let (object, property_selector) = match expr {
            Expression::Access(Access::Property(prop_access)) => (prop_access.object, &prop_access.property),
            Expression::Access(Access::NullSafeProperty(prop_access)) => (prop_access.object, &prop_access.property),
            _ => return Ok(ExpressionHookResult::Continue),
        };

        // Extract the property name from the selector.
        let property_name = match property_selector {
            ClassLikeMemberSelector::Identifier(ident) => ident.value,
            _ => return Ok(ExpressionHookResult::Continue),
        };

        // Get the type of the object being accessed.
        //
        // The ExpressionHook fires *before* the property access expression is
        // analyzed, so the object sub-expression has not yet been typed in
        // `expression_types`.  For the common case where the object is a
        // simple variable (`$user->prop`), we look up the variable's type
        // directly from the block-context locals / scope.  For already-typed
        // sub-expressions (e.g. chained calls) we fall back to
        // `get_expression_type`.
        let object_type = match resolve_object_type(object, context) {
            Some(ty) => ty,
            None => return Ok(ExpressionHookResult::Continue),
        };

        // Find a named object type that is an Eloquent Model subclass.
        let class_name = match find_model_class_name(object_type, context) {
            Some(name) => name,
            None => return Ok(ExpressionHookResult::Continue),
        };

        // Get the class metadata.
        let class_metadata = match context.codebase().get_class_like(&class_name) {
            Some(meta) => meta,
            None => return Ok(ExpressionHookResult::Continue),
        };

        // Check if the property already exists as a real declared property on the class.
        // If so, let the normal analysis handle it — we only provide types for
        // virtual (undeclared) properties.
        let dollar_name = if property_name.starts_with('$') {
            atom(property_name)
        } else {
            let s = format!("${}", property_name);
            atom(&s)
        };

        if class_metadata.properties.contains_key(&dollar_name) {
            return Ok(ExpressionHookResult::Continue);
        }

        // Try to resolve the virtual property type using the priority order
        // from phpantom's LaravelModelProvider::provide():
        //
        // 1. Cast properties (highest priority)
        // 2. Attribute defaults (skipped if in casts)
        // 3. Column name properties (skipped if in casts or defaults)
        // 4. Accessors (legacy and modern)
        // 5. Relationship properties
        // 6. Relationship count properties

        let prop_str = strip_dollar(property_name);

        // 1. Cast properties.
        if let Some(ty) = resolve_cast_property_type(prop_str, class_metadata, context.codebase()) {
            return Ok(ExpressionHookResult::SkipWithType(ty));
        }

        // 2. Attribute defaults.
        if let Some(ty) = resolve_attribute_default_type(prop_str, class_metadata) {
            return Ok(ExpressionHookResult::SkipWithType(ty));
        }

        // 3. Column name properties ($fillable, $guarded, $hidden).
        if is_column_name_property(prop_str, class_metadata) {
            return Ok(ExpressionHookResult::SkipWithType(get_mixed()));
        }

        // 4. Accessor properties.
        if let Some(ty) = resolve_accessor_property_type(prop_str, class_metadata, context.codebase()) {
            return Ok(ExpressionHookResult::SkipWithType(ty));
        }

        // 5. Relationship properties.
        if let Some(ty) = resolve_relationship_property_type(prop_str, class_metadata, context.codebase()) {
            return Ok(ExpressionHookResult::SkipWithType(ty));
        }

        // 6. Relationship count properties.
        if let Some(ty) = resolve_count_property_type(prop_str, class_metadata, context.codebase()) {
            return Ok(ExpressionHookResult::SkipWithType(ty));
        }

        // Not a recognized virtual property — let normal analysis proceed.
        Ok(ExpressionHookResult::Continue)
    }
}

/// Resolve the type of the object in a property access expression.
///
/// The ExpressionHook fires before the property access is analyzed, so the
/// object sub-expression may not yet have a type in `expression_types`.
/// We handle the most common patterns:
///
/// 1. **Simple variable** (`$user`) — look up in block-context locals.
/// 2. **`$this`** — look up in block-context locals as `$this`.
/// 3. **Already-typed expression** — fall back to `get_expression_type`
///    (works for chained calls where the earlier call has already been typed).
fn resolve_object_type<'a>(object: &Expression<'_>, context: &'a HookContext<'_, '_>) -> Option<&'a TUnion> {
    // Try variable lookup first (handles $user, $this, etc.)
    if let Expression::Variable(Variable::Direct(direct)) = object {
        let var_name = direct.name;
        if let Some(rc_ty) = context.get_variable_type(var_name) {
            return Some(rc_ty.as_ref());
        }
    }

    // Fall back to expression_types (for sub-expressions already analyzed).
    context.get_expression_type(object)
}

/// Find the class name of an Eloquent Model from an object type union.
///
/// Walks through the atomic types in the union looking for a named object
/// whose class extends `Illuminate\Database\Eloquent\Model`.  Also follows
/// generic parameter constraints (e.g. `T of User` where User is a Model).
fn find_model_class_name(object_type: &TUnion, context: &HookContext<'_, '_>) -> Option<Atom> {
    let mut atomics: Vec<&TAtomic> = object_type.types.iter().collect();

    while let Some(atomic) = atomics.pop() {
        match atomic {
            TAtomic::Object(TObject::Named(named)) => {
                if is_model_class(&named.name, context) {
                    return Some(named.name);
                }
            }
            TAtomic::GenericParameter(TGenericParameter { constraint, .. }) => {
                // Follow the generic constraint (e.g. `T of User`).
                atomics.extend(constraint.types.iter());
            }
            _ => {}
        }
    }

    None
}

/// Check if a class name refers to an Eloquent Model (or subclass thereof).
fn is_model_class(class_name: &Atom, context: &HookContext<'_, '_>) -> bool {
    let name_str: &str = class_name.as_ref();

    // Check via the codebase metadata's hierarchy.
    if let Some(class_meta) = context.codebase().get_class_like(name_str) {
        return is_eloquent_model_parent(&class_meta.all_parent_classes);
    }

    false
}

/// Strip the leading `$` from a property name if present.
fn strip_dollar(name: &str) -> &str {
    name.strip_prefix('$').unwrap_or(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_dollar_with_prefix() {
        assert_eq!(strip_dollar("$name"), "name");
    }

    #[test]
    fn strip_dollar_without_prefix() {
        assert_eq!(strip_dollar("name"), "name");
    }

    #[test]
    fn strip_dollar_empty() {
        assert_eq!(strip_dollar(""), "");
    }

    #[test]
    fn strip_dollar_only_dollar() {
        assert_eq!(strip_dollar("$"), "");
    }
}
