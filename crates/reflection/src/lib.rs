use std::collections::hash_map::Entry;

use ahash::HashMap;
use ahash::HashSet;
use serde::Deserialize;
use serde::Serialize;

use mago_interner::StringIdentifier;
use mago_span::HasPosition;

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

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct CodebaseReflection {
    pub constant_reflections: HashMap<Name, ConstantReflection>,
    pub constant_identifiers: HashMap<StringIdentifier, Name>,

    pub function_like_reflections: HashMap<FunctionLikeName, FunctionLikeReflection>,
    pub function_identifiers: HashMap<StringIdentifier, FunctionLikeName>,

    pub class_like_reflections: HashMap<ClassLikeName, ClassLikeReflection>,
    pub class_like_names: HashMap<StringIdentifier, ClassLikeName>,

    pub direct_classlike_descendants: HashMap<StringIdentifier, HashSet<StringIdentifier>>,
    pub all_classlike_descendants: HashMap<StringIdentifier, HashSet<StringIdentifier>>,

    pub populated: bool,
}

impl CodebaseReflection {
    /// Creates a new, empty `CodebaseReflection`.
    pub fn new() -> Self {
        Self { populated: false, ..Default::default() }
    }

    /// Registers a new constant in the codebase.
    ///
    /// If the constant already exists, it will not be added again.
    ///
    /// Returns `false` if the constant already exists.
    pub fn register_constant(&mut self, constant: ConstantReflection) -> bool {
        if self.constant_reflections.contains_key(&constant.name) {
            return false;
        }

        self.constant_identifiers.insert(constant.name.value, constant.name);
        self.constant_reflections.insert(constant.name, constant);

        true
    }

    /// Registers a new function-like entity in the codebase.
    ///
    /// If the function-like entity already exists, it will not be added again.
    ///
    /// Returns `false` if the function-like entity already exists.
    pub fn register_function_like(&mut self, function_like: FunctionLikeReflection) -> bool {
        let mut exists = false;

        if let FunctionLikeName::Function(id) = function_like.name {
            if let Entry::Vacant(e) = self.function_identifiers.entry(id.lower) {
                e.insert(function_like.name);
            } else {
                exists = true;
            }
        }

        if !exists {
            self.function_like_reflections.insert(function_like.name, function_like);
        }

        exists
    }

    /// Registers a new class-like entity (class, enum, interface, or trait) in the codebase.
    ///
    /// If the class-like entity already exists, it will not be added again.
    ///
    /// Returns `false` if the class-like entity already exists.
    pub fn register_class_like(&mut self, class_like: ClassLikeReflection) -> bool {
        let mut exists = false;

        match class_like.name {
            ClassLikeName::Class(identifier) => {
                if let Entry::Vacant(e) = self.class_like_names.entry(identifier.lower) {
                    e.insert(class_like.name);
                } else {
                    exists = true;
                }
            }
            ClassLikeName::Enum(identifier) => {
                if let Entry::Vacant(e) = self.class_like_names.entry(identifier.lower) {
                    e.insert(class_like.name);
                } else {
                    exists = true;
                }
            }
            ClassLikeName::Interface(identifier) => {
                if let Entry::Vacant(e) = self.class_like_names.entry(identifier.lower) {
                    e.insert(class_like.name);
                } else {
                    exists = true;
                }
            }
            ClassLikeName::Trait(identifier) => {
                if let Entry::Vacant(e) = self.class_like_names.entry(identifier.lower) {
                    e.insert(class_like.name);
                } else {
                    exists = true;
                }
            }
            _ => {}
        }

        if !exists {
            self.class_like_reflections.insert(class_like.name, class_like);
        }

        exists
    }

    /// Checks if a constant with the given name exists.
    pub fn constant_exists(&self, name: &StringIdentifier) -> bool {
        self.constant_identifiers.contains_key(name)
    }

