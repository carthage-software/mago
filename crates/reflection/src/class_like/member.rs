use std::hash::Hash;

use ahash::HashMap;
use ahash::HashSet;
use serde::Deserialize;
use serde::Serialize;

use mago_interner::StringIdentifier;
use mago_span::Span;

use crate::identifier::ClassLikeName;

/// Represents a collection of members (e.g., properties, methods, constants) associated with a class-like entity.
///
/// This structure maintains the details of each member, such as their identifiers and inheritance information,
/// allowing reflection on declared, inherited, overridden, and inheritable members.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemeberCollection<T: Eq + PartialEq> {
    pub members: HashMap<StringIdentifier, T>,
    pub appering_members: HashMap<StringIdentifier, ClassLikeName>,
    pub declaring_members: HashMap<StringIdentifier, ClassLikeName>,
    pub overriden_members: HashMap<StringIdentifier, HashSet<ClassLikeName>>,
    pub inheritable_members: HashMap<StringIdentifier, ClassLikeName>,
}

impl<T: Eq + PartialEq> Default for MemeberCollection<T> {
    fn default() -> Self {
        Self {
            members: Default::default(),
            appering_members: Default::default(),
            declaring_members: Default::default(),
            overriden_members: Default::default(),
            inheritable_members: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ClassLikeMemberVisibilityReflection {
    Public { span: Span },
    Protected { span: Span },
    Private { span: Span },
}

impl ClassLikeMemberVisibilityReflection {
    #[inline(always)]
    pub const fn is_public(&self) -> bool {
        matches!(self, ClassLikeMemberVisibilityReflection::Public { .. })
    }

    #[inline(always)]
    pub const fn is_protected(&self) -> bool {
        matches!(self, ClassLikeMemberVisibilityReflection::Protected { .. })
    }

    #[inline(always)]
    pub const fn is_private(&self) -> bool {
        matches!(self, ClassLikeMemberVisibilityReflection::Private { .. })
    }
}
