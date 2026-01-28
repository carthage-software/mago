use std::borrow::Cow;
use std::cell::RefCell;

use ahash::HashSet;
use mago_atom::Atom;
use mago_atom::ascii_lowercase_atom;

use crate::identifier::function_like::FunctionLikeIdentifier;
use crate::metadata::CodebaseMetadata;
use crate::metadata::function_like::FunctionLikeMetadata;
use crate::ttype::TType;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::alias::TAlias;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::callable::TCallable;
use crate::ttype::atomic::callable::TCallableSignature;
use crate::ttype::atomic::callable::parameter::TCallableParameter;
use crate::ttype::atomic::derived::TDerived;
use crate::ttype::atomic::derived::index_access::TIndexAccess;
use crate::ttype::atomic::derived::int_mask::TIntMask;
use crate::ttype::atomic::derived::int_mask_of::TIntMaskOf;
use crate::ttype::atomic::derived::key_of::TKeyOf;
use crate::ttype::atomic::derived::properties_of::TPropertiesOf;
use crate::ttype::atomic::derived::value_of::TValueOf;
use crate::ttype::atomic::mixed::TMixed;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::object::named::TNamedObject;
use crate::ttype::atomic::reference::TReference;
use crate::ttype::atomic::reference::TReferenceMemberSelector;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeString;
use crate::ttype::combiner;
use crate::ttype::get_mixed;
use crate::ttype::union::TUnion;

thread_local! {
    /// Thread-local set for tracking currently expanding aliases (cycle detection).
    /// Uses a HashSet for accurate tracking without false positives from hash collisions.
    static EXPANDING_ALIASES: RefCell<HashSet<(Atom, Atom)>> = const { RefCell::new(HashSet::with_hasher(ahash::RandomState::with_seeds(0, 0, 0, 0))) };

    /// Thread-local set for tracking objects whose type parameters are being expanded (cycle detection).
    static EXPANDING_OBJECT_PARAMS: RefCell<HashSet<Atom>> = const { RefCell::new(HashSet::with_hasher(ahash::RandomState::with_seeds(0, 0, 0, 0))) };
}

/// Resets the thread-local alias expansion state.
///
/// This is primarily useful for testing to ensure a clean state between tests.
/// In normal usage, the RAII guards handle cleanup automatically.
#[inline]
pub fn reset_expansion_state() {
    EXPANDING_ALIASES.with(|set| set.borrow_mut().clear());
    EXPANDING_OBJECT_PARAMS.with(|set| set.borrow_mut().clear());
}

/// RAII guard to ensure alias expansion state is properly cleaned up.
/// This guarantees the alias is removed from the set even if the expansion panics.
struct AliasExpansionGuard {
    class_name: Atom,
    alias_name: Atom,
}

impl AliasExpansionGuard {
    fn new(class_name: Atom, alias_name: Atom) -> Self {
        EXPANDING_ALIASES.with(|set| set.borrow_mut().insert((class_name, alias_name)));
        Self { class_name, alias_name }
    }
}

impl Drop for AliasExpansionGuard {
    fn drop(&mut self) {
        EXPANDING_ALIASES.with(|set| set.borrow_mut().remove(&(self.class_name, self.alias_name)));
    }
}

/// RAII guard for object type parameter expansion cycle detection.
struct ObjectParamsExpansionGuard {
    object_name: Atom,
}

impl ObjectParamsExpansionGuard {
    fn try_new(object_name: Atom) -> Option<Self> {
        EXPANDING_OBJECT_PARAMS.with(|set| {
            let mut set = set.borrow_mut();
            if set.contains(&object_name) {
                None
            } else {
                set.insert(object_name);
                Some(Self { object_name })
            }
        })
    }
}

impl Drop for ObjectParamsExpansionGuard {
    fn drop(&mut self) {
        EXPANDING_OBJECT_PARAMS.with(|set| set.borrow_mut().remove(&self.object_name));
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum StaticClassType {
    #[default]
    None,
    Name(Atom),
    Object(TObject),
}

#[derive(Debug)]
pub struct TypeExpansionOptions {
    pub self_class: Option<Atom>,
    pub static_class_type: StaticClassType,
    pub parent_class: Option<Atom>,
    pub evaluate_class_constants: bool,
    pub evaluate_conditional_types: bool,
    pub function_is_final: bool,
    pub expand_generic: bool,
    pub expand_templates: bool,
}

impl Default for TypeExpansionOptions {
    fn default() -> Self {
        Self {
            self_class: None,
            static_class_type: StaticClassType::default(),
            parent_class: None,
            evaluate_class_constants: true,
            evaluate_conditional_types: false,
            function_is_final: false,
            expand_generic: false,
            expand_templates: true,
        }
    }
}

/// Expands a type union, resolving special types like `self`, `static`, `parent`,
/// type aliases, class constants, and generic type parameters.
pub fn expand_union(codebase: &CodebaseMetadata, return_type: &mut TUnion, options: &TypeExpansionOptions) {
    if !return_type.is_expandable() {
        return;
    }

    let mut types = std::mem::take(&mut return_type.types).into_owned();
    let mut new_return_type_parts: Vec<TAtomic> = Vec::new();
    let mut skip_mask: u64 = 0;

    for (i, return_type_part) in types.iter_mut().enumerate() {
        let mut skip_key = false;
        expand_atomic(return_type_part, codebase, options, &mut skip_key, &mut new_return_type_parts);

        if skip_key && i < 64 {
            skip_mask |= 1u64 << i;
        }
    }

    if skip_mask != 0 {
        let mut idx = 0usize;
        types.retain(|_| {
            let retain = idx >= 64 || (skip_mask & (1u64 << idx)) == 0;
            idx += 1;
            retain
        });

        new_return_type_parts.append(&mut types);

        if new_return_type_parts.is_empty() {
            new_return_type_parts.push(TAtomic::Mixed(TMixed::new()));
        }

        types = if new_return_type_parts.len() > 1 {
            combiner::combine(new_return_type_parts, codebase, false)
        } else {
            new_return_type_parts
        };
    } else if types.len() > 1 {
        types = combiner::combine(types, codebase, false);
    }

    return_type.types = Cow::Owned(types);
}

pub(crate) fn expand_atomic(
    return_type_part: &mut TAtomic,
    codebase: &CodebaseMetadata,
    options: &TypeExpansionOptions,
    skip_key: &mut bool,
    new_return_type_parts: &mut Vec<TAtomic>,
) {
    match return_type_part {
        TAtomic::Array(array_type) => match array_type {
            TArray::Keyed(keyed_data) => {
                if let Some((key_parameter, value_parameter)) = &mut keyed_data.parameters {
                    expand_union(codebase, key_parameter, options);
                    expand_union(codebase, value_parameter, options);
                }

                if let Some(known_items) = &mut keyed_data.known_items {
                    for (_, item_type) in known_items.values_mut() {
                        expand_union(codebase, item_type, options);
                    }
                }
            }
            TArray::List(list_data) => {
                expand_union(codebase, &mut list_data.element_type, options);

                if let Some(known_elements) = &mut list_data.known_elements {
                    for (_, element_type) in known_elements.values_mut() {
                        expand_union(codebase, element_type, options);
                    }
                }
            }
        },
        TAtomic::Object(object) => {
            expand_object(object, codebase, options);
        }
        TAtomic::Callable(TCallable::Signature(signature)) => {
            if let Some(return_type) = signature.get_return_type_mut() {
                expand_union(codebase, return_type, options);
            }

            for param in signature.get_parameters_mut() {
                if let Some(param_type) = param.get_type_signature_mut() {
                    expand_union(codebase, param_type, options);
                }
            }
        }
        TAtomic::GenericParameter(parameter) => {
            expand_union(codebase, parameter.constraint.as_mut(), options);
        }
        TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::OfType { constraint, .. })) => {
            let mut atomic_return_type_parts = vec![];
            expand_atomic(constraint, codebase, options, &mut false, &mut atomic_return_type_parts);

            if !atomic_return_type_parts.is_empty() {
                **constraint = atomic_return_type_parts.remove(0);
            }
        }
        TAtomic::Reference(TReference::Member { class_like_name, member_selector }) => {
            *skip_key = true;
            expand_member_reference(*class_like_name, member_selector, codebase, options, new_return_type_parts);
        }
        TAtomic::Callable(TCallable::Alias(id)) => {
            if let Some(value) = get_atomic_of_function_like_identifier(id, codebase) {
                *skip_key = true;
                new_return_type_parts.push(value);
            }
        }
        TAtomic::Conditional(conditional) => {
            *skip_key = true;

            let mut then = conditional.then.clone();
            let mut otherwise = conditional.otherwise.clone();

            expand_union(codebase, &mut then, options);
            expand_union(codebase, &mut otherwise, options);

            new_return_type_parts.extend(then.types.into_owned());
            new_return_type_parts.extend(otherwise.types.into_owned());
        }
        TAtomic::Alias(alias) => {
            *skip_key = true;
            new_return_type_parts.extend(expand_alias(alias, codebase, options));
        }
        TAtomic::Derived(derived) => match derived {
            TDerived::KeyOf(key_of) => {
                *skip_key = true;
                new_return_type_parts.extend(expand_key_of(key_of, codebase, options));
            }
            TDerived::ValueOf(value_of) => {
                *skip_key = true;
                new_return_type_parts.extend(expand_value_of(value_of, codebase, options));
            }
            TDerived::IndexAccess(index_access) => {
                *skip_key = true;
                new_return_type_parts.extend(expand_index_access(index_access, codebase, options));
            }
            TDerived::IntMask(int_mask) => {
                *skip_key = true;
                new_return_type_parts.extend(expand_int_mask(int_mask, codebase, options));
            }
            TDerived::IntMaskOf(int_mask_of) => {
                *skip_key = true;
                new_return_type_parts.extend(expand_int_mask_of(int_mask_of, codebase, options));
            }
            TDerived::PropertiesOf(properties_of) => {
                *skip_key = true;
                new_return_type_parts.extend(expand_properties_of(properties_of, codebase, options));
            }
        },
        TAtomic::Iterable(iterable) => {
            expand_union(codebase, &mut iterable.key_type, options);
            expand_union(codebase, &mut iterable.value_type, options);
        }
        _ => {}
    }
}

