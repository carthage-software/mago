use std::borrow::Cow;

use mago_atom::Atom;
use mago_atom::AtomSet;
use mago_atom::ascii_lowercase_atom;
use mago_atom::ascii_lowercase_constant_name_atom;
use mago_atom::atom;
use mago_atom::empty_atom;
use mago_atom::u32_atom;
use mago_atom::u64_atom;
use mago_database::file::FileId;
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
use crate::metadata::ttype::TypeMetadata;
use crate::symbol::SymbolKind;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::object::TObject;
use crate::ttype::union::TUnion;

pub mod assertion;
pub mod consts;
pub mod context;
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
pub fn function_exists(codebase: &CodebaseMetadata, function_name: &str) -> bool {
    let lowercase_function_name = ascii_lowercase_atom(function_name);
    let function_identifier = (empty_atom(), lowercase_function_name);

    codebase.function_likes.contains_key(&function_identifier)
}

/// Checks if a global constant exists in the codebase.
///
/// The lookup for the namespace part of the constant name is case-insensitive,
/// but the constant name itself is case-sensitive, matching PHP's behavior.
pub fn constant_exists(codebase: &CodebaseMetadata, constant_name: &str) -> bool {
    let lowercase_constant_name = ascii_lowercase_constant_name_atom(constant_name);

    codebase.constants.contains_key(&lowercase_constant_name)
}

/// Checks if a class exists in the codebase.
///
/// This lookup is case-insensitive, in line with PHP's behavior for class names.
pub fn class_exists(codebase: &CodebaseMetadata, name: &str) -> bool {
    let lowercase_name = ascii_lowercase_atom(name);

    matches!(codebase.symbols.get_kind(&lowercase_name), Some(SymbolKind::Class))
}

/// Checks if a class or trait exists in the codebase.
///
/// This lookup is case-insensitive, in line with PHP's behavior for class names.
pub fn class_or_trait_exists(codebase: &CodebaseMetadata, name: &str) -> bool {
    let lowercase_name = ascii_lowercase_atom(name);

    matches!(codebase.symbols.get_kind(&lowercase_name), Some(SymbolKind::Class | SymbolKind::Trait))
}

/// Checks if an interface exists in the codebase.
///
/// This lookup is case-insensitive, in line with PHP's behavior for interface names.
pub fn interface_exists(codebase: &CodebaseMetadata, name: &str) -> bool {
    let lowercase_name = ascii_lowercase_atom(name);

    matches!(codebase.symbols.get_kind(&lowercase_name), Some(SymbolKind::Interface))
}

/// Checks if a class or interface exists in the codebase.
///
/// This lookup is case-insensitive, in line with PHP's behavior for class names.
pub fn class_or_interface_exists(codebase: &CodebaseMetadata, name: &str) -> bool {
    let lowercase_name = ascii_lowercase_atom(name);

    matches!(codebase.symbols.get_kind(&lowercase_name), Some(SymbolKind::Class | SymbolKind::Interface))
}

/// Checks if an enum exists in the codebase.
///
/// This lookup is case-insensitive, in line with PHP's behavior for enum names.
pub fn enum_exists(codebase: &CodebaseMetadata, name: &str) -> bool {
    let lowercase_name = ascii_lowercase_atom(name);

    matches!(codebase.symbols.get_kind(&lowercase_name), Some(SymbolKind::Enum))
}

/// Checks if a trait exists in the codebase.
///
/// This lookup is case-insensitive, in line with PHP's behavior for trait names.
pub fn trait_exists(codebase: &CodebaseMetadata, name: &str) -> bool {
    let lowercase_name = ascii_lowercase_atom(name);

    matches!(codebase.symbols.get_kind(&lowercase_name), Some(SymbolKind::Trait))
}

/// Checks if a class-like (class, interface, enum, or trait) exists in the codebase.
///
/// This lookup is case-insensitive.
pub fn class_like_exists(codebase: &CodebaseMetadata, name: &str) -> bool {
    let lowercase_name = ascii_lowercase_atom(name);

    codebase.symbols.contains(&lowercase_name)
}

