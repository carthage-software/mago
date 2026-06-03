use mago_syntax::cst;

use crate::ir::name::Name;
use crate::lower::Lowering;

impl<'arena> Lowering<'arena> {
    pub(crate) fn lower_name(&self, name: &'arena cst::LocalIdentifier<'arena>) -> Name<'arena> {
        Name { span: name.span, value: name.value }
    }
}