    /// Checks if a function with the given name exists.
    pub fn function_exists(&self, name: &StringIdentifier) -> bool {
        self.function_identifiers.contains_key(name)
    }

    /// Checks if a class with the given name exists.
    pub fn class_exists(&self, name: &StringIdentifier) -> bool {
        matches!(self.class_like_names.get(name), Some(ClassLikeName::Class(_)))
    }

    /// Checks if an enum with the given name exists.
    pub fn enum_exists(&self, name: &StringIdentifier) -> bool {
        matches!(self.class_like_names.get(name), Some(ClassLikeName::Enum(_)))
    }

    /// Checks if an interface with the given name exists.
    pub fn interface_exists(&self, name: &StringIdentifier) -> bool {
        matches!(self.class_like_names.get(name), Some(ClassLikeName::Interface(_)))
    }

    /// Checks if a trait with the given name exists.
    pub fn trait_exists(&self, name: &StringIdentifier) -> bool {
        matches!(self.class_like_names.get(name), Some(ClassLikeName::Trait(_)))
    }

    /// Retrieves a constant by name, if it exists.
    pub fn get_constant(&self, name: &StringIdentifier) -> Option<&ConstantReflection> {
        if let Some(identifier) = self.constant_identifiers.get(name) {
            self.constant_reflections.get(identifier)
        } else {
            None
        }
    }

    /// Retrieves a function-like by its identifier, if it exists.
    pub fn get_function_like(&self, identifier: FunctionLikeName) -> Option<&FunctionLikeReflection> {
        self.function_like_reflections.get(&identifier)
    }

    /// Retrieves a function by name, if it exists.
    pub fn get_function(&self, identifier: &StringIdentifier) -> Option<&FunctionLikeReflection> {
        if let Some(name) = self.function_identifiers.get(identifier) {
            self.function_like_reflections.get(name)
        } else {
            None
        }
    }

    /// Retrieves a closure by its position, if it exists.
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

    /// Retrieves an arrow function by its position, if it exists.
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

    /// Retrieves a class-like entity by its identifier, if it exists.
    pub fn get_class_like(&self, identifier: ClassLikeName) -> Option<&ClassLikeReflection> {
        self.class_like_reflections.get(&identifier)
    }

    /// Retrieves a class-like entity by its name, if it exists.
    pub fn get_named_class_like(&self, name: &StringIdentifier) -> Option<&ClassLikeReflection> {
        if let Some(identifier) = self.class_like_names.get(name) {
            self.class_like_reflections.get(identifier)
        } else {
            None
        }
    }

    /// Retrieves a class by name, if it exists.
    pub fn get_class(&self, name: &StringIdentifier) -> Option<&ClassLikeReflection> {
        if let Some(identifier @ ClassLikeName::Class(_)) = self.class_like_names.get(name) {
            self.class_like_reflections.get(identifier)
        } else {
            None
        }
    }

    /// Retrieves an enum by name, if it exists.
    pub fn get_enum(&self, name: &StringIdentifier) -> Option<&ClassLikeReflection> {
        if let Some(identifier @ ClassLikeName::Enum(_)) = self.class_like_names.get(name) {
            self.class_like_reflections.get(identifier)
        } else {
            None
        }
    }

    /// Retrieves an interface by name, if it exists.
    pub fn get_interface(&self, name: &StringIdentifier) -> Option<&ClassLikeReflection> {
        if let Some(identifier @ ClassLikeName::Interface(_)) = self.class_like_names.get(name) {
            self.class_like_reflections.get(identifier)
        } else {
            None
        }
    }

    /// Retrieves a trait by name, if it exists.
    pub fn get_trait(&self, name: &StringIdentifier) -> Option<&ClassLikeReflection> {
        if let Some(identifier @ ClassLikeName::Trait(_)) = self.class_like_names.get(name) {
            self.class_like_reflections.get(identifier)
        } else {
            None
        }
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
