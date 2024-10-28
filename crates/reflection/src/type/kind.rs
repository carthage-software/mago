use serde::Deserialize;
use serde::Serialize;

use fennec_interner::StringIdentifier;

use crate::identifier::ClassLikeIdentifier;

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
    Static(ClassLikeIdentifier),
    Self_(ClassLikeIdentifier),
    Parent(ClassLikeIdentifier),
    Void,
    Never,
    Float,
    Bool,
    Integer,
    String,
    Object,
    Mixed,
    Iterable,
    InvalidStatic,
    InvalidSelf,
    InvalidParent,
}

impl TypeKind {
    pub fn is_nullable(&self) -> bool {
        match &self {
            TypeKind::Union(kinds) => kinds.iter().any(|k| k.is_nullable()),
            TypeKind::Null => true,
            TypeKind::Mixed => true,
            _ => false,
        }
    }
}
