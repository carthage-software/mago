use std::borrow::Cow;

use ahash::HashSet;
use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use mago_span::Position;
use mago_span::Span;

use crate::identifier::method::MethodIdentifier;
use crate::metadata::CodebaseMetadata;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::metadata::class_like_constant::ClassLikeConstantMetadata;
use crate::metadata::constant::ConstantMetadata;
use crate::metadata::enum_case::EnumCaseMetadata;
use crate::metadata::function_like::FunctionLikeMetadata;
use crate::metadata::property::PropertyMetadata;
use crate::symbol::SymbolKind;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::object::TObject;
use crate::ttype::union::TUnion;

pub mod assertion;
pub mod consts;
pub mod context;
pub mod data_flow;
pub mod diff;
pub mod flags;
pub mod identifier;
pub mod issue;
pub mod metadata;
pub mod misc;
pub mod populator;
pub mod reference;
pub mod scanner;
pub mod symbol;
pub mod ttype;
pub mod visibility;

mod utils;

/// Checks if a global function exists in the codebase.
///
/// This lookup is case-insensitive, in line with PHP's behavior for function names.
pub fn function_exists(codebase: &CodebaseMetadata, interner: &ThreadedInterner, id: &StringIdentifier) -> bool {
    let lowered_id = interner.lowered(id);
    codebase.function_likes.contains_key(&(StringIdentifier::empty(), lowered_id))
}

/// Checks if a global constant exists in the codebase.
///
/// The lookup for the namespace part of the constant name is case-insensitive,
/// but the constant name itself is case-sensitive, matching PHP's behavior.
pub fn constant_exists(codebase: &CodebaseMetadata, interner: &ThreadedInterner, id: &StringIdentifier) -> bool {
    let lowered_id = lower_constant_name(interner, id);
    codebase.constants.contains_key(&lowered_id)
}

/// Checks if a class exists in the codebase.
///
/// This lookup is case-insensitive, in line with PHP's behavior for class names.
pub fn class_exists(codebase: &CodebaseMetadata, interner: &ThreadedInterner, id: &StringIdentifier) -> bool {
    let lowered_id = interner.lowered(id);

    matches!(codebase.symbols.get_kind(&lowered_id), Some(SymbolKind::Class))
}

/// Checks if a class or trait exists in the codebase.
///
/// This lookup is case-insensitive, in line with PHP's behavior for class names.
pub fn class_or_trait_exists(codebase: &CodebaseMetadata, interner: &ThreadedInterner, id: &StringIdentifier) -> bool {
    let lowered_id = interner.lowered(id);

    matches!(codebase.symbols.get_kind(&lowered_id), Some(SymbolKind::Class | SymbolKind::Trait))
}

/// Checks if an interface exists in the codebase.
///
/// This lookup is case-insensitive, in line with PHP's behavior for interface names.
pub fn interface_exists(codebase: &CodebaseMetadata, interner: &ThreadedInterner, id: &StringIdentifier) -> bool {
    let lowered_id = interner.lowered(id);

    matches!(codebase.symbols.get_kind(&lowered_id), Some(SymbolKind::Interface))
}

/// Checks if a class or interface exists in the codebase.
///
/// This lookup is case-insensitive, in line with PHP's behavior for class names.
pub fn class_or_interface_exists(
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    id: &StringIdentifier,
) -> bool {
    let lowered_id = interner.lowered(id);

    matches!(codebase.symbols.get_kind(&lowered_id), Some(SymbolKind::Class | SymbolKind::Interface))
}

/// Checks if an enum exists in the codebase.
///
/// This lookup is case-insensitive, in line with PHP's behavior for enum names.
pub fn enum_exists(codebase: &CodebaseMetadata, interner: &ThreadedInterner, id: &StringIdentifier) -> bool {
    let lowered_id = interner.lowered(id);

    matches!(codebase.symbols.get_kind(&lowered_id), Some(SymbolKind::Enum))
}

/// Checks if a trait exists in the codebase.
///
/// This lookup is case-insensitive, in line with PHP's behavior for trait names.
pub fn trait_exists(codebase: &CodebaseMetadata, interner: &ThreadedInterner, id: &StringIdentifier) -> bool {
    let lowered_id = interner.lowered(id);

    matches!(codebase.symbols.get_kind(&lowered_id), Some(SymbolKind::Trait))
}

