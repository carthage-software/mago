use std::sync::Arc;

use mago_word::Word;
use mago_word::concat_word;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::union::TUnion;

/// Represents `int-mask-of<Foo::*>` utility type.
///
/// This type extracts integer constants from a class and expands to
/// a union of all possible bitmask combinations.
///
/// For example, if `Foo` has constants `READ = 1`, `WRITE = 2`, `EXECUTE = 4`,
/// then `int-mask-of<Foo::*>` expands to `0|1|2|3|4|5|6|7`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TIntMaskOf(Arc<TUnion>);

impl TIntMaskOf {
    #[must_use]
    pub fn new(target: Arc<TUnion>) -> Self {
        Self(target)
    }

    #[inline]
    #[must_use]
    pub fn get_target_type(&self) -> &TUnion {
        &self.0
    }

    #[inline]
    pub fn get_target_type_mut(&mut self) -> &mut TUnion {
        Arc::make_mut(&mut self.0)
    }
}

impl TType for TIntMaskOf {
    fn get_child_nodes(&self) -> Vec<TypeRef<'_>> {
        vec![TypeRef::Union(&self.0)]
    }

    fn needs_population(&self) -> bool {
        self.0.needs_population()
    }

    fn is_expandable(&self) -> bool {
        true
    }

    fn is_complex(&self) -> bool {
        false
    }

    fn get_id(&self) -> Word {
        concat_word!(b"int-mask-of<", self.0.get_id(), b">")
    }

    fn get_pretty_id_with_indent(&self, _indent: usize) -> Word {
        self.get_id()
    }
}
