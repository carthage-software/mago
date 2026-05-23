use std::borrow::Cow;
use std::collections::hash_map::Entry;

use foldhash::HashMap;
use foldhash::HashSet;
use serde::Deserialize;
use serde::Serialize;

use mago_database::file::File;
use mago_database::file::FileId;
use mago_reporting::IssueCollection;
use mago_span::Span;
use mago_word::Word;
use mago_word::WordMap;
use mago_word::WordSet;
use mago_word::ascii_lowercase_constant_name_word;
use mago_word::ascii_lowercase_word;
use mago_word::empty_word;
use mago_word::word;

use crate::diff::CodebaseDiff;
use crate::identifier::method::MethodIdentifier;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::metadata::class_like_constant::ClassLikeConstantMetadata;
use crate::metadata::constant::ConstantMetadata;
use crate::metadata::enum_case::EnumCaseMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::function_like::FunctionLikeMetadata;
use crate::metadata::property::PropertyMetadata;
use crate::metadata::ttype::TypeMetadata;
use crate::reference::SymbolReferences;
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
pub mod version_constraint;

/// Lightweight set of keys extracted from a per-file [`CodebaseMetadata`].
///
/// Used by the incremental engine to efficiently remove a file's contributions from the
/// merged codebase without keeping a full `CodebaseMetadata` clone per file.
/// Created via [`CodebaseMetadata::extract_keys()`].
#[derive(Debug, Clone)]
pub struct CodebaseEntryKeys {
    /// Class-like FQCN atoms (also used for symbol removal).
    pub class_like_names: Vec<Word>,
    /// Function-like `(scope, name)` tuples.
    pub function_like_keys: Vec<(Word, Word)>,
    /// Constant FQN atoms.
    pub constant_names: Vec<Word>,
    /// File IDs that had signatures in this metadata.
    pub file_ids: Vec<FileId>,
}

/// Holds all analyzed information about the symbols, structures, and relationships within a codebase.
///
/// This acts as the central repository for metadata gathered during static analysis,
/// including details about classes, interfaces, traits, enums, functions, constants,
/// their members, inheritance, dependencies, and associated types.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[non_exhaustive]
#[allow(clippy::unsafe_derive_deserialize)]
pub struct CodebaseMetadata {
    /// Configuration flag: Should types be inferred based on usage patterns?
    pub infer_types_from_usage: bool,
    /// Map from class-like FQCN (`Word`) to its detailed metadata (`ClassLikeMetadata`).
    pub class_likes: WordMap<ClassLikeMetadata>,
    /// Map from a function/method identifier tuple `(scope_id, function_id)` to its metadata (`FunctionLikeMetadata`).
    /// `scope_id` is the FQCN for methods or often `Word::empty()` for global functions.
    pub function_likes: HashMap<(Word, Word), FunctionLikeMetadata>,
    /// Stores the kind (Class, Interface, etc.) for every known symbol FQCN.
    pub symbols: Symbols,
    /// Map from global constant FQN (`Word`) to its metadata (`ConstantMetadata`).
    pub constants: WordMap<ConstantMetadata>,
    /// Map from class/interface FQCN to the set of all its descendants (recursive).
    pub all_class_like_descendants: WordMap<WordSet>,
    /// Map from class/interface FQCN to the set of its direct descendants (children).
    pub direct_classlike_descendants: WordMap<WordSet>,
    /// Set of symbols (FQCNs) that are considered safe/validated.
    pub safe_symbols: WordSet,
    /// Set of specific members `(SymbolFQCN, MemberName)` that are considered safe/validated.
    pub safe_symbol_members: HashSet<(Word, Word)>,
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
    pub fn class_exists(&self, name: &[u8]) -> bool {
        let lowercase_name = ascii_lowercase_word(name);
        matches!(self.symbols.get_kind(lowercase_name), Some(SymbolKind::Class))
    }

    /// Checks if an interface exists in the codebase (case-insensitive).
    #[inline]
    #[must_use]
    pub fn interface_exists(&self, name: &[u8]) -> bool {
        let lowercase_name = ascii_lowercase_word(name);
        matches!(self.symbols.get_kind(lowercase_name), Some(SymbolKind::Interface))
    }

    /// Checks if a trait exists in the codebase (case-insensitive).
    #[inline]
    #[must_use]
    pub fn trait_exists(&self, name: &[u8]) -> bool {
        let lowercase_name = ascii_lowercase_word(name);
        matches!(self.symbols.get_kind(lowercase_name), Some(SymbolKind::Trait))
    }

    /// Checks if an enum exists in the codebase (case-insensitive).
    #[inline]
    #[must_use]
    pub fn enum_exists(&self, name: &[u8]) -> bool {
        let lowercase_name = ascii_lowercase_word(name);
        matches!(self.symbols.get_kind(lowercase_name), Some(SymbolKind::Enum))
    }

    /// Checks if a class-like (class, interface, trait, or enum) exists (case-insensitive).
    #[inline]
    #[must_use]
    pub fn class_like_exists(&self, name: &[u8]) -> bool {
        let lowercase_name = ascii_lowercase_word(name);
        self.symbols.contains(lowercase_name)
    }

    /// Checks if a namespace exists (case-insensitive).
    #[inline]
    #[must_use]
    pub fn namespace_exists(&self, name: &[u8]) -> bool {
        let lowercase_name = ascii_lowercase_word(name);
        self.symbols.contains_namespace(lowercase_name)
    }

