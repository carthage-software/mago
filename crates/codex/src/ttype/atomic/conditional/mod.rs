use serde::Deserialize;
use serde::Serialize;

use mago_interner::ThreadedInterner;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::union::TUnion;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct TConditional {
    pub subject: Box<TUnion>,
    pub target: Box<TUnion>,
    pub then: Box<TUnion>,
    pub otherwise: Box<TUnion>,
    pub negated: bool,
}

impl TConditional {
    pub fn new(
        subject: Box<TUnion>,
        target: Box<TUnion>,
        then: Box<TUnion>,
        otherwise: Box<TUnion>,
        negated: bool,
    ) -> Self {
        Self { subject, target, then, otherwise, negated }
    }

    pub fn get_subject(&self) -> &TUnion {
        &self.subject
    }

    pub fn get_subject_mut(&mut self) -> &mut TUnion {
        &mut self.subject
    }

    pub fn get_target(&self) -> &TUnion {
        &self.target
    }

    pub fn get_target_mut(&mut self) -> &mut TUnion {
        &mut self.target
    }

    pub fn get_then(&self) -> &TUnion {
        &self.then
    }

    pub fn get_then_mut(&mut self) -> &mut TUnion {
        &mut self.then
    }

    pub fn get_otherwise(&self) -> &TUnion {
        &self.otherwise
    }

    pub fn get_otherwise_mut(&mut self) -> &mut TUnion {
        &mut self.otherwise
    }

    pub fn is_negated(&self) -> bool {
        self.negated
    }
}

impl TType for TConditional {
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
        vec![
            TypeRef::Union(self.subject.as_ref()),
            TypeRef::Union(self.target.as_ref()),
            TypeRef::Union(self.then.as_ref()),
            TypeRef::Union(self.otherwise.as_ref()),
        ]
    }

    fn get_id(&self, interner: Option<&ThreadedInterner>) -> String {
        let mut id = "(".to_string();

        id += &self.subject.get_id(interner);
        id += " is ";
        if self.negated {
            id += "not ";
        }

        id += &self.target.get_id(interner);
        id += " ? ";
        id += &self.then.get_id(interner);
        id += " : ";
        id += &self.otherwise.get_id(interner);
        id += ")";

        id
    }
}
