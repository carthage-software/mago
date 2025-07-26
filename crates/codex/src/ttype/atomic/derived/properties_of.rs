use serde::Deserialize;
use serde::Serialize;

use mago_interner::ThreadedInterner;

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

    fn get_id(&self, interner: Option<&ThreadedInterner>) -> String {
        let mut id = String::new();
        if let Some(visibility) = &self.visibility {
            id += visibility.as_str();
            id += "-";
        }

        id += "properties-of<";
        id += &self.target_type.get_id(interner);
        id += ">";

        id
    }
}
