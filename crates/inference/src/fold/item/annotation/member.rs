use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::vec::Vec;
use mago_hir::ir::delimited::Delimited;
use mago_hir::ir::item::annotation::member::MethodAnnotation;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub(crate) fn infer_method_annotation(
        &mut self,
        method: &MethodAnnotation<'source, SymbolId, S, E>,
    ) -> InferenceResult<MethodAnnotation<'arena, SymbolId, Flow, Type<'arena>>> {
        let mut parameters = Vec::new_in(self.arena);
        for parameter in method.parameters.items {
            parameters.push(self.infer_parameter_annotation(parameter)?);
        }

        Ok(MethodAnnotation {
            span: method.span,
            visibility: method.visibility,
            r#static: method.r#static,
            name: method.name.copy_into(self.arena),
            type_parameters: method.type_parameters.map(|type_parameters| type_parameters.copy_into(self.arena)),
            parameters: Delimited { span: method.parameters.span, items: parameters.leak() },
            return_type: method.return_type.map(|return_type| copy_ref_into(return_type, self.arena)),
        })
    }
}
