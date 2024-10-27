use serde::Deserialize;
use serde::Serialize;

use fennec_interner::StringIdentifier;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum TypeKind {
    Identifier(StringIdentifier),
    Union(Vec<TypeKind>),
    Intersection(Vec<TypeKind>),
    Null,
    True,
    False,
    Array,
    Callable,
    Static(StringIdentifier),
    Self_(StringIdentifier),
    Parent(StringIdentifier),
    Void,
    Never,
    Float,
    Bool,
    Integer,
    String,
    Object,
    Mixed,
    Iterable,
}

impl TypeKind {
    pub fn is_nullable(&self) -> bool {
        match &self{
            TypeKind::Union(kinds) => {
                kinds.iter().any(|k| k.is_nullable())
            }
            TypeKind::Null => true,
            TypeKind::Mixed => true,
            _ => false,
        }
    }
}
