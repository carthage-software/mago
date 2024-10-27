use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;

use crate::identifier::ClassLikeIdentifier;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct InheritanceReflection {
    pub direct_implemented_interfaces: HashSet<ClassLikeIdentifier>,
    pub all_implemented_interfaces: HashSet<ClassLikeIdentifier>,
    pub direct_extended_class: Option<ClassLikeIdentifier>,
    pub all_extendeded_classes: HashSet<ClassLikeIdentifier>,
    pub direct_extended_interfaces: HashSet<ClassLikeIdentifier>,
    pub all_extended_interfaces: HashSet<ClassLikeIdentifier>,
    pub require_implementations: HashSet<ClassLikeIdentifier>,
    pub require_extensions: HashSet<ClassLikeIdentifier>,
    pub children: HashSet<ClassLikeIdentifier>,
}
