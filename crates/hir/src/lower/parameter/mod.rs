pub mod annotation;

use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::parameter::Parameter;
use crate::lower::Lowering;

impl<'arena> Lowering<'_, 'arena> {
    pub(crate) fn lower_parameter_list(
        &mut self,
        parameter_list: &'arena cst::FunctionLikeParameterList<'arena>,
    ) -> &'arena [Parameter<'arena, (), (), ()>] {
        self.arena
            .alloc_slice_fill_iter(parameter_list.parameters.iter().map(|parameter| self.lower_parameter(parameter)))
    }

    fn lower_parameter(
        &mut self,
        parameter: &'arena cst::FunctionLikeParameter<'arena>,
    ) -> Parameter<'arena, (), (), ()> {
        let attributes = self.lower_attribute_lists(&parameter.attribute_lists);
        let modifiers = self.lower_modifiers(&parameter.modifiers);
        let r#type = parameter.hint.as_ref().map(|hint| self.lower_type(hint));
        let variable = self.lower_direct_variable(&parameter.variable);

        let document = self.phpdoc_resolution.get(parameter.span());
        let type_annotation = self.lower_parameter_var_annotation(document.as_ref(), variable.name);

        let default_value = match &parameter.default_value {
            Some(default) => Some(&*self.arena.alloc(self.lower_expression(default.value))),
            None => None,
        };

        let hooks = match &parameter.hooks {
            Some(hook_list) => self.lower_property_hooks(hook_list),
            None => &[],
        };

        Parameter {
            attributes,
            modifiers,
            r#type,
            type_annotation,
            out_annotation: None,
            is_by_reference: parameter.ampersand.is_some(),
            is_variadic: parameter.ellipsis.is_some(),
            variable,
            default_value,
            hooks,
        }
    }
}