/// Checks if a class-like (class, interface, enum, or trait) exists in the codebase.
///
/// This lookup is case-insensitive.
pub fn class_like_exists(codebase: &CodebaseMetadata, interner: &ThreadedInterner, id: &StringIdentifier) -> bool {
    let lowered_id = interner.lowered(id);

    matches!(
        codebase.symbols.get_kind(&lowered_id),
        Some(SymbolKind::Class | SymbolKind::Interface | SymbolKind::Enum | SymbolKind::Trait)
    )
}

/// Checks if a method exists on a given class-like (including inherited methods).
///
/// The lookup for both the class-like name and the method name is case-insensitive.
pub fn method_exists(
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    fqc_id: &StringIdentifier,
    method_id: &StringIdentifier,
) -> bool {
    let lowered_fqc_id = interner.lowered(fqc_id);
    let lowered_method_id = interner.lowered(method_id);

    codebase
        .class_likes
        .get(&lowered_fqc_id)
        .is_some_and(|meta| meta.appearing_method_ids.contains_key(&lowered_method_id))
}

pub fn method_id_exists(
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    method_id: &MethodIdentifier,
) -> bool {
    let lowered_fqc_id = interner.lowered(method_id.get_class_name());
    let lowered_method_id = interner.lowered(method_id.get_method_name());

    codebase.function_likes.contains_key(&(lowered_fqc_id, lowered_method_id))
}

pub fn is_method_abstract(
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    fqc_id: &StringIdentifier,
    method_id: &StringIdentifier,
) -> bool {
    let lowered_fqc_id = interner.lowered(fqc_id);
    let lowered_method_id = interner.lowered(method_id);

    codebase
        .function_likes
        .get(&(lowered_fqc_id, lowered_method_id))
        .and_then(|meta| meta.get_method_metadata())
        .is_some_and(|method| method.is_abstract())
}

pub fn is_method_static(
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    fqc_id: &StringIdentifier,
    method_id: &StringIdentifier,
) -> bool {
    let lowered_fqc_id = interner.lowered(fqc_id);
    let lowered_method_id = interner.lowered(method_id);

    codebase
        .function_likes
        .get(&(lowered_fqc_id, lowered_method_id))
        .and_then(|meta| meta.get_method_metadata())
        .is_some_and(|method| method.is_static())
}

pub fn is_method_final(
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    fqc_id: &StringIdentifier,
    method_id: &StringIdentifier,
) -> bool {
    let lowered_fqc_id = interner.lowered(fqc_id);
    let lowered_method_id = interner.lowered(method_id);

    codebase
        .function_likes
        .get(&(lowered_fqc_id, lowered_method_id))
        .and_then(|meta| meta.get_method_metadata())
        .is_some_and(|method| method.is_final())
}

/// Checks if a property exists on a given class-like (including inherited properties).
///
/// The lookup for the class-like name is case-insensitive, but the property name is case-sensitive.
pub fn property_exists(
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    fqc_id: &StringIdentifier,
    property_id: &StringIdentifier,
) -> bool {
    let lowered_fqc_id = interner.lowered(fqc_id);

    codebase.class_likes.get(&lowered_fqc_id).is_some_and(|meta| meta.appearing_property_ids.contains_key(property_id))
}

/// Checks if a method is declared directly on a given class-like (not inherited).
///
/// The lookup for both the class-like name and the method name is case-insensitive.
pub fn declaring_method_exists(
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    fqc_id: &StringIdentifier,
    method_id: &StringIdentifier,
) -> bool {
    let lowered_fqc_id = interner.lowered(fqc_id);
    let lowered_method_id = interner.lowered(method_id);

    codebase
        .class_likes
        .get(&lowered_fqc_id)
        .is_some_and(|meta| meta.declaring_method_ids.contains_key(&lowered_method_id))
}

/// Checks if a property is declared directly on a given class-like (not inherited).
///
/// The lookup for the class-like name is case-insensitive, but the property name is case-sensitive.
pub fn declaring_property_exists(
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    fqc_id: &StringIdentifier,
    property_id: &StringIdentifier,
) -> bool {
    let lowered_fqc_id = interner.lowered(fqc_id);

    codebase.class_likes.get(&lowered_fqc_id).is_some_and(|meta| meta.properties.contains_key(property_id))
}

