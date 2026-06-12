use mago_allocator::Arena;
use mago_flags::U8Flags;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::identifier::Identifier;
use crate::ir::item::annotation::generics::TypeParameterDefiningEntity;
use crate::ir::item::member::method::Method;
use crate::ir::item::member::method::MethodFlag;
use crate::ir::item::modifier::ModifierKind;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_method(
        &mut self,
        method: &'scratch cst::Method<'scratch>,
        owner: Identifier<'arena>,
    ) -> Method<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&method.attribute_lists);
        let version_constraint = self.lower_version_constraint(&method.attribute_lists);
        let modifiers = self.lower_modifiers(&method.modifiers);
        let name = self.lower_name(&method.name);
        let return_type = method.return_type_hint.as_ref().map(|hint| self.lower_type(&hint.hint));

        let document = self.phpdoc_resolution.get(method.span());
        let is_static = modifiers.iter().any(|modifier| modifier.kind == ModifierKind::Static);
        self.type_resolution.enter_scope_with(TypeParameterDefiningEntity::Method(owner, name), is_static);
        let type_parameters = self.register_item_type_parameters(document.as_ref(), None);

        let parameters = self.lower_parameter_list(&method.parameter_list);
        let outer_effects = self.enter_function_like_body();
        let body = match &method.body {
            cst::MethodBody::Abstract(_) => None,
            cst::MethodBody::Concrete(block) => {
                Some(self.statements_to_statement(block.statements.as_slice(), block.span()))
            }
        };
        let effects = self.leave_function_like_body(outer_effects);

        let return_expression = match &method.body {
            cst::MethodBody::Concrete(block) => self.single_return_expression(block),
            cst::MethodBody::Abstract(_) => None,
        };
        let inferred_assertions = self.infer_function_like_assertions(return_expression, parameters.as_slice());
        let (annotation, assertions_inferred) =
            self.build_item_annotation(document.as_ref(), None, type_parameters, inferred_assertions);

        self.type_resolution.leave_scope();

        let mut flags = U8Flags::new();
        if method.ampersand.is_some() {
            flags.set(MethodFlag::ReturnsByReference);
        }
        if assertions_inferred {
            flags.set(MethodFlag::AssertionsInferred);
        }
        if effects.yields {
            flags.set(MethodFlag::Yields);
        }
        if effects.throws {
            flags.set(MethodFlag::Throws);
        }

        Method {
            span: method.span(),
            annotation,
            attributes,
            version_constraint,
            flags,
            modifiers,
            name,
            parameters,
            return_type,
            body,
            direct_accessed_globals: effects.accessed_globals.leak(),
        }
    }
}
