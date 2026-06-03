use bumpalo::collections::Vec;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::expression::definition::AnonymousClass;
use crate::ir::expression::definition::ArrowFunction;
use crate::ir::expression::definition::Closure;
use crate::ir::expression::definition::ClosureUseClauseVariable;
use crate::ir::generics::TypeParameterDefiningEntity;
use crate::ir::identifier::Identifier;
use crate::ir::identifier::IdentifierKind;
use crate::lower::Lowering;

impl<'arena> Lowering<'arena> {
    pub(crate) fn lower_closure(
        &mut self,
        closure: &'arena cst::Closure<'arena>,
    ) -> &'arena Closure<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&closure.attribute_lists);
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

        let document = self.phpdoc_resolution.get(closure.span());
        self.type_resolution.enter_scope(TypeParameterDefiningEntity::Closure(closure.span()));
        let annotations = self.lower_function_like_annotations(document.as_ref());

        let lowered_parameters = self.lower_parameter_list(&closure.parameter_list);
        let parameters =
            self.merge_parameter_annotations(lowered_parameters, annotations.parameters, annotations.parameter_outs);
        let body = self.statements_to_statement(closure.body.statements.as_slice(), closure.body.span());

        self.type_resolution.leave_scope();

        self.arena.alloc(Closure {
            attributes,
            is_static: closure.r#static.is_some(),
            type_parameter_annotations: annotations.type_parameters,
            parameters,
            return_by_reference: closure.ampersand.is_some(),
            return_type,
            return_type_annotation: annotations.return_type,
            throws_annotations: annotations.throws,
            assert_annotations: annotations.asserts,
            assert_if_true_annotations: annotations.asserts_if_true,
            assert_if_false_annotations: annotations.asserts_if_false,
            use_variables,
            body,
        })
    }

    pub(crate) fn lower_arrow_function(
        &mut self,
        arrow_function: &'arena cst::ArrowFunction<'arena>,
    ) -> &'arena ArrowFunction<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&arrow_function.attribute_lists);
        let return_type = arrow_function.return_type_hint.as_ref().map(|hint| self.lower_type(&hint.hint));

        let document = self.phpdoc_resolution.get(arrow_function.span());
        self.type_resolution.enter_scope(TypeParameterDefiningEntity::Closure(arrow_function.span()));
        let annotations = self.lower_function_like_annotations(document.as_ref());

        let lowered_parameters = self.lower_parameter_list(&arrow_function.parameter_list);
        let parameters =
            self.merge_parameter_annotations(lowered_parameters, annotations.parameters, annotations.parameter_outs);
        let expression = self.arena.alloc(self.lower_expression(arrow_function.expression));

        self.type_resolution.leave_scope();

        self.arena.alloc(ArrowFunction {
            attributes,
            is_static: arrow_function.r#static.is_some(),
            type_parameter_annotations: annotations.type_parameters,
            parameters,
            return_by_reference: arrow_function.ampersand.is_some(),
            return_type,
            return_type_annotation: annotations.return_type,
            throws_annotations: annotations.throws,
            assert_annotations: annotations.asserts,
            assert_if_true_annotations: annotations.asserts_if_true,
            assert_if_false_annotations: annotations.asserts_if_false,
            expression,
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

        let owner = Identifier { span: anonymous_class.span(), value: b"class@anonymous", kind: IdentifierKind::Local };
        let document = self.phpdoc_resolution.get(anonymous_class.span());
        self.type_resolution.enter_scope(TypeParameterDefiningEntity::ClassLike(owner));
        let annotations = self.lower_class_like_annotations(document.as_ref(), owner);

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
                cst::ClassLikeMember::Method(method) => methods.push(self.lower_method(method, owner)),
                cst::ClassLikeMember::EnumCase(_) => {}
            }
        }

        self.type_resolution.leave_scope();

        self.arena.alloc(AnonymousClass {
            attributes,
            arguments,
            extends,
            extends_annotations: annotations.extends,
            implements,
            implements_annotations: annotations.implements,
            mixin_annotations: annotations.mixins,
            trait_uses: trait_uses.into_bump_slice(),
            constants: constants.into_bump_slice(),
            properties: properties.into_bump_slice(),
            hooked_properties: hooked_properties.into_bump_slice(),
            methods: methods.into_bump_slice(),
        })
    }
}
