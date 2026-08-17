use std::sync::Arc;

use mago_word::Word;
use mago_word::concat_word;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::union::TUnion;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TConditional {
    pub subject: Arc<TUnion>,
    pub target: Arc<TUnion>,
    pub then: Arc<TUnion>,
    pub otherwise: Arc<TUnion>,
    pub negated: bool,
}

impl TConditional {
    #[must_use]
    pub fn new(
        subject: Arc<TUnion>,
        target: Arc<TUnion>,
        then: Arc<TUnion>,
        otherwise: Arc<TUnion>,
        negated: bool,
    ) -> Self {
        Self { subject, target, then, otherwise, negated }
    }

    pub fn get_subject_mut(&mut self) -> &mut TUnion {
        Arc::make_mut(&mut self.subject)
    }

    pub fn get_target_mut(&mut self) -> &mut TUnion {
        Arc::make_mut(&mut self.target)
    }

    pub fn get_then_mut(&mut self) -> &mut TUnion {
        Arc::make_mut(&mut self.then)
    }

    pub fn get_otherwise_mut(&mut self) -> &mut TUnion {
        Arc::make_mut(&mut self.otherwise)
    }

    #[must_use]
    pub fn is_negated(&self) -> bool {
        self.negated
    }
}

impl TType for TConditional {
    fn get_child_nodes(&self) -> Vec<TypeRef<'_>> {
        vec![
            TypeRef::Union(self.subject.as_ref()),
            TypeRef::Union(self.target.as_ref()),
            TypeRef::Union(self.then.as_ref()),
            TypeRef::Union(self.otherwise.as_ref()),
        ]
    }

    fn needs_population(&self) -> bool {
        self.subject.needs_population()
            || self.target.needs_population()
            || self.then.needs_population()
            || self.otherwise.needs_population()
    }

    fn is_expandable(&self) -> bool {
        true
    }

    fn get_id(&self) -> Word {
        concat_word!(
            "(",
            self.subject.get_id(),
            if self.negated { " is not " } else { " is " },
            self.target.get_id(),
            " ? ",
            self.then.get_id(),
            " : ",
            self.otherwise.get_id(),
            ")"
        )
    }
}
