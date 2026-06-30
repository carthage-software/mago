use mago_allocator::Arena;
use mago_allocator::copy::copy_ref_into;
use mago_hir::ir::item::expression::arrow_function::ArrowFunction;
use mago_oracle::id::SymbolId;
use mago_oracle::symbol::function_like::part::parameter::SignatureParameter;
use mago_oracle::ty::Type;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub(crate) fn infer_arrow_function(
        &mut self,
        arrow: &'source ArrowFunction<'source, SymbolId, S, E>,
        signature: &'arena [SignatureParameter<'arena>],
        declared_return: Option<Type<'arena>>,
    ) -> InferenceResult<(Type<'arena>, ArrowFunction<'arena, SymbolId, Flow, Type<'arena>>)> {
        let attributes = self.infer_attributes(arrow.attributes)?;
        let annotation = match arrow.annotation {
            Some(annotation) => Some(&*self.arena.alloc(self.infer_item_annotation(annotation)?)),
            None => None,
        };

        let outer = self.environment.clone();
        let reachable = self.reachable;
        self.bind_signature_parameters(arrow.parameters.items, signature);

        let parameters = self.infer_parameters(&arrow.parameters, None)?;
        let expression = self.infer_expression(arrow.expression)?;

        self.environment = outer;
        self.reachable = reachable;

        let return_type = declared_return.unwrap_or(expression.meta);
        let node = ArrowFunction {
            span: arrow.span,
            name: self.arena.alloc_slice_copy(arrow.name),
            annotation,
            attributes,
            flags: arrow.flags,
            parameters,
            return_type: arrow.return_type.map(|return_type| copy_ref_into(return_type, self.arena)),
            expression: self.arena.alloc(expression),
        };

        Ok((self.build_callable(signature, return_type), node))
    }
}
