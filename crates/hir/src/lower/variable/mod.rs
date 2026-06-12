pub mod annotation;

use mago_allocator::Arena;
use mago_syntax::cst;

use crate::ir::variable::DirectVariable;
use crate::ir::variable::Variable;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_variable(
        &mut self,
        variable: &'scratch cst::Variable<'scratch>,
    ) -> Variable<'arena, (), (), ()> {
        match variable {
            cst::Variable::Direct(direct) => Variable::Direct(self.lower_direct_variable(direct)),
            cst::Variable::Indirect(indirect) => {
                Variable::Indirect(self.arena.alloc(self.lower_expression(indirect.expression)))
            }
            cst::Variable::Nested(nested) => Variable::Nested(self.arena.alloc(self.lower_variable(nested.variable))),
        }
    }

    pub(crate) fn lower_direct_variable(
        &mut self,
        variable: &'scratch cst::DirectVariable<'scratch>,
    ) -> DirectVariable<'arena> {
        DirectVariable { span: variable.span, name: self.interner.intern(variable.name) }
    }
}
