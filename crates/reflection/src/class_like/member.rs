use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;

use serde::Deserialize;
use serde::Serialize;

use fennec_span::Span;

use crate::identifier::ClassLikeIdentifier;
use crate::identifier::ClassLikeMemberIdentifier;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemeberCollection<T: Eq + PartialEq> {
    members: HashMap<ClassLikeMemberIdentifier, T>,
    declared_members: HashSet<ClassLikeMemberIdentifier>,
    inherited_members: HashMap<ClassLikeMemberIdentifier, ClassLikeIdentifier>,
    overriden_members: HashMap<ClassLikeMemberIdentifier, HashSet<ClassLikeIdentifier>>,
    inheritable_members: HashSet<ClassLikeMemberIdentifier>,
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