#[cold]
fn expand_member_reference(
    class_like_name: Atom,
    member_selector: &TReferenceMemberSelector,
    codebase: &CodebaseMetadata,
    options: &TypeExpansionOptions,
    new_return_type_parts: &mut Vec<TAtomic>,
) {
    let Some(class_like) = codebase.get_class_like(&class_like_name) else {
        new_return_type_parts.push(TAtomic::Mixed(TMixed::new()));
        return;
    };

    for (constant_name, constant) in &class_like.constants {
        if !member_selector.matches(constant_name) {
            continue;
        }

        if let Some(inferred_type) = constant.inferred_type.as_ref() {
            let mut inferred_type = inferred_type.clone();
            let mut skip_inferred_type = false;
            expand_atomic(&mut inferred_type, codebase, options, &mut skip_inferred_type, new_return_type_parts);

            if !skip_inferred_type {
                new_return_type_parts.push(inferred_type);
            }
        } else if let Some(type_metadata) = constant.type_metadata.as_ref() {
            let mut constant_type = type_metadata.type_union.clone();
            expand_union(codebase, &mut constant_type, options);
            new_return_type_parts.extend(constant_type.types.into_owned());
        } else {
            new_return_type_parts.push(TAtomic::Mixed(TMixed::new()));
        }
    }

    for enum_case_name in class_like.enum_cases.keys() {
        if !member_selector.matches(enum_case_name) {
            continue;
        }
        new_return_type_parts.push(TAtomic::Object(TObject::new_enum_case(class_like.original_name, *enum_case_name)));
    }

    if let TReferenceMemberSelector::Identifier(member_name) = member_selector
        && let Some(type_alias) = class_like.type_aliases.get(member_name)
    {
        let mut alias_type = type_alias.type_union.clone();
        expand_union(codebase, &mut alias_type, options);
        new_return_type_parts.extend(alias_type.types.into_owned());
    }

    if new_return_type_parts.is_empty() {
        new_return_type_parts.push(TAtomic::Mixed(TMixed::new()));
    }
}

fn expand_object(object: &mut TObject, codebase: &CodebaseMetadata, options: &TypeExpansionOptions) {
    resolve_special_class_names(object, codebase, options);

    let TObject::Named(named) = object else {
        return;
    };

    let Some(_guard) = ObjectParamsExpansionGuard::try_new(named.name) else {
        return;
    };

    if let Some(class_metadata) = codebase.get_class_like(&named.name) {
        for &required in class_metadata.require_extends.iter().chain(&class_metadata.require_implements) {
            named.add_intersection_type(TAtomic::Object(TObject::Named(TNamedObject::new(required))));
        }
    }

    expand_or_fill_type_parameters(named, codebase, options);
}

/// Resolves `static`, `$this`, `self`, and `parent` to their concrete class names.
fn resolve_special_class_names(object: &mut TObject, codebase: &CodebaseMetadata, options: &TypeExpansionOptions) {
    if let TObject::Named(named) = object {
        let name_lc = ascii_lowercase_atom(&named.name);
        let needs_static_resolution = matches!(name_lc.as_str(), "static" | "$this") || named.is_this;

        if needs_static_resolution
            && let StaticClassType::Object(TObject::Enum(static_enum)) = &options.static_class_type
        {
            *object = TObject::Enum(static_enum.clone());
            return;
        }
    }

    let TObject::Named(named) = object else {
        return;
    };

    let name_lc = ascii_lowercase_atom(&named.name);
    let is_this = named.is_this;

    match name_lc.as_str() {
        "static" | "$this" => resolve_static_type(named, false, codebase, options),
        "self" => {
            if let Some(self_class) = options.self_class {
                named.name = self_class;
            }
        }
        "parent" => {
            if let Some(self_class) = options.self_class
                && let Some(class_metadata) = codebase.get_class_like(&self_class)
                && let Some(parent) = class_metadata.direct_parent_class
            {
                named.name = parent;
            }
        }
        _ if is_this => resolve_static_type(named, true, codebase, options),
        _ => {}
    }
}

/// Resolves a `static` or `$this` type to a named object using the static class type from options.
/// When `check_compatibility` is true, verifies the static type is compatible before resolving.
fn resolve_static_type(
    named: &mut TNamedObject,
    check_compatibility: bool,
    codebase: &CodebaseMetadata,
    options: &TypeExpansionOptions,
) {
    match &options.static_class_type {
        StaticClassType::Object(TObject::Named(static_obj)) => {
            if check_compatibility && !is_static_type_compatible(named, static_obj, codebase) {
                return;
            }

            if let Some(intersections) = &static_obj.intersection_types {
                named.intersection_types.get_or_insert_with(Vec::new).extend(intersections.iter().cloned());
            }

            if static_obj.type_parameters.is_some() && should_use_static_type_params(named, static_obj, codebase) {
                named.type_parameters.clone_from(&static_obj.type_parameters);
            }

            named.name = static_obj.name;
            named.is_this = true;
        }
        StaticClassType::Name(static_class) => {
            if !check_compatibility || codebase.is_instance_of(static_class, &named.name) {
                named.name = *static_class;
                named.is_this = false;
            }
        }
        _ => {}
    }
}

/// Checks if the static object type is compatible with a type that has is_this=true.
fn is_static_type_compatible(named: &TNamedObject, static_obj: &TNamedObject, codebase: &CodebaseMetadata) -> bool {
    codebase.is_instance_of(&static_obj.name, &named.name)
        || static_obj
            .intersection_types
            .iter()
            .flatten()
            .filter_map(|t| if let TAtomic::Object(obj) = t { obj.get_name() } else { None })
            .any(|name| codebase.is_instance_of(name, &named.name))
}

/// Returns true if we should use the static object's type parameters instead of the current ones.
/// This is true when current params are None or match the class's default template bounds.
fn should_use_static_type_params(named: &TNamedObject, static_obj: &TNamedObject, codebase: &CodebaseMetadata) -> bool {
    let Some(current_params) = &named.type_parameters else {
        return true;
    };

    let Some(class_metadata) = codebase.get_class_like(&static_obj.name) else {
        return false;
    };

    let templates = &class_metadata.template_types;

    current_params.len() == templates.len()
        && current_params.iter().zip(templates.iter()).all(|(current, (_, template_map))| {
            template_map.iter().next().map_or_else(|| current.is_mixed(), |(_, bound)| current == bound)
        })
}

/// Expands existing type parameters or fills them with default template bounds.
fn expand_or_fill_type_parameters(
    named: &mut TNamedObject,
    codebase: &CodebaseMetadata,
    options: &TypeExpansionOptions,
) {
    if let Some(params) = &mut named.type_parameters
        && !params.is_empty()
    {
        for param in params.iter_mut() {
            expand_union(codebase, param, options);
        }
        return;
    }

    let Some(class_metadata) = codebase.get_class_like(&named.name) else {
        return;
    };

    if class_metadata.template_types.is_empty() {
        return;
    }

    let defaults: Vec<TUnion> = class_metadata
        .template_types
        .iter()
        .map(|(_, template_map)| template_map.iter().next().map_or_else(get_mixed, |(_, t)| t.clone()))
        .collect();

    named.type_parameters = Some(defaults);
}

