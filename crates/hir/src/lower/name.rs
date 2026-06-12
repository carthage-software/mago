use mago_allocator::Arena;
use mago_syntax::cst;

use crate::ir::name::Name;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_name(&mut self, name: &'scratch cst::LocalIdentifier<'scratch>) -> Name<'arena> {
        Name { span: name.span, value: self.interner.intern(name.value) }
    }
}