/// Checks if a constant or enum case exists on a given class-like.
///
/// The lookup for the class-like name is case-insensitive, but the constant/case name is case-sensitive.
pub fn class_like_constant_or_enum_case_exists(
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    fqc_id: &StringIdentifier,
    constant_id: &StringIdentifier,
) -> bool {
    let lowered_fqc_id = interner.lowered(fqc_id);

    if let Some(meta) = codebase.class_likes.get(&lowered_fqc_id) {
        return meta.constants.contains_key(constant_id) || meta.enum_cases.contains_key(constant_id);
    }

    false
}

/// Retrieves the metadata for a global function.
///
/// This lookup is case-insensitive.
pub fn get_function<'a>(
    codebase: &'a CodebaseMetadata,
    interner: &ThreadedInterner,
    id: &StringIdentifier,
) -> Option<&'a FunctionLikeMetadata> {
    let lowered_id = interner.lowered(id);

    codebase.function_likes.get(&(StringIdentifier::empty(), lowered_id))
}

/// Retrieves the metadata for a closure based on its position in the source code.
///
/// This function uses the source ID and the closure's position to uniquely identify it.
pub fn get_closure<'a>(
    codebase: &'a CodebaseMetadata,
    interner: &ThreadedInterner,
    position: &Position,
) -> Option<&'a FunctionLikeMetadata> {
    let source_id = position.source.value();
    let closure_id = interner.intern(position.to_string());

    codebase.function_likes.get(&(source_id, closure_id))
}

/// Retrieves the metadata for a global constant.
///
/// The namespace lookup is case-insensitive, but the constant name itself is case-sensitive.
pub fn get_constant<'a>(
    codebase: &'a CodebaseMetadata,
    interner: &ThreadedInterner,
    id: &StringIdentifier,
) -> Option<&'a ConstantMetadata> {
    let lowered_id = lower_constant_name(interner, id);

    codebase.constants.get(&lowered_id)
}

/// Retrieves the metadata for a class.
///
/// This lookup is case-insensitive.
pub fn get_class<'a>(
    codebase: &'a CodebaseMetadata,
    interner: &ThreadedInterner,
    id: &StringIdentifier,
) -> Option<&'a ClassLikeMetadata> {
    let lowered_id = interner.lowered(id);

    if class_exists(codebase, interner, id) { codebase.class_likes.get(&lowered_id) } else { None }
}

/// Retrieves the metadata for an interface.
///
/// This lookup is case-insensitive.
pub fn get_interface<'a>(
    codebase: &'a CodebaseMetadata,
    interner: &ThreadedInterner,
    id: &StringIdentifier,
) -> Option<&'a ClassLikeMetadata> {
    let lowered_id = interner.lowered(id);

    if interface_exists(codebase, interner, id) { codebase.class_likes.get(&lowered_id) } else { None }
}

/// Retrieves the metadata for an enum.
///
/// This lookup is case-insensitive.
pub fn get_enum<'a>(
    codebase: &'a CodebaseMetadata,
    interner: &ThreadedInterner,
    id: &StringIdentifier,
) -> Option<&'a ClassLikeMetadata> {
    let lowered_id = interner.lowered(id);

    if enum_exists(codebase, interner, id) { codebase.class_likes.get(&lowered_id) } else { None }
}

/// Retrieves the metadata for a trait.
///
/// This lookup is case-insensitive.
pub fn get_trait<'a>(
    codebase: &'a CodebaseMetadata,
    interner: &ThreadedInterner,
    id: &StringIdentifier,
) -> Option<&'a ClassLikeMetadata> {
    let lowered_id = interner.lowered(id);

    if trait_exists(codebase, interner, id) { codebase.class_likes.get(&lowered_id) } else { None }
}

pub fn get_anonymous_class_name(interner: &ThreadedInterner, span: Span) -> StringIdentifier {
    interner.intern(format!(
        "class@anonymous:{}-{}:{}",
        span.start.source.0.value(),
        span.start.offset,
        span.end.offset,
    ))
}

/// Retrieves the metadata for an anonymous class based on its span.
///
/// This function generates a unique name for the anonymous class based on its span,
/// which includes the source file and the start and end offsets.
pub fn get_anonymous_class<'a>(
    codebase: &'a CodebaseMetadata,
    interner: &ThreadedInterner,
    span: Span,
) -> Option<&'a ClassLikeMetadata> {
    let name = get_anonymous_class_name(interner, span);

    if class_exists(codebase, interner, &name) { codebase.class_likes.get(&name) } else { None }
}

