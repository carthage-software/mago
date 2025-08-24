use std::collections::hash_map::Entry;

use ahash::HashMap;
use ahash::HashSet;
use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::AtomMap;
use mago_atom::AtomSet;
use mago_reporting::IssueCollection;

use crate::get_closure;
use crate::get_function;
use crate::get_method;
use crate::identifier::function_like::FunctionLikeIdentifier;
use crate::identifier::method::MethodIdentifier;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::metadata::constant::ConstantMetadata;
use crate::metadata::function_like::FunctionLikeMetadata;
use crate::metadata::property::PropertyMetadata;
use crate::metadata::ttype::TypeMetadata;
use crate::symbol::SymbolKind;
use crate::symbol::Symbols;
use crate::ttype::atomic::TAtomic;
use crate::ttype::union::TUnion;

pub mod attribute;
pub mod class_like;
pub mod class_like_constant;
pub mod constant;
pub mod enum_case;
pub mod flags;
pub mod function_like;
pub mod parameter;
pub mod property;
pub mod ttype;

/// Holds all analyzed information about the symbols, structures, and relationships within a codebase.
///
/// This acts as the central repository for metadata gathered during static analysis,
/// including details about classes, interfaces, traits, enums, functions, constants,
/// their members, inheritance, dependencies, and associated types.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct CodebaseMetadata {
    /// Configuration flag: Should types be inferred based on usage patterns?
    pub infer_types_from_usage: bool,
    /// Map from type alias name (`Atom`) to its metadata (`TypeMetadata`).
    pub aliases: AtomMap<TypeMetadata>,
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
    /// Set of symbols (FQCNs).
    pub safe_symbols: AtomSet,
    /// Set of specific members `(SymbolFQCN, MemberName)`.
    pub safe_symbol_members: HashSet<(Atom, Atom)>,
}

impl CodebaseMetadata {
    /// Creates a new, empty `CodebaseMetadata` with default values.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks if a class-like structure can be part of an intersection.
    /// Generally, only final classes cannot be intersected further down the hierarchy.
    #[inline]
    pub fn is_inheritable(&self, fq_class_name: &Atom) -> bool {
        match self.symbols.get_kind(fq_class_name) {
            Some(SymbolKind::Class) => {
                // Check if the class metadata exists and if it's NOT final
                self.class_likes.get(fq_class_name).is_some_and(|meta| !meta.flags.is_final())
            }
            Some(SymbolKind::Enum) => {
                // Enums are final and cannot be part of intersections
                false
            }
            Some(SymbolKind::Interface) | Some(SymbolKind::Trait) | None => {
                // Interfaces, Enums, Traits, or non-existent symbols can conceptually be part of intersections
                true
            }
        }
    }

    #[inline]
    pub fn class_or_trait_can_use_trait(&self, child_class: &Atom, parent_trait: &Atom) -> bool {
        if let Some(metadata) = self.class_likes.get(child_class) {
            if metadata.used_traits.contains(parent_trait) {
                return true;
            }

            return metadata.used_traits.contains(parent_trait);
        }
        false
    }

    /// Retrieves the literal value (as a `TAtomic`) of a class constant, if it was inferred.
    /// Returns `None` if the class/constant doesn't exist or the value type wasn't inferred.
    #[inline]
    pub fn get_classconst_literal_value(&self, fq_class_name: &Atom, const_name: &Atom) -> Option<&TAtomic> {
        self.class_likes
            .get(fq_class_name)
            .and_then(|class_metadata| class_metadata.constants.get(const_name))
            .and_then(|constant_metadata| constant_metadata.inferred_type.as_ref())
    }

    /// Checks if a property with the given name exists (is declared or inherited) within the class-like structure.
    /// Relies on `ClassLikeMetadata::has_appearing_property`.
    #[inline]
    pub fn property_exists(&self, classlike_name: &Atom, property_name: &Atom) -> bool {
        self.class_likes
            .get(classlike_name)
            .is_some_and(|metadata| metadata.appearing_property_ids.contains_key(property_name))
    }

