use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_hir::ir::item::annotation::parameter::ParameterAnnotation;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub(crate) fn infer_parameter_annotation(
        &mut self,
        parameter: &ParameterAnnotation<'source, SymbolId, S, E>,
    ) -> InferenceResult<ParameterAnnotation<'arena, SymbolId, Flow, Type<'arena>>> {
        let default_value = match parameter.default_value {
            Some(default_value) => Some(&*self.arena.alloc(self.infer_expression(default_value)?)),
            None => None,
        };

        Ok(ParameterAnnotation {
            span: parameter.span,
            r#type: parameter.r#type.map(|r#type| copy_ref_into(r#type, self.arena)),
            is_by_reference: parameter.is_by_reference,
            is_variadic: parameter.is_variadic,
            variable: parameter.variable.map(|variable| variable.copy_into(self.arena)),
            default_value,
        })
    }
}
