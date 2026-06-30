use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::copy::copy_slice_into;
use mago_hir::ir::item::expression::closure::Closure;
use mago_oracle::id::SymbolId;
use mago_oracle::symbol::function_like::part::parameter::SignatureParameter;
use mago_oracle::ty::Type;
use mago_oracle::var::Var;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::Environment;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub(crate) fn infer_closure(
        &mut self,
        closure: &'source Closure<'source, SymbolId, S, E>,
        signature: &'arena [SignatureParameter<'arena>],
        declared_return: Option<Type<'arena>>,
    ) -> InferenceResult<(Type<'arena>, Closure<'arena, SymbolId, Flow, Type<'arena>>)> {
        let attributes = self.infer_attributes(closure.attributes)?;
        let annotation = match closure.annotation {
            Some(annotation) => Some(&*self.arena.alloc(self.infer_item_annotation(annotation)?)),
            None => None,
        };

        let outer = std::mem::replace(&mut self.environment, Environment::new_in(self.source));
        let reachable = self.reachable;
        self.reachable = true;

        if let Some(use_variables) = closure.use_variables {
            for use_variable in use_variables.items {
                let name = Var::new(self.arena.alloc_slice_copy(use_variable.variable.name));
                let captured = outer.get(name);
                self.environment.set(name, captured);
            }
        }

        self.bind_signature_parameters(closure.parameters.items, signature);

        let parameters = self.infer_parameters(&closure.parameters, None)?;
        let body = self.infer_statement(closure.body)?;

        self.environment = outer;
        self.reachable = reachable;

        let node = Closure {
            span: closure.span,
            name: self.arena.alloc_slice_copy(closure.name),
            annotation,
            attributes,
            flags: closure.flags,
            parameters,
            return_type: closure.return_type.map(|return_type| copy_ref_into(return_type, self.arena)),
            use_variables: closure.use_variables.map(|use_variables| use_variables.copy_into(self.arena)),
            direct_accessed_globals: copy_slice_into(closure.direct_accessed_globals, self.arena),
            body: self.arena.alloc(body),
        };

        let return_type = match declared_return {
            Some(declared) => declared,
            None => self.infer_returned_type(node.body),
        };

        Ok((self.build_callable(signature, return_type), node))
    }
}