/// Checks if a method exists on a given class-like (including inherited methods).
///
/// The lookup for both the class-like name and the method name is case-insensitive.
pub fn method_exists(codebase: &CodebaseMetadata, fqcn: &str, method_name: &str) -> bool {
    let lowercase_fqcn = ascii_lowercase_atom(fqcn);
    let lowercase_method_name = ascii_lowercase_atom(method_name);

    codebase
        .class_likes
        .get(&lowercase_fqcn)
        .is_some_and(|meta| meta.appearing_method_ids.contains_key(&lowercase_method_name))
}

pub fn method_identifier_exists(codebase: &CodebaseMetadata, method_identifier: &MethodIdentifier) -> bool {
    let lowercase_fqcn = ascii_lowercase_atom(method_identifier.get_class_name());
    let lowercase_method_name = ascii_lowercase_atom(method_identifier.get_method_name());

    let method_identifier = (lowercase_fqcn, lowercase_method_name);

    codebase.function_likes.contains_key(&method_identifier)
}

pub fn is_method_abstract(codebase: &CodebaseMetadata, fqcn: &str, method_name: &str) -> bool {
    let lowercase_fqcn = ascii_lowercase_atom(fqcn);
    let lowercase_method_name = ascii_lowercase_atom(method_name);

    let method_identifier = (lowercase_fqcn, lowercase_method_name);

    codebase
        .function_likes
        .get(&method_identifier)
        .and_then(|meta| meta.method_metadata.as_ref())
        .is_some_and(|method| method.is_abstract)
}

pub fn is_method_static(codebase: &CodebaseMetadata, fqcn: &str, method_name: &str) -> bool {
    let lowercase_fqcn = ascii_lowercase_atom(fqcn);
    let lowercase_method_name = ascii_lowercase_atom(method_name);

    let method_identifier = (lowercase_fqcn, lowercase_method_name);

    codebase
        .function_likes
        .get(&method_identifier)
        .and_then(|meta| meta.method_metadata.as_ref())
        .is_some_and(|method| method.is_static)
}

pub fn is_method_final(codebase: &CodebaseMetadata, fqcn: &str, method_name: &str) -> bool {
    let lowercase_fqcn = ascii_lowercase_atom(fqcn);
    let lowercase_method_name = ascii_lowercase_atom(method_name);

    let method_identifier = (lowercase_fqcn, lowercase_method_name);

    codebase
        .function_likes
        .get(&method_identifier)
        .and_then(|meta| meta.method_metadata.as_ref())
        .is_some_and(|method| method.is_final)
}

/// Checks if a property exists on a given class-like (including inherited properties).
///
/// The lookup for the class-like name is case-insensitive, but the property name is case-sensitive.
pub fn property_exists(codebase: &CodebaseMetadata, fqcn: &str, property_name: &str) -> bool {
    let lowercase_fqcn = ascii_lowercase_atom(fqcn);

    codebase
        .class_likes
        .get(&lowercase_fqcn)
        .is_some_and(|meta| meta.appearing_property_ids.contains_key(&atom(property_name)))
}

/// Checks if a method is declared directly on a given class-like (not inherited).
///
/// The lookup for both the class-like name and the method name is case-insensitive.
pub fn declaring_method_exists(codebase: &CodebaseMetadata, fqcn: &str, method_name: &str) -> bool {
    let lowercase_fqcn = ascii_lowercase_atom(fqcn);
    let lowercase_method_name = ascii_lowercase_atom(method_name);

    codebase
        .class_likes
        .get(&lowercase_fqcn)
        .is_some_and(|meta| meta.declaring_method_ids.contains_key(&lowercase_method_name))
}

/// Checks if a property is declared directly on a given class-like (not inherited).
///
/// The lookup for the class-like name is case-insensitive, but the property name is case-sensitive.
pub fn declaring_property_exists(codebase: &CodebaseMetadata, fqcn: &str, property_name: &str) -> bool {
    let lowercase_fqcn = ascii_lowercase_atom(fqcn);
    let property_name = atom(property_name);

    codebase.class_likes.get(&lowercase_fqcn).is_some_and(|meta| meta.properties.contains_key(&property_name))
}

/// Checks if a constant or enum case exists on a given class-like.
///
/// The lookup for the class-like name is case-insensitive, but the constant/case name is case-sensitive.
pub fn class_like_constant_or_enum_case_exists(codebase: &CodebaseMetadata, fqcn: &str, constant_name: &str) -> bool {
    let lowercase_fqcn = ascii_lowercase_atom(fqcn);
    let constant_name = atom(constant_name);

    if let Some(meta) = codebase.class_likes.get(&lowercase_fqcn) {
        return meta.constants.contains_key(&constant_name) || meta.enum_cases.contains_key(&constant_name);
    }

    false
}

