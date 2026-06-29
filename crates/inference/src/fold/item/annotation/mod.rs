mod member;
mod parameter;

use mago_allocator::Arena;
use mago_allocator::copy::copy_slice_into;
use mago_allocator::vec::Vec;
use mago_hir::ir::item::annotation::ItemAnnotation;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;

use crate::error::InferenceResult;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    /// Infers a docblock annotation: the `@method`/`@param` default-value
    /// expressions are inferred, every other tag (which carries only types,
    /// names, or flags) is carried through.
    pub(crate) fn infer_item_annotation(
        &mut self,
        annotation: &ItemAnnotation<'source, SymbolId, S, E>,
    ) -> InferenceResult<ItemAnnotation<'arena, SymbolId, Flow, Type<'arena>>> {
        let mut methods = Vec::new_in(self.arena);
        for method in annotation.methods {
            methods.push(self.infer_method_annotation(method)?);
        }

        let mut parameters = Vec::new_in(self.arena);
        for parameter in annotation.parameters {
            parameters.push(self.infer_parameter_annotation(parameter)?);
        }

        Ok(ItemAnnotation {
            span: annotation.span,
            type_aliases: copy_slice_into(annotation.type_aliases, self.arena),
            imported_type_aliases: copy_slice_into(annotation.imported_type_aliases, self.arena),
            type_parameters: copy_slice_into(annotation.type_parameters, self.arena),
            inherited_type_parameters: copy_slice_into(annotation.inherited_type_parameters, self.arena),
            extends: copy_slice_into(annotation.extends, self.arena),
            require_extends: copy_slice_into(annotation.require_extends, self.arena),
            implements: copy_slice_into(annotation.implements, self.arena),
            require_implements: copy_slice_into(annotation.require_implements, self.arena),
            uses: copy_slice_into(annotation.uses, self.arena),
            sealings: copy_slice_into(annotation.sealings, self.arena),
            mixins: copy_slice_into(annotation.mixins, self.arena),
            methods: methods.leak(),
            properties: copy_slice_into(annotation.properties, self.arena),
            parameters: parameters.leak(),
            parameter_outs: copy_slice_into(annotation.parameter_outs, self.arena),
            where_constraints: copy_slice_into(annotation.where_constraints, self.arena),
            return_type: copy_slice_into(annotation.return_type, self.arena),
            throws: copy_slice_into(annotation.throws, self.arena),
            asserts: copy_slice_into(annotation.asserts, self.arena),
            asserts_if_true: copy_slice_into(annotation.asserts_if_true, self.arena),
            asserts_if_false: copy_slice_into(annotation.asserts_if_false, self.arena),
            self_out: copy_slice_into(annotation.self_out, self.arena),
            pure_unless_callable_impure: copy_slice_into(annotation.pure_unless_callable_impure, self.arena),
            var: copy_slice_into(annotation.var, self.arena),
            tags: annotation.tags,
            errors: self.arena.alloc_slice_copy(annotation.errors),
        })
    }
}