    /// Checks if a method with the given name exists within the class-like structure.
    /// Relies on `ClassLikeMetadata::has_method`.
    #[inline]
    pub fn method_exists(&self, classlike_name: &Atom, method_name: &Atom) -> bool {
        self.class_likes.get(classlike_name).is_some_and(|metadata| metadata.methods.contains(method_name))
    }

    /// Checks if a method with the given name exists (is declared or inherited) within the class-like structure.
    /// Relies on `ClassLikeMetadata::has_appearing_method`.
    #[inline]
    pub fn appearing_method_exists(&self, classlike_name: &Atom, method_name: &Atom) -> bool {
        self.class_likes.get(classlike_name).is_some_and(|metadata| metadata.has_appearing_method(method_name))
    }

    /// Checks specifically if a method is *declared* directly within the given class-like (not just inherited).
    #[inline]
    pub fn declaring_method_exists(&self, classlike_name: &Atom, method_name: &Atom) -> bool {
        self.class_likes.get(classlike_name).and_then(|metadata| metadata.declaring_method_ids.get(method_name))
            == Some(classlike_name) // Check if declaring class is this class
    }

    /// Finds the FQCN of the class/trait where a property was originally declared for a given class context.
    /// Returns `None` if the property doesn't appear in the class hierarchy.
    #[inline]
    pub fn get_declaring_class_for_property(&self, fq_class_name: &Atom, property_name: &Atom) -> Option<&Atom> {
        self.class_likes.get(fq_class_name).and_then(|metadata| metadata.declaring_property_ids.get(property_name))
    }

    /// Retrieves the full metadata for a property as it appears in the context of a specific class.
    /// This might be the metadata from the declaring class.
    /// Returns `None` if the class or property doesn't exist in this context.
    #[inline]
    pub fn get_property_metadata(&self, fq_class_name: &Atom, property_name: &Atom) -> Option<&PropertyMetadata> {
        // Find where the property appears (could be inherited)
        let appearing_class_fqcn =
            self.class_likes.get(fq_class_name).and_then(|meta| meta.appearing_property_ids.get(property_name)); // Assumes get_appearing_property_ids

        // Get the metadata from the class where it appears
        appearing_class_fqcn
            .and_then(|fqcn| self.class_likes.get(fqcn))
            .and_then(|meta| meta.properties.get(property_name))
    }

    /// Retrieves the type union for a property within the context of a specific class.
    /// It finds the declaring class of the property and returns its type signature.
    /// Returns `None` if the property or its type cannot be found.
    #[inline]
    pub fn get_property_type(&self, fq_class_name: &Atom, property_name: &Atom) -> Option<&TUnion> {
        // Find the class where the property was originally declared
        let declaring_class_fqcn = self.get_declaring_class_for_property(fq_class_name, property_name)?;
        // Get the metadata for that property from its declaring class
        let property_metadata = self.class_likes.get(declaring_class_fqcn)?.properties.get(property_name)?;

        // Return the type metadata's union from that metadata
        property_metadata.type_metadata.as_ref().map(|tm| &tm.type_union)
    }

    /// Resolves a `MethodIdentifier` to the identifier of the method as it *appears* in the given class context.
    /// This could be the declaring class or an ancestor if inherited.
    #[inline]
    pub fn get_appearing_method_id(&self, method_id: &MethodIdentifier) -> MethodIdentifier {
        self.class_likes
            .get(method_id.get_class_name())
            .and_then(|metadata| metadata.appearing_method_ids.get(method_id.get_method_name()))
            .map_or(*method_id, |appearing_fqcn| MethodIdentifier::new(*appearing_fqcn, *method_id.get_method_name()))
    }

