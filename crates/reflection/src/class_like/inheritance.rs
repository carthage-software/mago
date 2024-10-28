use ahash::HashSet;

use serde::Deserialize;
use serde::Serialize;

use fennec_interner::StringIdentifier;

use crate::identifier::ClassLikeIdentifier;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct InheritanceReflection {
    pub direct_implemented_interfaces: HashSet<StringIdentifier>,
    pub all_implemented_interfaces: HashSet<StringIdentifier>,
    pub direct_extended_class: Option<StringIdentifier>,
    pub all_extended_classes: HashSet<StringIdentifier>,
    pub direct_extended_interfaces: HashSet<StringIdentifier>,
    pub all_extended_interfaces: HashSet<StringIdentifier>,
    pub require_implementations: HashSet<StringIdentifier>,
    pub require_extensions: HashSet<StringIdentifier>,
    pub children: HashSet<ClassLikeIdentifier>,
}
