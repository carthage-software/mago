use ahash::HashMap;
use ahash::HashSet;
use fennec_interner::StringIdentifier;
use serde::Deserialize;
use serde::Serialize;

use fennec_span::Span;

use crate::attribute::AttributeReflection;
use crate::class_like::constant::ClassLikeConstantReflection;
use crate::class_like::enum_case::EnumCaseReflection;
use crate::class_like::inheritance::InheritanceReflection;
use crate::class_like::member::MemeberCollection;
use crate::class_like::property::PropertyReflection;
use crate::function_like::FunctionLikeReflection;
use crate::identifier::ClassLikeName;
use crate::identifier::Name;
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
    pub name: ClassLikeName,
    pub inheritance: InheritanceReflection,
    pub constants: HashMap<StringIdentifier, ClassLikeConstantReflection>,
    pub cases: MemeberCollection<EnumCaseReflection>,
    pub properties: MemeberCollection<PropertyReflection>,
    pub methods: MemeberCollection<FunctionLikeReflection>,
    pub used_traits: HashSet<StringIdentifier>,
    pub used_trait_names: HashMap<StringIdentifier, Name>,
    pub backing_type: Option<TypeReflection>,
    pub is_final: bool,
    pub is_readonly: bool,
    pub is_abstract: bool,
    pub is_anonymous: bool,
    pub span: Span,
    pub is_populated: bool,
}

impl ClassLikeReflection {
    pub fn is_trait(&self) -> bool {
        matches!(self.name, ClassLikeName::Trait(_))
    }

    pub fn is_interface(&self) -> bool {
        matches!(self.name, ClassLikeName::Interface(_))
    }

    pub fn is_class(&self) -> bool {
        matches!(self.name, ClassLikeName::Class(_))
    }

    pub fn is_enum(&self) -> bool {
        matches!(self.name, ClassLikeName::Enum(_))
    }

    pub fn is_anonymous_class(&self) -> bool {
        matches!(self.name, ClassLikeName::AnonymousClass(_))
    }

    /// Checks if this class-like entity extends the given class.
    pub fn extends_class(&self, class_like_identifier: &Name) -> bool {
        self.inheritance.all_extended_classes.contains(class_like_identifier)
    }

    /// Checks if this class-like entity implements the given interface.
    pub fn implements_interface(&self, interface_identifier: &Name) -> bool {
        self.inheritance.all_implemented_interfaces.contains(interface_identifier)
    }

    /// Checks if this interface extends the given interface.
    pub fn extends_interface(&self, interface_identifier: &Name) -> bool {
        self.inheritance.all_extended_interfaces.contains(interface_identifier)
    }

    /// Checks if this class-like entity uses the given trait.
    pub fn uses_trait(&self, trait_identifier: &StringIdentifier) -> bool {
        self.used_traits.contains(trait_identifier)
    }

    /// Checks if this class-like entity contains a constant with the given name.
    pub fn has_constant(&self, constant_name: &StringIdentifier) -> bool {
        self.constants.contains_key(constant_name)
    }

    /// Checks if this class-like entity contains an enum case with the given name.
    pub fn has_enum_case(&self, case_name: &StringIdentifier) -> bool {
        self.cases.appering_members.contains_key(case_name)
    }

    /// Checks if this class-like entity has a property with the given name.
    pub fn has_property(&self, property_name: &StringIdentifier) -> bool {
        self.properties.appering_members.contains_key(property_name)
    }

    /// Checks if this class-like entity has a method with the given name.
    pub fn has_method(&self, method_name: &StringIdentifier) -> bool {
        self.methods.appering_members.contains_key(method_name)
    }

    /// Retrieves a constant by name, if it exists.
    pub fn get_constant(&self, constant_name: &StringIdentifier) -> Option<&ClassLikeConstantReflection> {
        self.constants.get(constant_name)
    }

    /// Retrieves an enum case by name, if it exists.
    pub fn get_enum_case(&self, case_name: &StringIdentifier) -> Option<&EnumCaseReflection> {
        self.cases.members.get(case_name)
    }

    /// Retrieves a property by name, if it exists.
    pub fn get_property(&self, property_name: &StringIdentifier) -> Option<&PropertyReflection> {
        self.properties.members.get(property_name)
    }

    /// Retrieves a method by name, if it exists.
    pub fn get_method(&self, method_name: &StringIdentifier) -> Option<&FunctionLikeReflection> {
        self.methods.members.get(method_name)
    }
}
