use mago_phpdoc_syntax::cst::tag::MethodTagValueParameter;

use crate::ir::parameter::annotation::ParameterAnnotation;
use crate::ir::variable::DirectVariable;
use crate::lower::Lowering;

impl<'arena> Lowering<'arena> {
    pub(crate) fn lower_parameter_annotation(
        &self,
        parameter: &'arena MethodTagValueParameter<'arena>,
    ) -> ParameterAnnotation<'arena, (), (), ()> {
        ParameterAnnotation {
            r#type: parameter.r#type.map(|r#type| self.lower_type_annotation(r#type)),
            is_by_reference: parameter.ampersand.is_some(),
            is_variadic: parameter.ellipsis.is_some(),
            variable: DirectVariable { span: parameter.parameter.span, name: parameter.parameter.value },
            default_value: parameter
                .default
                .as_ref()
                .map(|default| &*self.arena.alloc(self.lower_constant_expression(default.value))),
        }
    }
}
