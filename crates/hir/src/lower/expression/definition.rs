use bumpalo::collections::Vec;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::expression::definition::AnonymousClass;
use crate::ir::expression::definition::ArrowFunction;
use crate::ir::expression::definition::Closure;
use crate::ir::expression::definition::ClosureUseClauseVariable;
use crate::lower::Lowering;

impl<'arena> Lowering<'arena> {
    pub(crate) fn lower_closure(
        &mut self,
        closure: &'arena cst::Closure<'arena>,
    ) -> &'arena Closure<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&closure.attribute_lists);
        let parameters = self.lower_parameter_list(&closure.parameter_list);
        let return_type = closure.return_type_hint.as_ref().map(|hint| self.lower_type(&hint.hint));
        let use_variables: &[ClosureUseClauseVariable<'arena>] = match &closure.use_clause {
            Some(use_clause) => {
                self.arena.alloc_slice_fill_iter(use_clause.variables.iter().map(|variable| ClosureUseClauseVariable {
                    is_by_reference: variable.ampersand.is_some(),
                    variable: self.lower_direct_variable(&variable.variable),
                }))
            }
            None => &[],
        };

        self.arena.alloc(Closure {
            attributes,
            is_static: closure.r#static.is_some(),
            type_parameter_annotations: &[],
            parameters,
            return_by_reference: closure.ampersand.is_some(),
            return_type,
            return_type_annotation: None,
            throws: &[],
            asserts: &[],
            asserts_if_true: &[],
            asserts_if_false: &[],
            use_variables,
            body: self.statements_to_statement(closure.body.statements.as_slice(), closure.body.span()),
        })
    }

    pub(crate) fn lower_arrow_function(
        &mut self,
        arrow_function: &'arena cst::ArrowFunction<'arena>,
    ) -> &'arena ArrowFunction<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&arrow_function.attribute_lists);
        let parameters = self.lower_parameter_list(&arrow_function.parameter_list);
        let return_type = arrow_function.return_type_hint.as_ref().map(|hint| self.lower_type(&hint.hint));

        self.arena.alloc(ArrowFunction {
            attributes,
            is_static: arrow_function.r#static.is_some(),
            type_parameter_annotations: &[],
            parameters,
            return_by_reference: arrow_function.ampersand.is_some(),
            return_type,
            return_type_annotation: None,
            throws: &[],
            asserts: &[],
            asserts_if_true: &[],
            asserts_if_false: &[],
            expression: self.arena.alloc(self.lower_expression(arrow_function.expression)),
        })
    }

    pub(crate) fn lower_anonymous_class(
        &mut self,
        anonymous_class: &'arena cst::AnonymousClass<'arena>,
    ) -> &'arena AnonymousClass<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&anonymous_class.attribute_lists);
        let arguments = match &anonymous_class.argument_list {
            Some(argument_list) => self.lower_argument_list(argument_list),
            None => &[],
        };

        let extends = anonymous_class.extends.as_ref().and_then(|extends| self.lower_extends_one(extends));
        let implements = anonymous_class.implements.as_ref().map(|implements| self.lower_implements(implements));

        let mut trait_uses = Vec::new_in(self.arena);
        let mut constants = Vec::new_in(self.arena);
        let mut properties = Vec::new_in(self.arena);
        let mut hooked_properties = Vec::new_in(self.arena);
        let mut methods = Vec::new_in(self.arena);

        for member in anonymous_class.members.iter() {
            match member {
                cst::ClassLikeMember::TraitUse(trait_use) => trait_uses.push(self.lower_trait_use(trait_use)),
                cst::ClassLikeMember::Constant(constant) => constants.push(self.lower_class_like_constant(constant)),
                cst::ClassLikeMember::Property(cst::Property::Plain(property)) => {
                    properties.push(self.lower_plain_property(property));
                }
                cst::ClassLikeMember::Property(cst::Property::Hooked(property)) => {
                    hooked_properties.push(self.lower_hooked_property(property));
                }
                cst::ClassLikeMember::Method(method) => methods.push(self.lower_method(method)),
                cst::ClassLikeMember::EnumCase(_) => {}
            }
        }

        self.arena.alloc(AnonymousClass {
            attributes,
            arguments,
            extends,
            extends_annotations: &[],
            implements,
            implements_annotations: &[],
            mixin_annotations: &[],
            trait_uses: trait_uses.into_bump_slice(),
            constants: constants.into_bump_slice(),
            properties: properties.into_bump_slice(),
            hooked_properties: hooked_properties.into_bump_slice(),
            methods: methods.into_bump_slice(),
        })
    }
}