    /// Checks if a class or trait exists in the codebase (case-insensitive).
    #[inline]
    #[must_use]
    pub fn class_or_trait_exists(&self, name: &[u8]) -> bool {
        let lowercase_name = ascii_lowercase_word(name);
        matches!(self.symbols.get_kind(lowercase_name), Some(SymbolKind::Class | SymbolKind::Trait))
    }

    /// Checks if a class or interface exists in the codebase (case-insensitive).
    #[inline]
    #[must_use]
    pub fn class_or_interface_exists(&self, name: &[u8]) -> bool {
        let lowercase_name = ascii_lowercase_word(name);
        matches!(self.symbols.get_kind(lowercase_name), Some(SymbolKind::Class | SymbolKind::Interface))
    }

    /// Checks if a method identifier exists in the codebase.
    #[inline]
    #[must_use]
    pub fn method_identifier_exists(&self, method_id: &MethodIdentifier) -> bool {
        let lowercase_class = ascii_lowercase_word(method_id.get_class_name().as_bytes());
        let lowercase_method = ascii_lowercase_word(method_id.get_method_name().as_bytes());
        let identifier = (lowercase_class, lowercase_method);
        self.function_likes.contains_key(&identifier)
    }

    /// Checks if a global function exists in the codebase (case-insensitive).
    #[inline]
    #[must_use]
    pub fn function_exists(&self, name: &[u8]) -> bool {
        let lowercase_name = ascii_lowercase_word(name);
        let identifier = (empty_word(), lowercase_name);
        self.function_likes.contains_key(&identifier)
    }

    /// Checks if a global constant exists in the codebase.
    /// The namespace part is case-insensitive, but the constant name is case-sensitive.
    #[inline]
    #[must_use]
    pub fn constant_exists(&self, name: &[u8]) -> bool {
        let lowercase_name = ascii_lowercase_constant_name_word(name);
        self.constants.contains_key(&lowercase_name)
    }

    /// Checks if a method exists on a class-like, including inherited methods (case-insensitive).
    #[inline]
    #[must_use]
    pub fn method_exists(&self, class: &[u8], method: &[u8]) -> bool {
        let lowercase_class = ascii_lowercase_word(class);
        let lowercase_method = ascii_lowercase_word(method);
        self.class_likes
            .get(&lowercase_class)
            .is_some_and(|meta| meta.appearing_method_ids.contains_key(&lowercase_method))
    }

    /// Checks if a property exists on a class-like, including inherited properties.
    /// Class name is case-insensitive, property name is case-sensitive.
    #[inline]
    #[must_use]
    pub fn property_exists(&self, class: &[u8], property: &[u8]) -> bool {
        let lowercase_class = ascii_lowercase_word(class);
        let property_name = word(property);
        self.class_likes
            .get(&lowercase_class)
            .is_some_and(|meta| meta.appearing_property_ids.contains_key(&property_name))
    }

    /// Checks if a class constant or enum case exists on a class-like.
    /// Class name is case-insensitive, constant/case name is case-sensitive.
    #[inline]
    #[must_use]
    pub fn class_constant_exists(&self, class: &[u8], constant: &[u8]) -> bool {
        let lowercase_class = ascii_lowercase_word(class);
        let constant_name = word(constant);
        self.class_likes.get(&lowercase_class).is_some_and(|meta| {
            meta.constants.contains_key(&constant_name) || meta.enum_cases.contains_key(&constant_name)
        })
    }

    /// Checks if a method is declared directly in a class (not inherited).
    #[inline]
    #[must_use]
    pub fn method_is_declared_in_class(&self, class: &[u8], method: &[u8]) -> bool {
        let lowercase_class = ascii_lowercase_word(class);
        let lowercase_method = ascii_lowercase_word(method);
        self.class_likes
            .get(&lowercase_class)
            .and_then(|meta| meta.declaring_method_ids.get(&lowercase_method))
            .is_some_and(|method_id| method_id.get_class_name() == lowercase_class)
    }

    /// Checks if a property is declared directly in a class (not inherited).
    #[inline]
    #[must_use]
    pub fn property_is_declared_in_class(&self, class: &[u8], property: &[u8]) -> bool {
        let lowercase_class = ascii_lowercase_word(class);
        let property_name = word(property);
        self.class_likes.get(&lowercase_class).is_some_and(|meta| meta.properties.contains_key(&property_name))
    }

    /// Retrieves metadata for a class (case-insensitive).
    /// Returns `None` if the name doesn't correspond to a class.
    #[inline]
    #[must_use]
    pub fn get_class(&self, name: &[u8]) -> Option<&ClassLikeMetadata> {
        let lowercase_name = ascii_lowercase_word(name);
        if self.symbols.contains_class(lowercase_name) { self.class_likes.get(&lowercase_name) } else { None }
    }

    /// Retrieves metadata for an interface (case-insensitive).
    #[inline]
    #[must_use]
    pub fn get_interface(&self, name: &[u8]) -> Option<&ClassLikeMetadata> {
        let lowercase_name = ascii_lowercase_word(name);
        if self.symbols.contains_interface(lowercase_name) { self.class_likes.get(&lowercase_name) } else { None }
    }

    /// Retrieves metadata for a trait (case-insensitive).
    #[inline]
    #[must_use]
    pub fn get_trait(&self, name: &[u8]) -> Option<&ClassLikeMetadata> {
        let lowercase_name = ascii_lowercase_word(name);
        if self.symbols.contains_trait(lowercase_name) { self.class_likes.get(&lowercase_name) } else { None }
    }