/// Retrieves the metadata for a global function.
///
/// This lookup is case-insensitive.
pub fn get_function<'a>(codebase: &'a CodebaseMetadata, function_name: &str) -> Option<&'a FunctionLikeMetadata> {
    let lowercase_function_name = ascii_lowercase_atom(function_name);
    let function_identifier = (empty_atom(), lowercase_function_name);

    codebase.function_likes.get(&function_identifier)
}

/// Retrieves the metadata for a closure based on its position in the source code.
///
/// This function uses the source ID and the closure's position to uniquely identify it.
pub fn get_closure<'a>(
    codebase: &'a CodebaseMetadata,
    file_id: &FileId,
    position: &Position,
) -> Option<&'a FunctionLikeMetadata> {
    let file_ref = u64_atom(file_id.as_u64());
    let closure_ref = u32_atom(position.offset);
    let identifier = (file_ref, closure_ref);

    codebase.function_likes.get(&identifier)
}

/// Retrieves the metadata for a global constant.
///
/// The namespace lookup is case-insensitive, but the constant name itself is case-sensitive.
pub fn get_constant<'a>(codebase: &'a CodebaseMetadata, constant_name: &str) -> Option<&'a ConstantMetadata> {
    let lowercase_constant_name = ascii_lowercase_constant_name_atom(constant_name);

    codebase.constants.get(&lowercase_constant_name)
}

/// Retrieves the metadata for a class.
///
/// This lookup is case-insensitive.
pub fn get_class<'a>(codebase: &'a CodebaseMetadata, name: &str) -> Option<&'a ClassLikeMetadata> {
    let lowercase_name = ascii_lowercase_atom(name);

    if codebase.symbols.contains_class(&lowercase_name) { codebase.class_likes.get(&lowercase_name) } else { None }
}

/// Retrieves the metadata for an interface.
///
/// This lookup is case-insensitive.
pub fn get_interface<'a>(codebase: &'a CodebaseMetadata, name: &str) -> Option<&'a ClassLikeMetadata> {
    let lowercase_name = ascii_lowercase_atom(name);

    if codebase.symbols.contains_interface(&lowercase_name) { codebase.class_likes.get(&lowercase_name) } else { None }
}

/// Retrieves the metadata for an enum.
///
/// This lookup is case-insensitive.
pub fn get_enum<'a>(codebase: &'a CodebaseMetadata, name: &str) -> Option<&'a ClassLikeMetadata> {
    let lowercase_name = ascii_lowercase_atom(name);

    if codebase.symbols.contains_enum(&lowercase_name) { codebase.class_likes.get(&lowercase_name) } else { None }
}

/// Retrieves the metadata for a trait.
///
/// This lookup is case-insensitive.
pub fn get_trait<'a>(codebase: &'a CodebaseMetadata, name: &str) -> Option<&'a ClassLikeMetadata> {
    let lowercase_name = ascii_lowercase_atom(name);

    if codebase.symbols.contains_trait(&lowercase_name) { codebase.class_likes.get(&lowercase_name) } else { None }
}

pub fn get_anonymous_class_name(span: Span) -> Atom {
    use std::io::Write;

    // A 64-byte buffer on the stack. This is ample space for the prefix,
    // u64 file id, and 2 u32 integers, preventing any chance of a heap allocation.
    let mut buffer = [0u8; 64];

    // Use a block to limit the scope of the mutable writer
    // `writer` is a mutable slice that implements `std::io::Write`.
    let mut writer = &mut buffer[..];

    // SAFETY: We use `unwrap_unchecked` here because we are writing to a fixed-size buffer
    unsafe {
        write!(writer, "class@anonymous:{}-{}:{}", span.file_id, span.start.offset, span.end.offset).unwrap_unchecked()
    };

    // Determine how many bytes were written by checking the length of the original buffer
    // against what the `writer` had left. This is a common pattern for `io::Write` on slices.
    let written_len = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());

    atom(
        // SAFETY: We use `unwrap_unchecked` here because we are certain the bytes
        // up to `written_len` are valid UTF-8.
        unsafe { std::str::from_utf8(&buffer[..written_len]).unwrap_unchecked() },
    )
}