    /// Retrieves the metadata for a specific function-like construct using its identifier.
    #[inline]
    pub fn get_function_like(&self, identifier: &FunctionLikeIdentifier) -> Option<&FunctionLikeMetadata> {
        match identifier {
            FunctionLikeIdentifier::Function(fq_function_name) => get_function(self, fq_function_name),
            FunctionLikeIdentifier::Method(fq_classlike_name, method_name) => {
                get_method(self, fq_classlike_name, method_name)
            }
            FunctionLikeIdentifier::Closure(file_id, position) => get_closure(self, file_id, position),
        }
    }

    /// Merges information from another `CodebaseMetadata` into this one.
    /// Collections are extended. For HashMaps, entries in `other` may overwrite existing ones.
    #[inline]
    pub fn extend(&mut self, other: CodebaseMetadata) {
        for (k, v) in other.aliases {
            self.aliases.entry(k).or_insert(v);
        }

        // Merge class-likes with priority
        for (k, v) in other.class_likes {
            let metadata_to_keep = match self.class_likes.entry(k) {
                Entry::Occupied(entry) => {
                    let existing_metadata = entry.remove();

                    if v.flags.is_user_defined() {
                        v
                    } else if existing_metadata.flags.is_user_defined() {
                        existing_metadata
                    } else if v.flags.is_built_in() {
                        v
                    } else if existing_metadata.flags.is_built_in() {
                        existing_metadata
                    } else {
                        v
                    }
                }
                Entry::Vacant(_) => v,
            };
            self.class_likes.insert(k, metadata_to_keep);
        }

        for (k, v) in other.function_likes {
            let metadata_to_keep = match self.function_likes.entry(k) {
                Entry::Occupied(entry) => {
                    let existing_metadata = entry.remove();

                    if v.flags.is_user_defined() {
                        v
                    } else if existing_metadata.flags.is_user_defined() {
                        existing_metadata
                    } else if v.flags.is_built_in() {
                        v
                    } else if existing_metadata.flags.is_built_in() {
                        existing_metadata
                    } else {
                        v
                    }
                }
                Entry::Vacant(_) => v,
            };
            self.function_likes.insert(k, metadata_to_keep);
        }

        for (k, v) in other.constants {
            let metadata_to_keep = match self.constants.entry(k) {
                Entry::Occupied(entry) => {
                    let existing_metadata = entry.remove();

                    if v.flags.is_user_defined() {
                        v
                    } else if existing_metadata.flags.is_user_defined() {
                        existing_metadata
                    } else if v.flags.is_built_in() {
                        v
                    } else if existing_metadata.flags.is_built_in() {
                        existing_metadata
                    } else {
                        v
                    }
                }
                Entry::Vacant(_) => v,
            };
            self.constants.insert(k, metadata_to_keep);
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

    pub fn take_issues(&mut self, user_defined: bool) -> IssueCollection {
        let mut issues = IssueCollection::new();

        for metadata in self.class_likes.values_mut() {
            if user_defined && !metadata.flags.is_user_defined() {
                continue;
            }

            issues.extend(metadata.take_issues());
        }

        for metadata in self.function_likes.values_mut() {
            if user_defined && !metadata.flags.is_user_defined() {
                continue;
            }

            issues.extend(metadata.take_issues());
        }

        for metadata in self.constants.values_mut() {
            if user_defined && !metadata.flags.is_user_defined() {
                continue;
            }

            issues.extend(metadata.take_issues());
        }

        issues
    }
}

/// Provides a default, empty `CodebaseMetadata`.
impl Default for CodebaseMetadata {
    #[inline]
    fn default() -> Self {
        Self {
            class_likes: AtomMap::default(),
            aliases: AtomMap::default(),
            function_likes: HashMap::default(),
            symbols: Symbols::new(),
            infer_types_from_usage: false,
            constants: AtomMap::default(),
            all_class_like_descendants: AtomMap::default(),
            direct_classlike_descendants: AtomMap::default(),
            safe_symbols: AtomSet::default(),
            safe_symbol_members: HashSet::default(),
        }
    }
}
