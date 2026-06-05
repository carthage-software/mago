use mago_syntax::cst;

use crate::ir::variable::DirectVariable;
use crate::ir::variable::Variable;
use crate::lower::Lowering;

impl<'arena> Lowering<'_, 'arena> {
    pub(crate) fn lower_variable(&mut self, variable: &'arena cst::Variable<'arena>) -> Variable<'arena, (), (), ()> {
        match variable {
            cst::Variable::Direct(direct) => Variable::Direct(self.lower_direct_variable(direct)),
            cst::Variable::Indirect(indirect) => {
                Variable::Indirect(self.arena.alloc(self.lower_expression(indirect.expression)))
            }
            cst::Variable::Nested(nested) => Variable::Nested(self.arena.alloc(self.lower_variable(nested.variable))),
        }
    }

    pub(crate) fn lower_direct_variable(
        &self,
        variable: &'arena cst::DirectVariable<'arena>,
    ) -> DirectVariable<'arena> {
        DirectVariable { span: variable.span, name: variable.name }
    }
}
