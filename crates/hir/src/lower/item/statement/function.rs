use mago_allocator::Arena;
use mago_flags::U8Flags;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::item::annotation::generics::TypeParameterDefiningEntity;
use crate::ir::item::statement::function::Function;
use crate::ir::item::statement::function::FunctionFlag;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_function(
        &mut self,
        function: &'scratch cst::Function<'scratch>,
    ) -> &'arena Function<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&function.attribute_lists);
        let version_constraint = self.lower_version_constraint(&function.attribute_lists);
        let name = self.lower_declaration_name(&function.name);
        let return_type = function.return_type_hint.as_ref().map(|hint| self.lower_type(&hint.hint));

        let document = self.phpdoc_resolution.get(function.span());
        self.type_resolution.enter_scope(TypeParameterDefiningEntity::Function(name));
        let type_parameters = self.register_item_type_parameters(document.as_ref(), None);

        let parameters = self.lower_parameter_list(&function.parameter_list);

        let outer_effects = self.enter_function_like_body();
        let body = self.arena.alloc(self.lower_block(&function.body));
        let effects = self.leave_function_like_body(outer_effects);

        let return_expression = self.single_return_expression(&function.body);
        let inferred_assertions = self.infer_function_like_assertions(return_expression, parameters.as_slice());
        let (annotation, assertions_inferred) =
            self.build_item_annotation(document.as_ref(), None, type_parameters, inferred_assertions);

        self.type_resolution.leave_scope();

        let mut flags = U8Flags::new();
        if function.ampersand.is_some() {
            flags.set(FunctionFlag::ReturnsByReference);
        }
        if assertions_inferred {
            flags.set(FunctionFlag::AssertionsInferred);
        }
        if effects.yields {
            flags.set(FunctionFlag::Yields);
        }
        if effects.throws {
            flags.set(FunctionFlag::Throws);
        }

        self.arena.alloc(Function {
            span: function.span(),
            annotation,
            attributes,
            flags,
            version_constraint,
            name,
            parameters,
            return_type,
            body,
            direct_accessed_globals: effects.accessed_globals.leak(),
        })
    }
}