/// Retrieves the metadata for any class-like (class, interface, enum, or trait).
///
/// This lookup is case-insensitive.
pub fn get_class_like<'a>(
    codebase: &'a CodebaseMetadata,
    interner: &ThreadedInterner,
    id: &StringIdentifier,
) -> Option<&'a ClassLikeMetadata> {
    let lowered_id = interner.lowered(id);
    codebase.class_likes.get(&lowered_id)
}

pub fn get_declaring_class_for_property(
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    fqc_id: &StringIdentifier,
    property_id: &StringIdentifier,
) -> Option<StringIdentifier> {
    let lowered_fqc_id = interner.lowered(fqc_id);

    let class_like = codebase.class_likes.get(&lowered_fqc_id)?;

    class_like.declaring_property_ids.get(property_id).copied()
}

/// Retrieves the metadata for a property, searching the inheritance hierarchy.
///
/// This function finds where the property was originally declared and returns its metadata.
/// The lookup for the class-like name is case-insensitive, but the property name is case-sensitive.
pub fn get_declaring_property<'a>(
    codebase: &'a CodebaseMetadata,
    interner: &ThreadedInterner,
    fqc_id: &StringIdentifier,
    property_id: &StringIdentifier,
) -> Option<&'a PropertyMetadata> {
    let declaring_fqc_id = get_declaring_class_for_property(codebase, interner, fqc_id, property_id)?;
    let declaring_class_like = codebase.class_likes.get(&declaring_fqc_id)?;

    declaring_class_like.properties.get(property_id)
}

pub fn get_method_id(fqc_id: &StringIdentifier, method_name_id: &StringIdentifier) -> MethodIdentifier {
    MethodIdentifier::new(*fqc_id, *method_name_id)
}

pub fn get_declaring_method_id(
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    method_id: &MethodIdentifier,
) -> MethodIdentifier {
    let lowered_fqc_id = interner.lowered(method_id.get_class_name());
    let lowered_method_id = interner.lowered(method_id.get_method_name());

    let Some(class_like_metadata) = codebase.class_likes.get(&lowered_fqc_id) else {
        // If the class-like doesn't exist, return the method ID as is
        return *method_id;
    };

    let declaring_method_ids = class_like_metadata.get_declaring_method_ids();
    if let Some(declaring_fqcn) = declaring_method_ids.get(&lowered_method_id)
        && let Some(declaring_class_metadata) = codebase.class_likes.get(declaring_fqcn)
    {
        return MethodIdentifier::new(declaring_class_metadata.original_name, *method_id.get_method_name());
    };

    if class_like_metadata.is_abstract {
        let overridden_method_ids = class_like_metadata.get_overridden_method_ids();
        if let Some(overridden_classes) = overridden_method_ids.get(&lowered_method_id)
            && let Some(first_class) = overridden_classes.iter().next()
            && let Some(first_class_metadata) = codebase.class_likes.get(first_class)
        {
            return MethodIdentifier::new(first_class_metadata.original_name, *method_id.get_method_name());
        }
    }

    // If the method isn't declared in this class, return the method ID as is
    *method_id
}

/// Retrieves the metadata for a method, searching the inheritance hierarchy.
///
/// This function finds where the method is declared (which could be an ancestor class/trait)
/// and returns the metadata from there.
///
/// The lookup for both the class-like name and the method name is case-insensitive.
pub fn get_declaring_method<'a>(
    codebase: &'a CodebaseMetadata,
    interner: &ThreadedInterner,
    fqc_id: &StringIdentifier,
    method_name_id: &StringIdentifier,
) -> Option<&'a FunctionLikeMetadata> {
    let method_id = MethodIdentifier::new(interner.lowered(fqc_id), interner.lowered(method_name_id));
    let declaring_method_id = get_declaring_method_id(codebase, interner, &method_id);

    get_method(codebase, interner, declaring_method_id.get_class_name(), declaring_method_id.get_method_name())
}

pub fn get_method_by_id<'a>(
    codebase: &'a CodebaseMetadata,
    interner: &ThreadedInterner,
    method_id: &MethodIdentifier,
) -> Option<&'a FunctionLikeMetadata> {
    let lowered_fqc_id = interner.lowered(method_id.get_class_name());
    let lowered_method_id = interner.lowered(method_id.get_method_name());

    codebase.function_likes.get(&(lowered_fqc_id, lowered_method_id))
}

pub fn get_method<'a>(
    codebase: &'a CodebaseMetadata,
    interner: &ThreadedInterner,
    fqc_id: &StringIdentifier,
    method_name_id: &StringIdentifier,
) -> Option<&'a FunctionLikeMetadata> {
    let lowered_fqc_id = interner.lowered(fqc_id);
    let lowered_method_id = interner.lowered(method_name_id);

    codebase.function_likes.get(&(lowered_fqc_id, lowered_method_id))
}

