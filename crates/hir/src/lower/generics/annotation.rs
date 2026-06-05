use mago_phpdoc_syntax::cst::tag::TemplateTagValue;
use mago_phpdoc_syntax::cst::tag::WhereTagValue;
use mago_span::HasSpan;

use crate::ir::generics::Variance;
use crate::ir::generics::annotation::TypeParameterAnnotation;
use crate::ir::generics::annotation::WhereConstraintAnnotation;
use crate::lower::Lowering;

impl<'arena> Lowering<'_, 'arena> {
    pub(crate) fn lower_type_parameter_annotation(
        &self,
        template: &'arena TemplateTagValue<'arena>,
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
        &self,
        constraint: &'arena WhereTagValue<'arena>,
    ) -> WhereConstraintAnnotation<'arena> {
        WhereConstraintAnnotation {
            span: constraint.span(),
            type_parameter: self.phpdoc_name(&constraint.name),
            constraint: self.lower_type_annotation(constraint.r#type),
        }
    }
}
