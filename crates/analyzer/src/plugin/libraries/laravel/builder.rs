//! Builder forwarding and query chain support for Laravel Eloquent models.
//!
//! This module implements Phase 4 of the Laravel migration:
//!
//! **Builder-as-static forwarding (`StaticMethodCallHook`):**
//! When a static call like `User::where(...)` is made on a Model subclass,
//! the method doesn't exist as a static method on the model.  Eloquent
//! forwards these via `__callStatic` to an Eloquent Builder instance.
//! This hook intercepts such calls and returns the mapped Builder return
//! type so the chain resolves correctly.
//!
//! **Builder return type mapping (`MethodReturnTypeProvider`):**
//! When chaining methods on `Builder<User>`, return types containing
//! `static`, `$this`, `self`, or `TModel` need to be mapped to the
//! concrete model type or `Builder<ConcreteModel>` so the chain
//! continues with proper type information.
//!
//! **Scope methods on Builder (Phase 6):**
//! When a method call is made on a `Builder<T>` instance and the method
//! doesn't exist on Builder, checks if it's a scope method on `T`.
//! This is implemented via the same `MethodReturnTypeProvider` targeting
//! `Builder::*`.
//!
//! Derived from phpantom_lsp's `build_builder_forwarded_methods()` and
//! `build_scope_methods_for_builder()`.

use mago_atom::Atom;
use mago_atom::atom;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::generic::TGenericParameter;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::union::TUnion;
use mago_codex::visibility::Visibility;
use mago_syntax::ast::ClassLikeMemberSelector;
use mago_syntax::ast::Expression;
use mago_syntax::ast::MethodCall;
use mago_syntax::ast::StaticMethodCall;

use crate::plugin::context::HookContext;
use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::hook::ExpressionHookResult;
use crate::plugin::hook::HookResult;
use crate::plugin::hook::MethodCallHook;
use crate::plugin::hook::StaticMethodCallHook;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::method::MethodReturnTypeProvider;
use crate::plugin::provider::method::MethodTarget;

use super::utils::ELOQUENT_BUILDER;
use super::utils::is_convention_scope_method;
use super::utils::is_eloquent_builder_parent;
use super::utils::is_eloquent_model_parent;
use super::utils::scope_method_name;

// ────────────────────────────────────────────────────────────────────────────────
// Constants
// ────────────────────────────────────────────────────────────────────────────────

/// Query Builder FQN — Eloquent Builder mixes in Query\Builder.
const QUERY_BUILDER: &str = "Illuminate\\Database\\Query\\Builder";

/// The `#[Scope]` attribute FQN.
const SCOPE_ATTRIBUTE: &str = "Illuminate\\Database\\Eloquent\\Attributes\\Scope";

// ────────────────────────────────────────────────────────────────────────────────
// StaticMethodCallHook — Builder-as-static forwarding on Model subclasses
// ────────────────────────────────────────────────────────────────────────────────

static STATIC_HOOK_META: ProviderMeta = ProviderMeta::new(
    "laravel-builder-forwarding",
    "Laravel Builder Forwarding",
    "Forwards static method calls on Eloquent Model subclasses to Builder methods",
);

/// Hook that intercepts static method calls on Eloquent Model subclasses
/// and forwards them to Builder when the method exists on Builder but
/// not on the model.
///
/// For example, `User::where('active', true)` is forwarded to
/// `Builder::where(...)`, and the return type is mapped to
/// `Builder<User>`.
pub struct BuilderForwardingHook;

impl Provider for BuilderForwardingHook {
    fn meta() -> &'static ProviderMeta {
        &STATIC_HOOK_META
    }
}