    /// Retrieves metadata for an enum (case-insensitive).
    #[inline]
    #[must_use]
    pub fn get_enum(&self, name: &[u8]) -> Option<&ClassLikeMetadata> {
        let lowercase_name = ascii_lowercase_word(name);
        if self.symbols.contains_enum(lowercase_name) { self.class_likes.get(&lowercase_name) } else { None }
    }

    /// Retrieves metadata for any class-like structure (case-insensitive).
    #[inline]
    #[must_use]
    pub fn get_class_like(&self, name: &[u8]) -> Option<&ClassLikeMetadata> {
        let lowercase_name = ascii_lowercase_word(name);
        self.class_likes.get(&lowercase_name)
    }

    /// Retrieves metadata for a global function (case-insensitive).
    #[inline]
    #[must_use]
    pub fn get_function(&self, name: &[u8]) -> Option<&FunctionLikeMetadata> {
        let lowercase_name = ascii_lowercase_word(name);
        let identifier = (empty_word(), lowercase_name);
        self.function_likes.get(&identifier)
    }

    /// Retrieves metadata for a method (case-insensitive for both class and method names).
    #[inline]
    #[must_use]
    pub fn get_method(&self, class: &[u8], method: &[u8]) -> Option<&FunctionLikeMetadata> {
        let lowercase_class = ascii_lowercase_word(class);
        let lowercase_method = ascii_lowercase_word(method);
        let identifier = (lowercase_class, lowercase_method);
        self.function_likes.get(&identifier)
    }

    /// Retrieves metadata for a closure or arrow function by its synthetic
    /// name (e.g. `{closure:src/foo.php:12:5}`).
    #[inline]
    #[must_use]
    pub fn get_closure(&self, synthetic_name: &Word) -> Option<&FunctionLikeMetadata> {
        self.function_likes.get(&(empty_word(), *synthetic_name))
    }

    /// Retrieves metadata for a closure declared at the given file and span.
    /// Convenience wrapper that rebuilds the synthetic name and delegates to
    /// [`Self::get_closure`].
    #[inline]
    #[must_use]
    pub fn get_closure_at(&self, file: &File, span: Span) -> Option<&FunctionLikeMetadata> {
        let name = crate::build_synthetic_name("closure", file, span);
        self.get_closure(&name)
    }

    /// Retrieves method metadata by `MethodIdentifier`.
    #[inline]
    #[must_use]
    pub fn get_method_by_id(&self, method_id: &MethodIdentifier) -> Option<&FunctionLikeMetadata> {
        let lowercase_class = ascii_lowercase_word(method_id.get_class_name().as_bytes());
        let lowercase_method = ascii_lowercase_word(method_id.get_method_name().as_bytes());
        let identifier = (lowercase_class, lowercase_method);
        self.function_likes.get(&identifier)
    }

