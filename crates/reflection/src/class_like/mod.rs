use ahash::HashMap;
use ahash::HashSet;
use serde::Deserialize;
use serde::Serialize;

use mago_interner::StringIdentifier;
use mago_reporting::IssueCollection;
use mago_source::HasSource;
use mago_source::SourceIdentifier;
use mago_span::HasSpan;
use mago_span::Span;

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
use crate::Reflection;

pub mod constant;
pub mod enum_case;
pub mod inheritance;
pub mod member;
pub mod property;

/// Represents reflection data for a PHP class, interface, enum, or trait.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClassLikeReflection {
    /// Attributes (e.g., annotations) associated with the class-like entity.
    pub attribute_reflections: Vec<AttributeReflection>,

    /// The name of the class-like entity, such as its fully qualified name.
    pub name: ClassLikeName,

    /// Inheritance information for the class-like entity, including parent classes and implemented interfaces.
    pub inheritance: InheritanceReflection,

    /// Constants defined in the class-like entity.
    pub constants: HashMap<StringIdentifier, ClassLikeConstantReflection>,

    /// Enum cases defined in the class-like entity, if it is an enum.
    pub cases: MemeberCollection<EnumCaseReflection>,

    /// Properties defined in the class-like entity.
    pub properties: MemeberCollection<PropertyReflection>,

    /// Methods defined in the class-like entity.
    pub methods: MemeberCollection<FunctionLikeReflection>,

    /// Traits used by the class-like entity.
    pub used_traits: HashSet<StringIdentifier>,

    /// The backing type of the entity, used if it is an enum.
    pub backing_type: Option<TypeReflection>,

    /// Whether the class-like entity is declared as `final`.
    pub is_final: bool,

    /// Whether the class-like entity is declared as `readonly`.
    pub is_readonly: bool,

    /// Whether the class-like entity is declared as `abstract`.
    pub is_abstract: bool,

    /// Whether the entity is an anonymous class.
    pub is_anonymous: bool,

    /// The span in the source code where the class-like entity is declared.
    pub span: Span,

    /// Indicates whether the reflection is fully populated with all metadata.
    pub is_populated: bool,

    /// Issues encountered while processing the class-like entity.
    pub issues: IssueCollection,
}

impl ClassLikeReflection {
    /// Checks if this class-like entity is a trait.
    pub fn is_trait(&self) -> bool {
        matches!(self.name, ClassLikeName::Trait(_))
    }

    /// Checks if this class-like entity is an interface.
    pub fn is_interface(&self) -> bool {
        matches!(self.name, ClassLikeName::Interface(_))
    }

    /// Checks if this class-like entity is a class.
    pub fn is_class(&self) -> bool {
        matches!(self.name, ClassLikeName::Class(_))
    }

    /// Checks if this class-like entity is an enum.
    pub fn is_enum(&self) -> bool {
        matches!(self.name, ClassLikeName::Enum(_))
    }

    /// Checks if this class-like entity is a trait.
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

impl HasSpan for ClassLikeReflection {
    /// Returns the span of the class-like entity in the source code.
    ///
    /// The span covers the entire declaration of the entity, including its attributes,
    /// inheritance, and body.
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSource for ClassLikeReflection {
    /// Returns the source identifier of the file containing this class-like entity.
    ///
    /// The source identifier provides metadata about the origin of the entity,
    /// such as whether it is user-defined, vendor-provided, or built-in.
    fn source(&self) -> SourceIdentifier {
        self.span.source()
    }
}

impl Reflection for ClassLikeReflection {
    /// Returns the source category of the class-like entity.
    ///
    /// The category indicates whether the entity is part of the project (`UserDefined`),
    /// vendor-provided (`Vendor`), or built-in (`BuiltIn`).
    fn get_category(&self) -> crate::SourceCategory {
        self.source().category()
    }

    /// Indicates whether the class-like entity's reflection data is fully populated.
    ///
    /// If `is_populated` is `false`, additional processing may be required to resolve
    /// all metadata for this entity.
    fn is_populated(&self) -> bool {
        self.is_populated
    }

    /// Returns the issues encountered while processing the class-like entity.
    fn get_issues(&self) -> &IssueCollection {
        &self.issues
    }
}