impl StaticMethodCallHook for BuilderForwardingHook {
    fn before_static_method_call(
        &self,
        call: &StaticMethodCall<'_>,
        context: &mut HookContext<'_, '_>,
    ) -> HookResult<ExpressionHookResult> {
        // Extract the method name.
        let method_name = match &call.method {
            ClassLikeMemberSelector::Identifier(ident) => ident.value,
            _ => return Ok(ExpressionHookResult::Continue),
        };

        // Skip magic methods.
        if method_name.starts_with("__") {
            return Ok(ExpressionHookResult::Continue);
        }

        // Resolve the class name from the static call target.
        let class_name = match resolve_class_name_from_expression(call.class, context) {
            Some(name) => name,
            None => return Ok(ExpressionHookResult::Continue),
        };

        // Check if this class is an Eloquent Model subclass.
        let model_metadata = match context.codebase().get_class_like(&class_name) {
            Some(meta) if is_eloquent_model_parent(&meta.all_parent_classes) => meta,
            _ => return Ok(ExpressionHookResult::Continue),
        };

        // Check if the method already exists as a static method on the model.
        // If it does, let normal analysis handle it.
        if has_static_method(model_metadata, method_name, context.codebase()) {
            return Ok(ExpressionHookResult::Continue);
        }

        // Check if the method is a scope method on the model.
        // Scope methods take priority over builder methods.
        if let Some(scope_type) = resolve_scope_as_static(method_name, &class_name, model_metadata, context.codebase())
        {
            return Ok(ExpressionHookResult::SkipWithType(scope_type));
        }

        // Try to find the method on the Builder class.
        if let Some(return_type) = resolve_builder_forwarded_type(method_name, &class_name, context.codebase()) {
            return Ok(ExpressionHookResult::SkipWithType(return_type));
        }

        Ok(ExpressionHookResult::Continue)
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// MethodCallHook — Scope methods on Builder instances (Phase 6)
// ────────────────────────────────────────────────────────────────────────────────

static METHOD_HOOK_META: ProviderMeta = ProviderMeta::new(
    "laravel-builder-scope",
    "Laravel Builder Scope",
    "Resolves scope method calls on Eloquent Builder instances",
);

/// Hook that intercepts method calls on `Builder<T>` instances and
/// resolves scope methods defined on the model `T`.
///
/// For example, `User::query()->active()` where `active` is a scope
/// on `User` (defined as `scopeActive` or `#[Scope] active`).
pub struct BuilderScopeHook;

impl Provider for BuilderScopeHook {
    fn meta() -> &'static ProviderMeta {
        &METHOD_HOOK_META
    }
}

impl MethodCallHook for BuilderScopeHook {
    fn before_method_call(
        &self,
        call: &MethodCall<'_>,
        context: &mut HookContext<'_, '_>,
    ) -> HookResult<ExpressionHookResult> {
        // Extract the method name.
        let method_name = match &call.method {
            ClassLikeMemberSelector::Identifier(ident) => ident.value,
            _ => return Ok(ExpressionHookResult::Continue),
        };

        // Get the type of the object being called on.
        let object_type = match context.get_expression_type(call.object) {
            Some(ty) => ty,
            None => return Ok(ExpressionHookResult::Continue),
        };

        // Check if the object is a Builder<T> and extract the model type T.
        let model_name = match extract_model_from_builder_type(object_type, context.codebase()) {
            Some(name) => name,
            None => return Ok(ExpressionHookResult::Continue),
        };

        // Check that the method doesn't already exist on Builder.
        // If it does, let normal analysis handle it.
        if let Some(builder_meta) = context.codebase().get_class_like(ELOQUENT_BUILDER)
            && builder_meta.methods.contains(&mago_atom::ascii_lowercase_atom(method_name))
        {
            return Ok(ExpressionHookResult::Continue);
        }
        // Also check Query\Builder methods.
        if let Some(query_meta) = context.codebase().get_class_like(QUERY_BUILDER)
            && query_meta.methods.contains(&mago_atom::ascii_lowercase_atom(method_name))
        {
            return Ok(ExpressionHookResult::Continue);
        }

        // Try to resolve as a scope method on the model.
        let model_name_str: &str = model_name.as_ref();
        if let Some(model_metadata) = context.codebase().get_class_like(model_name_str)
            && let Some(scope_type) =
                resolve_scope_on_builder(method_name, model_name_str, model_metadata, context.codebase())
        {
            return Ok(ExpressionHookResult::SkipWithType(scope_type));
        }

        Ok(ExpressionHookResult::Continue)
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// MethodReturnTypeProvider — Builder return type mapping
// ────────────────────────────────────────────────────────────────────────────────

static PROVIDER_META: ProviderMeta = ProviderMeta::new(
    "laravel-builder-return-type",
    "Laravel Builder Return Type",
    "Maps Builder method return types for proper query chain resolution",
);

/// Targets all methods on the Eloquent Builder class.
static TARGETS: [MethodTarget; 2] =
    [MethodTarget::all_methods(ELOQUENT_BUILDER), MethodTarget::all_methods(QUERY_BUILDER)];

/// Method return type provider that maps `static`/`$this`/`self`/`TModel`
/// in Builder method return types to the concrete model's Builder type,
/// keeping query chains properly typed.
pub struct BuilderReturnTypeProvider;

impl Provider for BuilderReturnTypeProvider {
    fn meta() -> &'static ProviderMeta {
        &PROVIDER_META
    }
}

impl MethodReturnTypeProvider for BuilderReturnTypeProvider {
    fn targets() -> &'static [MethodTarget]
    where
        Self: Sized,
    {
        &TARGETS
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        class_name: &str,
        method_name: &str,
        _invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        // We only handle Builder or Query\Builder.
        if !is_builder_class(class_name, context.codebase()) {
            return None;
        }

        // Get the method metadata.
        let method_meta = context.codebase().get_method(class_name, method_name)?;
        let return_type = method_meta.return_type_metadata.as_ref()?;
        let return_union = &return_type.type_union;

        // Check if the return type contains types that need substitution.
        if !needs_builder_substitution(return_union) {
            return None;
        }

        // We need to know the concrete model type to perform substitution.
        // This comes from the Builder's generic parameter.
        // For now, we map self/static/$this → Builder (preserving generics
        // from the calling context).  The actual model type extraction
        // happens at the call site level.
        let substituted = substitute_builder_return_type(return_union, class_name);
        Some(substituted)
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// Helper: Resolve class name from a static call expression
// ────────────────────────────────────────────────────────────────────────────────

/// Resolve a class name from the class expression in a static method call.
fn resolve_class_name_from_expression(expr: &Expression<'_>, context: &HookContext<'_, '_>) -> Option<Atom> {
    match expr {
        Expression::Identifier(ident) => {
            // Resolve the identifier to its fully-qualified name using
            // the resolved names map (e.g. `User` → `App\Models\User`).
            let resolved = context.resolve_name(ident);
            Some(atom(resolved))
        }
        _ => {
            // Try to get the type of the expression and extract a class name.
            let ty = context.get_expression_type(expr)?;
            for atomic in ty.types.iter() {
                if let TAtomic::Object(TObject::Named(named)) = atomic {
                    return Some(named.name);
                }
            }
            None
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// Helper: Check if a static method exists on a model
// ────────────────────────────────────────────────────────────────────────────────

/// Check if a class has a static method with the given name.
///
/// Checks both the class's own methods and inherited methods.
fn has_static_method(class_metadata: &ClassLikeMetadata, method_name: &str, codebase: &CodebaseMetadata) -> bool {
    let lowercase_method = mago_atom::ascii_lowercase_atom(method_name);

    // Check if the method exists on the class.
    if !class_metadata.methods.contains(&lowercase_method) {
        return false;
    }

    // Get the method metadata and check if it's static.
    if let Some(method_meta) = codebase.get_method(&class_metadata.name, method_name) {
        return method_meta.flags.is_static();
    }

    false
}

// ────────────────────────────────────────────────────────────────────────────────
// Helper: Resolve builder forwarded return type
// ────────────────────────────────────────────────────────────────────────────────

/// Try to find a method on the Eloquent Builder and map its return type
/// for a static call forwarded from a Model subclass.
///
/// This mirrors phpantom's `build_builder_forwarded_methods()`:
/// 1. Look up the method on Eloquent Builder (including inherited/mixin methods)
/// 2. Map return types: `static`/`$this`/`self` → `Builder<Model>`,
///    `TModel` → concrete model class
fn resolve_builder_forwarded_type(method_name: &str, model_name: &str, codebase: &CodebaseMetadata) -> Option<TUnion> {
    // Try Eloquent Builder first, then Query Builder.
    let method_meta = find_builder_method(method_name, codebase)?;

    // Only forward public, non-magic methods.
    if let Some(ref mm) = method_meta.method_metadata
        && mm.visibility != Visibility::Public
    {
        return None;
    }
    if method_meta.flags.is_magic_method() {
        return None;
    }

    // Get the return type and apply substitutions.
    let return_type = method_meta.return_type_metadata.as_ref()?;
    let return_union = &return_type.type_union;

    let substituted = substitute_for_model(return_union, model_name, codebase);
    Some(substituted)
}

/// Find a method on the Eloquent Builder class, including methods
/// from Query\Builder (via @mixin) and inherited traits.
fn find_builder_method<'a>(method_name: &str, codebase: &'a CodebaseMetadata) -> Option<&'a FunctionLikeMetadata> {
    // Try Eloquent Builder first.
    if let Some(meta) = codebase.get_method(ELOQUENT_BUILDER, method_name) {
        return Some(meta);
    }

    // Try Query Builder (mixed in via @mixin).
    if let Some(meta) = codebase.get_method(QUERY_BUILDER, method_name) {
        return Some(meta);
    }

    // Check if Eloquent Builder has the method in its method set
    // (may come from traits).
    if let Some(builder_meta) = codebase.get_class_like(ELOQUENT_BUILDER) {
        let lowercase = mago_atom::ascii_lowercase_atom(method_name);
        if builder_meta.methods.contains(&lowercase) {
            // Try to find it through the declaring method ID.
            if let Some(declaring_class) = builder_meta.declaring_method_ids.get(&lowercase) {
                return codebase.get_method(&declaring_class.get_class_name(), method_name);
            }
        }
    }

    None
}

// ────────────────────────────────────────────────────────────────────────────────
// Helper: Scope method resolution
// ────────────────────────────────────────────────────────────────────────────────

/// Resolve a scope method call as a static method on a Model.
///
/// When `User::active()` is called, checks if `User` has a `scopeActive`
/// method or a method with `#[Scope]` attribute named `active`, and
/// returns the scope's return type mapped to `Builder<User>`.
fn resolve_scope_as_static(
    method_name: &str,
    model_name: &str,
    model_metadata: &ClassLikeMetadata,
    codebase: &CodebaseMetadata,
) -> Option<TUnion> {
    resolve_scope_return_type(method_name, model_name, model_metadata, codebase)
}

/// Resolve a scope method call on a Builder<T> instance.
///
/// When `User::query()->active()` is called, checks if the model `T`
/// has a `scopeActive` method or `#[Scope] active` method, and returns
/// the scope's return type.
fn resolve_scope_on_builder(
    method_name: &str,
    model_name: &str,
    model_metadata: &ClassLikeMetadata,
    codebase: &CodebaseMetadata,
) -> Option<TUnion> {
    resolve_scope_return_type(method_name, model_name, model_metadata, codebase)
}

/// Core scope resolution logic shared between static and instance contexts.
///
/// Looks for:
/// 1. Convention-based scopes: `scopeActive()` → callable as `active()`
/// 2. Attribute-based scopes: `#[Scope] active()` → callable as `active()`
///
/// Returns the scope's return type with `static`/`$this`/`self` mapped
/// to the concrete model name within `Builder<Model>`.
fn resolve_scope_return_type(
    method_name: &str,
    model_name: &str,
    model_metadata: &ClassLikeMetadata,
    codebase: &CodebaseMetadata,
) -> Option<TUnion> {
    // Strategy 1: Convention-based scope (scopeActive → active).
    let convention_name = scope_method_name(method_name);
    if let Some(method_meta) = codebase.get_method(&model_metadata.name, &convention_name)
        && is_scope_method_meta(method_meta)
    {
        return Some(build_scope_return_type(method_meta, model_name));
    }

    // Strategy 2: Attribute-based scope (#[Scope] active).
    if let Some(method_meta) = codebase.get_method(&model_metadata.name, method_name)
        && has_scope_attribute(method_meta)
    {
        return Some(build_scope_return_type(method_meta, model_name));
    }

    None
}

/// Check if a method is a scope method (convention-based).
fn is_scope_method_meta(method_meta: &FunctionLikeMetadata) -> bool {
    if let Some(name) = &method_meta.original_name {
        let name_str: &str = name.as_ref();
        return is_convention_scope_method(name_str);
    }
    if let Some(name) = &method_meta.name {
        let name_str: &str = name.as_ref();
        // Check case-insensitively since `name` is lowercased.
        return name_str.len() > 5 && name_str.starts_with("scope");
    }
    false
}

/// Check if a method has the `#[Scope]` attribute.
fn has_scope_attribute(method_meta: &FunctionLikeMetadata) -> bool {
    method_meta.attributes.iter().any(|attr| {
        let name: &str = attr.name.as_ref();
        name == SCOPE_ATTRIBUTE || name.ends_with("\\Scope")
    })
}

/// Build the return type for a scope method.
///
/// If the scope has a declared return type that is not `void`, use it
/// (with `static`/`$this`/`self` substituted for the model name).
/// Otherwise, default to `Builder<Model>`.
fn build_scope_return_type(method_meta: &FunctionLikeMetadata, model_name: &str) -> TUnion {
    // Check if the method has a non-void return type.
    if let Some(return_type) = &method_meta.return_type_metadata {
        let return_union = &return_type.type_union;

        // Check if return type is void.
        if !return_union.is_void() {
            // Substitute static/$this/self → model name in the return type.
            return substitute_self_types_for_model(return_union, model_name);
        }
    }

    // Default: Builder<Model>
    make_builder_type(model_name)
}

// ────────────────────────────────────────────────────────────────────────────────
// Helper: Extract model type from Builder<T>
// ────────────────────────────────────────────────────────────────────────────────

/// Extract the concrete model type `T` from a `Builder<T>` type.
///
/// Walks through the atomic types in the union looking for a named
/// object that is a Builder subclass, then extracts its first generic
/// type parameter.
fn extract_model_from_builder_type(ty: &TUnion, codebase: &CodebaseMetadata) -> Option<Atom> {
    for atomic in ty.types.iter() {
        match atomic {
            TAtomic::Object(TObject::Named(named)) => {
                let name_str: &str = named.name.as_ref();
                if is_builder_class(name_str, codebase) {
                    // Extract the first type parameter (TModel).
                    if let Some(first_param) = named.type_parameters.as_ref().and_then(|tp| tp.first()) {
                        for param_atomic in first_param.types.iter() {
                            if let TAtomic::Object(TObject::Named(model_named)) = param_atomic {
                                return Some(model_named.name);
                            }
                        }
                    }
                    // Builder without type parameters — can't determine model.
                    return None;
                }
            }
            TAtomic::GenericParameter(TGenericParameter { constraint, .. }) => {
                // Follow generic parameter constraints.
                if let Some(model) = extract_model_from_builder_type(constraint, codebase) {
                    return Some(model);
                }
            }
            _ => {}
        }
    }
    None
}

/// Check if a class name refers to an Eloquent Builder or subclass thereof.
fn is_builder_class(class_name: &str, codebase: &CodebaseMetadata) -> bool {
    if class_name.eq_ignore_ascii_case(ELOQUENT_BUILDER) || class_name.eq_ignore_ascii_case(QUERY_BUILDER) {
        return true;
    }

    if let Some(class_meta) = codebase.get_class_like(class_name) {
        return is_eloquent_builder_parent(&class_meta.all_parent_classes);
    }

    false
}

// ────────────────────────────────────────────────────────────────────────────────
// Helper: Type substitution
// ────────────────────────────────────────────────────────────────────────────────

/// Check if a return type union contains types that need builder substitution
/// (`static`, `$this`, `self`, or template parameters).
fn needs_builder_substitution(ty: &TUnion) -> bool {
    for atomic in ty.types.iter() {
        match atomic {
            TAtomic::Object(TObject::Named(named)) => {
                if named.is_this {
                    return true;
                }
                let name_str: &str = named.name.as_ref();
                if name_str == "static" || name_str == "self" {
                    return true;
                }
            }
            TAtomic::GenericParameter(_) => {
                return true;
            }
            _ => {}
        }
    }
    false
}

/// Substitute builder self-types in a return type.
///
/// Maps `static`/`$this`/`self` → the builder class name itself
/// (preserving generic context from the caller).
fn substitute_builder_return_type(return_type: &TUnion, builder_class: &str) -> TUnion {
    let mut new_types = Vec::with_capacity(return_type.types.len());

    for atomic in return_type.types.iter() {
        let substituted = substitute_atomic_for_builder(atomic, builder_class);
        new_types.push(substituted);
    }

    TUnion::from_vec(new_types)
}

/// Substitute types in a return type for a specific model.
///
/// Maps:
/// - `static`/`$this`/`self` → `Builder<Model>`
/// - `TModel` template params → concrete model class
/// - Eloquent Collection → custom collection if applicable
fn substitute_for_model(return_type: &TUnion, model_name: &str, _codebase: &CodebaseMetadata) -> TUnion {
    let mut new_types = Vec::with_capacity(return_type.types.len());

    for atomic in return_type.types.iter() {
        let substituted = substitute_atomic_for_model(atomic, model_name);
        new_types.push(substituted);
    }

    TUnion::from_vec(new_types)
}

/// Substitute self-types (`static`, `$this`, `self`) in a return type,
/// replacing them with the concrete model name.  Used for scope return
/// types where `static` refers to the model, not the builder.
fn substitute_self_types_for_model(return_type: &TUnion, model_name: &str) -> TUnion {
    let mut new_types = Vec::with_capacity(return_type.types.len());

    for atomic in return_type.types.iter() {
        let substituted = substitute_self_atomic_for_model(atomic, model_name);
        new_types.push(substituted);
    }

    TUnion::from_vec(new_types)
}

/// Substitute a single atomic type for builder context.
fn substitute_atomic_for_builder(atomic: &TAtomic, builder_class: &str) -> TAtomic {
    match atomic {
        TAtomic::Object(TObject::Named(named)) => {
            if named.is_this {
                // $this → Builder class
                TAtomic::Object(TObject::Named(TNamedObject::new(atom(builder_class))))
            } else {
                let name_str: &str = named.name.as_ref();
                if name_str == "static" || name_str == "self" {
                    TAtomic::Object(TObject::Named(TNamedObject::new(atom(builder_class))))
                } else {
                    atomic.clone()
                }
            }
        }
        _ => atomic.clone(),
    }
}

/// Substitute a single atomic type for a specific model in builder forwarding.
///
/// - `static`/`$this`/`self` → `Builder<Model>`
/// - Template parameters (TModel) → concrete model class
/// - Named objects with type parameters → recursively substitute
fn substitute_atomic_for_model(atomic: &TAtomic, model_name: &str) -> TAtomic {
    match atomic {
        TAtomic::Object(TObject::Named(named)) => {
            if named.is_this {
                // $this → Builder<Model>
                return make_builder_atomic(model_name);
            }

            let name_str: &str = named.name.as_ref();
            if name_str == "static" || name_str == "self" {
                return make_builder_atomic(model_name);
            }

            // Recursively substitute type parameters.
            if let Some(type_params) = &named.type_parameters {
                let new_params: Vec<TUnion> =
                    type_params.iter().map(|p| substitute_for_model_simple(p, model_name)).collect();
                return TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
                    named.name,
                    Some(new_params),
                )));
            }

            atomic.clone()
        }
        TAtomic::GenericParameter(param) => {
            // Template parameters like TModel → concrete model class.
            let param_name: &str = param.parameter_name.as_ref();
            if param_name.eq_ignore_ascii_case("TModel")
                || param_name.eq_ignore_ascii_case("T")
                || param_name.eq_ignore_ascii_case("TModelClass")
            {
                TAtomic::Object(TObject::Named(TNamedObject::new(atom(model_name))))
            } else {
                atomic.clone()
            }
        }
        _ => atomic.clone(),
    }
}

/// Simple substitution for type parameters within generic arguments.
fn substitute_for_model_simple(ty: &TUnion, model_name: &str) -> TUnion {
    let new_types: Vec<TAtomic> =
        ty.types.iter().map(|atomic| substitute_atomic_for_model(atomic, model_name)).collect();
    TUnion::from_vec(new_types)
}

/// Substitute self-types in an atomic type, mapping to model name directly
/// (not wrapped in Builder<>).  Used for scope return type substitution
/// where `static` in `Builder<static>` means the model.
fn substitute_self_atomic_for_model(atomic: &TAtomic, model_name: &str) -> TAtomic {
    match atomic {
        TAtomic::Object(TObject::Named(named)) => {
            if named.is_this {
                return TAtomic::Object(TObject::Named(TNamedObject::new(atom(model_name))));
            }

            let name_str: &str = named.name.as_ref();
            if name_str == "static" || name_str == "self" {
                return TAtomic::Object(TObject::Named(TNamedObject::new(atom(model_name))));
            }

            // Recursively substitute type parameters.
            if let Some(type_params) = &named.type_parameters {
                let new_params: Vec<TUnion> = type_params
                    .iter()
                    .map(|p| {
                        let new_types: Vec<TAtomic> =
                            p.types.iter().map(|a| substitute_self_atomic_for_model(a, model_name)).collect();
                        TUnion::from_vec(new_types)
                    })
                    .collect();
                return TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
                    named.name,
                    Some(new_params),
                )));
            }