    /// Retrieves the declaring method metadata, following the inheritance chain.
    /// This finds where the method is actually implemented.
    #[inline]
    #[must_use]
    pub fn get_declaring_method(&self, class: &[u8], method: &[u8]) -> Option<&FunctionLikeMetadata> {
        let method_id = MethodIdentifier::new(word(class), word(method));
        let declaring_method_id = self.get_declaring_method_identifier(&method_id);
        self.get_method(
            declaring_method_id.get_class_name().as_bytes(),
            declaring_method_id.get_method_name().as_bytes(),
        )
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
            FunctionLikeIdentifier::Function(name) => self.get_function(name.as_bytes()),
            FunctionLikeIdentifier::Method(class, method) => self.get_method(class.as_bytes(), method.as_bytes()),
            FunctionLikeIdentifier::Closure(name) => self.get_closure(name),
        }
    }

    /// Retrieves metadata for a global constant.
    /// Namespace lookup is case-insensitive, constant name is case-sensitive.
    #[inline]
    #[must_use]
    pub fn get_constant(&self, name: &[u8]) -> Option<&ConstantMetadata> {
        let lowercase_name = ascii_lowercase_constant_name_word(name);
        self.constants.get(&lowercase_name)
    }

    /// Retrieves metadata for a class constant.
    /// Class name is case-insensitive, constant name is case-sensitive.
    #[inline]
    #[must_use]
    pub fn get_class_constant(&self, class: &[u8], constant: &[u8]) -> Option<&ClassLikeConstantMetadata> {
        let lowercase_class = ascii_lowercase_word(class);
        let constant_name = word(constant);
        self.class_likes.get(&lowercase_class).and_then(|meta| meta.constants.get(&constant_name))
    }

    /// Retrieves metadata for an enum case.
    #[inline]
    #[must_use]
    pub fn get_enum_case(&self, class: &[u8], case: &[u8]) -> Option<&EnumCaseMetadata> {
        let lowercase_class = ascii_lowercase_word(class);
        let case_name = word(case);
        self.class_likes.get(&lowercase_class).and_then(|meta| meta.enum_cases.get(&case_name))
    }

    /// Retrieves metadata for a property directly from the class where it's declared.
    /// Class name is case-insensitive, property name is case-sensitive.
    #[inline]
    #[must_use]
    pub fn get_property(&self, class: &[u8], property: &[u8]) -> Option<&PropertyMetadata> {
        let lowercase_class = ascii_lowercase_word(class);
        let property_name = word(property);
        self.class_likes.get(&lowercase_class)?.properties.get(&property_name)
    }

    /// Retrieves the property metadata, potentially from a parent class if inherited.
    #[inline]
    #[must_use]
    pub fn get_declaring_property(&self, class: &[u8], property: &[u8]) -> Option<&PropertyMetadata> {
        let lowercase_class = ascii_lowercase_word(class);
        let property_name = word(property);
        let declaring_class = self.class_likes.get(&lowercase_class)?.declaring_property_ids.get(&property_name)?;
        self.class_likes.get(declaring_class)?.properties.get(&property_name)
    }
    // Type Resolution

    /// Gets the type of a property, resolving it from the declaring class if needed.
    #[inline]
    #[must_use]
    pub fn get_property_type(&self, class: &[u8], property: &[u8]) -> Option<&TUnion> {
        let lowercase_class = ascii_lowercase_word(class);
        let property_name = word(property);
        let declaring_class = self.class_likes.get(&lowercase_class)?.declaring_property_ids.get(&property_name)?;
        let property_meta = self.class_likes.get(declaring_class)?.properties.get(&property_name)?;
        property_meta.type_metadata.as_ref().map(|tm| &tm.type_union)
    }

    /// Gets the type of a class constant, considering both type hints and inferred types.
    #[must_use]
    pub fn get_class_constant_type<'meta>(&'meta self, class: &[u8], constant: &[u8]) -> Option<Cow<'meta, TUnion>> {
        let lowercase_class = ascii_lowercase_word(class);
        let constant_name = word(constant);
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
    pub fn get_class_constant_literal_value(&self, class: &[u8], constant: &[u8]) -> Option<&TAtomic> {
        let lowercase_class = ascii_lowercase_word(class);
        let constant_name = word(constant);
        self.class_likes
            .get(&lowercase_class)
            .and_then(|meta| meta.constants.get(&constant_name))
            .and_then(|constant_meta| constant_meta.inferred_type.as_ref())
    }
    // Inheritance Queries

    /// Checks if a child class extends a parent class (case-insensitive).
    #[inline]
    #[must_use]
    pub fn class_extends(&self, child: &[u8], parent: &[u8]) -> bool {
        let lowercase_child = ascii_lowercase_word(child);
        let lowercase_parent = ascii_lowercase_word(parent);
        self.class_likes.get(&lowercase_child).is_some_and(|meta| meta.all_parent_classes.contains(&lowercase_parent))
    }

    /// Checks if a class directly extends a parent class (case-insensitive).
    #[inline]
    #[must_use]
    pub fn class_directly_extends(&self, child: &[u8], parent: &[u8]) -> bool {
        let lowercase_child = ascii_lowercase_word(child);
        let lowercase_parent = ascii_lowercase_word(parent);
        self.class_likes
            .get(&lowercase_child)
            .is_some_and(|meta| meta.direct_parent_class.as_ref() == Some(&lowercase_parent))
    }

    /// Checks if a class implements an interface (case-insensitive).
    #[inline]
    #[must_use]
    pub fn class_implements(&self, class: &[u8], interface: &[u8]) -> bool {
        let lowercase_class = ascii_lowercase_word(class);
        let lowercase_interface = ascii_lowercase_word(interface);
        self.class_likes
            .get(&lowercase_class)
            .is_some_and(|meta| meta.all_parent_interfaces.contains(&lowercase_interface))
    }

    /// Checks if a class directly implements an interface (case-insensitive).
    #[inline]
    #[must_use]
    pub fn class_directly_implements(&self, class: &[u8], interface: &[u8]) -> bool {
        let lowercase_class = ascii_lowercase_word(class);
        let lowercase_interface = ascii_lowercase_word(interface);
        self.class_likes
            .get(&lowercase_class)
            .is_some_and(|meta| meta.direct_parent_interfaces.contains(&lowercase_interface))
    }

    /// Checks if a class uses a trait (case-insensitive).
    #[inline]
    #[must_use]
    pub fn class_uses_trait(&self, class: &[u8], trait_name: &[u8]) -> bool {
        let lowercase_class = ascii_lowercase_word(class);
        let lowercase_trait = ascii_lowercase_word(trait_name);
        self.class_likes.get(&lowercase_class).is_some_and(|meta| meta.used_traits.contains(&lowercase_trait))
    }

    /// Checks if a trait has `@require-extends` for a class (case-insensitive).
    /// Returns true if the trait requires extending the specified class or any of its parents.
    #[inline]
    #[must_use]
    pub fn trait_requires_extends(&self, trait_name: &[u8], class_name: &[u8]) -> bool {
        let lowercase_trait = ascii_lowercase_word(trait_name);

        self.class_likes.get(&lowercase_trait).is_some_and(|meta| {
            meta.require_extends.iter().any(|required| self.is_instance_of(class_name, required.as_bytes()))
        })
    }

    /// Checks if child is an instance of parent (via extends or implements).
    #[inline]
    #[must_use]
    pub fn is_instance_of(&self, child: &[u8], parent: &[u8]) -> bool {
        if child == parent {
            return true;
        }

        let lowercase_child = ascii_lowercase_word(child);
        let lowercase_parent = ascii_lowercase_word(parent);

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
    pub fn is_enum_or_final_class(&self, name: &[u8]) -> bool {
        let lowercase_name = ascii_lowercase_word(name);
        self.class_likes.get(&lowercase_name).is_some_and(|meta| meta.kind.is_enum() || meta.flags.is_final())
    }

    /// Checks if a class-like can be part of an intersection.
    /// Generally, only final classes and enums cannot be intersected.
    #[inline]
    #[must_use]
    pub fn is_inheritable(&self, name: &[u8]) -> bool {
        let lowercase_name = ascii_lowercase_word(name);
        match self.symbols.get_kind(lowercase_name) {
            Some(SymbolKind::Class) => self.class_likes.get(&lowercase_name).is_some_and(|meta| !meta.flags.is_final()),
            Some(SymbolKind::Enum) => false,
            Some(SymbolKind::Interface | SymbolKind::Trait) | None => true,
        }
    }

    /// Gets all descendants of a class (recursive).
    #[inline]
    #[must_use]
    pub fn get_class_descendants(&self, class: &[u8]) -> WordSet {
        let lowercase_class = ascii_lowercase_word(class);
        let mut all_descendants = WordSet::default();
        let mut queue = vec![&lowercase_class];
        let mut visited = WordSet::default();
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
    pub fn get_class_ancestors(&self, class: &[u8]) -> WordSet {
        let lowercase_class = ascii_lowercase_word(class);
        let mut ancestors = WordSet::default();
        if let Some(meta) = self.class_likes.get(&lowercase_class) {
            ancestors.extend(meta.all_parent_classes.iter().copied());
            ancestors.extend(meta.all_parent_interfaces.iter().copied());
        }
        ancestors
    }

    /// Gets the class where a method is declared (following inheritance).
    #[inline]
    #[must_use]
    pub fn get_declaring_method_class(&self, class: &[u8], method: &[u8]) -> Option<Word> {
        let lowercase_class = ascii_lowercase_word(class);
        let lowercase_method = ascii_lowercase_word(method);

        self.class_likes
            .get(&lowercase_class)?
            .declaring_method_ids
            .get(&lowercase_method)
            .map(|method_id| method_id.get_class_name())
    }

    /// Gets the class where a method appears (could be the declaring class or child class).
    #[inline]
    #[must_use]
    pub fn get_appearing_method_class(&self, class: &[u8], method: &[u8]) -> Option<Word> {
        let lowercase_class = ascii_lowercase_word(class);
        let lowercase_method = ascii_lowercase_word(method);
        self.class_likes
            .get(&lowercase_class)?
            .appearing_method_ids
            .get(&lowercase_method)
            .map(|method_id| method_id.get_class_name())
    }

    /// Gets the declaring method identifier for a method.
    #[must_use]
    pub fn get_declaring_method_identifier(&self, method_id: &MethodIdentifier) -> MethodIdentifier {
        let lowercase_class = ascii_lowercase_word(method_id.get_class_name().as_bytes());
        let lowercase_method = ascii_lowercase_word(method_id.get_method_name().as_bytes());

        let Some(class_meta) = self.class_likes.get(&lowercase_class) else {
            return *method_id;
        };

        if let Some(declaring_method_id) = class_meta.declaring_method_ids.get(&lowercase_method) {
            return *declaring_method_id;
        }

        if class_meta.flags.is_abstract()
            && let Some(overridden_map) = class_meta.overridden_method_ids.get(&lowercase_method)
            && let Some((_, first_method_id)) = overridden_map.first()
        {
            return *first_method_id;
        }

        *method_id
    }

    /// Checks if a method is overriding a parent method.
    #[inline]
    #[must_use]
    pub fn method_is_overriding(&self, class: &[u8], method: &[u8]) -> bool {
        let lowercase_class = ascii_lowercase_word(class);
        let lowercase_method = ascii_lowercase_word(method);
        self.class_likes
            .get(&lowercase_class)
            .is_some_and(|meta| meta.overridden_method_ids.contains_key(&lowercase_method))
    }

    /// Checks if a method is abstract.
    #[inline]
    #[must_use]
    pub fn method_is_abstract(&self, class: &[u8], method: &[u8]) -> bool {
        let lowercase_class = ascii_lowercase_word(class);
        let lowercase_method = ascii_lowercase_word(method);
        let identifier = (lowercase_class, lowercase_method);
        self.function_likes
            .get(&identifier)
            .and_then(|meta| meta.method_metadata.as_ref())
            .is_some_and(|method_meta| method_meta.is_abstract)
    }

    /// Checks if a method is static.
    #[inline]
    #[must_use]
    pub fn method_is_static(&self, class: &[u8], method: &[u8]) -> bool {
        let lowercase_class = ascii_lowercase_word(class);
        let lowercase_method = ascii_lowercase_word(method);
        let identifier = (lowercase_class, lowercase_method);
        self.function_likes
            .get(&identifier)
            .and_then(|meta| meta.method_metadata.as_ref())
            .is_some_and(|method_meta| method_meta.is_static)
    }

    /// Checks if a method is final.
    #[inline]
    #[must_use]
    pub fn method_is_final(&self, class: &[u8], method: &[u8]) -> bool {
        let lowercase_class = ascii_lowercase_word(class);
        let lowercase_method = ascii_lowercase_word(method);
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
    pub fn get_method_visibility(&self, class: &[u8], method: &[u8]) -> Option<Visibility> {
        let lowercase_class = ascii_lowercase_word(class);
        let lowercase_method = ascii_lowercase_word(method);

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
    pub fn get_function_like_thrown_types<'meta>(
        &'meta self,
        class_like: Option<&'meta ClassLikeMetadata>,
        function_like: &'meta FunctionLikeMetadata,
    ) -> &'meta [TypeMetadata] {
        if !function_like.thrown_types.is_empty() {
            return function_like.thrown_types.as_slice();
        }

        if !function_like.kind.is_method() {
            return &[];
        }

        let Some(class_like) = class_like else {
            return &[];
        };

        let method_name = &function_like.name;

        if let Some(overridden_map) = class_like.overridden_method_ids.get(method_name) {
            for (parent_class_name, parent_method_id) in overridden_map {
                if class_like.name.as_bytes().eq_ignore_ascii_case(parent_class_name.as_bytes()) {
                    continue; // Skip self-recursion if the method overrides itself
                }

                let Some(parent_class) = self.class_likes.get(parent_class_name) else {
                    continue;
                };

                let parent_method_key = (parent_method_id.get_class_name(), parent_method_id.get_method_name());
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
    pub fn get_declaring_property_class(&self, class: &[u8], property: &[u8]) -> Option<Word> {
        let lowercase_class = ascii_lowercase_word(class);
        let property_name = word(property);
        self.class_likes.get(&lowercase_class)?.declaring_property_ids.get(&property_name).copied()
    }

    /// Gets the class where a property appears.
    #[inline]
    #[must_use]
    pub fn get_appearing_property_class(&self, class: &[u8], property: &[u8]) -> Option<Word> {
        let lowercase_class = ascii_lowercase_word(class);
        let property_name = word(property);
        self.class_likes.get(&lowercase_class)?.appearing_property_ids.get(&property_name).copied()
    }

    /// Gets all descendants of a class (recursive).
    #[must_use]
    pub fn get_all_descendants(&self, class: &[u8]) -> WordSet {
        let lowercase_class = ascii_lowercase_word(class);
        let mut all_descendants = WordSet::default();
        let mut queue = vec![&lowercase_class];
        let mut visited = WordSet::default();
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

    /// Generates the synthetic display name for an anonymous class based on
    /// its declaring file and span. Delegates to [`crate::get_anonymous_class_name`].
    #[must_use]
    pub fn get_anonymous_class_name(file: &File, span: Span) -> Word {
        crate::get_anonymous_class_name(file, span)
    }

    /// Retrieves the metadata for an anonymous class based on its declaring
    /// file and span.
    #[must_use]
    pub fn get_anonymous_class(&self, file: &File, span: Span) -> Option<&ClassLikeMetadata> {
        let name = Self::get_anonymous_class_name(file, span);
        if self.class_exists(name.as_bytes()) { self.class_likes.get(&name) } else { None }
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

    /// Marks safe symbols based on diff and invalidation cascade.
    ///
    /// After this function runs, `self.safe_symbols` and `self.safe_symbol_members`
    /// will contain all symbols that can be safely skipped during analysis.
    ///
    /// # Arguments
    ///
    /// * `diff` - The computed diff between old and new code
    /// * `references` - Symbol reference graph from previous run
    ///
    /// # Returns
    /// Returns `Some(global_scope_invalid)` on success, where `global_scope_invalid`
    /// is `true` when global-scope code (the `(empty, empty)` pseudo-symbol) references
    /// something that changed. Returns `None` if the cascade was too large to compute.
    pub fn mark_safe_symbols(&mut self, diff: &CodebaseDiff, references: &SymbolReferences) -> Option<bool> {
        let (invalid_symbols, partially_invalid) = references.get_invalid_symbols(diff)?;

        // Mark all symbols in 'keep' set as safe (unless invalidated by cascade)
        for keep_symbol in diff.get_keep() {
            if !invalid_symbols.contains(keep_symbol) {
                if keep_symbol.1.is_empty() {
                    // Top-level symbol (class, function, constant)
                    if !partially_invalid.contains(&keep_symbol.0) {
                        self.safe_symbols.insert(keep_symbol.0);
                    }
                } else {
                    // Member (method, property, class constant)
                    self.safe_symbol_members.insert(*keep_symbol);
                }
            }
        }

        Some(invalid_symbols.contains(&(empty_word(), empty_word())))
    }

    /// Merges information from another `CodebaseMetadata` into this one.
    ///
    /// When both metadata have the same priority, the one with the smaller span is kept
    /// for deterministic results regardless of scan order.
    pub fn extend(&mut self, other: CodebaseMetadata) {
        for (k, mut v) in other.class_likes {
            match self.class_likes.entry(k) {
                Entry::Occupied(mut entry) => {
                    if should_replace_metadata(entry.get().flags, entry.get().span, v.flags, v.span) {
                        v.version_constraint.merge(entry.get().version_constraint.clone());
                        entry.insert(v);
                    } else {
                        entry.get_mut().version_constraint.merge(v.version_constraint);
                    }
                }
                Entry::Vacant(entry) => {
                    entry.insert(v);
                }
            }
        }

        for (k, mut v) in other.function_likes {
            match self.function_likes.entry(k) {
                Entry::Occupied(mut entry) => {
                    if should_replace_metadata(entry.get().flags, entry.get().span, v.flags, v.span) {
                        v.version_constraint.merge(entry.get().version_constraint.clone());
                        entry.insert(v);
                    } else {
                        entry.get_mut().version_constraint.merge(v.version_constraint);
                    }
                }
                Entry::Vacant(entry) => {
                    entry.insert(v);
                }
            }
        }

        for (k, mut v) in other.constants {
            match self.constants.entry(k) {
                Entry::Occupied(mut entry) => {
                    if should_replace_metadata(entry.get().flags, entry.get().span, v.flags, v.span) {
                        v.version_constraint.merge(entry.get().version_constraint.clone());
                        entry.insert(v);
                    } else {
                        entry.get_mut().version_constraint.merge(v.version_constraint);
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

        self.file_signatures.extend(other.file_signatures);
        self.safe_symbols.extend(other.safe_symbols);
        self.safe_symbol_members.extend(other.safe_symbol_members);
        self.infer_types_from_usage |= other.infer_types_from_usage;
    }

    /// Extends this codebase with another by reference, cloning only individual entries.
    ///
    /// This is more efficient than `extend(other.clone())` because it avoids allocating
    /// a full clone of the source metadata's outer HashMap/WordMap structures. Only
    /// individual entries that need insertion are cloned.
    pub fn extend_ref(&mut self, other: &CodebaseMetadata) {
        for (k, v) in &other.class_likes {
            match self.class_likes.entry(*k) {
                Entry::Occupied(mut entry) => {
                    if should_replace_metadata(entry.get().flags, entry.get().span, v.flags, v.span) {
                        let mut new = v.clone();
                        new.version_constraint.merge(entry.get().version_constraint.clone());
                        entry.insert(new);
                    } else {
                        entry.get_mut().version_constraint.merge(v.version_constraint.clone());
                    }
                }
                Entry::Vacant(entry) => {
                    entry.insert(v.clone());
                }
            }
        }

        for (k, v) in &other.function_likes {
            match self.function_likes.entry(*k) {
                Entry::Occupied(mut entry) => {
                    if should_replace_metadata(entry.get().flags, entry.get().span, v.flags, v.span) {
                        let mut new = v.clone();
                        new.version_constraint.merge(entry.get().version_constraint.clone());
                        entry.insert(new);
                    } else {
                        entry.get_mut().version_constraint.merge(v.version_constraint.clone());
                    }
                }
                Entry::Vacant(entry) => {
                    entry.insert(v.clone());
                }
            }
        }

        for (k, v) in &other.constants {
            match self.constants.entry(*k) {
                Entry::Occupied(mut entry) => {
                    if should_replace_metadata(entry.get().flags, entry.get().span, v.flags, v.span) {
                        let mut new = v.clone();
                        new.version_constraint.merge(entry.get().version_constraint.clone());
                        entry.insert(new);
                    } else {
                        entry.get_mut().version_constraint.merge(v.version_constraint.clone());
                    }
                }
                Entry::Vacant(entry) => {
                    entry.insert(v.clone());
                }
            }
        }

        self.symbols.extend_ref(&other.symbols);

        for (k, v) in &other.all_class_like_descendants {
            self.all_class_like_descendants.entry(*k).or_default().extend(v.iter().copied());
        }

        for (k, v) in &other.direct_classlike_descendants {
            self.direct_classlike_descendants.entry(*k).or_default().extend(v.iter().copied());
        }

        for (k, v) in &other.file_signatures {
            self.file_signatures.insert(*k, v.clone());
        }
        self.safe_symbols.extend(other.safe_symbols.iter().copied());
        self.safe_symbol_members.extend(other.safe_symbol_members.iter().copied());
        self.infer_types_from_usage |= other.infer_types_from_usage;
    }

    /// Removes all entries that were contributed by the given per-file scan metadata.
    ///
    /// This is the inverse of [`extend_ref()`]: it removes class_likes, function_likes,
    /// constants, symbols, and file_signatures whose keys match those in `file_metadata`.
    ///
    /// Used by the incremental engine to patch the codebase in-place when files change,
    /// avoiding a full rebuild from base + all files.
    ///
    /// Note: This does NOT remove descendant map entries — those are rebuilt from scratch
    /// by `populate_codebase()` on every run.
    pub fn remove_entries_of(&mut self, file_metadata: &CodebaseMetadata) {
        for k in file_metadata.class_likes.keys() {
            self.class_likes.remove(k);
        }

        for k in file_metadata.function_likes.keys() {
            self.function_likes.remove(k);
        }

        for k in file_metadata.constants.keys() {
            self.constants.remove(k);
        }

        // Remove symbols that were contributed by this file.
        // We can only remove class-like symbols (not namespaces, since they may be shared).
        for k in file_metadata.class_likes.keys() {
            self.symbols.remove(*k);
        }

        for k in file_metadata.file_signatures.keys() {
            self.file_signatures.remove(k);
        }
    }

    /// Extracts the set of keys from this metadata for use with [`remove_entries_by_keys()`].
    ///
    /// This is much cheaper than keeping a full `CodebaseMetadata` clone — it only stores
    /// the keys needed to undo an `extend_ref()` operation.
    #[must_use]
    pub fn extract_keys(&self) -> CodebaseEntryKeys {
        CodebaseEntryKeys {
            class_like_names: self.class_likes.keys().copied().collect(),
            function_like_keys: self.function_likes.keys().copied().collect(),
            constant_names: self.constants.keys().copied().collect(),
            file_ids: self.file_signatures.keys().copied().collect(),
        }
    }

    /// Extracts only the keys that this per-file metadata currently "owns" in the given
    /// merged codebase; i.e. keys whose span in `merged` matches this metadata's span.
    ///
    /// This is what you want for incremental fingerprints. [`extract_keys`](Self::extract_keys)
    /// captures *every* key the scan produced, including ones that lost the tiebreak in
    /// [`extend`](Self::extend) / [`extend_ref`](Self::extend_ref) when another file defined
    /// the same FQN. Using `extract_keys` as a removal fingerprint then causes a nasty
    /// cross-file bug: touching file *B* can remove an entry that file *A* actually owns,
    /// because [`remove_entries_by_keys`](Self::remove_entries_by_keys) deletes by FQN
    /// without checking who the current owner is. The analyzer then reports a spurious
    /// "duplicate definition" when it walks *A* and finds *B*'s span in the codebase.
    ///
    /// By only recording the keys whose spans still match *this* metadata, removing the
    /// fingerprint later becomes a safe no-op when another file won the merge. The
    /// removal only drops the entries this file genuinely put into the merged codebase.
    #[must_use]
    pub fn extract_owned_keys(&self, merged: &CodebaseMetadata) -> CodebaseEntryKeys {
        let class_like_names = self
            .class_likes
            .iter()
            .filter(|(name, meta)| merged.class_likes.get(*name).is_some_and(|m| m.span == meta.span))
            .map(|(name, _)| *name)
            .collect();

        let function_like_keys = self
            .function_likes
            .iter()
            .filter(|(key, meta)| merged.function_likes.get(*key).is_some_and(|m| m.span == meta.span))
            .map(|(key, _)| *key)
            .collect();

        let constant_names = self
            .constants
            .iter()
            .filter(|(name, meta)| merged.constants.get(*name).is_some_and(|m| m.span == meta.span))
            .map(|(name, _)| *name)
            .collect();

        // A file signature is always owned by its file (there is at most one per file).
        let file_ids = self.file_signatures.keys().copied().collect();

        CodebaseEntryKeys { class_like_names, function_like_keys, constant_names, file_ids }
    }

    /// Removes entries whose keys match the given [`CodebaseEntryKeys`].
    ///
    /// This is the lightweight equivalent of [`remove_entries_of()`] — it performs the
    /// same removals but from a compact key set instead of a full `CodebaseMetadata` reference.
    pub fn remove_entries_by_keys(&mut self, keys: &CodebaseEntryKeys) {
        for k in &keys.class_like_names {
            self.class_likes.remove(k);
            self.symbols.remove(*k);
        }

        for k in &keys.function_like_keys {
            self.function_likes.remove(k);
        }

        for k in &keys.constant_names {
            self.constants.remove(k);
        }

        for k in &keys.file_ids {
            self.file_signatures.remove(k);
        }
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
            class_likes: WordMap::default(),
            function_likes: HashMap::default(),
            symbols: Symbols::new(),
            infer_types_from_usage: false,
            constants: WordMap::default(),
            all_class_like_descendants: WordMap::default(),
            direct_classlike_descendants: WordMap::default(),
            safe_symbols: WordSet::default(),
            safe_symbol_members: HashSet::default(),
            file_signatures: HashMap::default(),
        }
    }
}

/// Determines which metadata value to keep when merging duplicates.
///
/// Priority:
///   1. user-defined > built-in > other.
///   2. non-polyfill > polyfill — tools like rector/phpstan/psalm ship
///      skeleton stubs gated by `if (!class_exists('X'))` that should never
///      shadow a concrete definition.
///   3. smaller span wins as a deterministic tie-breaker.
///
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

    let new_is_polyfill = new_flags.is_polyfill();
    let existing_is_polyfill = existing_flags.is_polyfill();

    if new_is_polyfill != existing_is_polyfill {
        return !new_is_polyfill;
    }

    new_span < existing_span
}

#[cfg(test)]
mod should_replace_metadata_tests {
    use super::*;

    #[test]
    fn non_polyfill_replaces_polyfill() {
        let polyfill = MetadataFlags::POLYFILL;
        let real = MetadataFlags::empty();
        assert!(should_replace_metadata(polyfill, Span::dummy(0, 100), real, Span::dummy(0, 100)));
        assert!(!should_replace_metadata(real, Span::dummy(0, 100), polyfill, Span::dummy(0, 100)));
    }

    #[test]
    fn polyfill_does_not_replace_non_polyfill_even_with_smaller_span() {
        let real = MetadataFlags::empty();
        let polyfill = MetadataFlags::POLYFILL;
        assert!(!should_replace_metadata(real, Span::dummy(500, 600), polyfill, Span::dummy(0, 10)));
    }

    #[test]
    fn user_defined_beats_polyfill_flag() {
        let polyfill_user = MetadataFlags::POLYFILL | MetadataFlags::USER_DEFINED;
        let plain = MetadataFlags::empty();
        assert!(!should_replace_metadata(polyfill_user, Span::dummy(0, 10), plain, Span::dummy(0, 10)));
        assert!(should_replace_metadata(plain, Span::dummy(0, 10), polyfill_user, Span::dummy(0, 10)));
    }

    #[test]
    fn two_user_defined_fall_through_to_polyfill_check() {
        let a = MetadataFlags::POLYFILL | MetadataFlags::USER_DEFINED;
        let b = MetadataFlags::USER_DEFINED;
        assert!(should_replace_metadata(a, Span::dummy(0, 10), b, Span::dummy(0, 10)));
        assert!(!should_replace_metadata(b, Span::dummy(0, 10), a, Span::dummy(0, 10)));
    }

    #[test]
    fn two_non_polyfills_fall_through_to_priority_rules() {
        let user = MetadataFlags::USER_DEFINED;
        let builtin = MetadataFlags::BUILTIN;
        assert!(!should_replace_metadata(user, Span::dummy(0, 10), builtin, Span::dummy(0, 10)));
        assert!(should_replace_metadata(builtin, Span::dummy(0, 10), user, Span::dummy(0, 10)));
    }
}
