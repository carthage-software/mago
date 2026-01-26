use std::borrow::Cow;
use std::collections::hash_map::Entry;

use ahash::HashMap;
use ahash::HashSet;
use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::AtomMap;
use mago_atom::AtomSet;
use mago_atom::ascii_lowercase_atom;
use mago_atom::ascii_lowercase_constant_name_atom;
use mago_atom::atom;
use mago_atom::empty_atom;
use mago_atom::u32_atom;
use mago_atom::u64_atom;
use mago_database::file::FileId;
use mago_reporting::IssueCollection;
use mago_span::Position;
use mago_span::Span;

use crate::identifier::method::MethodIdentifier;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::metadata::class_like_constant::ClassLikeConstantMetadata;
use crate::metadata::constant::ConstantMetadata;
use crate::metadata::enum_case::EnumCaseMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::function_like::FunctionLikeMetadata;
use crate::metadata::property::PropertyMetadata;
use crate::metadata::ttype::TypeMetadata;
use crate::signature::FileSignature;
use crate::symbol::SymbolKind;
use crate::symbol::Symbols;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::object::TObject;
use crate::ttype::union::TUnion;
use crate::visibility::Visibility;

pub mod attribute;
pub mod class_like;
pub mod class_like_constant;
pub mod constant;
pub mod enum_case;
pub mod flags;
pub mod function_like;
pub mod parameter;
pub mod property;
pub mod property_hook;
pub mod ttype;

/// Holds all analyzed information about the symbols, structures, and relationships within a codebase.
///
/// This acts as the central repository for metadata gathered during static analysis,
/// including details about classes, interfaces, traits, enums, functions, constants,
/// their members, inheritance, dependencies, and associated types.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[non_exhaustive]
pub struct CodebaseMetadata {
    /// Configuration flag: Should types be inferred based on usage patterns?
    pub infer_types_from_usage: bool,
    /// Map from class-like FQCN (`Atom`) to its detailed metadata (`ClassLikeMetadata`).
    pub class_likes: AtomMap<ClassLikeMetadata>,
    /// Map from a function/method identifier tuple `(scope_id, function_id)` to its metadata (`FunctionLikeMetadata`).
    /// `scope_id` is the FQCN for methods or often `Atom::empty()` for global functions.
    pub function_likes: HashMap<(Atom, Atom), FunctionLikeMetadata>,
    /// Stores the kind (Class, Interface, etc.) for every known symbol FQCN.
    pub symbols: Symbols,
    /// Map from global constant FQN (`Atom`) to its metadata (`ConstantMetadata`).
    pub constants: AtomMap<ConstantMetadata>,
    /// Map from class/interface FQCN to the set of all its descendants (recursive).
    pub all_class_like_descendants: AtomMap<AtomSet>,
    /// Map from class/interface FQCN to the set of its direct descendants (children).
    pub direct_classlike_descendants: AtomMap<AtomSet>,
    /// Set of symbols (FQCNs) that are considered safe/validated.
    pub safe_symbols: AtomSet,
    /// Set of specific members `(SymbolFQCN, MemberName)` that are considered safe/validated.
    pub safe_symbol_members: HashSet<(Atom, Atom)>,
    /// Each `FileSignature` contains a hierarchical tree of `DefSignatureNode` representing
    /// top-level symbols (classes, functions, constants) and their nested members (methods, properties).
    pub file_signatures: HashMap<FileId, FileSignature>,
}

impl CodebaseMetadata {
    /// Creates a new, empty `CodebaseMetadata` with default values.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks if a class exists in the codebase (case-insensitive).
    ///
    /// # Examples
    /// ```ignore
    /// if codebase.class_exists("MyClass") {
    ///     // MyClass is a class
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn class_exists(&self, name: &str) -> bool {
        let lowercase_name = ascii_lowercase_atom(name);
        matches!(self.symbols.get_kind(&lowercase_name), Some(SymbolKind::Class))
    }

    /// Checks if an interface exists in the codebase (case-insensitive).
    #[inline]
    #[must_use]
    pub fn interface_exists(&self, name: &str) -> bool {
        let lowercase_name = ascii_lowercase_atom(name);
        matches!(self.symbols.get_kind(&lowercase_name), Some(SymbolKind::Interface))
    }

    /// Checks if a trait exists in the codebase (case-insensitive).
    #[inline]
    #[must_use]
    pub fn trait_exists(&self, name: &str) -> bool {
        let lowercase_name = ascii_lowercase_atom(name);
        matches!(self.symbols.get_kind(&lowercase_name), Some(SymbolKind::Trait))
    }

    /// Checks if an enum exists in the codebase (case-insensitive).
    #[inline]
    #[must_use]
    pub fn enum_exists(&self, name: &str) -> bool {
        let lowercase_name = ascii_lowercase_atom(name);
        matches!(self.symbols.get_kind(&lowercase_name), Some(SymbolKind::Enum))
    }