/// Retrieves the metadata for an anonymous class based on its span.
///
/// This function generates a unique name for the anonymous class based on its span,
/// which includes the source file and the start and end offsets.
pub fn get_anonymous_class(codebase: &CodebaseMetadata, span: Span) -> Option<&ClassLikeMetadata> {
    let name = get_anonymous_class_name(span);

    if class_exists(codebase, &name) { codebase.class_likes.get(&name) } else { None }
}

/// Retrieves the metadata for any class-like (class, interface, enum, or trait).
///
/// This lookup is case-insensitive.
pub fn get_class_like<'a>(codebase: &'a CodebaseMetadata, name: &str) -> Option<&'a ClassLikeMetadata> {
    let lowercase_name = ascii_lowercase_atom(name);

    codebase.class_likes.get(&lowercase_name)
}

pub fn get_declaring_class_for_property(codebase: &CodebaseMetadata, fqcn: &str, property_name: &str) -> Option<Atom> {
    let lowercase_fqcn = ascii_lowercase_atom(fqcn);
    let property_name = atom(property_name);

    codebase.class_likes.get(&lowercase_fqcn)?.declaring_property_ids.get(&property_name).copied()
}

/// Retrieves the metadata for a property, searching the inheritance hierarchy.
///
/// This function finds where the property was originally declared and returns its metadata.
/// The lookup for the class-like name is case-insensitive, but the property name is case-sensitive.
pub fn get_declaring_property<'a>(
    codebase: &'a CodebaseMetadata,
    fqcn: &str,
    property_name: &str,
) -> Option<&'a PropertyMetadata> {
    let lowercase_fqcn = ascii_lowercase_atom(fqcn);
    let property_name = atom(property_name);

    let declaring_fqcn = codebase.class_likes.get(&lowercase_fqcn)?.declaring_property_ids.get(&property_name)?;

    codebase.class_likes.get(declaring_fqcn)?.properties.get(&property_name)
}

pub fn get_method_identifier(fqcn: &str, method_name: &str) -> MethodIdentifier {
    MethodIdentifier::new(atom(fqcn), atom(method_name))
}

pub fn get_declaring_method_identifier(
    codebase: &CodebaseMetadata,
    method_identifier: &MethodIdentifier,
) -> MethodIdentifier {
    let lowercase_fqcn = ascii_lowercase_atom(method_identifier.get_class_name());
    let lowercase_method_name = ascii_lowercase_atom(method_identifier.get_method_name());

    let Some(class_like_metadata) = codebase.class_likes.get(&lowercase_fqcn) else {
        // If the class-like doesn't exist, return the method ID as is
        return *method_identifier;
    };

    if let Some(declaring_fqcn) = class_like_metadata.declaring_method_ids.get(&lowercase_method_name)
        && let Some(declaring_class_metadata) = codebase.class_likes.get(declaring_fqcn)
    {
        return MethodIdentifier::new(declaring_class_metadata.original_name, *method_identifier.get_method_name());
    };

    if class_like_metadata.flags.is_abstract()
        && let Some(overridden_classes) = class_like_metadata.overridden_method_ids.get(&lowercase_method_name)
        && let Some(first_class) = overridden_classes.iter().next()
        && let Some(first_class_metadata) = codebase.class_likes.get(first_class)
    {
        return MethodIdentifier::new(first_class_metadata.original_name, *method_identifier.get_method_name());
    }

    // If the method isn't declared in this class, return the method ID as is
    *method_identifier
}

/// Retrieves the metadata for a method, searching the inheritance hierarchy.
///
/// This function finds where the method is declared (which could be an ancestor class/trait)
/// and returns the metadata from there.
///
/// The lookup for both the class-like name and the method name is case-insensitive.
pub fn get_declaring_method<'a>(
    codebase: &'a CodebaseMetadata,
    fqcn: &str,
    method_name: &str,
) -> Option<&'a FunctionLikeMetadata> {
    let method_id = MethodIdentifier::new(atom(fqcn), atom(method_name));
    let declaring_method_id = get_declaring_method_identifier(codebase, &method_id);

    get_method(codebase, declaring_method_id.get_class_name(), declaring_method_id.get_method_name())
}