#[must_use]
pub fn get_signature_of_function_like_identifier(
    function_like_identifier: &FunctionLikeIdentifier,
    codebase: &CodebaseMetadata,
) -> Option<TCallableSignature> {
    Some(match function_like_identifier {
        FunctionLikeIdentifier::Function(name) => {
            let function_like_metadata = codebase.get_function(name)?;

            get_signature_of_function_like_metadata(
                function_like_identifier,
                function_like_metadata,
                codebase,
                &TypeExpansionOptions::default(),
            )
        }
        FunctionLikeIdentifier::Closure(file_id, position) => {
            let function_like_metadata = codebase.get_closure(file_id, position)?;

            get_signature_of_function_like_metadata(
                function_like_identifier,
                function_like_metadata,
                codebase,
                &TypeExpansionOptions::default(),
            )
        }
        FunctionLikeIdentifier::Method(classlike_name, method_name) => {
            let function_like_metadata = codebase.get_declaring_method(classlike_name, method_name)?;

            get_signature_of_function_like_metadata(
                function_like_identifier,
                function_like_metadata,
                codebase,
                &TypeExpansionOptions {
                    self_class: Some(*classlike_name),
                    static_class_type: StaticClassType::Name(*classlike_name),
                    ..Default::default()
                },
            )
        }
    })
}

#[must_use]
pub fn get_atomic_of_function_like_identifier(
    function_like_identifier: &FunctionLikeIdentifier,
    codebase: &CodebaseMetadata,
) -> Option<TAtomic> {
    let signature = get_signature_of_function_like_identifier(function_like_identifier, codebase)?;

    Some(TAtomic::Callable(TCallable::Signature(signature)))
}

#[must_use]
pub fn get_signature_of_function_like_metadata(
    function_like_identifier: &FunctionLikeIdentifier,
    function_like_metadata: &FunctionLikeMetadata,
    codebase: &CodebaseMetadata,
    options: &TypeExpansionOptions,
) -> TCallableSignature {
    let parameters: Vec<_> = function_like_metadata
        .parameters
        .iter()
        .map(|parameter_metadata| {
            let type_signature = if let Some(t) = parameter_metadata.get_type_metadata() {
                let mut t = t.type_union.clone();
                expand_union(codebase, &mut t, options);
                Some(Box::new(t))
            } else {
                None
            };

            TCallableParameter::new(
                type_signature,
                parameter_metadata.flags.is_by_reference(),
                parameter_metadata.flags.is_variadic(),
                parameter_metadata.flags.has_default(),
            )
        })
        .collect();

    let return_type = if let Some(type_metadata) = function_like_metadata.return_type_metadata.as_ref() {
        let mut return_type = type_metadata.type_union.clone();
        expand_union(codebase, &mut return_type, options);
        Some(Box::new(return_type))
    } else {
        None
    };

    let mut signature = TCallableSignature::new(function_like_metadata.flags.is_pure(), true)
        .with_parameters(parameters)
        .with_return_type(return_type)
        .with_source(Some(*function_like_identifier));

    if let FunctionLikeIdentifier::Closure(file_id, closure_position) = function_like_identifier {
        signature = signature.with_closure_location(Some((*file_id, *closure_position)));
    }

    signature
}

#[cold]
fn expand_key_of(
    return_type_key_of: &TKeyOf,
    codebase: &CodebaseMetadata,
    options: &TypeExpansionOptions,
) -> Vec<TAtomic> {
    let mut target_type = return_type_key_of.get_target_type().clone();
    expand_union(codebase, &mut target_type, options);

    let Some(new_return_types) = TKeyOf::get_key_of_targets(&target_type.types, codebase, false) else {
        return vec![TAtomic::Derived(TDerived::KeyOf(return_type_key_of.clone()))];
    };

    new_return_types.types.into_owned()
}

#[cold]
fn expand_value_of(
    return_type_value_of: &TValueOf,
    codebase: &CodebaseMetadata,
    options: &TypeExpansionOptions,
) -> Vec<TAtomic> {
    let mut target_type = return_type_value_of.get_target_type().clone();
    expand_union(codebase, &mut target_type, options);

    let Some(new_return_types) = TValueOf::get_value_of_targets(&target_type.types, codebase, false) else {
        return vec![TAtomic::Derived(TDerived::ValueOf(return_type_value_of.clone()))];
    };

    new_return_types.types.into_owned()
}

#[cold]
fn expand_index_access(
    return_type_index_access: &TIndexAccess,
    codebase: &CodebaseMetadata,
    options: &TypeExpansionOptions,
) -> Vec<TAtomic> {
    let mut target_type = return_type_index_access.get_target_type().clone();
    expand_union(codebase, &mut target_type, options);

    let mut index_type = return_type_index_access.get_index_type().clone();
    expand_union(codebase, &mut index_type, options);

    let Some(new_return_types) = TIndexAccess::get_indexed_access_result(&target_type.types, &index_type.types, false)
    else {
        return vec![TAtomic::Derived(TDerived::IndexAccess(return_type_index_access.clone()))];
    };

    new_return_types.types.into_owned()
}

#[cold]
fn expand_int_mask(int_mask: &TIntMask, codebase: &CodebaseMetadata, options: &TypeExpansionOptions) -> Vec<TAtomic> {
    let mut literal_values = Vec::new();

    for value in int_mask.get_values() {
        let mut expanded = value.clone();
        expand_union(codebase, &mut expanded, options);

        if let Some(int_val) = expanded.get_single_literal_int_value() {
            literal_values.push(int_val);
        }
    }

    if literal_values.is_empty() {
        return vec![TAtomic::Scalar(TScalar::int())];
    }

    let combinations = TIntMask::calculate_mask_combinations(&literal_values);
    combinations.into_iter().map(|v| TAtomic::Scalar(TScalar::literal_int(v))).collect()
}

#[cold]
fn expand_int_mask_of(
    int_mask_of: &TIntMaskOf,
    codebase: &CodebaseMetadata,
    options: &TypeExpansionOptions,
) -> Vec<TAtomic> {
    let mut target = int_mask_of.get_target_type().clone();
    expand_union(codebase, &mut target, options);

    let mut literal_values = Vec::new();
    for atomic in target.types.iter() {
        if let Some(int_val) = atomic.get_literal_int_value() {
            literal_values.push(int_val);
        }
    }

    if literal_values.is_empty() {
        return vec![TAtomic::Scalar(TScalar::int())];
    }

    let combinations = TIntMask::calculate_mask_combinations(&literal_values);
    combinations.into_iter().map(|v| TAtomic::Scalar(TScalar::literal_int(v))).collect()
}

#[cold]
fn expand_properties_of(
    properties_of: &TPropertiesOf,
    codebase: &CodebaseMetadata,
    options: &TypeExpansionOptions,
) -> Vec<TAtomic> {
    let mut target_type = properties_of.get_target_type().clone();
    expand_union(codebase, &mut target_type, options);

    let Some(keyed_array) =
        TPropertiesOf::get_properties_of_targets(&target_type.types, codebase, properties_of.visibility(), false)
    else {
        return vec![TAtomic::Derived(TDerived::PropertiesOf(properties_of.clone()))];
    };

    vec![keyed_array]
}