    /// Checks if a class-like (class, interface, trait, or enum) exists (case-insensitive).
    #[inline]
    #[must_use]
    pub fn class_like_exists(&self, name: &str) -> bool {
        let lowercase_name = ascii_lowercase_atom(name);
        self.symbols.contains(&lowercase_name)
    }

    /// Checks if a namespace exists (case-insensitive).
    #[inline]
    #[must_use]
    pub fn namespace_exists(&self, name: &str) -> bool {
        let lowercase_name = ascii_lowercase_atom(name);
        self.symbols.contains_namespace(&lowercase_name)
    }

    /// Checks if a class or trait exists in the codebase (case-insensitive).
    #[inline]
    #[must_use]
    pub fn class_or_trait_exists(&self, name: &str) -> bool {
        let lowercase_name = ascii_lowercase_atom(name);
        matches!(self.symbols.get_kind(&lowercase_name), Some(SymbolKind::Class | SymbolKind::Trait))
    }

    /// Checks if a class or interface exists in the codebase (case-insensitive).
    #[inline]
    #[must_use]
    pub fn class_or_interface_exists(&self, name: &str) -> bool {
        let lowercase_name = ascii_lowercase_atom(name);
        matches!(self.symbols.get_kind(&lowercase_name), Some(SymbolKind::Class | SymbolKind::Interface))
    }

    /// Checks if a method identifier exists in the codebase.
    #[inline]
    #[must_use]
    pub fn method_identifier_exists(&self, method_id: &MethodIdentifier) -> bool {
        let lowercase_class = ascii_lowercase_atom(method_id.get_class_name());
        let lowercase_method = ascii_lowercase_atom(method_id.get_method_name());
        let identifier = (lowercase_class, lowercase_method);
        self.function_likes.contains_key(&identifier)
    }

    /// Checks if a global function exists in the codebase (case-insensitive).
    #[inline]
    #[must_use]
    pub fn function_exists(&self, name: &str) -> bool {
        let lowercase_name = ascii_lowercase_atom(name);
        let identifier = (empty_atom(), lowercase_name);
        self.function_likes.contains_key(&identifier)
    }

    /// Checks if a global constant exists in the codebase.
    /// The namespace part is case-insensitive, but the constant name is case-sensitive.
    #[inline]
    #[must_use]
    pub fn constant_exists(&self, name: &str) -> bool {
        let lowercase_name = ascii_lowercase_constant_name_atom(name);
        self.constants.contains_key(&lowercase_name)
    }

    /// Checks if a method exists on a class-like, including inherited methods (case-insensitive).
    #[inline]
    #[must_use]
    pub fn method_exists(&self, class: &str, method: &str) -> bool {
        let lowercase_class = ascii_lowercase_atom(class);
        let lowercase_method = ascii_lowercase_atom(method);
        self.class_likes
            .get(&lowercase_class)
            .is_some_and(|meta| meta.appearing_method_ids.contains_key(&lowercase_method))
    }

    /// Checks if a property exists on a class-like, including inherited properties.
    /// Class name is case-insensitive, property name is case-sensitive.
    #[inline]
    #[must_use]
    pub fn property_exists(&self, class: &str, property: &str) -> bool {
        let lowercase_class = ascii_lowercase_atom(class);
        let property_name = atom(property);
        self.class_likes
            .get(&lowercase_class)
            .is_some_and(|meta| meta.appearing_property_ids.contains_key(&property_name))
    }

    /// Checks if a class constant or enum case exists on a class-like.
    /// Class name is case-insensitive, constant/case name is case-sensitive.
    #[inline]
    #[must_use]
    pub fn class_constant_exists(&self, class: &str, constant: &str) -> bool {
        let lowercase_class = ascii_lowercase_atom(class);
        let constant_name = atom(constant);
        self.class_likes.get(&lowercase_class).is_some_and(|meta| {
            meta.constants.contains_key(&constant_name) || meta.enum_cases.contains_key(&constant_name)
        })
    }

    /// Checks if a method is declared directly in a class (not inherited).
    #[inline]
    #[must_use]
    pub fn method_is_declared_in_class(&self, class: &str, method: &str) -> bool {
        let lowercase_class = ascii_lowercase_atom(class);
        let lowercase_method = ascii_lowercase_atom(method);
        self.class_likes
            .get(&lowercase_class)
            .and_then(|meta| meta.declaring_method_ids.get(&lowercase_method))
            .is_some_and(|method_id| method_id.get_class_name() == &lowercase_class)
    }

    /// Checks if a property is declared directly in a class (not inherited).
    #[inline]
    #[must_use]
    pub fn property_is_declared_in_class(&self, class: &str, property: &str) -> bool {
        let lowercase_class = ascii_lowercase_atom(class);
        let property_name = atom(property);
        self.class_likes.get(&lowercase_class).is_some_and(|meta| meta.properties.contains_key(&property_name))
    }

