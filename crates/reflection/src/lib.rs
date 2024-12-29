use std::collections::hash_map::Entry;

use ahash::HashMap;
use ahash::HashSet;
use mago_reporting::IssueCollection;
use serde::Deserialize;
use serde::Serialize;

use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use mago_source::HasSource;
use mago_source::SourceCategory;
use mago_span::HasPosition;
use mago_span::HasSpan;

use crate::class_like::ClassLikeReflection;
use crate::constant::ConstantReflection;
use crate::function_like::FunctionLikeReflection;
use crate::identifier::ClassLikeName;
use crate::identifier::FunctionLikeName;
use crate::identifier::Name;

pub mod assertion;
pub mod attribute;
pub mod class_like;
pub mod constant;
pub mod function_like;
pub mod identifier;
pub mod r#type;

/// The `Reflection` trait is implemented by all reflection types in the system.
///
/// It provides a consistent interface for querying metadata about PHP constructs
/// such as classes, functions, and other entities. This trait allows the system
/// to introspect and categorize these constructs based on their origin, source,
/// and other attributes.
pub trait Reflection: HasSpan + HasSource {
    /// Retrieves the `SourceCategory` for the entity.
    ///
    /// The `SourceCategory` indicates whether the entity belongs to one of the following:
    ///
    /// - `BuiltIn`: A PHP construct that is part of the PHP core or standard library.
    /// - `External`: A construct defined in third-party or vendor-provided libraries.
    /// - `UserDefined`: A construct written by the user or part of the current project.
    ///
    /// # Returns
    /// - A `SourceCategory` enum variant corresponding to the entity's origin.
    fn get_category(&self) -> SourceCategory;

    /// Indicates whether the entity is user-defined or part of the current project.
    ///
    /// # Returns
    ///
    /// - `true` if the entity's `SourceCategory` is `UserDefined`.
    /// - `false` otherwise.
    fn is_user_defined(&self) -> bool {
        self.get_category() == SourceCategory::UserDefined
    }

    /// Indicates whether the entity originates from an external source (e.g., vendor libraries).
    ///
    /// # Returns
    ///
    /// - `true` if the entity's `SourceCategory` is `Vendor` or similar external categories.
    /// - `false` otherwise.
    fn is_external(&self) -> bool {
        self.get_category() == SourceCategory::External
    }

    /// Indicates whether the entity is a built-in PHP construct.
    ///
    /// Built-in constructs include classes, functions, and constants that are
    /// part of the PHP core or extensions.
    ///
    /// # Returns
    ///
    /// - `true` if the entity's `SourceCategory` is `BuiltIn`.
    /// - `false` otherwise.
    #[inline(always)]
    fn is_builtin(&self) -> bool {
        self.get_category() == SourceCategory::BuiltIn
    }

    /// Indicates whether the entity has been fully populated with metadata.
    ///
    /// This can be useful to determine whether lazy-loaded or partially
    /// processed entities have had their information fully resolved.
    ///
    /// # Returns
    ///
    /// - `true` if the entity's metadata is fully populated.
    /// - `false` if additional processing is needed to populate the metadata.
    fn is_populated(&self) -> bool;

    /// Retrieves any issues found during the population of the reflection.
    ///
    /// The returned `IssueCollection` contains errors, warnings, or notices
    /// related to the metadata of the entity.
    ///
    /// This method is particularly useful for static analysis tools or compilers
    /// to report potential problems in the code being analyzed.
    ///
    /// # Returns
    ///
    /// - A reference to an `IssueCollection` containing all detected issues.
    fn get_issues(&self) -> &IssueCollection;
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct CodebaseReflection {
    /// Reflections for all constants in the codebase, keyed by their `Name`.
    pub constant_reflections: HashMap<Name, ConstantReflection>,

    /// Mapping of constant names to their canonical `Name` representations.
    pub constant_names: HashMap<StringIdentifier, Name>,

    /// Reflections for all function-like entities (functions, closures, etc.), keyed by their `FunctionLikeName`.
    pub function_like_reflections: HashMap<FunctionLikeName, FunctionLikeReflection>,

    /// Mapping of function-like names to their canonical `FunctionLikeName` representations.
    pub function_names: HashMap<StringIdentifier, FunctionLikeName>,

    /// Reflections for all class-like entities (classes, traits, enums, interfaces), keyed by their `ClassLikeName`.
    pub class_like_reflections: HashMap<ClassLikeName, ClassLikeReflection>,

    /// Mapping of class-like names to their canonical `ClassLikeName` representations.
    pub class_like_names: HashMap<StringIdentifier, ClassLikeName>,

    /// Direct descendants of each class-like entity, useful for hierarchy traversal.
    pub direct_classlike_descendants: HashMap<StringIdentifier, HashSet<StringIdentifier>>,

    /// All descendants of each class-like entity, useful for comprehensive hierarchy analysis.
    pub all_classlike_descendants: HashMap<StringIdentifier, HashSet<StringIdentifier>>,

    /// Indicates whether all entities in the codebase have been fully populated.
    pub populated: bool,
}

impl CodebaseReflection {
    /// Creates a new, empty `CodebaseReflection`.
    ///
    /// # Returns
    ///
    /// A new instance of `CodebaseReflection` with `populated` set to `false`
    /// and all internal collections initialized to their default states.
    pub fn new() -> Self {
        Self { populated: false, ..Default::default() }
    }

