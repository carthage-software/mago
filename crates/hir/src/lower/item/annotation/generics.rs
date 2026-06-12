use mago_allocator::Arena;
use mago_phpdoc_syntax::cst::tag::TemplateTagValue;
use mago_phpdoc_syntax::cst::tag::WhereTagValue;
use mago_span::HasSpan;

use crate::ir::item::annotation::generics::TypeParameterAnnotation;
use crate::ir::item::annotation::generics::Variance;
use crate::ir::item::annotation::generics::WhereConstraintAnnotation;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_type_parameter_annotation(
        &mut self,
        template: &'scratch TemplateTagValue<'scratch>,
        variance: Variance,
    ) -> TypeParameterAnnotation<'arena> {
        TypeParameterAnnotation {
            span: template.span(),
            variance,
            name: self.phpdoc_name(&template.name),
            bound: template.bound.as_ref().map(|bound| self.lower_type_annotation(bound.r#type)),
            default: template.default.as_ref().map(|default| self.lower_type_annotation(default.r#type)),
        }
    }

    pub(crate) fn lower_where_constraint_annotation(
        &mut self,
        constraint: &'scratch WhereTagValue<'scratch>,
    ) -> WhereConstraintAnnotation<'arena> {
        WhereConstraintAnnotation {
            span: constraint.span(),
            type_parameter: self.phpdoc_name(&constraint.name),
            constraint: self.lower_type_annotation(constraint.r#type),
        }
    }
}
