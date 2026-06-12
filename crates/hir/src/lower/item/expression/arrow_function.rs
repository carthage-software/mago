use mago_allocator::Arena;
use mago_flags::U8Flags;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::item::annotation::generics::TypeParameterDefiningEntity;
use crate::ir::item::expression::arrow_function::ArrowFunction;
use crate::ir::item::expression::arrow_function::ArrowFunctionFlag;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_arrow_function(
        &mut self,
        arrow_function: &'scratch cst::ArrowFunction<'scratch>,
    ) -> &'arena ArrowFunction<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&arrow_function.attribute_lists);
        let return_type = arrow_function.return_type_hint.as_ref().map(|hint| self.lower_type(&hint.hint));

        let document = self.phpdoc_resolution.get(arrow_function.span());
        self.type_resolution.enter_scope(TypeParameterDefiningEntity::Closure(arrow_function.span()));
        let type_parameters = self.register_item_type_parameters(document.as_ref(), None);

        let parameters = self.lower_parameter_list(&arrow_function.parameter_list);
        let inferred_assertions =
            self.infer_function_like_assertions(Some(arrow_function.expression), parameters.as_slice());
        let (annotation, assertions_inferred) =
            self.build_item_annotation(document.as_ref(), None, type_parameters, inferred_assertions);

        let outer_effects = self.enter_function_like_body();
        let expression = self.arena.alloc(self.lower_expression(arrow_function.expression));
        let effects = self.leave_function_like_body(outer_effects);

        self.type_resolution.leave_scope();

        let mut flags = U8Flags::new();
        if arrow_function.r#static.is_some() {
            flags.set(ArrowFunctionFlag::Static);
        }
        if arrow_function.ampersand.is_some() {
            flags.set(ArrowFunctionFlag::ReturnsByReference);
        }
        if assertions_inferred {
            flags.set(ArrowFunctionFlag::AssertionsInferred);
        }
        if effects.yields {
            flags.set(ArrowFunctionFlag::Yields);
        }
        if effects.throws {
            flags.set(ArrowFunctionFlag::Throws);
        }

        self.arena.alloc(ArrowFunction {
            span: arrow_function.span(),
            annotation,
            attributes,
            flags,
            parameters,
            return_type,
            expression,
        })
    }
}
