use mago_allocator::Arena;
use mago_flags::U8Flags;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::delimited::Delimited;
use crate::ir::item::parameter::Parameter;
use crate::ir::item::parameter::ParameterFlag;
use crate::ir::variable::annotation::VariableAnnotation;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_parameter_list(
        &mut self,
        parameter_list: &'scratch cst::FunctionLikeParameterList<'scratch>,
    ) -> Delimited<'arena, Parameter<'arena, (), (), ()>> {
        Delimited {
            span: parameter_list.left_parenthesis.join(parameter_list.right_parenthesis),
            items: self.arena.alloc_slice_fill_iter(
                parameter_list.parameters.iter().map(|parameter| self.lower_parameter(parameter)),
            ),
        }
    }

    fn lower_parameter(
        &mut self,
        parameter: &'scratch cst::FunctionLikeParameter<'scratch>,
    ) -> Parameter<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&parameter.attribute_lists);
        let modifiers = self.lower_modifiers(&parameter.modifiers);
        let r#type = parameter.hint.as_ref().map(|hint| self.lower_type(hint));
        let variable = self.lower_direct_variable(&parameter.variable);

        let document = self.phpdoc_resolution.get(parameter.span());
        let annotation = self.lower_parameter_var_annotation(document.as_ref(), variable.name).map(|type_annotation| {
            &*self.arena.alloc(VariableAnnotation {
                span: parameter.span(),
                type_annotation,
                variable: Some(variable),
                errors: self.lower_annotation_errors(document.as_ref()),
            })
        });

        let default_value = match &parameter.default_value {
            Some(default) => Some(&*self.arena.alloc(self.lower_expression(default.value))),
            None => None,
        };

        let hooks = parameter.hooks.as_ref().map(|hook_list| self.lower_property_hooks(hook_list));

        let mut flags = U8Flags::new();
        if parameter.ampersand.is_some() {
            flags.set(ParameterFlag::ByReference);
        }
        if parameter.ellipsis.is_some() {
            flags.set(ParameterFlag::IsVariadic);
        }

        Parameter {
            span: parameter.span(),
            annotation,
            attributes,
            flags,
            version_constraint: &[],
            modifiers,
            r#type,
            variable,
            default_value,
            hooks,
        }
    }
}