    /// Retrieves metadata for a class (case-insensitive).
    /// Returns `None` if the name doesn't correspond to a class.
    #[inline]
    #[must_use]
    pub fn get_class(&self, name: &str) -> Option<&ClassLikeMetadata> {
        let lowercase_name = ascii_lowercase_atom(name);
        if self.symbols.contains_class(&lowercase_name) { self.class_likes.get(&lowercase_name) } else { None }
    }

    /// Retrieves metadata for an interface (case-insensitive).
    #[inline]
    #[must_use]
    pub fn get_interface(&self, name: &str) -> Option<&ClassLikeMetadata> {
        let lowercase_name = ascii_lowercase_atom(name);
        if self.symbols.contains_interface(&lowercase_name) { self.class_likes.get(&lowercase_name) } else { None }
    }

    /// Retrieves metadata for a trait (case-insensitive).
    #[inline]
    #[must_use]
    pub fn get_trait(&self, name: &str) -> Option<&ClassLikeMetadata> {
        let lowercase_name = ascii_lowercase_atom(name);
        if self.symbols.contains_trait(&lowercase_name) { self.class_likes.get(&lowercase_name) } else { None }
    }

    /// Retrieves metadata for an enum (case-insensitive).
    #[inline]
    #[must_use]
    pub fn get_enum(&self, name: &str) -> Option<&ClassLikeMetadata> {
        let lowercase_name = ascii_lowercase_atom(name);
        if self.symbols.contains_enum(&lowercase_name) { self.class_likes.get(&lowercase_name) } else { None }
    }

    /// Retrieves metadata for any class-like structure (case-insensitive).
    #[inline]
    #[must_use]
    pub fn get_class_like(&self, name: &str) -> Option<&ClassLikeMetadata> {
        let lowercase_name = ascii_lowercase_atom(name);
        self.class_likes.get(&lowercase_name)
    }

    /// Retrieves metadata for a global function (case-insensitive).
    #[inline]
    #[must_use]
    pub fn get_function(&self, name: &str) -> Option<&FunctionLikeMetadata> {
        let lowercase_name = ascii_lowercase_atom(name);
        let identifier = (empty_atom(), lowercase_name);
        self.function_likes.get(&identifier)
    }

    /// Retrieves metadata for a method (case-insensitive for both class and method names).
    #[inline]
    #[must_use]
    pub fn get_method(&self, class: &str, method: &str) -> Option<&FunctionLikeMetadata> {
        let lowercase_class = ascii_lowercase_atom(class);
        let lowercase_method = ascii_lowercase_atom(method);
        let identifier = (lowercase_class, lowercase_method);
        self.function_likes.get(&identifier)
    }

    /// Retrieves metadata for a closure based on its file and position.
    #[inline]
    #[must_use]
    pub fn get_closure(&self, file_id: &FileId, position: &Position) -> Option<&FunctionLikeMetadata> {
        let file_ref = u64_atom(file_id.as_u64());
        let closure_ref = u32_atom(position.offset);
        let identifier = (file_ref, closure_ref);
        self.function_likes.get(&identifier)
    }

    /// Retrieves method metadata by `MethodIdentifier`.
    #[inline]
    #[must_use]
    pub fn get_method_by_id(&self, method_id: &MethodIdentifier) -> Option<&FunctionLikeMetadata> {
        let lowercase_class = ascii_lowercase_atom(method_id.get_class_name());
        let lowercase_method = ascii_lowercase_atom(method_id.get_method_name());
        let identifier = (lowercase_class, lowercase_method);
        self.function_likes.get(&identifier)
    }

    /// Retrieves the declaring method metadata, following the inheritance chain.
    /// This finds where the method is actually implemented.
    #[inline]
    #[must_use]
    pub fn get_declaring_method(&self, class: &str, method: &str) -> Option<&FunctionLikeMetadata> {
        let method_id = MethodIdentifier::new(atom(class), atom(method));
        let declaring_method_id = self.get_declaring_method_identifier(&method_id);
        self.get_method(declaring_method_id.get_class_name(), declaring_method_id.get_method_name())
    }

    /// Retrieves metadata for any function-like construct (function, method, or closure).
    /// This is a convenience method that delegates to the appropriate getter based on the identifier type.
    #[inline]
    #[must_use]
    pub fn get_function_like(
        &self,
        identifier: &crate::identifier::function_like::FunctionLikeIdentifier,
    ) -> Option<&FunctionLikeMetadata> {
        use crate::identifier::function_like::FunctionLikeIdentifier;
        match identifier {
            FunctionLikeIdentifier::Function(name) => self.get_function(name),
            FunctionLikeIdentifier::Method(class, method) => self.get_method(class, method),
            FunctionLikeIdentifier::Closure(file_id, position) => self.get_closure(file_id, position),
        }
    }