#[cold]
fn expand_alias(alias: &TAlias, codebase: &CodebaseMetadata, options: &TypeExpansionOptions) -> Vec<TAtomic> {
    let class_name = alias.get_class_name();
    let alias_name = alias.get_alias_name();

    // Check for cycle using the HashSet
    let is_cycle = EXPANDING_ALIASES.with(|set| set.borrow().contains(&(class_name, alias_name)));

    if is_cycle {
        return vec![TAtomic::Alias(alias.clone())];
    }

    let Some(mut expanded_union) = alias.resolve(codebase).cloned() else {
        return vec![TAtomic::Alias(alias.clone())];
    };

    let _ = AliasExpansionGuard::new(class_name, alias_name);

    expand_union(codebase, &mut expanded_union, options);

    expanded_union.types.into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::borrow::Cow;

    use bumpalo::Bump;

    use mago_atom::atom;
    use mago_database::Database;
    use mago_database::DatabaseReader;
    use mago_database::file::File;
    use mago_database::file::FileId;
    use mago_names::resolver::NameResolver;
    use mago_span::Position;
    use mago_syntax::parser::parse_file;

    use crate::metadata::CodebaseMetadata;
    use crate::misc::GenericParent;
    use crate::populator::populate_codebase;
    use crate::reference::SymbolReferences;
    use crate::scanner::scan_program;
    use crate::ttype::atomic::array::TArray;
    use crate::ttype::atomic::array::keyed::TKeyedArray;
    use crate::ttype::atomic::array::list::TList;
    use crate::ttype::atomic::callable::TCallable;
    use crate::ttype::atomic::callable::TCallableSignature;
    use crate::ttype::atomic::callable::parameter::TCallableParameter;
    use crate::ttype::atomic::conditional::TConditional;
    use crate::ttype::atomic::derived::TDerived;
    use crate::ttype::atomic::derived::index_access::TIndexAccess;
    use crate::ttype::atomic::derived::key_of::TKeyOf;
    use crate::ttype::atomic::derived::value_of::TValueOf;
    use crate::ttype::atomic::generic::TGenericParameter;
    use crate::ttype::atomic::iterable::TIterable;
    use crate::ttype::atomic::object::r#enum::TEnum;
    use crate::ttype::atomic::object::named::TNamedObject;
    use crate::ttype::atomic::reference::TReference;
    use crate::ttype::atomic::reference::TReferenceMemberSelector;
    use crate::ttype::atomic::scalar::TScalar;
    use crate::ttype::atomic::scalar::class_like_string::TClassLikeString;
    use crate::ttype::atomic::scalar::class_like_string::TClassLikeStringKind;
    use crate::ttype::get_int;
    use crate::ttype::get_mixed;
    use crate::ttype::get_never;
    use crate::ttype::get_null;
    use crate::ttype::get_string;
    use crate::ttype::get_void;
    use crate::ttype::union::UnionFlags;

    fn create_test_codebase(code: &'static str) -> CodebaseMetadata {
        let file = File::ephemeral(Cow::Borrowed("code.php"), Cow::Borrowed(code));
        let config =
            mago_database::DatabaseConfiguration::new(std::path::Path::new("/"), vec![], vec![], vec![], vec![])
                .into_static();
        let database = Database::single(file, config);

        let mut codebase = CodebaseMetadata::new();
        let arena = Bump::new();
        for file in database.files() {
            let program = parse_file(&arena, &file).0;
            let resolved_names = NameResolver::new(&arena).resolve(program);
            let program_codebase = scan_program(&arena, &file, program, &resolved_names);

            codebase.extend(program_codebase);
        }

        populate_codebase(&mut codebase, &mut SymbolReferences::new(), Default::default(), Default::default());

        codebase
    }

    fn options_with_self(self_class: &str) -> TypeExpansionOptions {
        TypeExpansionOptions { self_class: Some(ascii_lowercase_atom(self_class)), ..Default::default() }
    }

    fn options_with_static(static_class: &str) -> TypeExpansionOptions {
        TypeExpansionOptions {
            self_class: Some(ascii_lowercase_atom(static_class)),
            static_class_type: StaticClassType::Name(ascii_lowercase_atom(static_class)),
            ..Default::default()
        }
    }

    fn options_with_static_object(object: TObject) -> TypeExpansionOptions {
        let name = object.get_name().copied();
        TypeExpansionOptions {
            self_class: name,
            static_class_type: StaticClassType::Object(object),
            ..Default::default()
        }
    }

    macro_rules! assert_expands_to {
        ($codebase:expr, $input:expr, $expected:expr) => {
            assert_expands_to!($codebase, $input, $expected, &TypeExpansionOptions::default())
        };
        ($codebase:expr, $input:expr, $expected:expr, $options:expr) => {{
            let mut actual = $input.clone();
            expand_union($codebase, &mut actual, $options);
            assert_eq!(
                actual.types.as_ref(),
                $expected.types.as_ref(),
                "Type expansion mismatch.\nInput: {:?}\nExpected: {:?}\nActual: {:?}",
                $input,
                $expected,
                actual
            );
        }};
    }

    fn make_self_object() -> TUnion {
        TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom("self")))))
    }

    fn make_static_object() -> TUnion {
        TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom("static")))))
    }

    fn make_parent_object() -> TUnion {
        TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom("parent")))))
    }

    fn make_named_object(name: &str) -> TUnion {
        TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_atom(name)))))
    }

    #[test]
    fn test_expand_null_type() {
        let codebase = CodebaseMetadata::new();
        let null_type = get_null();
        assert_expands_to!(&codebase, null_type, get_null());
    }

    #[test]
    fn test_expand_void_type() {
        let codebase = CodebaseMetadata::new();
        let void_type = get_void();
        assert_expands_to!(&codebase, void_type, get_void());
    }

    #[test]
    fn test_expand_never_type() {
        let codebase = CodebaseMetadata::new();
        let never_type = get_never();
        assert_expands_to!(&codebase, never_type, get_never());
    }

    #[test]
    fn test_expand_int_type() {
        let codebase = CodebaseMetadata::new();
        let int_type = get_int();
        assert_expands_to!(&codebase, int_type, get_int());
    }

    #[test]
    fn test_expand_mixed_type() {
        let codebase = CodebaseMetadata::new();
        let mixed_type = get_mixed();
        assert_expands_to!(&codebase, mixed_type, get_mixed());
    }

    #[test]
    fn test_expand_keyed_array_with_self_key() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let mut keyed = TKeyedArray::new();
        keyed.parameters = Some((Box::new(make_self_object()), Box::new(get_int())));
        let input = TUnion::from_atomic(TAtomic::Array(TArray::Keyed(keyed)));

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        if let TAtomic::Array(TArray::Keyed(keyed)) = &actual.types[0]
            && let Some((key, _)) = &keyed.parameters
        {
            assert!(key.types.iter().any(|t| {
                if let TAtomic::Object(TObject::Named(named)) = t {
                    named.name == ascii_lowercase_atom("foo")
                } else {
                    false
                }
            }));
        }
    }

    #[test]
    fn test_expand_keyed_array_with_self_value() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let mut keyed = TKeyedArray::new();
        keyed.parameters = Some((Box::new(get_string()), Box::new(make_self_object())));
        let input = TUnion::from_atomic(TAtomic::Array(TArray::Keyed(keyed)));

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        if let TAtomic::Array(TArray::Keyed(keyed)) = &actual.types[0]
            && let Some((_, value)) = &keyed.parameters
        {
            assert!(value.types.iter().any(|t| {
                if let TAtomic::Object(TObject::Named(named)) = t {
                    named.name == ascii_lowercase_atom("foo")
                } else {
                    false
                }
            }));
        }
    }

    #[test]
    fn test_expand_keyed_array_known_items() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        use crate::ttype::atomic::array::key::ArrayKey;
        use std::collections::BTreeMap;

        let mut keyed = TKeyedArray::new();
        let mut known_items = BTreeMap::new();
        known_items.insert(ArrayKey::String(atom("key")), (false, make_self_object()));
        keyed.known_items = Some(known_items);
        let input = TUnion::from_atomic(TAtomic::Array(TArray::Keyed(keyed)));

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        if let TAtomic::Array(TArray::Keyed(keyed)) = &actual.types[0]
            && let Some(items) = &keyed.known_items
        {
            let (_, item_type) = items.get(&ArrayKey::String(atom("key"))).unwrap();
            assert!(item_type.types.iter().any(|t| {
                if let TAtomic::Object(TObject::Named(named)) = t {
                    named.name == ascii_lowercase_atom("foo")
                } else {
                    false
                }
            }));
        }
    }

    #[test]
    fn test_expand_list_with_self_element() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let list = TList::new(Box::new(make_self_object()));
        let input = TUnion::from_atomic(TAtomic::Array(TArray::List(list)));

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        if let TAtomic::Array(TArray::List(list)) = &actual.types[0] {
            assert!(list.element_type.types.iter().any(|t| {
                if let TAtomic::Object(TObject::Named(named)) = t {
                    named.name == ascii_lowercase_atom("foo")
                } else {
                    false
                }
            }));
        }
    }

    #[test]
    fn test_expand_list_known_elements() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        use std::collections::BTreeMap;

        let mut list = TList::new(Box::new(get_mixed()));
        let mut known_elements = BTreeMap::new();
        known_elements.insert(0, (false, make_self_object()));
        list.known_elements = Some(known_elements);
        let input = TUnion::from_atomic(TAtomic::Array(TArray::List(list)));

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        if let TAtomic::Array(TArray::List(list)) = &actual.types[0]
            && let Some(elements) = &list.known_elements
        {
            let (_, element_type) = elements.get(&0).unwrap();
            assert!(element_type.types.iter().any(|t| {
                if let TAtomic::Object(TObject::Named(named)) = t {
                    named.name == ascii_lowercase_atom("foo")
                } else {
                    false
                }
            }));
        }
    }

    #[test]
    fn test_expand_nested_array() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let inner_list = TList::new(Box::new(make_self_object()));
        let inner_array = TUnion::from_atomic(TAtomic::Array(TArray::List(inner_list)));

        let mut outer = TKeyedArray::new();
        outer.parameters = Some((Box::new(make_self_object()), Box::new(inner_array)));
        let input = TUnion::from_atomic(TAtomic::Array(TArray::Keyed(outer)));

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        if let TAtomic::Array(TArray::Keyed(keyed)) = &actual.types[0]
            && let Some((key, value)) = &keyed.parameters
        {
            assert!(key.types.iter().any(|t| {
                if let TAtomic::Object(TObject::Named(named)) = t {
                    named.name == ascii_lowercase_atom("foo")
                } else {
                    false
                }
            }));
            if let TAtomic::Array(TArray::List(inner)) = &value.types[0] {
                assert!(inner.element_type.types.iter().any(|t| {
                    if let TAtomic::Object(TObject::Named(named)) = t {
                        named.name == ascii_lowercase_atom("foo")
                    } else {
                        false
                    }
                }));
            }
        }
    }

    #[test]
    fn test_expand_empty_array() {
        let codebase = CodebaseMetadata::new();
        let keyed = TKeyedArray::new();
        let input = TUnion::from_atomic(TAtomic::Array(TArray::Keyed(keyed.clone())));
        let expected = TUnion::from_atomic(TAtomic::Array(TArray::Keyed(keyed)));
        assert_expands_to!(&codebase, input, expected);
    }

    #[test]
    fn test_expand_non_empty_list() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let mut list = TList::new(Box::new(make_self_object()));
        list.non_empty = true;
        let input = TUnion::from_atomic(TAtomic::Array(TArray::List(list)));

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        if let TAtomic::Array(TArray::List(list)) = &actual.types[0] {
            assert!(list.non_empty);
            assert!(list.element_type.types.iter().any(|t| {
                if let TAtomic::Object(TObject::Named(named)) = t {
                    named.name == ascii_lowercase_atom("foo")
                } else {
                    false
                }
            }));
        }
    }

    #[test]
    fn test_expand_self_to_class_name() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let input = make_self_object();
        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        assert!(actual.types.iter().any(|t| {
            if let TAtomic::Object(TObject::Named(named)) = t {
                named.name == ascii_lowercase_atom("foo")
            } else {
                false
            }
        }));
    }

    #[test]
    fn test_expand_static_to_class_name() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let input = make_static_object();
        let options = options_with_static("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        assert!(actual.types.iter().any(|t| {
            if let TAtomic::Object(TObject::Named(named)) = t {
                named.name == ascii_lowercase_atom("foo")
            } else {
                false
            }
        }));
    }

    #[test]
    fn test_expand_static_with_object_type() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let input = make_static_object();
        let static_obj = TObject::Named(TNamedObject::new(ascii_lowercase_atom("foo")));
        let options = options_with_static_object(static_obj);
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        assert!(actual.types.iter().any(|t| {
            if let TAtomic::Object(TObject::Named(named)) = t {
                named.name == ascii_lowercase_atom("foo") && named.is_this
            } else {
                false
            }
        }));
    }

    #[test]
    fn test_expand_static_with_enum_type() {
        let code = r"<?php enum Status { case Active; case Inactive; }";
        let codebase = create_test_codebase(code);

        let input = make_static_object();
        let static_enum = TObject::Enum(TEnum::new(ascii_lowercase_atom("status")));
        let options = options_with_static_object(static_enum);
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        assert!(actual.types.iter().any(|t| matches!(t, TAtomic::Object(TObject::Enum(_)))));
    }

    #[test]
    fn test_expand_parent_to_parent_class() {
        let code = r"<?php
            class BaseClass {}
            class ChildClass extends BaseClass {}
        ";
        let codebase = create_test_codebase(code);

        let input = make_parent_object();
        let options = options_with_self("ChildClass");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        assert!(actual.types.iter().any(|t| {
            if let TAtomic::Object(TObject::Named(named)) = t {
                named.name == ascii_lowercase_atom("baseclass")
            } else {
                false
            }
        }));
    }

    #[test]
    fn test_expand_parent_without_parent_class() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let input = make_parent_object();
        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        assert!(actual.types.iter().any(|t| {
            if let TAtomic::Object(TObject::Named(named)) = t { named.name == atom("parent") } else { false }
        }));
    }

    #[test]
    fn test_expand_this_variable() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let input = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_this(atom("$this")))));
        let options = options_with_static("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        assert!(actual.types.iter().any(|t| {
            if let TAtomic::Object(TObject::Named(named)) = t {
                named.name == ascii_lowercase_atom("foo")
            } else {
                false
            }
        }));
    }

    #[test]
    fn test_expand_this_with_final_function() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let input = make_static_object();
        let options = TypeExpansionOptions {
            self_class: Some(ascii_lowercase_atom("foo")),
            static_class_type: StaticClassType::Name(ascii_lowercase_atom("foo")),
            function_is_final: true,
            ..Default::default()
        };
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        assert!(actual.types.iter().any(|t| {
            if let TAtomic::Object(TObject::Named(named)) = t {
                named.name == ascii_lowercase_atom("foo") && !named.is_this
            } else {
                false
            }
        }));
    }

    #[test]
    fn test_expand_object_with_type_parameters() {
        let code = r"<?php class Container {}";
        let codebase = create_test_codebase(code);

        let named =
            TNamedObject::new_with_type_parameters(ascii_lowercase_atom("container"), Some(vec![make_self_object()]));
        let input = TUnion::from_atomic(TAtomic::Object(TObject::Named(named)));

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        if let TAtomic::Object(TObject::Named(named)) = &actual.types[0]
            && let Some(params) = &named.type_parameters
        {
            assert!(params[0].types.iter().any(|t| {
                if let TAtomic::Object(TObject::Named(named)) = t {
                    named.name == ascii_lowercase_atom("foo")
                } else {
                    false
                }
            }));
        }
    }

    #[test]
    fn test_expand_object_gets_default_type_params() {
        let code = r"<?php
            /** @template T */
            class Container {}
        ";
        let codebase = create_test_codebase(code);

        let named = TNamedObject::new(ascii_lowercase_atom("container"));
        let input = TUnion::from_atomic(TAtomic::Object(TObject::Named(named)));

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &TypeExpansionOptions::default());

        if let TAtomic::Object(TObject::Named(named)) = &actual.types[0] {
            assert!(named.type_parameters.is_some());
        }
    }

    #[test]
    fn test_expand_object_intersection_from_static() {
        let code = r"<?php
            interface Stringable {}
            class Foo implements Stringable {}
        ";
        let codebase = create_test_codebase(code);

        let input = make_static_object();

        let mut static_named = TNamedObject::new(ascii_lowercase_atom("foo"));
        static_named.intersection_types =
            Some(vec![TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_atom("stringable"))))]);
        let static_obj = TObject::Named(static_named);
        let options = options_with_static_object(static_obj);

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        if let TAtomic::Object(TObject::Named(named)) = &actual.types[0] {
            assert!(named.intersection_types.is_some());
        }
    }

    #[test]
    fn test_expand_self_without_self_class_option() {
        let codebase = CodebaseMetadata::new();

        let input = make_self_object();
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &TypeExpansionOptions::default());

        assert!(actual.types.iter().any(|t| {
            if let TAtomic::Object(TObject::Named(named)) = t { named.name == atom("self") } else { false }
        }));
    }

    #[test]
    fn test_expand_callable_return_type() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let sig = TCallableSignature::new(false, false).with_return_type(Some(Box::new(make_self_object())));
        let input = TUnion::from_atomic(TAtomic::Callable(TCallable::Signature(sig)));

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        if let TAtomic::Callable(TCallable::Signature(sig)) = &actual.types[0]
            && let Some(ret) = sig.get_return_type()
        {
            assert!(ret.types.iter().any(|t| {
                if let TAtomic::Object(TObject::Named(named)) = t {
                    named.name == ascii_lowercase_atom("foo")
                } else {
                    false
                }
            }));
        }
    }

    #[test]
    fn test_expand_callable_parameter_types() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let param = TCallableParameter::new(Some(Box::new(make_self_object())), false, false, false);
        let sig = TCallableSignature::new(false, false).with_parameters(vec![param]);
        let input = TUnion::from_atomic(TAtomic::Callable(TCallable::Signature(sig)));

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        if let TAtomic::Callable(TCallable::Signature(sig)) = &actual.types[0]
            && let Some(param) = sig.get_parameters().first()
            && let Some(param_type) = param.get_type_signature()
        {
            assert!(param_type.types.iter().any(|t| {
                if let TAtomic::Object(TObject::Named(named)) = t {
                    named.name == ascii_lowercase_atom("foo")
                } else {
                    false
                }
            }));
        }
    }

    #[test]
    fn test_expand_callable_alias_to_function() {
        let code = r"<?php
            function myFunc(): int { return 1; }
        ";
        let codebase = create_test_codebase(code);

        let alias = TCallable::Alias(FunctionLikeIdentifier::Function(ascii_lowercase_atom("myfunc")));
        let input = TUnion::from_atomic(TAtomic::Callable(alias));

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &TypeExpansionOptions::default());

        assert!(actual.types.iter().any(|t| matches!(t, TAtomic::Callable(TCallable::Signature(_)))));
    }

    #[test]
    fn test_expand_callable_alias_to_method() {
        let code = r"<?php
            class Foo {
                public function bar(): int { return 1; }
            }
        ";
        let codebase = create_test_codebase(code);

        let alias =
            TCallable::Alias(FunctionLikeIdentifier::Method(ascii_lowercase_atom("foo"), ascii_lowercase_atom("bar")));
        let input = TUnion::from_atomic(TAtomic::Callable(alias));

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &TypeExpansionOptions::default());

        assert!(actual.types.iter().any(|t| matches!(t, TAtomic::Callable(TCallable::Signature(_)))));
    }

    #[test]
    fn test_expand_callable_alias_unknown() {
        let codebase = CodebaseMetadata::new();

        let alias = TCallable::Alias(FunctionLikeIdentifier::Function(atom("nonexistent")));
        let input = TUnion::from_atomic(TAtomic::Callable(alias.clone()));

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &TypeExpansionOptions::default());

        assert!(actual.types.iter().any(|t| matches!(t, TAtomic::Callable(TCallable::Alias(_)))));
    }

    #[test]
    fn test_expand_closure_signature() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let sig = TCallableSignature::new(false, true).with_return_type(Some(Box::new(make_self_object())));
        let input = TUnion::from_atomic(TAtomic::Callable(TCallable::Signature(sig)));

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        if let TAtomic::Callable(TCallable::Signature(sig)) = &actual.types[0]
            && let Some(ret) = sig.get_return_type()
        {
            assert!(ret.types.iter().any(|t| {
                if let TAtomic::Object(TObject::Named(named)) = t {
                    named.name == ascii_lowercase_atom("foo")
                } else {
                    false
                }
            }));
        }
    }

    #[test]
    fn test_expand_generic_parameter_constraint() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let generic = TGenericParameter::new(
            atom("T"),
            Box::new(make_self_object()),
            GenericParent::ClassLike(ascii_lowercase_atom("foo")),
        );
        let input = TUnion::from_atomic(TAtomic::GenericParameter(generic));

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        if let TAtomic::GenericParameter(param) = &actual.types[0] {
            assert!(param.constraint.types.iter().any(|t| {
                if let TAtomic::Object(TObject::Named(named)) = t {
                    named.name == ascii_lowercase_atom("foo")
                } else {
                    false
                }
            }));
        }
    }

    #[test]
    fn test_expand_nested_generic_constraint() {
        let code = r"<?php class Foo {} class Bar {}";
        let codebase = create_test_codebase(code);

        let container =
            TNamedObject::new_with_type_parameters(ascii_lowercase_atom("container"), Some(vec![make_self_object()]));
        let constraint = TUnion::from_atomic(TAtomic::Object(TObject::Named(container)));

        let generic = TGenericParameter::new(
            atom("T"),
            Box::new(constraint),
            GenericParent::ClassLike(ascii_lowercase_atom("bar")),
        );
        let input = TUnion::from_atomic(TAtomic::GenericParameter(generic));

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        if let TAtomic::GenericParameter(param) = &actual.types[0]
            && let TAtomic::Object(TObject::Named(named)) = &param.constraint.types[0]
            && let Some(params) = &named.type_parameters
        {
            assert!(params[0].types.iter().any(|t| {
                if let TAtomic::Object(TObject::Named(named)) = t {
                    named.name == ascii_lowercase_atom("foo")
                } else {
                    false
                }
            }));
        }
    }

    #[test]
    fn test_expand_generic_with_intersection() {
        let code = r"<?php
            interface Stringable {}
            class Foo {}
        ";
        let codebase = create_test_codebase(code);

        let mut generic = TGenericParameter::new(
            atom("T"),
            Box::new(make_self_object()),
            GenericParent::ClassLike(ascii_lowercase_atom("foo")),
        );
        generic.intersection_types =
            Some(vec![TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_atom("stringable"))))]);
        let input = TUnion::from_atomic(TAtomic::GenericParameter(generic));

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        if let TAtomic::GenericParameter(param) = &actual.types[0] {
            assert!(param.intersection_types.is_some());
            assert!(param.constraint.types.iter().any(|t| {
                if let TAtomic::Object(TObject::Named(named)) = t {
                    named.name == ascii_lowercase_atom("foo")
                } else {
                    false
                }
            }));
        }
    }

    #[test]
    fn test_expand_class_string_of_self() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let constraint = Box::new(TAtomic::Object(TObject::Named(TNamedObject::new(atom("self")))));
        let class_string = TClassLikeString::OfType { kind: TClassLikeStringKind::Class, constraint };
        let input = TUnion::from_atomic(TAtomic::Scalar(TScalar::ClassLikeString(class_string)));

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        if let TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::OfType { constraint, .. })) = &actual.types[0]
            && let TAtomic::Object(TObject::Named(named)) = constraint.as_ref()
        {
            assert_eq!(named.name, ascii_lowercase_atom("foo"));
        }
    }

    #[test]
    fn test_expand_class_string_of_static() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let constraint = Box::new(TAtomic::Object(TObject::Named(TNamedObject::new(atom("static")))));
        let class_string = TClassLikeString::OfType { kind: TClassLikeStringKind::Class, constraint };
        let input = TUnion::from_atomic(TAtomic::Scalar(TScalar::ClassLikeString(class_string)));

        let options = options_with_static("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        if let TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::OfType { constraint, .. })) = &actual.types[0]
            && let TAtomic::Object(TObject::Named(named)) = constraint.as_ref()
        {
            assert_eq!(named.name, ascii_lowercase_atom("foo"));
        }
    }

    #[test]
    fn test_expand_interface_string_of_type() {
        let code = r"<?php interface MyInterface {}";
        let codebase = create_test_codebase(code);

        let constraint = Box::new(TAtomic::Object(TObject::Named(TNamedObject::new(atom("self")))));
        let class_string = TClassLikeString::OfType { kind: TClassLikeStringKind::Interface, constraint };
        let input = TUnion::from_atomic(TAtomic::Scalar(TScalar::ClassLikeString(class_string)));

        let options = options_with_self("MyInterface");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        if let TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::OfType { kind, constraint })) =
            &actual.types[0]
        {
            assert!(matches!(kind, TClassLikeStringKind::Interface));
            if let TAtomic::Object(TObject::Named(named)) = constraint.as_ref() {
                assert_eq!(named.name, ascii_lowercase_atom("myinterface"));
            }
        }
    }

    #[test]
    fn test_expand_member_reference_wildcard_constants() {
        let code = r"<?php
            class Foo {
                public const A = 1;
                public const B = 2;
            }
        ";
        let codebase = create_test_codebase(code);

        let reference = TReference::new_member(ascii_lowercase_atom("foo"), TReferenceMemberSelector::Wildcard);
        let input = TUnion::from_atomic(TAtomic::Reference(reference));

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &TypeExpansionOptions::default());

        assert!(!actual.types.is_empty());
    }

    #[test]
    fn test_expand_member_reference_wildcard_enum_cases() {
        let code = r"<?php
            enum Status {
                case Active;
                case Inactive;
            }
        ";
        let codebase = create_test_codebase(code);

        let reference = TReference::new_member(ascii_lowercase_atom("status"), TReferenceMemberSelector::Wildcard);
        let input = TUnion::from_atomic(TAtomic::Reference(reference));

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &TypeExpansionOptions::default());

        assert_eq!(actual.types.len(), 2);
        assert!(actual.types.iter().all(|t| matches!(t, TAtomic::Object(TObject::Enum(_)))));
    }

    #[test]
    fn test_expand_member_reference_starts_with() {
        let code = r"<?php
            class Foo {
                public const STATUS_ACTIVE = 1;
                public const STATUS_INACTIVE = 2;
                public const OTHER = 3;
            }
        ";
        let codebase = create_test_codebase(code);

        let reference =
            TReference::new_member(ascii_lowercase_atom("foo"), TReferenceMemberSelector::StartsWith(atom("STATUS_")));
        let input = TUnion::from_atomic(TAtomic::Reference(reference));

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &TypeExpansionOptions::default());

        assert!(!actual.types.is_empty());
    }

    #[test]
    fn test_expand_member_reference_ends_with() {
        let code = r"<?php
            class Foo {
                public const READ_ERROR = 1;
                public const WRITE_ERROR = 2;
                public const SUCCESS = 0;
            }
        ";
        let codebase = create_test_codebase(code);

        let reference =
            TReference::new_member(ascii_lowercase_atom("foo"), TReferenceMemberSelector::EndsWith(atom("_ERROR")));
        let input = TUnion::from_atomic(TAtomic::Reference(reference));

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &TypeExpansionOptions::default());

        assert!(!actual.types.is_empty());
    }

    #[test]
    fn test_expand_member_reference_identifier_constant() {
        let code = r"<?php
            class Foo {
                public const BAR = 42;
            }
        ";
        let codebase = create_test_codebase(code);

        let reference =
            TReference::new_member(ascii_lowercase_atom("foo"), TReferenceMemberSelector::Identifier(atom("BAR")));
        let input = TUnion::from_atomic(TAtomic::Reference(reference));

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &TypeExpansionOptions::default());

        assert_eq!(actual.types.len(), 1);
    }

    #[test]
    fn test_expand_member_reference_identifier_enum_case() {
        let code = r"<?php
            enum Status {
                case Active;
            }
        ";
        let codebase = create_test_codebase(code);

        let reference = TReference::new_member(
            ascii_lowercase_atom("status"),
            TReferenceMemberSelector::Identifier(atom("Active")),
        );
        let input = TUnion::from_atomic(TAtomic::Reference(reference));

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &TypeExpansionOptions::default());

        assert_eq!(actual.types.len(), 1);
        assert!(matches!(&actual.types[0], TAtomic::Object(TObject::Enum(_))));
    }

    #[test]
    fn test_expand_member_reference_unknown_class() {
        let codebase = CodebaseMetadata::new();

        let reference = TReference::new_member(atom("NonExistent"), TReferenceMemberSelector::Identifier(atom("FOO")));
        let input = TUnion::from_atomic(TAtomic::Reference(reference));

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &TypeExpansionOptions::default());

        assert!(actual.types.iter().any(|t| matches!(t, TAtomic::Mixed(_))));
    }

    #[test]
    fn test_expand_member_reference_unknown_member() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let reference = TReference::new_member(
            ascii_lowercase_atom("foo"),
            TReferenceMemberSelector::Identifier(atom("NONEXISTENT")),
        );
        let input = TUnion::from_atomic(TAtomic::Reference(reference));

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &TypeExpansionOptions::default());

        assert!(actual.types.iter().any(|t| matches!(t, TAtomic::Mixed(_))));
    }

    #[test]
    fn test_expand_member_reference_constant_with_inferred_type() {
        let code = r#"<?php
            class Foo {
                public const VALUE = "hello";
            }
        "#;
        let codebase = create_test_codebase(code);

        let reference =
            TReference::new_member(ascii_lowercase_atom("foo"), TReferenceMemberSelector::Identifier(atom("VALUE")));
        let input = TUnion::from_atomic(TAtomic::Reference(reference));

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &TypeExpansionOptions::default());

        assert_eq!(actual.types.len(), 1);
    }

    #[test]
    fn test_expand_member_reference_constant_with_type_metadata() {
        let code = r"<?php
            class Foo {
                /** @var int */
                public const VALUE = 42;
            }
        ";
        let codebase = create_test_codebase(code);

        let reference =
            TReference::new_member(ascii_lowercase_atom("foo"), TReferenceMemberSelector::Identifier(atom("VALUE")));
        let input = TUnion::from_atomic(TAtomic::Reference(reference));

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &TypeExpansionOptions::default());

        assert_eq!(actual.types.len(), 1);
    }

    #[test]
    fn test_expand_conditional_both_branches() {
        let code = r"<?php class Foo {} class Bar {}";
        let codebase = create_test_codebase(code);

        let conditional = TConditional::new(
            Box::new(get_mixed()),
            Box::new(get_string()),
            Box::new(make_self_object()),
            Box::new(make_self_object()),
            false,
        );
        let input = TUnion::from_atomic(TAtomic::Conditional(conditional));

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        assert!(actual.types.iter().any(|t| {
            if let TAtomic::Object(TObject::Named(named)) = t {
                named.name == ascii_lowercase_atom("foo")
            } else {
                false
            }
        }));
    }

    #[test]
    fn test_expand_conditional_with_self_in_then() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let conditional = TConditional::new(
            Box::new(get_mixed()),
            Box::new(get_string()),
            Box::new(make_self_object()),
            Box::new(get_int()),
            false,
        );
        let input = TUnion::from_atomic(TAtomic::Conditional(conditional));

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        assert!(!actual.types.is_empty());
    }

    #[test]
    fn test_expand_conditional_with_self_in_otherwise() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let conditional = TConditional::new(
            Box::new(get_mixed()),
            Box::new(get_string()),
            Box::new(get_int()),
            Box::new(make_self_object()),
            false,
        );
        let input = TUnion::from_atomic(TAtomic::Conditional(conditional));

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        assert!(!actual.types.is_empty());
    }

    #[test]
    fn test_expand_simple_alias() {
        let code = r"<?php
            class Foo {
                /** @phpstan-type MyInt = int */
            }
        ";
        let codebase = create_test_codebase(code);

        let alias = TAlias::new(ascii_lowercase_atom("foo"), atom("MyInt"));
        let input = TUnion::from_atomic(TAtomic::Alias(alias));

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &TypeExpansionOptions::default());

        assert!(!actual.types.is_empty());
    }

    #[test]
    fn test_expand_nested_alias() {
        let code = r"<?php
            class Foo {
                /** @phpstan-type Inner = int */
                /** @phpstan-type Outer = Inner */
            }
        ";
        let codebase = create_test_codebase(code);

        let alias = TAlias::new(ascii_lowercase_atom("foo"), atom("Outer"));
        let input = TUnion::from_atomic(TAtomic::Alias(alias));

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &TypeExpansionOptions::default());

        assert!(!actual.types.is_empty());
    }

    #[test]
    fn test_expand_alias_cycle_detection() {
        let codebase = CodebaseMetadata::new();

        let alias = TAlias::new(atom("Foo"), atom("SelfRef"));
        let input = TUnion::from_atomic(TAtomic::Alias(alias.clone()));

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &TypeExpansionOptions::default());

        assert!(actual.types.iter().any(|t| matches!(t, TAtomic::Alias(_))));
    }

    #[test]
    fn test_expand_alias_unknown() {
        let codebase = CodebaseMetadata::new();

        let alias = TAlias::new(atom("NonExistent"), atom("Unknown"));
        let input = TUnion::from_atomic(TAtomic::Alias(alias.clone()));

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &TypeExpansionOptions::default());

        assert!(actual.types.iter().any(|t| matches!(t, TAtomic::Alias(_))));
    }

    #[test]
    fn test_expand_alias_with_self_inside() {
        let code = r"<?php
            class Foo {
                /** @phpstan-type MySelf = self */
            }
        ";
        let codebase = create_test_codebase(code);

        let alias = TAlias::new(ascii_lowercase_atom("foo"), atom("MySelf"));
        let input = TUnion::from_atomic(TAtomic::Alias(alias));

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        assert!(!actual.types.is_empty());
    }

    #[test]
    fn test_expand_key_of_array() {
        let codebase = CodebaseMetadata::new();

        let mut keyed = TKeyedArray::new();
        keyed.parameters = Some((Box::new(get_string()), Box::new(get_int())));
        let array_type = TUnion::from_atomic(TAtomic::Array(TArray::Keyed(keyed)));

        let key_of = TKeyOf::new(Box::new(array_type));
        let input = TUnion::from_atomic(TAtomic::Derived(TDerived::KeyOf(key_of)));

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &TypeExpansionOptions::default());

        assert!(actual.types.iter().any(super::super::atomic::TAtomic::is_string));
    }

    #[test]
    fn test_expand_key_of_with_self() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let mut keyed = TKeyedArray::new();
        keyed.parameters = Some((Box::new(make_self_object()), Box::new(get_int())));
        let array_type = TUnion::from_atomic(TAtomic::Array(TArray::Keyed(keyed)));

        let key_of = TKeyOf::new(Box::new(array_type));
        let input = TUnion::from_atomic(TAtomic::Derived(TDerived::KeyOf(key_of)));

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        assert!(!actual.types.is_empty());
    }

    #[test]
    fn test_expand_value_of_array() {
        let codebase = CodebaseMetadata::new();

        let mut keyed = TKeyedArray::new();
        keyed.parameters = Some((Box::new(get_string()), Box::new(get_int())));
        let array_type = TUnion::from_atomic(TAtomic::Array(TArray::Keyed(keyed)));

        let value_of = TValueOf::new(Box::new(array_type));
        let input = TUnion::from_atomic(TAtomic::Derived(TDerived::ValueOf(value_of)));

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &TypeExpansionOptions::default());

        assert!(actual.types.iter().any(super::super::atomic::TAtomic::is_int));
    }

    #[test]
    fn test_expand_value_of_enum() {
        let code = r"<?php
            enum Status: string {
                case Active = 'active';
                case Inactive = 'inactive';
            }
        ";
        let codebase = create_test_codebase(code);

        let enum_type = TUnion::from_atomic(TAtomic::Object(TObject::Enum(TEnum::new(ascii_lowercase_atom("status")))));

        let value_of = TValueOf::new(Box::new(enum_type));
        let input = TUnion::from_atomic(TAtomic::Derived(TDerived::ValueOf(value_of)));

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &TypeExpansionOptions::default());

        assert!(!actual.types.is_empty());
    }

    #[test]
    fn test_expand_index_access() {
        let codebase = CodebaseMetadata::new();

        use crate::ttype::atomic::array::key::ArrayKey;
        use std::collections::BTreeMap;

        let mut keyed = TKeyedArray::new();
        let mut known_items = BTreeMap::new();
        known_items.insert(ArrayKey::String(atom("key")), (false, get_int()));
        keyed.known_items = Some(known_items);
        let array_type = TUnion::from_atomic(TAtomic::Array(TArray::Keyed(keyed)));

        use crate::ttype::get_literal_string;
        let index_type = get_literal_string(atom("key"));

        let index_access = TIndexAccess::new(array_type, index_type);
        let input = TUnion::from_atomic(TAtomic::Derived(TDerived::IndexAccess(index_access)));

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &TypeExpansionOptions::default());

        assert!(!actual.types.is_empty());
    }

    #[test]
    fn test_expand_index_access_with_self() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        use crate::ttype::atomic::array::key::ArrayKey;
        use std::collections::BTreeMap;

        let mut keyed = TKeyedArray::new();
        let mut known_items = BTreeMap::new();
        known_items.insert(ArrayKey::String(atom("key")), (false, make_self_object()));
        keyed.known_items = Some(known_items);
        let array_type = TUnion::from_atomic(TAtomic::Array(TArray::Keyed(keyed)));

        use crate::ttype::get_literal_string;
        let index_type = get_literal_string(atom("key"));

        let index_access = TIndexAccess::new(array_type, index_type);
        let input = TUnion::from_atomic(TAtomic::Derived(TDerived::IndexAccess(index_access)));

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        assert!(!actual.types.is_empty());
    }

    #[test]
    fn test_expand_iterable_key_type() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let iterable = TIterable::new(Box::new(make_self_object()), Box::new(get_int()));
        let input = TUnion::from_atomic(TAtomic::Iterable(iterable));

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        if let TAtomic::Iterable(iter) = &actual.types[0] {
            assert!(iter.get_key_type().types.iter().any(|t| {
                if let TAtomic::Object(TObject::Named(named)) = t {
                    named.name == ascii_lowercase_atom("foo")
                } else {
                    false
                }
            }));
        }
    }

    #[test]
    fn test_expand_iterable_value_type() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let iterable = TIterable::new(Box::new(get_int()), Box::new(make_self_object()));
        let input = TUnion::from_atomic(TAtomic::Iterable(iterable));

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        if let TAtomic::Iterable(iter) = &actual.types[0] {
            assert!(iter.get_value_type().types.iter().any(|t| {
                if let TAtomic::Object(TObject::Named(named)) = t {
                    named.name == ascii_lowercase_atom("foo")
                } else {
                    false
                }
            }));
        }
    }

    #[test]
    fn test_get_signature_of_function() {
        let code = r#"<?php
            function myFunc(int $a): string { return ""; }
        "#;
        let codebase = create_test_codebase(code);

        let id = FunctionLikeIdentifier::Function(ascii_lowercase_atom("myfunc"));

        let sig = get_signature_of_function_like_identifier(&id, &codebase);
        assert!(sig.is_some());

        let sig = sig.unwrap();
        assert_eq!(sig.get_parameters().len(), 1);
        assert!(sig.get_return_type().is_some());
    }

    #[test]
    fn test_get_signature_of_method() {
        let code = r"<?php
            class Foo {
                public function bar(string $s): int { return 0; }
            }
        ";
        let codebase = create_test_codebase(code);

        let id = FunctionLikeIdentifier::Method(ascii_lowercase_atom("foo"), ascii_lowercase_atom("bar"));

        let sig = get_signature_of_function_like_identifier(&id, &codebase);
        assert!(sig.is_some());

        let sig = sig.unwrap();
        assert_eq!(sig.get_parameters().len(), 1);
    }

    #[test]
    fn test_get_signature_of_closure() {
        let codebase = CodebaseMetadata::new();

        let id = FunctionLikeIdentifier::Closure(FileId::new("test"), Position::new(0));
        let sig = get_signature_of_function_like_identifier(&id, &codebase);

        assert!(sig.is_none());
    }

    #[test]
    fn test_get_atomic_of_function() {
        let code = r"<?php
            function myFunc(): void {}
        ";
        let codebase = create_test_codebase(code);

        let id = FunctionLikeIdentifier::Function(ascii_lowercase_atom("myfunc"));

        let atomic = get_atomic_of_function_like_identifier(&id, &codebase);
        assert!(atomic.is_some());
        assert!(matches!(atomic.unwrap(), TAtomic::Callable(TCallable::Signature(_))));
    }

    #[test]
    fn test_get_signature_with_parameters() {
        let code = r"<?php
            function multiParam(int $a, string $b, ?float $c = null): bool { return true; }
        ";
        let codebase = create_test_codebase(code);

        let id = FunctionLikeIdentifier::Function(ascii_lowercase_atom("multiparam"));

        let sig = get_signature_of_function_like_identifier(&id, &codebase);
        assert!(sig.is_some());

        let sig = sig.unwrap();
        assert_eq!(sig.get_parameters().len(), 3);

        let third_param = &sig.get_parameters()[2];
        assert!(third_param.has_default());
    }

    #[test]
    fn test_expand_preserves_by_reference_flag() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let mut input = make_self_object();
        input.flags.insert(UnionFlags::BY_REFERENCE);

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        assert!(actual.flags.contains(UnionFlags::BY_REFERENCE));
    }

    #[test]
    fn test_expand_preserves_possibly_undefined_flag() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let mut input = make_self_object();
        input.flags.insert(UnionFlags::POSSIBLY_UNDEFINED);

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        assert!(actual.flags.contains(UnionFlags::POSSIBLY_UNDEFINED));
    }

    #[test]
    fn test_expand_multiple_self_in_union() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let input = TUnion::from_vec(vec![
            TAtomic::Object(TObject::Named(TNamedObject::new(atom("self")))),
            TAtomic::Object(TObject::Named(TNamedObject::new(atom("self")))),
        ]);

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        assert!(actual.types.len() <= 2);
    }

    #[test]
    fn test_expand_deeply_nested_types() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let inner = TList::new(Box::new(make_self_object()));
        let middle = TList::new(Box::new(TUnion::from_atomic(TAtomic::Array(TArray::List(inner)))));
        let outer = TList::new(Box::new(TUnion::from_atomic(TAtomic::Array(TArray::List(middle)))));
        let input = TUnion::from_atomic(TAtomic::Array(TArray::List(outer)));

        let options = options_with_self("Foo");
        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        if let TAtomic::Array(TArray::List(outer)) = &actual.types[0]
            && let TAtomic::Array(TArray::List(middle)) = &outer.element_type.types[0]
            && let TAtomic::Array(TArray::List(inner)) = &middle.element_type.types[0]
        {
            assert!(inner.element_type.types.iter().any(|t| {
                if let TAtomic::Object(TObject::Named(named)) = t {
                    named.name == ascii_lowercase_atom("foo")
                } else {
                    false
                }
            }));
        }
    }

    #[test]
    fn test_expand_with_all_options_disabled() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let input = make_self_object();
        let options = TypeExpansionOptions {
            self_class: None,
            static_class_type: StaticClassType::None,
            parent_class: None,
            evaluate_class_constants: false,
            evaluate_conditional_types: false,
            function_is_final: false,
            expand_generic: false,
            expand_templates: false,
        };

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        assert!(actual.types.iter().any(|t| {
            if let TAtomic::Object(TObject::Named(named)) = t { named.name == atom("self") } else { false }
        }));
    }

    #[test]
    fn test_expand_already_expanded_type() {
        let code = r"<?php class Foo {}";
        let codebase = create_test_codebase(code);

        let input = make_named_object("Foo");
        let options = options_with_self("Foo");

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        let mut actual2 = actual.clone();
        expand_union(&codebase, &mut actual2, &options);

        assert_eq!(actual.types.as_ref(), actual2.types.as_ref());
    }

    #[test]
    fn test_expand_complex_generic_class() {
        let code = r"<?php
            /**
             * @template T
             * @template U
             */
            class Container {}
        ";
        let codebase = create_test_codebase(code);

        let named = TNamedObject::new_with_type_parameters(
            ascii_lowercase_atom("container"),
            Some(vec![make_self_object(), make_static_object()]),
        );
        let input = TUnion::from_atomic(TAtomic::Object(TObject::Named(named)));

        let options = TypeExpansionOptions {
            self_class: Some(ascii_lowercase_atom("foo")),
            static_class_type: StaticClassType::Name(ascii_lowercase_atom("bar")),
            ..Default::default()
        };

        let mut actual = input.clone();
        expand_union(&codebase, &mut actual, &options);

        if let TAtomic::Object(TObject::Named(named)) = &actual.types[0]
            && let Some(params) = &named.type_parameters
        {
            assert!(params[0].types.iter().any(|t| {
                if let TAtomic::Object(TObject::Named(named)) = t {
                    named.name == ascii_lowercase_atom("foo")
                } else {
                    false
                }
            }));
            assert!(params[1].types.iter().any(|t| {
                if let TAtomic::Object(TObject::Named(named)) = t {
                    named.name == ascii_lowercase_atom("bar")
                } else {
                    false
                }
            }));
        }
    }
}