    /// Registers a new constant in the codebase.
    ///
    /// This method ensures that the constant is uniquely registered,
    /// accounting for case-insensitive names. If a constant with the same name
    /// already exists, it will not be registered again.
    ///
    /// # Arguments
    ///
    /// - `interner`: A `ThreadedInterner` instance for name handling.
    /// - `reflection`: The `ConstantReflection` to register.
    ///
    /// # Returns
    ///
    /// - `true` if the constant was successfully registered.
    /// - `false` if the constant already exists.
    pub fn register_constant(&mut self, interner: &ThreadedInterner, reflection: ConstantReflection) -> bool {
        let lowercase_name = lower_constant_name(interner, &reflection.name.value);
        if self.constant_names.contains_key(&lowercase_name) {
            return false;
        }

        self.constant_names.insert(lowercase_name, reflection.name);
        self.constant_reflections.insert(reflection.name, reflection);

        true
    }

    /// Registers a new function-like entity (e.g., function, closure, or arrow function) in the codebase.
    ///
    /// This method ensures that the function-like entity is uniquely registered,
    /// accounting for case-insensitive names. If an entity with the same name already
    /// exists, it will not be registered again.
    ///
    /// # Arguments
    ///
    /// - `interner`: A `ThreadedInterner` instance for name handling.
    /// - `reflection`: The `FunctionLikeReflection` to register.
    ///
    /// # Returns
    ///
    /// - `true` if the entity was successfully registered.
    /// - `false` if the entity already exists.
    pub fn register_function_like(&mut self, interner: &ThreadedInterner, reflection: FunctionLikeReflection) -> bool {
        let mut exists = false;

        if let FunctionLikeName::Function(name) = reflection.name {
            let lowercase_name = interner.lowered(&name.value);
            if let Entry::Vacant(e) = self.function_names.entry(lowercase_name) {
                e.insert(reflection.name);
            } else {
                exists = true;
            }
        }

        if !exists {
            self.function_like_reflections.insert(reflection.name, reflection);
        }

        exists
    }

    /// Registers a new class-like entity (e.g., class, interface, trait, or enum) in the codebase.
    ///
    /// This method ensures that the class-like entity is uniquely registered,
    /// accounting for case-insensitive names. If an entity with the same name
    /// already exists, it will not be registered again.
    ///
    /// # Arguments
    ///
    /// - `interner`: A `ThreadedInterner` instance for name handling.
    /// - `reflection`: The `ClassLikeReflection` to register.
    ///
    /// # Returns
    ///
    /// - `true` if the entity was successfully registered.
    /// - `false` if the entity already exists.
    pub fn register_class_like(&mut self, interner: &ThreadedInterner, reflection: ClassLikeReflection) -> bool {
        let mut exists = false;

        match reflection.name {
            ClassLikeName::Class(name) => {
                let lowercase_name = interner.lowered(&name.value);

                if let Entry::Vacant(e) = self.class_like_names.entry(lowercase_name) {
                    e.insert(reflection.name);
                } else {
                    exists = true;
                }
            }
            ClassLikeName::Enum(name) => {
                let lowercase_name = interner.lowered(&name.value);
                if let Entry::Vacant(e) = self.class_like_names.entry(lowercase_name) {
                    e.insert(reflection.name);
                } else {
                    exists = true;
                }
            }
            ClassLikeName::Interface(name) => {
                let lowercase_name = interner.lowered(&name.value);

                if let Entry::Vacant(e) = self.class_like_names.entry(lowercase_name) {
                    e.insert(reflection.name);
                } else {
                    exists = true;
                }
            }
            ClassLikeName::Trait(name) => {
                let lowercase_name = interner.lowered(&name.value);

                if let Entry::Vacant(e) = self.class_like_names.entry(lowercase_name) {
                    e.insert(reflection.name);
                } else {
                    exists = true;
                }
            }
            _ => {}
        }

        if !exists {
            self.class_like_reflections.insert(reflection.name, reflection);
        }

        exists
    }