    /// Retrieves metadata for a global constant.
    /// Namespace lookup is case-insensitive, constant name is case-sensitive.
    #[inline]
    #[must_use]
    pub fn get_constant(&self, name: &str) -> Option<&ConstantMetadata> {
        let lowercase_name = ascii_lowercase_constant_name_atom(name);
        self.constants.get(&lowercase_name)
    }

    /// Retrieves metadata for a class constant.
    /// Class name is case-insensitive, constant name is case-sensitive.
    #[inline]
    #[must_use]
    pub fn get_class_constant(&self, class: &str, constant: &str) -> Option<&ClassLikeConstantMetadata> {
        let lowercase_class = ascii_lowercase_atom(class);
        let constant_name = atom(constant);
        self.class_likes.get(&lowercase_class).and_then(|meta| meta.constants.get(&constant_name))
    }

    /// Retrieves metadata for an enum case.
    #[inline]
    #[must_use]
    pub fn get_enum_case(&self, class: &str, case: &str) -> Option<&EnumCaseMetadata> {
        let lowercase_class = ascii_lowercase_atom(class);
        let case_name = atom(case);
        self.class_likes.get(&lowercase_class).and_then(|meta| meta.enum_cases.get(&case_name))
    }

    /// Retrieves metadata for a property directly from the class where it's declared.
    /// Class name is case-insensitive, property name is case-sensitive.
    #[inline]
    #[must_use]
    pub fn get_property(&self, class: &str, property: &str) -> Option<&PropertyMetadata> {
        let lowercase_class = ascii_lowercase_atom(class);
        let property_name = atom(property);
        self.class_likes.get(&lowercase_class)?.properties.get(&property_name)
    }

    /// Retrieves the property metadata, potentially from a parent class if inherited.
    #[inline]
    #[must_use]
    pub fn get_declaring_property(&self, class: &str, property: &str) -> Option<&PropertyMetadata> {
        let lowercase_class = ascii_lowercase_atom(class);
        let property_name = atom(property);
        let declaring_class = self.class_likes.get(&lowercase_class)?.declaring_property_ids.get(&property_name)?;
        self.class_likes.get(declaring_class)?.properties.get(&property_name)
    }
    // Type Resolution

    /// Gets the type of a property, resolving it from the declaring class if needed.
    #[inline]
    #[must_use]
    pub fn get_property_type(&self, class: &str, property: &str) -> Option<&TUnion> {
        let lowercase_class = ascii_lowercase_atom(class);
        let property_name = atom(property);
        let declaring_class = self.class_likes.get(&lowercase_class)?.declaring_property_ids.get(&property_name)?;
        let property_meta = self.class_likes.get(declaring_class)?.properties.get(&property_name)?;
        property_meta.type_metadata.as_ref().map(|tm| &tm.type_union)
    }