pub fn get_method_by_id<'a>(
    codebase: &'a CodebaseMetadata,
    method_identifier: &MethodIdentifier,
) -> Option<&'a FunctionLikeMetadata> {
    let lowercase_fqcn = ascii_lowercase_atom(method_identifier.get_class_name());
    let lowercase_method_name = ascii_lowercase_atom(method_identifier.get_method_name());

    let function_identifier = (lowercase_fqcn, lowercase_method_name);

    codebase.function_likes.get(&function_identifier)
}

pub fn get_method<'a>(
    codebase: &'a CodebaseMetadata,
    fqcn: &str,
    method_name: &str,
) -> Option<&'a FunctionLikeMetadata> {
    let lowercase_fqcn = ascii_lowercase_atom(fqcn);
    let lowercase_method_name = ascii_lowercase_atom(method_name);
    let function_like_identifier = (lowercase_fqcn, lowercase_method_name);

    codebase.function_likes.get(&function_like_identifier)
}

/// Retrieves the metadata for a property that is declared directly on the given class-like.
///
/// This does not search the inheritance hierarchy.
/// The lookup for the class-like name is case-insensitive, but the property name is case-sensitive.
pub fn get_property<'a>(
    codebase: &'a CodebaseMetadata,
    fqcn: &str,
    property_name: &str,
) -> Option<&'a PropertyMetadata> {
    let lowercase_fqcn = ascii_lowercase_atom(fqcn);
    let property_name = atom(property_name);

    codebase.class_likes.get(&lowercase_fqcn)?.properties.get(&property_name)
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
    fqcn: &str,
    constant_name: &str,
) -> Option<ClassConstantOrEnumCase<'a>> {
    let lowercase_fqcn = ascii_lowercase_atom(fqcn);
    let constant_name = atom(constant_name);

    let class_like = codebase.class_likes.get(&lowercase_fqcn)?;

    if let Some(constant_meta) = class_like.constants.get(&constant_name) {
        return Some(ClassConstantOrEnumCase::Constant(constant_meta));
    }

    if let Some(enum_case_meta) = class_like.enum_cases.get(&constant_name) {
        return Some(ClassConstantOrEnumCase::EnumCase(enum_case_meta));
    }

    None
}

/// Checks if a class-like is an instance of another class-like.
///
/// This function checks if the `child` class-like is an instance of the `parent` class-like
/// by looking up their metadata in the codebase.
pub fn is_instance_of(codebase: &CodebaseMetadata, child_name: &str, parent_name: &str) -> bool {
    if child_name == parent_name {
        return true;
    }

    let lowercase_child_name = ascii_lowercase_atom(child_name);
    let lowercase_parent_name = ascii_lowercase_atom(parent_name);

    if lowercase_child_name == lowercase_parent_name {
        return true;
    }

    let Some(child_meta) = codebase.class_likes.get(&lowercase_child_name) else {
        return false;
    };

    child_meta.has_parent(&lowercase_parent_name)
}

pub fn inherits_class(codebase: &CodebaseMetadata, child_name: &str, parent_name: &str) -> bool {
    let lowercase_child_name = ascii_lowercase_atom(child_name);
    let lowercase_parent_name = ascii_lowercase_atom(parent_name);

    let Some(child_meta) = codebase.class_likes.get(&lowercase_child_name) else {
        return false;
    };

    child_meta.all_parent_classes.contains(&lowercase_parent_name)
}

pub fn directly_inherits_class(codebase: &CodebaseMetadata, child_name: &str, parent_name: &str) -> bool {
    let lowercase_child_name = ascii_lowercase_atom(child_name);
    let lowercase_parent_name = ascii_lowercase_atom(parent_name);

    let Some(child_meta) = codebase.class_likes.get(&lowercase_child_name) else {
        return false;
    };

    child_meta.direct_parent_class.as_ref().is_some_and(|parent_class| parent_class == &lowercase_parent_name)
}

pub fn inherits_interface(codebase: &CodebaseMetadata, child_name: &str, parent_name: &str) -> bool {
    let lowercase_child_name = ascii_lowercase_atom(child_name);
    let lowercase_parent_name = ascii_lowercase_atom(parent_name);

    let Some(child_meta) = codebase.class_likes.get(&lowercase_child_name) else {
        return false;
    };

    child_meta.all_parent_interfaces.contains(&lowercase_parent_name)
}