/// Retrieves the metadata for a property that is declared directly on the given class-like.
///
/// This does not search the inheritance hierarchy.
/// The lookup for the class-like name is case-insensitive, but the property name is case-sensitive.
pub fn get_property<'a>(
    codebase: &'a CodebaseMetadata,
    interner: &ThreadedInterner,
    fqc_id: &StringIdentifier,
    property_id: &StringIdentifier,
) -> Option<&'a PropertyMetadata> {
    let lowered_fqc_id = interner.lowered(fqc_id);

    let class_like = codebase.class_likes.get(&lowered_fqc_id)?;

    class_like.properties.get(property_id)
}

/// An enum to represent either a class constant or an enum case.
#[derive(Debug, PartialEq)]
pub enum ClassConstantOrEnumCase<'a> {
    Constant(&'a ClassLikeConstantMetadata),
    EnumCase(&'a EnumCaseMetadata),
}

/// Retrieves the metadata for a class constant or an enum case from a class-like.
///
/// The lookup for the class-like name is case-insensitive, but the constant/case name is case-sensitive.
pub fn get_class_like_constant_or_enum_case<'a>(
    codebase: &'a CodebaseMetadata,
    interner: &ThreadedInterner,
    fqc_id: &StringIdentifier,
    constant_id: &StringIdentifier,
) -> Option<ClassConstantOrEnumCase<'a>> {
    let lowered_fqc_id = interner.lowered(fqc_id);
    let class_like = codebase.class_likes.get(&lowered_fqc_id)?;

    if let Some(constant_meta) = class_like.constants.get(constant_id) {
        return Some(ClassConstantOrEnumCase::Constant(constant_meta));
    }

    if let Some(enum_case_meta) = class_like.enum_cases.get(constant_id) {
        return Some(ClassConstantOrEnumCase::EnumCase(enum_case_meta));
    }

    None
}

/// Checks if a class-like is an instance of another class-like.
///
/// This function checks if the `child` class-like is an instance of the `parent` class-like
/// by looking up their metadata in the codebase.
pub fn is_instance_of(
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    child: &StringIdentifier,
    parent: &StringIdentifier,
) -> bool {
    let lowered_child = interner.lowered(child);
    let lowered_parent = interner.lowered(parent);

    if lowered_child == lowered_parent {
        return true;
    }

    let Some(child_meta) = codebase.class_likes.get(&lowered_child) else {
        return false;
    };

    child_meta.has_parent(&lowered_parent)
}

pub fn inherits_class(
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    child: &StringIdentifier,
    parent: &StringIdentifier,
) -> bool {
    let lowered_child = interner.lowered(child);
    let lowered_parent = interner.lowered(parent);

    let Some(child_meta) = codebase.class_likes.get(&lowered_child) else {
        return false;
    };

    child_meta.all_parent_classes.contains(&lowered_parent)
}

pub fn directly_inherits_class(
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    child: &StringIdentifier,
    parent: &StringIdentifier,
) -> bool {
    let lowered_child = interner.lowered(child);
    let lowered_parent = interner.lowered(parent);

    let Some(child_meta) = codebase.class_likes.get(&lowered_child) else {
        return false;
    };

    child_meta.direct_parent_class.as_ref().is_some_and(|parent_class| parent_class == &lowered_parent)
}

pub fn inherits_interface(
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    child: &StringIdentifier,
    parent: &StringIdentifier,
) -> bool {
    let lowered_child = interner.lowered(child);
    let lowered_parent = interner.lowered(parent);

    let Some(child_meta) = codebase.class_likes.get(&lowered_child) else {
        return false;
    };

    child_meta.all_parent_interfaces.contains(&lowered_parent)
}

pub fn directly_inherits_interface(
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    child: &StringIdentifier,
    parent: &StringIdentifier,
) -> bool {
    let lowered_child = interner.lowered(child);
    let lowered_parent = interner.lowered(parent);

    let Some(child_meta) = codebase.class_likes.get(&lowered_child) else {
        return false;
    };

    child_meta.direct_parent_interfaces.contains(&lowered_parent)
}

