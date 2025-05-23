use ahash::HashMap;
use ahash::HashSet;

use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use serde::Deserialize;
use serde::Serialize;

use crate::identifier::ClassLikeName;
use crate::identifier::Name;

use super::ClassLikeReflection;

/// Represents the inheritance details of a class-like entity, including implemented interfaces,
/// extended classes or interfaces, and any required inheritance constraints.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct InheritanceReflection {
    /// Interfaces directly implemented by the current class or enum.
    pub direct_implemented_interfaces: HashSet<Name>,

    /// All interfaces implemented by the current class or any of its ancestors,
    /// including `direct_implemented_interfaces`.
    pub all_implemented_interfaces: HashSet<Name>,

    /// The class directly extended by the current class, if applicable.
    pub direct_extended_class: Option<Name>,

    /// All classes extended by the current class or any of its ancestors,
    /// including `direct_extended_class`.
    pub all_extended_classes: HashSet<Name>,

    /// Interfaces directly extended by the current interface, if applicable.
    pub direct_extended_interfaces: HashSet<Name>,

    /// All interfaces extended by the current interface or any of its ancestors,
    /// including `direct_extended_interfaces`.
    pub all_extended_interfaces: HashSet<Name>,

    /// Interfaces that the current class-like entity requires any inheriting entity to implement,
    /// as specified by the `@require-implements` tag.
    pub require_implementations: HashSet<StringIdentifier>,

    /// Classes or interfaces that the current class-like entity requires any inheriting entity to extend,
    /// as specified by the `@require-extends` tag.
    pub require_extensions: HashSet<StringIdentifier>,

    /// Identifiers of class-like entities that directly extend or implement the current class-like entity.
    pub children: HashSet<ClassLikeName>,

    /// A lookup map of string identifiers to class-like names.
    pub names: HashMap<StringIdentifier, Name>,
}

impl InheritanceReflection {
    #[inline]
    pub fn implements_interfaces(&self) -> bool {
        !self.all_implemented_interfaces.is_empty()
    }

    #[inline]
    pub fn extends_classes(&self) -> bool {
        !self.all_extended_classes.is_empty()
    }

    #[inline]
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    #[inline]
    pub fn is_instance_of(&self, interner: &ThreadedInterner, other: &ClassLikeReflection) -> bool {
        let Some(other_name) = other.name.inner() else {
            return false;
        };

        let other_identifier = interner.lowered(&other_name.value);
        if let Some(this_name) = self.direct_extended_class {
            let this_identifier = interner.lowered(&this_name.value);
            if this_identifier == other_identifier {
                return true;
            }
        }

        let Some(other_fqcn) = self.names.get(&other_identifier) else {
            return false;
        };

        self.all_extended_classes.contains(other_fqcn)
            || self.all_implemented_interfaces.contains(other_fqcn)
            || self.all_extended_interfaces.contains(other_fqcn)
    }

    #[inline]
    pub fn extends_class(&self, interner: &ThreadedInterner, other: &ClassLikeReflection) -> bool {
        let Some(name) = other.name.inner() else {
            return false; // we can't extend a class that does not have a name, i.e. anonymous class
        };

        self.extends_class_with_name(interner, &name.value)
    }

    #[inline]
    pub fn extends_class_with_name(&self, interner: &ThreadedInterner, other: &StringIdentifier) -> bool {
        let identifier = interner.lowered(other);
        let Some(other) = self.names.get(&identifier) else {
            return false;
        };

        self.all_extended_classes.contains(other)
    }

    #[inline]
    pub fn extends_interface(&self, interner: &ThreadedInterner, other: &ClassLikeReflection) -> bool {
        let Some(name) = other.name.inner() else {
            return false;
        };

        self.extends_interface_with_name(interner, &name.value)
    }

    #[inline]
    pub fn extends_interface_with_name(&self, interner: &ThreadedInterner, other: &StringIdentifier) -> bool {
        let identifier = interner.lowered(other);
        let Some(other) = self.names.get(&identifier) else {
            return false;
        };

        self.all_extended_interfaces.contains(other)
    }

    #[inline]
    pub fn implements_interface(&self, interner: &ThreadedInterner, other: &ClassLikeReflection) -> bool {
        let Some(name) = other.name.inner() else {
            return false;
        };

        self.implements_interface_with_name(interner, &name.value)
    }

    #[inline]
    pub fn implements_interface_with_name(&self, interner: &ThreadedInterner, other: &StringIdentifier) -> bool {
        let identifier = interner.lowered(other);
        let Some(other) = self.names.get(&identifier) else {
            return false;
        };

        self.all_implemented_interfaces.contains(other)
    }
}