    /// Checks if a constant exists in the codebase.
    ///
    /// # Arguments
    ///
    /// - `interner`: A `ThreadedInterner` instance for name handling.
    /// - `id`: A `StringIdentifier` representing the constant's name.
    ///
    /// # Returns
    ///
    /// - `true` if the constant exists.
    /// - `false` otherwise.
    pub fn constant_exists(&self, interner: &ThreadedInterner, id: &StringIdentifier) -> bool {
        let id = lower_constant_name(interner, id);

        self.constant_names.contains_key(&id)
    }

    /// Checks if a function exists in the codebase.
    ///
    /// # Arguments
    ///
    /// - `interner`: A `ThreadedInterner` instance for name handling.
    /// - `id`: A `StringIdentifier` representing the function's name.
    ///
    /// # Returns
    ///
    /// - `true` if the function exists.
    /// - `false` otherwise.
    pub fn function_exists(&self, interner: &ThreadedInterner, id: &StringIdentifier) -> bool {
        let id = interner.lowered(id);

        self.function_names.contains_key(&id)
    }

    /// Checks if a class exists in the codebase.
    ///
    /// # Arguments
    ///
    /// - `interner`: A `ThreadedInterner` instance for name handling.
    /// - `id`: A `StringIdentifier` representing the class's name.
    ///
    /// # Returns
    ///
    /// - `true` if the class exists.
    /// - `false` otherwise.
    pub fn class_exists(&self, interner: &ThreadedInterner, id: &StringIdentifier) -> bool {
        let id = interner.lowered(id);

        matches!(self.class_like_names.get(&id), Some(ClassLikeName::Class(_)))
    }

    /// Checks if an enum exists in the codebase.
    ///
    /// # Arguments
    ///
    /// - `interner`: A `ThreadedInterner` instance for name handling.
    /// - `id`: A `StringIdentifier` representing the enum's name.
    ///
    /// # Returns
    ///
    /// - `true` if the enum exists.
    /// - `false` otherwise.
    pub fn enum_exists(&self, interner: &ThreadedInterner, id: &StringIdentifier) -> bool {
        let id = interner.lowered(id);

        matches!(self.class_like_names.get(&id), Some(ClassLikeName::Enum(_)))
    }

    /// Checks if an interface exists in the codebase.
    ///
    /// # Arguments
    ///
    /// - `interner`: A `ThreadedInterner` instance for name handling.
    /// - `id`: A `StringIdentifier` representing the interface's name.
    ///
    /// # Returns
    ///
    /// - `true` if the interface exists.
    /// - `false` otherwise.
    pub fn interface_exists(&self, interner: &ThreadedInterner, id: &StringIdentifier) -> bool {
        let name = interner.lowered(id);

        matches!(self.class_like_names.get(&name), Some(ClassLikeName::Interface(_)))
    }

    /// Checks if a trait exists in the codebase.
    ///
    /// # Arguments
    ///
    /// - `interner`: A `ThreadedInterner` instance for name handling.
    /// - `id`: A `StringIdentifier` representing the trait's name.
    ///
    /// # Returns
    ///
    /// - `true` if the trait exists.
    /// - `false` otherwise.
    pub fn trait_exists(&self, interner: &ThreadedInterner, id: &StringIdentifier) -> bool {
        let id = interner.lowered(id);

        matches!(self.class_like_names.get(&id), Some(ClassLikeName::Trait(_)))
    }

    /// Retrieves a constant reflection by name, if it exists.
    ///
    /// # Arguments
    ///
    /// - `interner`: A `ThreadedInterner` instance for name handling.
    /// - `id`: A `StringIdentifier` representing the constant's name.
    ///
    /// # Returns
    ///
    /// - `Some(&ConstantReflection)` if the constant exists.
    /// - `None` otherwise.
    pub fn get_constant(&self, interner: &ThreadedInterner, id: &StringIdentifier) -> Option<&ConstantReflection> {
        let id = lower_constant_name(interner, id);

        if let Some(name) = self.constant_names.get(&id) {
            self.constant_reflections.get(name)
        } else {
            None
        }
    }

    /// Retrieves a function-like reflection by its name, if it exists.
    ///
    /// # Arguments
    ///
    /// - `name`: The name of the function-like entity as a `FunctionLikeName`.
    ///
    /// # Returns
    ///
    /// - `Some(&FunctionLikeReflection)` if the function-like entity exists.
    /// - `None` otherwise.
    pub fn get_function_like(&self, name: FunctionLikeName) -> Option<&FunctionLikeReflection> {
        self.function_like_reflections.get(&name)
    }

