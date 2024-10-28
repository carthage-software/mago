use ahash::HashSet;

use serde::Deserialize;
use serde::Serialize;

use fennec_interner::StringIdentifier;
use fennec_span::Span;

use crate::attribute::AttributeReflection;
use crate::class_like::constant::ClassLikeConstantReflection;
use crate::class_like::enum_case::EnumCaseReflection;
use crate::class_like::inheritance::InheritanceReflection;
use crate::class_like::member::MemeberCollection;
use crate::class_like::property::PropertyReflection;
use crate::function_like::FunctionLikeReflection;
use crate::identifier::ClassLikeIdentifier;
use crate::r#type::TypeReflection;

pub mod constant;
pub mod enum_case;
pub mod inheritance;
pub mod member;
pub mod property;

/// Represents reflection data for a PHP class, interface, enum, or trait.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClassLikeReflection {
    pub attribute_reflections: Vec<AttributeReflection>,
    pub identifier: ClassLikeIdentifier,
    pub inheritance_reflection: InheritanceReflection,
    pub constant_reflections: MemeberCollection<ClassLikeConstantReflection>,
    pub case_reflections: MemeberCollection<EnumCaseReflection>,
    pub property_reflections: MemeberCollection<PropertyReflection>,
    pub method_reflections: MemeberCollection<FunctionLikeReflection>,
    pub used_traits: HashSet<StringIdentifier>,
    pub backing_type_reflection: Option<TypeReflection>,
    pub is_final: bool,
    pub is_readonly: bool,
    pub is_abstract: bool,
    pub span: Span,
}

impl ClassLikeReflection {
    /// Checks if this class-like entity extends the given class.
    pub fn extends_class(&self, class_like_identifier: &StringIdentifier) -> bool {
        self.inheritance_reflection.all_extended_classes.contains(class_like_identifier)
    }

    /// Checks if this class-like entity implements the given interface.
    pub fn implements_interface(&self, interface_identifier: &StringIdentifier) -> bool {
        self.inheritance_reflection.all_implemented_interfaces.contains(interface_identifier)
    }

    /// Checks if this interface extends the given interface.
    pub fn extends_interface(&self, interface_identifier: &StringIdentifier) -> bool {
        self.inheritance_reflection.all_extended_interfaces.contains(interface_identifier)
    }

    /// Checks if this class-like entity uses the given trait.
    pub fn uses_trait(&self, trait_identifier: &StringIdentifier) -> bool {
        self.used_traits.contains(trait_identifier)
    }

    /// Checks if this class-like entity contains a constant with the given name.
    pub fn has_constant(&self, constant_name: &StringIdentifier) -> bool {
        self.constant_reflections.appering_members.contains(constant_name)
    }

    /// Checks if this class-like entity contains an enum case with the given name.
    pub fn has_enum_case(&self, case_name: &StringIdentifier) -> bool {
        self.case_reflections.appering_members.contains(case_name)
    }

    /// Checks if this class-like entity has a property with the given name.
    pub fn has_property(&self, property_name: &StringIdentifier) -> bool {
        self.property_reflections.appering_members.contains(property_name)
    }

    /// Checks if this class-like entity has a method with the given name.
    pub fn has_method(&self, method_name: &StringIdentifier) -> bool {
        self.method_reflections.appering_members.contains(method_name)
    }

    /// Retrieves a constant by name, if it exists.
    pub fn get_constant(&self, constant_name: &StringIdentifier) -> Option<&ClassLikeConstantReflection> {
        self.constant_reflections.members.get(constant_name)
    }

    /// Retrieves an enum case by name, if it exists.
    pub fn get_enum_case(&self, case_name: &StringIdentifier) -> Option<&EnumCaseReflection> {
        self.case_reflections.members.get(case_name)
    }

    /// Retrieves a property by name, if it exists.
    pub fn get_property(&self, property_name: &StringIdentifier) -> Option<&PropertyReflection> {
        self.property_reflections.members.get(property_name)
    }

    /// Retrieves a method by name, if it exists.
    pub fn get_method(&self, method_name: &StringIdentifier) -> Option<&FunctionLikeReflection> {
        self.method_reflections.members.get(method_name)
    }
}