    /// Gets the type of a class constant, considering both type hints and inferred types.
    #[must_use]
    pub fn get_class_constant_type<'a>(&'a self, class: &str, constant: &str) -> Option<Cow<'a, TUnion>> {
        let lowercase_class = ascii_lowercase_atom(class);
        let constant_name = atom(constant);
        let class_meta = self.class_likes.get(&lowercase_class)?;

        // Check if it's an enum case
        if class_meta.kind.is_enum() && class_meta.enum_cases.contains_key(&constant_name) {
            let atomic = TAtomic::Object(TObject::new_enum_case(class_meta.original_name, constant_name));
            return Some(Cow::Owned(TUnion::from_atomic(atomic)));
        }

        // It's a regular class constant
        let constant_meta = class_meta.constants.get(&constant_name)?;

        // Prefer the type signature if available
        if let Some(type_meta) = constant_meta.type_metadata.as_ref() {
            return Some(Cow::Borrowed(&type_meta.type_union));
        }

        // Fall back to inferred type
        constant_meta.inferred_type.as_ref().map(|atomic| Cow::Owned(TUnion::from_atomic(atomic.clone())))
    }

    /// Gets the literal value of a class constant if it was inferred.
    #[inline]
    #[must_use]
    pub fn get_class_constant_literal_value(&self, class: &str, constant: &str) -> Option<&TAtomic> {
        let lowercase_class = ascii_lowercase_atom(class);
        let constant_name = atom(constant);
        self.class_likes
            .get(&lowercase_class)
            .and_then(|meta| meta.constants.get(&constant_name))
            .and_then(|constant_meta| constant_meta.inferred_type.as_ref())
    }
    // Inheritance Queries

    /// Checks if a child class extends a parent class (case-insensitive).
    #[inline]
    #[must_use]
    pub fn class_extends(&self, child: &str, parent: &str) -> bool {
        let lowercase_child = ascii_lowercase_atom(child);
        let lowercase_parent = ascii_lowercase_atom(parent);
        self.class_likes.get(&lowercase_child).is_some_and(|meta| meta.all_parent_classes.contains(&lowercase_parent))
    }

    /// Checks if a class directly extends a parent class (case-insensitive).
    #[inline]
    #[must_use]
    pub fn class_directly_extends(&self, child: &str, parent: &str) -> bool {
        let lowercase_child = ascii_lowercase_atom(child);
        let lowercase_parent = ascii_lowercase_atom(parent);
        self.class_likes
            .get(&lowercase_child)
            .is_some_and(|meta| meta.direct_parent_class.as_ref() == Some(&lowercase_parent))
    }

    /// Checks if a class implements an interface (case-insensitive).
    #[inline]
    #[must_use]
    pub fn class_implements(&self, class: &str, interface: &str) -> bool {
        let lowercase_class = ascii_lowercase_atom(class);
        let lowercase_interface = ascii_lowercase_atom(interface);
        self.class_likes
            .get(&lowercase_class)
            .is_some_and(|meta| meta.all_parent_interfaces.contains(&lowercase_interface))
    }

    /// Checks if a class directly implements an interface (case-insensitive).
    #[inline]
    #[must_use]
    pub fn class_directly_implements(&self, class: &str, interface: &str) -> bool {
        let lowercase_class = ascii_lowercase_atom(class);
        let lowercase_interface = ascii_lowercase_atom(interface);
        self.class_likes
            .get(&lowercase_class)
            .is_some_and(|meta| meta.direct_parent_interfaces.contains(&lowercase_interface))
    }

    /// Checks if a class uses a trait (case-insensitive).
    #[inline]
    #[must_use]
    pub fn class_uses_trait(&self, class: &str, trait_name: &str) -> bool {
        let lowercase_class = ascii_lowercase_atom(class);
        let lowercase_trait = ascii_lowercase_atom(trait_name);
        self.class_likes.get(&lowercase_class).is_some_and(|meta| meta.used_traits.contains(&lowercase_trait))
    }

    /// Checks if a trait has `@require-extends` for a class (case-insensitive).
    /// Returns true if the trait requires extending the specified class or any of its parents.
    #[inline]
    #[must_use]
    pub fn trait_requires_extends(&self, trait_name: &str, class_name: &str) -> bool {
        let lowercase_trait = ascii_lowercase_atom(trait_name);

        self.class_likes
            .get(&lowercase_trait)
            .is_some_and(|meta| meta.require_extends.iter().any(|required| self.is_instance_of(class_name, required)))
    }

    /// Checks if child is an instance of parent (via extends or implements).
    #[inline]
    #[must_use]
    pub fn is_instance_of(&self, child: &str, parent: &str) -> bool {
        if child == parent {
            return true;
        }

        let lowercase_child = ascii_lowercase_atom(child);
        let lowercase_parent = ascii_lowercase_atom(parent);

        if lowercase_child == lowercase_parent {
            return true;
        }

        self.class_likes.get(&lowercase_child).is_some_and(|meta| {
            meta.all_parent_classes.contains(&lowercase_parent)
                || meta.all_parent_interfaces.contains(&lowercase_parent)
                || meta.used_traits.contains(&lowercase_parent)
                || meta.require_extends.contains(&lowercase_parent)
                || meta.require_implements.contains(&lowercase_parent)
        })
    }

    /// Checks if the given name is an enum or final class.
    #[inline]
    #[must_use]
    pub fn is_enum_or_final_class(&self, name: &str) -> bool {
        let lowercase_name = ascii_lowercase_atom(name);
        self.class_likes.get(&lowercase_name).is_some_and(|meta| meta.kind.is_enum() || meta.flags.is_final())
    }

    /// Checks if a class-like can be part of an intersection.
    /// Generally, only final classes and enums cannot be intersected.
    #[inline]
    #[must_use]
    pub fn is_inheritable(&self, name: &str) -> bool {
        let lowercase_name = ascii_lowercase_atom(name);
        match self.symbols.get_kind(&lowercase_name) {
            Some(SymbolKind::Class) => self.class_likes.get(&lowercase_name).is_some_and(|meta| !meta.flags.is_final()),
            Some(SymbolKind::Enum) => false,
            Some(SymbolKind::Interface | SymbolKind::Trait) | None => true,
        }
    }

    /// Gets all descendants of a class (recursive).
    #[inline]
    #[must_use]
    pub fn get_class_descendants(&self, class: &str) -> AtomSet {
        let lowercase_class = ascii_lowercase_atom(class);
        let mut all_descendants = AtomSet::default();
        let mut queue = vec![&lowercase_class];
        let mut visited = AtomSet::default();
        visited.insert(lowercase_class);

        while let Some(current_name) = queue.pop() {
            if let Some(direct_descendants) = self.direct_classlike_descendants.get(current_name) {
                for descendant in direct_descendants {
                    if visited.insert(*descendant) {
                        all_descendants.insert(*descendant);
                        queue.push(descendant);
                    }
                }
            }
        }

        all_descendants
    }

    /// Gets all ancestors of a class (parents + interfaces).
    #[inline]
    #[must_use]
    pub fn get_class_ancestors(&self, class: &str) -> AtomSet {
        let lowercase_class = ascii_lowercase_atom(class);
        let mut ancestors = AtomSet::default();
        if let Some(meta) = self.class_likes.get(&lowercase_class) {
            ancestors.extend(meta.all_parent_classes.iter().copied());
            ancestors.extend(meta.all_parent_interfaces.iter().copied());
        }
        ancestors
    }

    /// Gets the class where a method is declared (following inheritance).
    #[inline]
    #[must_use]
    pub fn get_declaring_method_class(&self, class: &str, method: &str) -> Option<Atom> {
        let lowercase_class = ascii_lowercase_atom(class);
        let lowercase_method = ascii_lowercase_atom(method);

        self.class_likes
            .get(&lowercase_class)?
            .declaring_method_ids
            .get(&lowercase_method)
            .map(|method_id| *method_id.get_class_name())
    }

    /// Gets the class where a method appears (could be the declaring class or child class).
    #[inline]
    #[must_use]
    pub fn get_appearing_method_class(&self, class: &str, method: &str) -> Option<Atom> {
        let lowercase_class = ascii_lowercase_atom(class);
        let lowercase_method = ascii_lowercase_atom(method);
        self.class_likes
            .get(&lowercase_class)?
            .appearing_method_ids
            .get(&lowercase_method)
            .map(|method_id| *method_id.get_class_name())
    }

    /// Gets the declaring method identifier for a method.
    #[must_use]
    pub fn get_declaring_method_identifier(&self, method_id: &MethodIdentifier) -> MethodIdentifier {
        let lowercase_class = ascii_lowercase_atom(method_id.get_class_name());
        let lowercase_method = ascii_lowercase_atom(method_id.get_method_name());

        let Some(class_meta) = self.class_likes.get(&lowercase_class) else {
            return *method_id;
        };

        if let Some(declaring_method_id) = class_meta.declaring_method_ids.get(&lowercase_method) {
            return *declaring_method_id;
        }

        if class_meta.flags.is_abstract()
            && let Some(overridden_map) = class_meta.overridden_method_ids.get(&lowercase_method)
            && let Some((_, first_method_id)) = overridden_map.iter().next()
        {
            return *first_method_id;
        }

        *method_id
    }

    /// Checks if a method is overriding a parent method.
    #[inline]
    #[must_use]
    pub fn method_is_overriding(&self, class: &str, method: &str) -> bool {
        let lowercase_class = ascii_lowercase_atom(class);
        let lowercase_method = ascii_lowercase_atom(method);
        self.class_likes
            .get(&lowercase_class)
            .is_some_and(|meta| meta.overridden_method_ids.contains_key(&lowercase_method))
    }

    /// Checks if a method is abstract.
    #[inline]
    #[must_use]
    pub fn method_is_abstract(&self, class: &str, method: &str) -> bool {
        let lowercase_class = ascii_lowercase_atom(class);
        let lowercase_method = ascii_lowercase_atom(method);
        let identifier = (lowercase_class, lowercase_method);
        self.function_likes
            .get(&identifier)
            .and_then(|meta| meta.method_metadata.as_ref())
            .is_some_and(|method_meta| method_meta.is_abstract)
    }

    /// Checks if a method is static.
    #[inline]
    #[must_use]
    pub fn method_is_static(&self, class: &str, method: &str) -> bool {
        let lowercase_class = ascii_lowercase_atom(class);
        let lowercase_method = ascii_lowercase_atom(method);
        let identifier = (lowercase_class, lowercase_method);
        self.function_likes
            .get(&identifier)
            .and_then(|meta| meta.method_metadata.as_ref())
            .is_some_and(|method_meta| method_meta.is_static)
    }

    /// Checks if a method is final.
    #[inline]
    #[must_use]
    pub fn method_is_final(&self, class: &str, method: &str) -> bool {
        let lowercase_class = ascii_lowercase_atom(class);
        let lowercase_method = ascii_lowercase_atom(method);
        let identifier = (lowercase_class, lowercase_method);
        self.function_likes
            .get(&identifier)
            .and_then(|meta| meta.method_metadata.as_ref())
            .is_some_and(|method_meta| method_meta.is_final)
    }

    /// Gets the effective visibility of a method, taking into account trait alias visibility overrides.
    ///
    /// When a trait method is aliased with a visibility modifier (e.g., `use Trait { method as public aliasedMethod; }`),
    /// the visibility is stored in the class's `trait_visibility_map`. This method checks that map first,
    /// then falls back to the method's declared visibility.
    #[inline]
    #[must_use]
    pub fn get_method_visibility(&self, class: &str, method: &str) -> Option<Visibility> {
        let lowercase_class = ascii_lowercase_atom(class);
        let lowercase_method = ascii_lowercase_atom(method);

        // First check if there's a trait visibility override for this method
        if let Some(class_meta) = self.class_likes.get(&lowercase_class)
            && let Some(overridden_visibility) = class_meta.trait_visibility_map.get(&lowercase_method)
        {
            return Some(*overridden_visibility);
        }

        // Fall back to the method's declared visibility
        let declaring_class = self.get_declaring_method_class(class, method)?;
        let identifier = (declaring_class, lowercase_method);

        self.function_likes
            .get(&identifier)
            .and_then(|meta| meta.method_metadata.as_ref())
            .map(|method_meta| method_meta.visibility)
    }

    /// Gets thrown types for a function-like, including inherited throws.
    #[must_use]
    pub fn get_function_like_thrown_types<'a>(
        &'a self,
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

        if let Some(overridden_map) = class_like.overridden_method_ids.get(method_name) {
            for (parent_class_name, parent_method_id) in overridden_map {
                let Some(parent_class) = self.class_likes.get(parent_class_name) else {
                    continue;
                };

                let parent_method_key = (*parent_method_id.get_class_name(), *parent_method_id.get_method_name());
                if let Some(parent_method) = self.function_likes.get(&parent_method_key) {
                    let thrown = self.get_function_like_thrown_types(Some(parent_class), parent_method);
                    if !thrown.is_empty() {
                        return thrown;
                    }
                }
            }
        }

        &[]
    }

    /// Gets the class where a property is declared.
    #[inline]
    #[must_use]
    pub fn get_declaring_property_class(&self, class: &str, property: &str) -> Option<Atom> {
        let lowercase_class = ascii_lowercase_atom(class);
        let property_name = atom(property);
        self.class_likes.get(&lowercase_class)?.declaring_property_ids.get(&property_name).copied()
    }

    /// Gets the class where a property appears.
    #[inline]
    #[must_use]
    pub fn get_appearing_property_class(&self, class: &str, property: &str) -> Option<Atom> {
        let lowercase_class = ascii_lowercase_atom(class);
        let property_name = atom(property);
        self.class_likes.get(&lowercase_class)?.appearing_property_ids.get(&property_name).copied()
    }

    /// Gets all descendants of a class (recursive).
    #[must_use]
    pub fn get_all_descendants(&self, class: &str) -> AtomSet {
        let lowercase_class = ascii_lowercase_atom(class);
        let mut all_descendants = AtomSet::default();
        let mut queue = vec![&lowercase_class];
        let mut visited = AtomSet::default();
        visited.insert(lowercase_class);

        while let Some(current_name) = queue.pop() {
            if let Some(direct_descendants) = self.direct_classlike_descendants.get(current_name) {
                for descendant in direct_descendants {
                    if visited.insert(*descendant) {
                        all_descendants.insert(*descendant);
                        queue.push(descendant);
                    }
                }
            }
        }

        all_descendants
    }

    /// Generates a unique name for an anonymous class based on its span.
    #[must_use]
    pub fn get_anonymous_class_name(span: mago_span::Span) -> Atom {
        use std::io::Write;

        let mut buffer = [0u8; 64];
        let mut writer = &mut buffer[..];

        unsafe {
            write!(writer, "class@anonymous:{}-{}:{}", span.file_id, span.start.offset, span.end.offset)
                .unwrap_unchecked();
        };

        let written_len = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());

        atom(unsafe { std::str::from_utf8(&buffer[..written_len]).unwrap_unchecked() })
    }

    /// Retrieves the metadata for an anonymous class based on its span.
    #[must_use]
    pub fn get_anonymous_class(&self, span: mago_span::Span) -> Option<&ClassLikeMetadata> {
        let name = Self::get_anonymous_class_name(span);
        if self.class_exists(&name) { self.class_likes.get(&name) } else { None }
    }

    /// Gets the file signature for a given file ID.
    ///
    /// # Arguments
    ///
    /// * `file_id` - The file identifier
    ///
    /// # Returns
    ///
    /// A reference to the `FileSignature` if it exists, or `None` if the file has no signature.
    #[inline]
    #[must_use]
    pub fn get_file_signature(&self, file_id: &FileId) -> Option<&FileSignature> {
        self.file_signatures.get(file_id)
    }

    /// Adds or updates a file signature for a given file ID.
    ///
    /// # Arguments
    ///
    /// * `file_id` - The file identifier
    /// * `signature` - The file signature
    ///
    /// # Returns
    ///
    /// The previous `FileSignature` if it existed.
    #[inline]
    pub fn set_file_signature(&mut self, file_id: FileId, signature: FileSignature) -> Option<FileSignature> {
        self.file_signatures.insert(file_id, signature)
    }

    /// Removes the file signature for a given file ID.
    ///
    /// # Arguments
    ///
    /// * `file_id` - The file identifier
    ///
    /// # Returns
    ///
    /// The removed `FileSignature` if it existed.
    #[inline]
    pub fn remove_file_signature(&mut self, file_id: &FileId) -> Option<FileSignature> {
        self.file_signatures.remove(file_id)
    }

    /// Merges information from another `CodebaseMetadata` into this one.
    ///
    /// When both metadata have the same priority, the one with the smaller span is kept
    /// for deterministic results regardless of scan order.
    pub fn extend(&mut self, other: CodebaseMetadata) {
        for (k, v) in other.class_likes {
            match self.class_likes.entry(k) {
                Entry::Occupied(mut entry) => {
                    if should_replace_metadata(entry.get().flags, entry.get().span, v.flags, v.span) {
                        entry.insert(v);
                    }
                }
                Entry::Vacant(entry) => {
                    entry.insert(v);
                }
            }
        }

        for (k, v) in other.function_likes {
            match self.function_likes.entry(k) {
                Entry::Occupied(mut entry) => {
                    if should_replace_metadata(entry.get().flags, entry.get().span, v.flags, v.span) {
                        entry.insert(v);
                    }
                }
                Entry::Vacant(entry) => {
                    entry.insert(v);
                }
            }
        }

        for (k, v) in other.constants {
            match self.constants.entry(k) {
                Entry::Occupied(mut entry) => {
                    if should_replace_metadata(entry.get().flags, entry.get().span, v.flags, v.span) {
                        entry.insert(v);
                    }
                }
                Entry::Vacant(entry) => {
                    entry.insert(v);
                }
            }
        }

        self.symbols.extend(other.symbols);

        for (k, v) in other.all_class_like_descendants {
            self.all_class_like_descendants.entry(k).or_default().extend(v);
        }

        for (k, v) in other.direct_classlike_descendants {
            self.direct_classlike_descendants.entry(k).or_default().extend(v);
        }

        self.safe_symbols.extend(other.safe_symbols);
        self.safe_symbol_members.extend(other.safe_symbol_members);
        self.infer_types_from_usage |= other.infer_types_from_usage;
    }

    /// Takes all issues from the codebase metadata.
    pub fn take_issues(&mut self, user_defined: bool) -> IssueCollection {
        let mut issues = IssueCollection::new();

        for meta in self.class_likes.values_mut() {
            if user_defined && !meta.flags.is_user_defined() {
                continue;
            }
            issues.extend(meta.take_issues());
        }

        for meta in self.function_likes.values_mut() {
            if user_defined && !meta.flags.is_user_defined() {
                continue;
            }
            issues.extend(meta.take_issues());
        }

        for meta in self.constants.values_mut() {
            if user_defined && !meta.flags.is_user_defined() {
                continue;
            }
            issues.extend(meta.take_issues());
        }

        issues
    }

    /// Gets all file IDs that have signatures in this metadata.
    ///
    /// This is a helper method for incremental analysis to iterate over all files.
    #[must_use]
    pub fn get_all_file_ids(&self) -> Vec<FileId> {
        self.file_signatures.keys().copied().collect()
    }
}