pub fn directly_inherits_interface(codebase: &CodebaseMetadata, child_name: &str, parent_name: &str) -> bool {
    let lowercase_child_name = ascii_lowercase_atom(child_name);
    let lowercase_parent_name = ascii_lowercase_atom(parent_name);

    let Some(child_meta) = codebase.class_likes.get(&lowercase_child_name) else {
        return false;
    };

    child_meta.direct_parent_interfaces.contains(&lowercase_parent_name)
}

pub fn uses_trait(codebase: &CodebaseMetadata, child_name: &str, trait_name: &str) -> bool {
    let lowercase_child_name = ascii_lowercase_atom(child_name);
    let lowercase_trait_name = ascii_lowercase_atom(trait_name);

    let Some(child_meta) = codebase.class_likes.get(&lowercase_child_name) else {
        return false;
    };

    child_meta.used_traits.contains(&lowercase_trait_name)
}

/// Recursively collects all descendant class/interface/enum FQCNs for a given class-like structure.
/// Uses the pre-computed `all_classlike_descendants` map if available, otherwise might be empty.
/// Warning: Recursive; could stack overflow on extremely deep hierarchies if map isn't precomputed well.
#[inline]
pub fn get_all_descendants(codebase: &CodebaseMetadata, fqcn: &str) -> AtomSet {
    let lowercase_fqcn = ascii_lowercase_atom(fqcn);

    // This implementation assumes direct_classlike_descendants is populated correctly.
    let mut all_descendants = AtomSet::default();
    let mut queue = vec![&lowercase_fqcn];
    let mut visited = AtomSet::default();
    visited.insert(lowercase_fqcn); // Don't include self in descendants

    while let Some(current_name) = queue.pop() {
        if let Some(direct_descendants) = codebase.direct_classlike_descendants.get(current_name) {
            for descendant in direct_descendants {
                if visited.insert(*descendant) {
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
pub fn is_method_overriding(codebase: &CodebaseMetadata, fqcn: &str, method_name: &str) -> bool {
    let lowercase_method_name = ascii_lowercase_atom(method_name);

    get_class_like(codebase, fqcn)
        .is_some_and(|metadata| metadata.overridden_method_ids.contains_key(&lowercase_method_name))
}

pub fn get_function_like_thrown_types<'a>(
    codebase: &'a CodebaseMetadata,
    class_like: Option<&'a ClassLikeMetadata>,
    function_like: &'a FunctionLikeMetadata,
) -> &'a [TypeMetadata] {
    if !function_like.thrown_types.is_empty() {
        return function_like.thrown_types.as_slice();
    }

    if !function_like.kind.is_method() {
        return &[];
    }

    let Some(class_like) = class_like else {
        return &[];
    };

    let Some(method_name) = function_like.name.as_ref() else {
        return &[];
    };

    for parent_class_name_id in class_like.overridden_method_ids.get(method_name).into_iter().flatten() {
        let Some(parent_class) = codebase.class_likes.get(parent_class_name_id) else {
            continue;
        };

        let parent_method_id = (*parent_class_name_id, *method_name);
        if let Some(parent_method) = codebase.function_likes.get(&parent_method_id) {
            let thrown = get_function_like_thrown_types(codebase, Some(parent_class), parent_method);
            if !thrown.is_empty() {
                return thrown;
            }
        }
    }

    &[]
}

/// Retrieves the type of a class constant, considering type hints and inferred types.
/// Returns `None` if the class or constant doesn't exist, or type cannot be determined.
#[inline]
pub fn get_class_constant_type<'a>(
    codebase: &'a CodebaseMetadata,
    fq_class_name: &str,
    constant_name: &str,
) -> Option<Cow<'a, TUnion>> {
    let class_metadata = get_class_like(codebase, fq_class_name)?;
    let constant_name = atom(constant_name);

    if class_metadata.kind.is_enum() && class_metadata.enum_cases.contains_key(&constant_name) {
        let atomic = TAtomic::Object(TObject::new_enum_case(class_metadata.original_name, constant_name));

        return Some(Cow::Owned(TUnion::from_atomic(atomic)));
    }

    // It's a regular class constant
    let constant_metadata = class_metadata.constants.get(&constant_name)?;

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
        Cow::Owned(TUnion::from_atomic(atomic_type.clone()))
    })
}
