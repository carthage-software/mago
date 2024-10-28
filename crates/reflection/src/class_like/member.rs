use std::hash::Hash;

use ahash::HashMap;
use ahash::HashSet;
use serde::Deserialize;
use serde::Serialize;

use fennec_interner::StringIdentifier;
use fennec_span::Span;

use crate::identifier::ClassLikeIdentifier;

/// Represents a collection of members (e.g., properties, methods, constants) associated with a class-like entity.
///
/// This structure maintains the details of each member, such as their identifiers and inheritance information,
/// allowing reflection on declared, inherited, overridden, and inheritable members.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemeberCollection<T: Eq + PartialEq> {
    /// All instances of the members, mapped by their unique identifiers.
    pub members: HashMap<StringIdentifier, T>,

    /// Identifiers of all members that appear in the current class-like entity,
    /// regardless of whether they were declared or inherited.
    pub appering_members: HashSet<StringIdentifier>,

    /// Identifiers of members that were explicitly declared within the current class-like entity.
    pub declared_members: HashSet<StringIdentifier>,

    /// Identifiers of members inherited from parent class-like entities, mapped to the originating class-like entity.
    pub inherited_members: HashMap<StringIdentifier, ClassLikeIdentifier>,

    /// Identifiers of members overridden in the current class-like entity.
    /// 
    /// These are members that exist in both the current and a parent class-like entity.
    pub overriden_members: HashMap<StringIdentifier, HashSet<ClassLikeIdentifier>>,

    /// Identifiers of members that can be inherited by child class-like entities.
    pub inheritable_members: HashSet<StringIdentifier>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ClassLikeMemberVisibilityReflection {
    Public { span: Span },
    Protected { span: Span },
    Private { span: Span },
}

impl ClassLikeMemberVisibilityReflection {
    pub fn is_public(&self) -> bool {
        matches!(self, ClassLikeMemberVisibilityReflection::Public { .. })
    }

    pub fn is_protected(&self) -> bool {
        matches!(self, ClassLikeMemberVisibilityReflection::Protected { .. })
    }

    pub fn is_private(&self) -> bool {
        matches!(self, ClassLikeMemberVisibilityReflection::Private { .. })
    }
}
