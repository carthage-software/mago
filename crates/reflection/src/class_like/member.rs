use std::hash::Hash;

use ahash::HashMap;
use ahash::HashSet;
use serde::Deserialize;
use serde::Serialize;

use fennec_interner::StringIdentifier;
use fennec_span::Span;

use crate::identifier::ClassLikeIdentifier;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemeberCollection<T: Eq + PartialEq> {
    pub members: HashMap<StringIdentifier, T>,
    pub appering_members: HashSet<StringIdentifier>,
    pub declared_members: HashSet<StringIdentifier>,
    pub inherited_members: HashMap<StringIdentifier, ClassLikeIdentifier>,
    pub overriden_members: HashMap<StringIdentifier, HashSet<ClassLikeIdentifier>>,
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