pub fn uses_trait(
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    child: &StringIdentifier,
    trait_name: &StringIdentifier,
) -> bool {
    let lowered_child = interner.lowered(child);
    let lowered_trait_name = interner.lowered(trait_name);

    let Some(child_meta) = codebase.class_likes.get(&lowered_child) else {
        return false;
    };

    child_meta.used_traits.contains(&lowered_trait_name)
}

/// Recursively collects all descendant class/interface/enum FQCNs for a given class-like structure.
/// Uses the pre-computed `all_classlike_descendants` map if available, otherwise might be empty.
/// Warning: Recursive; could stack overflow on extremely deep hierarchies if map isn't precomputed well.
#[inline]
pub fn get_all_descendants(
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    class_like_name: &StringIdentifier,
) -> HashSet<StringIdentifier> {
    let fqc_id = interner.lowered(class_like_name);

    // This implementation assumes direct_classlike_descendants is populated correctly.
    let mut all_descendants = HashSet::default();
    let mut queue = vec![&fqc_id];
    let mut visited = HashSet::default();
    visited.insert(&fqc_id); // Don't include self in descendants

    while let Some(current_name) = queue.pop() {
        if let Some(direct_descendants) = codebase.direct_classlike_descendants.get(current_name) {
            for descendant in direct_descendants {
                if visited.insert(descendant) {
                    // Add to results only if not visited before
                    all_descendants.insert(*descendant);
                    queue.push(descendant); // Add to queue for further exploration
                }
            }
        }
    }
    all_descendants
}

/// Checks if a method is overridden from a parent class-like.
///
/// This function checks if the method with the given name in the specified class-like
/// is overridden from a parent class-like by looking up the metadata in the codebase.
///
/// The lookup for both the class-like name and the method name is case-insensitive.
pub fn is_method_overriding(
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    fqc_id: &StringIdentifier,
    method_name: &StringIdentifier,
) -> bool {
    let lowered_method_name = interner.lowered(method_name);

    get_class_like(codebase, interner, fqc_id)
        .is_some_and(|metadata| metadata.overridden_method_ids.contains_key(&lowered_method_name))
}

/// Retrieves the type of a class constant, considering type hints and inferred types.
/// Returns `None` if the class or constant doesn't exist, or type cannot be determined.
#[inline]
pub fn get_class_constant_type<'a>(
    codebase: &'a CodebaseMetadata,
    interner: &ThreadedInterner,
    fq_class_name: &StringIdentifier,
    constant_name: &StringIdentifier,
) -> Option<Cow<'a, TUnion>> {
    let class_metadata = get_class_like(codebase, interner, fq_class_name)?;

    if class_metadata.kind.is_enum() && class_metadata.enum_cases.contains_key(constant_name) {
        return Some(Cow::Owned(TUnion::new(vec![TAtomic::Object(TObject::new_enum_case(
            class_metadata.original_name,
            *constant_name,
        ))])));
    }

    // It's a regular class constant
    let constant_metadata = class_metadata.constants.get(constant_name)?;

    // Prefer the type signature if available
    if let Some(type_metadata) = constant_metadata.type_metadata.as_ref() {
        // Return borrowed signature type directly
        // (Original logic about boring scalars/is_this seemed complex and possibly specific
        //  to a particular analysis stage; simplifying here to return declared type if present)
        return Some(Cow::Borrowed(&type_metadata.type_union));
    }

    // Fall back to inferred type if no signature
    constant_metadata.inferred_type.as_ref().map(|atomic_type| {
        // Wrap the atomic type in a TUnion if returning inferred type
        Cow::Owned(TUnion::new(vec![atomic_type.clone()]))
    })
}

/// Lowers the namespace part of a fully qualified constant name while preserving the case of the constant name itself.
///
/// For example, `My\Namespace\MY_CONST` becomes `my\namespace\MY_CONST`. This is necessary because
/// PHP constant lookups are case-insensitive for the namespace but case-sensitive for the final constant name.
fn lower_constant_name(interner: &ThreadedInterner, name: &StringIdentifier) -> StringIdentifier {
    let name_str = interner.lookup(name);
    if !name_str.contains('\\') {
        return *name;
    }

    let mut parts: Vec<_> = name_str.split('\\').map(str::to_owned).collect();
    let total_parts = parts.len();
    if total_parts > 1 {
        parts = parts
            .into_iter()
            .enumerate()
            .map(|(i, part)| if i < total_parts - 1 { part.to_ascii_lowercase() } else { part })
            .collect::<Vec<_>>();
    }

    interner.intern(parts.join("\\"))
}