impl Default for CodebaseMetadata {
    #[inline]
    fn default() -> Self {
        Self {
            class_likes: AtomMap::default(),
            function_likes: HashMap::default(),
            symbols: Symbols::new(),
            infer_types_from_usage: false,
            constants: AtomMap::default(),
            all_class_like_descendants: AtomMap::default(),
            direct_classlike_descendants: AtomMap::default(),
            safe_symbols: AtomSet::default(),
            safe_symbol_members: HashSet::default(),
            file_signatures: HashMap::default(),
        }
    }
}

/// Determines which metadata value to keep when merging duplicates.
///
/// Priority: user-defined > built-in > other. Uses smaller span as tie-breaker.
/// Returns `true` if the new value should replace the existing one.
fn should_replace_metadata(
    existing_flags: MetadataFlags,
    existing_span: Span,
    new_flags: MetadataFlags,
    new_span: Span,
) -> bool {
    let new_is_user_defined = new_flags.is_user_defined();
    let existing_is_user_defined = existing_flags.is_user_defined();

    if new_is_user_defined != existing_is_user_defined {
        return new_is_user_defined;
    }

    let new_is_built_in = new_flags.is_built_in();
    let existing_is_built_in = existing_flags.is_built_in();

    if new_is_built_in != existing_is_built_in {
        return new_is_built_in;
    }

    new_span < existing_span
}