            atomic.clone()
        }
        _ => atomic.clone(),
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// Helper: Type construction
// ────────────────────────────────────────────────────────────────────────────────

/// Create a `Builder<Model>` type.
fn make_builder_type(model_name: &str) -> TUnion {
    TUnion::from_atomic(make_builder_atomic(model_name))
}

/// Create a `Builder<Model>` atomic type.
fn make_builder_atomic(model_name: &str) -> TAtomic {
    let model_union = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom(model_name)))));
    TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
        atom(ELOQUENT_BUILDER),
        Some(vec![model_union]),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use mago_codex::metadata::flags::MetadataFlags;
    use mago_codex::misc::GenericParent;
    use mago_codex::ttype::get_int;
    use mago_codex::ttype::get_mixed;
    use mago_codex::ttype::get_string;
    use mago_span::Span;

    use super::super::utils::ELOQUENT_COLLECTION;

    // ── make_builder_type ───────────────────────────────────────────

    #[test]
    fn make_builder_type_creates_correct_structure() {
        let ty = make_builder_type("App\\Models\\User");
        assert!(!ty.is_nullable());
        assert_eq!(ty.types.len(), 1);

        match ty.types.first().unwrap() {
            TAtomic::Object(TObject::Named(named)) => {
                let name: &str = named.name.as_ref();
                assert_eq!(name, ELOQUENT_BUILDER);
                assert!(named.has_type_parameters());
                let params = named.get_type_parameters().unwrap();
                assert_eq!(params.len(), 1);
                // First param should be the model type.
                match params[0].types.first().unwrap() {
                    TAtomic::Object(TObject::Named(model)) => {
                        let model_name: &str = model.name.as_ref();
                        assert_eq!(model_name, "App\\Models\\User");
                    }
                    other => panic!("Expected named object, got {:?}", other),
                }
            }
            other => panic!("Expected named object, got {:?}", other),
        }
    }

    // ── substitute_atomic_for_model ─────────────────────────────────

    #[test]
    fn substitute_static_becomes_builder() {
        let static_type = TAtomic::Object(TObject::Named(TNamedObject::new(atom("static"))));
        let result = substitute_atomic_for_model(&static_type, "App\\Models\\User");

        match &result {
            TAtomic::Object(TObject::Named(named)) => {
                let name: &str = named.name.as_ref();
                assert_eq!(name, ELOQUENT_BUILDER);
                assert!(named.has_type_parameters());
            }
            other => panic!("Expected Builder<User>, got {:?}", other),
        }
    }

    #[test]
    fn substitute_self_becomes_builder() {
        let self_type = TAtomic::Object(TObject::Named(TNamedObject::new(atom("self"))));
        let result = substitute_atomic_for_model(&self_type, "App\\Models\\Post");

        match &result {
            TAtomic::Object(TObject::Named(named)) => {
                let name: &str = named.name.as_ref();
                assert_eq!(name, ELOQUENT_BUILDER);
            }
            other => panic!("Expected Builder<Post>, got {:?}", other),
        }
    }

    #[test]
    fn substitute_this_becomes_builder() {
        let this_type = TAtomic::Object(TObject::Named(TNamedObject::new_this(atom(ELOQUENT_BUILDER))));
        let result = substitute_atomic_for_model(&this_type, "App\\Models\\User");

        match &result {
            TAtomic::Object(TObject::Named(named)) => {
                let name: &str = named.name.as_ref();
                assert_eq!(name, ELOQUENT_BUILDER);
                assert!(named.has_type_parameters());
            }
            other => panic!("Expected Builder<User>, got {:?}", other),
        }
    }

    #[test]
    fn substitute_tmodel_becomes_concrete_model() {
        let tmodel = TAtomic::GenericParameter(TGenericParameter {
            parameter_name: atom("TModel"),
            defining_entity: GenericParent::ClassLike(atom(ELOQUENT_BUILDER)),
            constraint: Arc::new(get_mixed()),
            intersection_types: None,
        });
        let result = substitute_atomic_for_model(&tmodel, "App\\Models\\User");

        match &result {
            TAtomic::Object(TObject::Named(named)) => {
                let name: &str = named.name.as_ref();
                assert_eq!(name, "App\\Models\\User");
            }
            other => panic!("Expected User, got {:?}", other),
        }
    }

    #[test]
    fn substitute_preserves_non_template_return_type() {
        let int_type = get_int();
        let result = substitute_for_model(&int_type, "App\\Models\\User", &CodebaseMetadata::default());
        // int should stay int.
        assert_eq!(result.types.len(), 1);
    }

    #[test]
    fn substitute_named_with_generic_params_recurses() {
        // Simulate Collection<static> → Collection<Builder<User>>
        let inner_static = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom("static")))));
        let collection = TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
            atom(ELOQUENT_COLLECTION),
            Some(vec![inner_static]),
        )));
        let result = substitute_atomic_for_model(&collection, "App\\Models\\User");

        match &result {
            TAtomic::Object(TObject::Named(named)) => {
                let name: &str = named.name.as_ref();
                assert_eq!(name, ELOQUENT_COLLECTION);
                let params = named.get_type_parameters().unwrap();
                assert_eq!(params.len(), 1);
                // The inner type should now be Builder<User>.
                match params[0].types.first().unwrap() {
                    TAtomic::Object(TObject::Named(inner)) => {
                        let inner_name: &str = inner.name.as_ref();
                        assert_eq!(inner_name, ELOQUENT_BUILDER);
                    }
                    other => panic!("Expected Builder inside Collection, got {:?}", other),
                }
            }
            other => panic!("Expected Collection, got {:?}", other),
        }
    }

    // ── substitute_self_types_for_model ──────────────────────────────

    #[test]
    fn scope_substitute_static_becomes_model() {
        let union = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom("static")))));
        let result = substitute_self_types_for_model(&union, "App\\Models\\User");

        match result.types.first().unwrap() {
            TAtomic::Object(TObject::Named(named)) => {
                let name: &str = named.name.as_ref();
                assert_eq!(name, "App\\Models\\User");
            }
            other => panic!("Expected User, got {:?}", other),
        }
    }

    #[test]
    fn scope_substitute_builder_static_becomes_builder_model() {
        // Builder<static> → Builder<User>
        let inner = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom("static")))));
        let builder = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
            atom(ELOQUENT_BUILDER),
            Some(vec![inner]),
        ))));
        let result = substitute_self_types_for_model(&builder, "App\\Models\\User");

        match result.types.first().unwrap() {
            TAtomic::Object(TObject::Named(named)) => {
                let name: &str = named.name.as_ref();
                assert_eq!(name, ELOQUENT_BUILDER);
                let params = named.get_type_parameters().unwrap();
                match params[0].types.first().unwrap() {
                    TAtomic::Object(TObject::Named(model)) => {
                        let model_name: &str = model.name.as_ref();
                        assert_eq!(model_name, "App\\Models\\User");
                    }
                    other => panic!("Expected User, got {:?}", other),
                }
            }
            other => panic!("Expected Builder, got {:?}", other),
        }
    }

    // ── needs_builder_substitution ───────────────────────────────────

    #[test]
    fn needs_substitution_for_static() {
        let ty = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom("static")))));
        assert!(needs_builder_substitution(&ty));
    }

    #[test]
    fn needs_substitution_for_self() {
        let ty = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom("self")))));
        assert!(needs_builder_substitution(&ty));
    }

    #[test]
    fn needs_substitution_for_this() {
        let ty = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_this(atom("Builder")))));
        assert!(needs_builder_substitution(&ty));
    }

    #[test]
    fn no_substitution_for_int() {
        assert!(!needs_builder_substitution(&get_int()));
    }

    #[test]
    fn no_substitution_for_string() {
        assert!(!needs_builder_substitution(&get_string()));
    }

    #[test]
    fn no_substitution_for_named_class() {
        let ty = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom("SomeClass")))));
        assert!(!needs_builder_substitution(&ty));
    }

    // ── extract_model_from_builder_type ──────────────────────────────

    #[test]
    fn extract_model_from_builder_with_params() {
        let codebase = CodebaseMetadata::default();
        let model_union =
            TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom("App\\Models\\User")))));
        let builder_type = TUnion::from_atomic(TAtomic::Object(TObject::Named(
            TNamedObject::new_with_type_parameters(atom(ELOQUENT_BUILDER), Some(vec![model_union])),
        )));

        let result = extract_model_from_builder_type(&builder_type, &codebase);
        assert!(result.is_some());
        let model_atom = result.unwrap();
        let model: &str = model_atom.as_ref();
        assert_eq!(model, "App\\Models\\User");
    }

    #[test]
    fn extract_model_from_builder_without_params() {
        let codebase = CodebaseMetadata::default();
        let builder_type =
            TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom(ELOQUENT_BUILDER)))));

        let result = extract_model_from_builder_type(&builder_type, &codebase);
        assert!(result.is_none());
    }

    #[test]
    fn extract_model_from_non_builder() {
        let codebase = CodebaseMetadata::default();
        let ty = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom("SomeRandomClass")))));

        let result = extract_model_from_builder_type(&ty, &codebase);
        assert!(result.is_none());
    }

    // ── has_static_method ───────────────────────────────────────────

    #[test]
    fn has_static_method_returns_false_for_empty_class() {
        let codebase = CodebaseMetadata::default();
        let metadata = ClassLikeMetadata::new(
            atom("App\\Models\\User"),
            atom("App\\Models\\User"),
            Span::dummy(0, 0),
            None,
            MetadataFlags::empty(),
        );
        assert!(!has_static_method(&metadata, "where", &codebase));
    }

    // ── scope method name helpers ───────────────────────────────────

    #[test]
    fn scope_method_name_active() {
        assert_eq!(scope_method_name("active"), "scopeActive");
    }

    #[test]
    fn scope_method_name_of_type() {
        assert_eq!(scope_method_name("ofType"), "scopeOfType");
    }

    // ── build_scope_return_type ─────────────────────────────────────

    #[test]
    fn scope_default_return_type_is_builder() {
        use mago_codex::metadata::function_like::{FunctionLikeKind, FunctionLikeMetadata};

        let method = FunctionLikeMetadata::new(FunctionLikeKind::Method, Span::dummy(0, 0), MetadataFlags::empty());
        let result = build_scope_return_type(&method, "App\\Models\\User");

        // Should default to Builder<User>.
        match result.types.first().unwrap() {
            TAtomic::Object(TObject::Named(named)) => {
                let name: &str = named.name.as_ref();
                assert_eq!(name, ELOQUENT_BUILDER);
                assert!(named.has_type_parameters());
            }
            other => panic!("Expected Builder<User>, got {:?}", other),
        }
    }
}