    /// Retrieves a function reflection by name, if it exists.
    ///
    /// # Arguments
    ///
    /// - `interner`: A `ThreadedInterner` instance for name handling.
    /// - `id`: A `StringIdentifier` representing the function's name.
    ///
    /// # Returns
    ///
    /// - `Some(&FunctionLikeReflection)` if the function exists.
    /// - `None` otherwise.
    pub fn get_function(&self, interner: &ThreadedInterner, id: &StringIdentifier) -> Option<&FunctionLikeReflection> {
        let id = interner.lowered(id);

        if let Some(name) = self.function_names.get(&id) {
            self.function_like_reflections.get(name)
        } else {
            None
        }
    }

    /// Retrieves a closure reflection by its position, if it exists.
    ///
    /// # Arguments
    ///
    /// - `position`: The position to search for as an implementation of `HasPosition`.
    ///
    /// # Returns
    ///
    /// - `Some(&FunctionLikeReflection)` if the closure exists at the given position.
    /// - `None` otherwise.
    pub fn get_closure(&self, position: &impl HasPosition) -> Option<&FunctionLikeReflection> {
        self.function_like_reflections.iter().find_map(|(identifier, function_like)| match identifier {
            FunctionLikeName::Closure(span) => {
                if span.contains(position) {
                    Some(function_like)
                } else {
                    None
                }
            }
            _ => None,
        })
    }

    /// Retrieves an arrow function reflection by its position, if it exists.
    ///
    /// # Arguments
    ///
    /// - `position`: The position to search for as an implementation of `HasPosition`.
    ///
    /// # Returns
    ///
    /// - `Some(&FunctionLikeReflection)` if the arrow function exists at the given position.
    /// - `None` otherwise.
    pub fn get_arrow_function(&self, position: &impl HasPosition) -> Option<&FunctionLikeReflection> {
        self.function_like_reflections.iter().find_map(|(identifier, function_like)| match identifier {
            FunctionLikeName::ArrowFunction(span) => {
                if span.contains(position) {
                    Some(function_like)
                } else {
                    None
                }
            }
            _ => None,
        })
    }

    /// Retrieves a class-like reflection by its identifier, if it exists.
    ///
    /// # Arguments
    ///
    /// - `name`: The `ClassLikeName` representing the class-like entity.
    ///
    /// # Returns
    ///
    /// - `Some(&ClassLikeReflection)` if the class-like entity exists.
    /// - `None` otherwise.
    pub fn get_class_like(&self, name: ClassLikeName) -> Option<&ClassLikeReflection> {
        self.class_like_reflections.get(&name)
    }

    /// Retrieves a class-like reflection by its name, if it exists.
    ///
    /// # Arguments
    ///
    /// - `interner`: A `ThreadedInterner` instance for name handling.
    /// - `id`: A `StringIdentifier` representing the class-like entity's name.
    ///
    /// # Returns
    ///
    /// - `Some(&ClassLikeReflection)` if the class-like entity exists.
    /// - `None` otherwise.
    pub fn get_named_class_like(
        &self,
        interner: &ThreadedInterner,
        id: &StringIdentifier,
    ) -> Option<&ClassLikeReflection> {
        let id = interner.lowered(id);

        if let Some(name) = self.class_like_names.get(&id) {
            self.class_like_reflections.get(name)
        } else {
            None
        }
    }

    /// Retrieves a class reflection by its name, if it exists.
    ///
    /// # Arguments
    ///
    /// - `interner`: A `ThreadedInterner` instance for name handling.
    /// - `id`: A `StringIdentifier` representing the class's name.
    ///
    /// # Returns
    ///
    /// - `Some(&ClassLikeReflection)` if the class exists.
    /// - `None` otherwise.
    pub fn get_class(&self, interner: &ThreadedInterner, id: &StringIdentifier) -> Option<&ClassLikeReflection> {
        let id = interner.lowered(id);

        if let Some(name @ ClassLikeName::Class(_)) = self.class_like_names.get(&id) {
            self.class_like_reflections.get(name)
        } else {
            None
        }
    }

