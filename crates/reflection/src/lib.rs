use std::collections::HashSet;

use fennec_interner::StringIdentifier;
use fennec_span::HasPosition;
use identifier::ClassLikeIdentifier;
use serde::Deserialize;
use serde::Serialize;

use crate::class_like::ClassLikeReflection;
use crate::constant::ConstantReflection;
use crate::function_like::FunctionLikeReflection;
use crate::identifier::FunctionLikeIdentifier;

pub mod attribute;
pub mod class_like;
pub mod constant;
pub mod function_like;
pub mod identifier;
pub mod r#type;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct CodebaseReflection {
    pub constant_reflections: Vec<ConstantReflection>,
    pub function_like_reflections: Vec<FunctionLikeReflection>,
    pub class_like_reflections: Vec<ClassLikeReflection>,
    pub constant_names: HashSet<StringIdentifier>,
    pub function_names: HashSet<StringIdentifier>,
    pub class_names: HashSet<StringIdentifier>,
    pub enum_names: HashSet<StringIdentifier>,
    pub interface_names: HashSet<StringIdentifier>,
    pub trait_names: HashSet<StringIdentifier>,
}

impl CodebaseReflection {
    /// Creates a new, empty `CodebaseReflection`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new constant in the codebase.
    pub fn register_constant(&mut self, constant: ConstantReflection) {
        self.constant_names.insert(constant.identifier.name);
        self.constant_reflections.push(constant);
    }

    /// Registers a new function-like entity in the codebase.
    pub fn register_function_like(&mut self, function_like: FunctionLikeReflection) {
        match function_like.identifier {
            FunctionLikeIdentifier::Function(name, _) => {
                self.function_names.insert(name);
            }
            _ => {}
        }
        self.function_like_reflections.push(function_like);
    }

    /// Registers a new class-like entity (class, enum, interface, or trait) in the codebase.
    pub fn register_class_like(&mut self, class_like: ClassLikeReflection) {
        match class_like.identifier {
            ClassLikeIdentifier::Class(name, _) => {
                self.class_names.insert(name);
            }
            ClassLikeIdentifier::Enum(name, _) => {
                self.enum_names.insert(name);
            }
            ClassLikeIdentifier::Interface(name, _) => {
                self.interface_names.insert(name);
            }
            ClassLikeIdentifier::Trait(name, _) => {
                self.trait_names.insert(name);
            }
            _ => {}
        }
        self.class_like_reflections.push(class_like);
    }

    /// Checks if a constant with the given name exists.
    pub fn constant_exists(&self, name: StringIdentifier) -> bool {
        self.constant_names.contains(&name)
    }

    /// Checks if a function with the given name exists.
    pub fn function_exists(&self, name: StringIdentifier) -> bool {
        self.function_names.contains(&name)
    }

    /// Checks if a class with the given name exists.
    pub fn class_exists(&self, name: StringIdentifier) -> bool {
        self.class_names.contains(&name)
    }

    /// Checks if an enum with the given name exists.
    pub fn enum_exists(&self, name: StringIdentifier) -> bool {
        self.enum_names.contains(&name)
    }

    /// Checks if an interface with the given name exists.
    pub fn interface_exists(&self, name: StringIdentifier) -> bool {
        self.interface_names.contains(&name)
    }

    /// Checks if a trait with the given name exists.
    pub fn trait_exists(&self, name: StringIdentifier) -> bool {
        self.trait_names.contains(&name)
    }

    /// Retrieves a constant by name, if it exists.
    pub fn get_constant(&self, name: StringIdentifier) -> Option<&ConstantReflection> {
        self.constant_reflections.iter().find(|constant| constant.identifier.name == name)
    }

    /// Retrieves a function by name, if it exists.
    pub fn get_function(&self, name: StringIdentifier) -> Option<&FunctionLikeReflection> {
        self.function_like_reflections.iter().find(|function_like| match function_like.identifier {
            FunctionLikeIdentifier::Function(function_name, _) => function_name == name,
            _ => false,
        })
    }

    /// Retrieves a closure by its position, if it exists.
    pub fn get_closure(&self, position: &impl HasPosition) -> Option<&FunctionLikeReflection> {
        self.function_like_reflections.iter().find(|function_like| match function_like.identifier {
            FunctionLikeIdentifier::Closure(span) => span.contains(position),
            _ => false,
        })
    }

    /// Retrieves an arrow function by its position, if it exists.
    pub fn get_arrow_function(&self, position: &impl HasPosition) -> Option<&FunctionLikeReflection> {
        self.function_like_reflections.iter().find(|function_like| match function_like.identifier {
            FunctionLikeIdentifier::ArrowFunction(span) => span.contains(position),
            _ => false,
        })
    }

    /// Retrieves a class-like entity by name, if it exists.
    pub fn get_class_like(&self, name: StringIdentifier) -> Option<&ClassLikeReflection> {
        self.class_like_reflections.iter().find(|class_like| match class_like.identifier {
            ClassLikeIdentifier::Class(class_name, _) => class_name == name,
            ClassLikeIdentifier::Enum(enum_name, _) => enum_name == name,
            ClassLikeIdentifier::Interface(interface_name, _) => interface_name == name,
            ClassLikeIdentifier::Trait(trait_name, _) => trait_name == name,
            _ => false,
        })
    }

    /// Retrieves a class by name, if it exists.
    pub fn get_class(&self, name: StringIdentifier) -> Option<&ClassLikeReflection> {
        self.class_like_reflections.iter().find(|class_like| match class_like.identifier {
            ClassLikeIdentifier::Class(class_name, _) => class_name == name,
            _ => false,
        })
    }

    /// Retrieves an enum by name, if it exists.
    pub fn get_enum(&self, name: StringIdentifier) -> Option<&ClassLikeReflection> {
        self.class_like_reflections.iter().find(|class_like| match class_like.identifier {
            ClassLikeIdentifier::Enum(enum_name, _) => enum_name == name,
            _ => false,
        })
    }

    /// Retrieves an interface by name, if it exists.
    pub fn get_interface(&self, name: StringIdentifier) -> Option<&ClassLikeReflection> {
        self.class_like_reflections.iter().find(|class_like| match class_like.identifier {
            ClassLikeIdentifier::Interface(interface_name, _) => interface_name == name,
            _ => false,
        })
    }

    /// Retrieves a trait by name, if it exists.
    pub fn get_trait(&self, name: StringIdentifier) -> Option<&ClassLikeReflection> {
        self.class_like_reflections.iter().find(|class_like| match class_like.identifier {
            ClassLikeIdentifier::Trait(trait_name, _) => trait_name == name,
            _ => false,
        })
    }
}
