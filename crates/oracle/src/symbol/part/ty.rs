#[cfg(feature = "serde")]
use serde::Serialize;

use crate::ty::Type;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct TypeSlot<'arena> {
    pub hint: Option<Type<'arena>>,
    pub annotation: Option<Type<'arena>>,
    pub inferred: Option<Type<'arena>>,
}

impl<'arena> TypeSlot<'arena> {
    #[must_use]
    pub fn new() -> Self {
        Self { hint: None, annotation: None, inferred: None }
    }

    #[must_use]
    pub fn effective(&self, with_inference: bool) -> Option<Type<'arena>> {
        if with_inference {
            return self.annotation.or(self.hint).or(self.inferred);
        }

        self.annotation.or(self.hint)
    }
}
