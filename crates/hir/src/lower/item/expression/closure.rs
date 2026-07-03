use mago_allocator::Arena;
use mago_flags::U8Flags;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::delimited::Delimited;
use crate::ir::item::annotation::generics::TypeParameterDefiningEntity;
use crate::ir::item::expression::closure::Closure;
use crate::ir::item::expression::closure::ClosureFlag;
use crate::ir::item::expression::closure::ClosureUseClauseVariable;
use crate::ir::item::expression::closure::ClosureUseClauseVariableFlag;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_closure(
        &mut self,
        closure: &'scratch cst::Closure<'scratch>,
    ) -> &'arena Closure<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&closure.attribute_lists);
        let return_type = closure.return_type_hint.as_ref().map(|hint| self.lower_type(&hint.hint));
        let use_variables = closure.use_clause.as_ref().map(|use_clause| Delimited {
            span: use_clause.left_parenthesis.join(use_clause.right_parenthesis),
            items: self.arena.alloc_slice_fill_iter(use_clause.variables.iter().map(|variable| {
                let mut flags = U8Flags::new();
                if variable.ampersand.is_some() {
                    flags.set(ClosureUseClauseVariableFlag::ByReference);
                }

                ClosureUseClauseVariable {
                    span: variable.span(),
                    flags,
                    variable: self.lower_direct_variable(&variable.variable),
                }
            })),
        });

        let document = self.phpdoc_resolution.get(closure.span());
        self.type_resolution.enter_scope(TypeParameterDefiningEntity::Closure(closure.span()));
        let type_parameters = self.register_item_type_parameters(document.as_ref(), None);

        let parameters = self.lower_parameter_list(&closure.parameter_list);

        let outer_effects = self.enter_function_like_body();
        let body = self.arena.alloc(self.lower_block(&closure.body));
        let effects = self.leave_function_like_body(outer_effects);

        let return_expression = self.single_return_expression(&closure.body);
        let inferred_assertions = self.infer_function_like_assertions(return_expression, parameters.as_slice());
        let (annotation, assertions_inferred) =
            self.build_item_annotation(document.as_ref(), None, type_parameters, inferred_assertions);

        self.type_resolution.leave_scope();

        let mut flags = U8Flags::new();
        if closure.r#static.is_some() {
            flags.set(ClosureFlag::Static);
        }

        if closure.ampersand.is_some() {
            flags.set(ClosureFlag::ReturnsByReference);
        }

        if assertions_inferred {
            flags.set(ClosureFlag::AssertionsInferred);
        }

        if effects.yields {
            flags.set(ClosureFlag::Yields);
        }

        if effects.throws {
            flags.set(ClosureFlag::Throws);
        }

        self.arena.alloc(Closure {
            span: closure.span(),
            name: self.build_synthetic_name(b"closure", closure.span()),
            annotation,
            attributes,
            flags,
            parameters,
            return_type,
            use_variables,
            body,
            direct_accessed_globals: effects.accessed_globals.leak(),
        })
    }
}
