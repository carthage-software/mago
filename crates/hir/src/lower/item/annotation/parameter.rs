use mago_allocator::Arena;
use mago_phpdoc_syntax::cst::tag::MethodTagValueParameter;
use mago_span::HasSpan;

use crate::ir::item::annotation::parameter::ParameterAnnotation;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_parameter_annotation(
        &mut self,
        parameter: &'scratch MethodTagValueParameter<'scratch>,
    ) -> ParameterAnnotation<'arena, (), (), ()> {
        ParameterAnnotation {
            span: parameter.span(),
            r#type: parameter.r#type.map(|r#type| self.lower_type_annotation(r#type)),
            is_by_reference: parameter.ampersand.is_some(),
            is_variadic: parameter.ellipsis.is_some(),
            variable: self.phpdoc_variable(&parameter.parameter),
            default_value: match parameter.default.as_ref() {
                Some(default) => {
                    let default = self.lower_constant_expression(default.value);

                    Some(self.arena.alloc(default))
                }
                _ => None,
            },
        }
    }
}
