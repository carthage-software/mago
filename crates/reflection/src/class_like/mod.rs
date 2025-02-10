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
    pub cases: HashMap<StringIdentifier, EnumCaseReflection>,

    /// Properties defined in the class-like entity.
    pub properties: MemeberCollection<PropertyReflection>,

    /// Methods defined in the class-like entity.
    pub methods: MemeberCollection<FunctionLikeReflection>,

    /// Traits used by the class-like entity.
    pub used_traits: HashSet<Name>,

    /// Traits used by the class-like entity.
    pub used_trait_names: HashMap<StringIdentifier, Name>,

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
    pub fn new(name: ClassLikeName, span: Span) -> Self {
        Self {
            attribute_reflections: Vec::new(),
            name,
            inheritance: InheritanceReflection::default(),
            constants: HashMap::default(),
            cases: HashMap::default(),
            properties: MemeberCollection::default(),
            methods: MemeberCollection::default(),
            used_traits: HashSet::default(),
            used_trait_names: HashMap::default(),
            backing_type: None,
            is_final: false,
            is_readonly: false,
            is_abstract: false,
            is_anonymous: false,
            span,
            is_populated: false,
            issues: IssueCollection::new(),
        }
    }

    /// Checks if this class-like entity is a trait.
    #[inline]
    pub const fn is_trait(&self) -> bool {
        matches!(self.name, ClassLikeName::Trait(_))
    }

    /// Checks if this class-like entity is an interface.
    #[inline]
    pub const fn is_interface(&self) -> bool {
        matches!(self.name, ClassLikeName::Interface(_))
    }

    /// Checks if this class-like entity is a class.
    #[inline]
    pub const fn is_class(&self) -> bool {
        matches!(self.name, ClassLikeName::Class(_))
    }

    /// Checks if this class-like entity is an enum.
    #[inline]
    pub const fn is_enum(&self) -> bool {
        matches!(self.name, ClassLikeName::Enum(_))
    }

    /// Checks if this class-like entity is a trait.
    #[inline]
    pub const fn is_anonymous_class(&self) -> bool {
        matches!(self.name, ClassLikeName::AnonymousClass(_))
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

    fn take_issues(&mut self) -> IssueCollection {
        std::mem::take(&mut self.issues)
    }
}
