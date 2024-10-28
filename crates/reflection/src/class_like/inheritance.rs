use ahash::HashSet;

use serde::Deserialize;
use serde::Serialize;

use fennec_interner::StringIdentifier;

use crate::identifier::ClassLikeIdentifier;

/// Represents the inheritance details of a class-like entity, including implemented interfaces,
/// extended classes or interfaces, and any required inheritance constraints.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct InheritanceReflection {
    /// Interfaces directly implemented by the current class or enum.
    pub direct_implemented_interfaces: HashSet<StringIdentifier>,

    /// All interfaces implemented by the current class or any of its ancestors,
    /// including `direct_implemented_interfaces`.
    pub all_implemented_interfaces: HashSet<StringIdentifier>,

    /// The class directly extended by the current class, if applicable.
    pub direct_extended_class: Option<StringIdentifier>,

    /// All classes extended by the current class or any of its ancestors,
    /// including `direct_extended_class`.
    pub all_extended_classes: HashSet<StringIdentifier>,

    /// Interfaces directly extended by the current interface, if applicable.
    pub direct_extended_interfaces: HashSet<StringIdentifier>,

    /// All interfaces extended by the current interface or any of its ancestors,
    /// including `direct_extended_interfaces`.
    pub all_extended_interfaces: HashSet<StringIdentifier>,

    /// Interfaces that the current class-like entity requires any inheriting entity to implement,
    /// as specified by the `@require-implements` tag.
    pub require_implementations: HashSet<StringIdentifier>,

    /// Classes or interfaces that the current class-like entity requires any inheriting entity to extend,
    /// as specified by the `@require-extends` tag.
    pub require_extensions: HashSet<StringIdentifier>,

    /// Identifiers of class-like entities that directly extend or implement the current class-like entity.
    pub children: HashSet<ClassLikeIdentifier>,
}