    /// Retrieves an enum reflection by its name, if it exists.
    ///
    /// # Arguments
    ///
    /// - `interner`: A `ThreadedInterner` instance for name handling.
    /// - `id`: A `StringIdentifier` representing the enum's name.
    ///
    /// # Returns
    ///
    /// - `Some(&ClassLikeReflection)` if the enum exists.
    /// - `None` otherwise.
    pub fn get_enum(&self, interner: &ThreadedInterner, id: &StringIdentifier) -> Option<&ClassLikeReflection> {
        let id = interner.lowered(id);

        if let Some(name @ ClassLikeName::Enum(_)) = self.class_like_names.get(&id) {
            self.class_like_reflections.get(name)
        } else {
            None
        }
    }

    /// Retrieves an interface reflection by its name, if it exists.
    ///
    /// # Arguments
    ///
    /// - `interner`: A `ThreadedInterner` instance for name handling.
    /// - `id`: A `StringIdentifier` representing the interface's name.
    ///
    /// # Returns
    ///
    /// - `Some(&ClassLikeReflection)` if the interface exists.
    /// - `None` otherwise.
    pub fn get_interface(&self, interner: &ThreadedInterner, id: &StringIdentifier) -> Option<&ClassLikeReflection> {
        let id = interner.lowered(id);

        if let Some(name @ ClassLikeName::Interface(_)) = self.class_like_names.get(&id) {
            self.class_like_reflections.get(name)
        } else {
            None
        }
    }

    /// Retrieves a trait reflection by its name, if it exists.
    ///
    /// # Arguments
    ///
    /// - `interner`: A `ThreadedInterner` instance for name handling.
    /// - `id`: A `StringIdentifier` representing the trait's name.
    ///
    /// # Returns
    ///
    /// - `Some(&ClassLikeReflection)` if the trait exists.
    /// - `None` otherwise.
    pub fn get_trait(&self, interner: &ThreadedInterner, id: &StringIdentifier) -> Option<&ClassLikeReflection> {
        let id = interner.lowered(id);

        if let Some(name @ ClassLikeName::Trait(_)) = self.class_like_names.get(&id) {
            self.class_like_reflections.get(name)
        } else {
            None
        }
    }

    /// Retrieves an anonymous class reflection by its span, if it exists.
    ///
    /// # Arguments
    ///
    /// - `node`: The node containing the span as an implementation of `HasSpan`.
    ///
    /// # Returns
    ///
    /// - `Some(&ClassLikeReflection)` if the anonymous class exists.
    /// - `None` otherwise.
    pub fn get_anonymous_class(&self, node: &impl HasSpan) -> Option<&ClassLikeReflection> {
        self.class_like_reflections.get(&ClassLikeName::AnonymousClass(node.span()))
    }

    /// Returns the function-like reflection (function, closure, etc.) that encloses the given offset.
    ///
    /// This method iterates through the reflections in the codebase, filtering for function-like reflections
    /// that contain the given offset in their definition range. It returns the reflection with the
    /// largest starting offset, effectively finding the innermost function-like reflection containing
    /// the offset.
    ///
    /// # Arguments
    ///
    /// * `has_position` - The position to search for.
    ///
    /// # Returns
    ///
    /// * `Option<&FunctionLikeReflection>` - The enclosing function-like reflection, if found.
    pub fn get_enclosing_function_like(&self, has_position: &impl HasPosition) -> Option<&FunctionLikeReflection> {
        self.function_like_reflections
            .iter()
            .filter(|(_, function_like)| function_like.span.has_offset(has_position.offset()))
            .max_by_key(|(_, function_like)| function_like.span.start.offset)
            .map(|(_, function_like)| function_like)
    }

    /// Returns the class-like reflection (class, trait, etc.) that encloses the given offset.
    ///
    /// This method iterates through the reflections in the codebase, filtering for class-like reflections
    /// that contain the given offset in their definition range. It returns the reflection with the
    /// largest starting offset, effectively finding the innermost class-like reflection containing
    /// the offset.
    ///
    /// # Arguments
    ///
    /// * `has_position` - The position to search for.
    ///
    /// # Returns
    ///
    /// * `Option<&ClassLikeReflection>` - The enclosing class-like reflection, if found.
    pub fn get_enclosing_class_like(&self, has_position: &impl HasPosition) -> Option<&ClassLikeReflection> {
        self.class_like_reflections
            .iter()
            .filter(|(_, class_like)| class_like.span.has_offset(has_position.offset()))
            .max_by_key(|(_, class_like)| class_like.span.start.offset)
            .map(|(_, class_like)| class_like)
    }
}

fn lower_constant_name(interner: &ThreadedInterner, name: &StringIdentifier) -> StringIdentifier {
    let name = interner.lookup(name);

    let mut parts: Vec<_> = name.split('\\').map(str::to_owned).collect();
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
