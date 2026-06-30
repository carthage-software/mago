use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::copy::copy_slice_into;
use mago_hir::ir::item::statement::function::Function;
use mago_oracle::id::SymbolId;
use mago_oracle::symbol::function_like::FunctionLikeSymbol;
use mago_oracle::symbol::function_like::part::parameter::SignatureParameter;
use mago_oracle::ty::Type;

use crate::error::InferenceError;
use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::Environment;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub(crate) fn infer_function(
        &mut self,
        function: &'source Function<'source, SymbolId, S, E>,
    ) -> InferenceResult<Function<'arena, SymbolId, Flow, Type<'arena>>> {
        let attributes = self.infer_attributes(function.attributes)?;
        let annotation = match function.annotation {
            Some(annotation) => Some(&*self.arena.alloc(self.infer_item_annotation(annotation)?)),
            None => None,
        };
        let signature = self.function_signature(function)?;

        let outer_environment = std::mem::replace(&mut self.environment, Environment::new_in(self.source));
        let outer_reachable = self.reachable;
        self.reachable = true;

        self.bind_signature_parameters(function.parameters.items, signature);

        let parameters = self.infer_parameters(&function.parameters)?;
        let body = self.infer_statement(function.body)?;

        self.environment = outer_environment;
        self.reachable = outer_reachable;

        Ok(Function {
            span: function.span,
            annotation,
            attributes,
            version_constraint: self.arena.alloc_slice_copy(function.version_constraint),
            flags: function.flags,
            name: function.name.copy_into(self.arena),
            parameters,
            return_type: function.return_type.map(|return_type| copy_ref_into(return_type, self.arena)),
            direct_accessed_globals: copy_slice_into(function.direct_accessed_globals, self.arena),
            body: self.arena.alloc(body),
        })
    }

    fn function_signature(
        &self,
        function: &'source Function<'source, SymbolId, S, E>,
    ) -> InferenceResult<&'arena [SignatureParameter<'arena>]> {
        match self.resolve_function_symbol(&function.name) {
            Some(FunctionLikeSymbol::Function(symbol)) => Ok(symbol.params),
            _ => Err(InferenceError::UnresolvedItemSymbol { span: function.span, kind: "function" }),
        }
    }
}
