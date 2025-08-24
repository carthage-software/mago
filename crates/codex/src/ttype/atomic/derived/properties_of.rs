use mago_atom::Atom;
use mago_atom::concat_atom;
use serde::Deserialize;
use serde::Serialize;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::visibility::Visibility;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord)]
pub struct TPropertiesOf {
    pub visibility: Option<Visibility>,
    pub target_type: Box<TAtomic>,
}

impl TPropertiesOf {
    #[inline]
    pub const fn new(target_type: Box<TAtomic>) -> Self {
        TPropertiesOf { visibility: None, target_type }
    }

    #[inline]
    pub const fn public(target_type: Box<TAtomic>) -> Self {
        TPropertiesOf { visibility: Some(Visibility::Public), target_type }
    }

    #[inline]
    pub const fn protected(target_type: Box<TAtomic>) -> Self {
        TPropertiesOf { visibility: Some(Visibility::Protected), target_type }
    }

    #[inline]
    pub const fn private(target_type: Box<TAtomic>) -> Self {
        TPropertiesOf { visibility: Some(Visibility::Private), target_type }
    }

    #[inline]
    pub const fn visibility(&self) -> Option<Visibility> {
        self.visibility
    }

    #[inline]
    pub const fn get_target_type(&self) -> &TAtomic {
        &self.target_type
    }

    #[inline]
    pub const fn get_target_type_mut(&mut self) -> &mut TAtomic {
        &mut self.target_type
    }
}

impl TType for TPropertiesOf {
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
        vec![TypeRef::Atomic(&self.target_type)]
    }

    fn needs_population(&self) -> bool {
        self.target_type.needs_population()
    }

    fn is_expandable(&self) -> bool {
        true
    }

    fn get_id(&self) -> Atom {
        if let Some(visibility) = &self.visibility {
            concat_atom!(visibility.as_str(), "-properties-of<", self.target_type.get_id().as_str(), ">")
        } else {
            concat_atom!("properties-of<", self.target_type.get_id().as_str(), ">")
        }
    }
}
