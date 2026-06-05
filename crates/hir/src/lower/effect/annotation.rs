use mago_phpdoc_syntax::cst::tag::AssertTagMethodValue;
use mago_phpdoc_syntax::cst::tag::AssertTagPropertyValue;
use mago_phpdoc_syntax::cst::tag::AssertTagValue;
use mago_phpdoc_syntax::cst::tag::SelfOutTagValue;
use mago_phpdoc_syntax::cst::tag::ThrowsTagValue;
use mago_span::HasSpan;

use crate::ir::effect::annotation::AssertAnnotation;
use crate::ir::effect::annotation::AssertAnnotationTarget;
use crate::ir::effect::annotation::SelfOutAnnotation;
use crate::ir::effect::annotation::ThrowsAnnotation;
use crate::ir::variable::DirectVariable;
use crate::lower::Lowering;

impl<'arena> Lowering<'_, 'arena> {
    pub(crate) fn lower_throws_annotation(&self, throws: &'arena ThrowsTagValue<'arena>) -> ThrowsAnnotation<'arena> {
        ThrowsAnnotation { span: throws.span(), r#type: self.lower_type_annotation(throws.r#type) }
    }

    pub(crate) fn lower_assert_annotation(&self, assert: &'arena AssertTagValue<'arena>) -> AssertAnnotation<'arena> {
        AssertAnnotation {
            span: assert.span(),
            negated: assert.is_negated(),
            equality: assert.is_equality(),
            r#type: self.lower_type_annotation(assert.r#type),
            target: AssertAnnotationTarget::Variable(DirectVariable {
                span: assert.parameter.span,
                name: assert.parameter.value,
            }),
        }
    }

    pub(crate) fn lower_assert_method_annotation(
        &self,
        assert: &'arena AssertTagMethodValue<'arena>,
    ) -> AssertAnnotation<'arena> {
        AssertAnnotation {
            span: assert.span(),
            negated: assert.is_negated(),
            equality: assert.is_equality(),
            r#type: self.lower_type_annotation(assert.r#type),
            target: AssertAnnotationTarget::Method(
                DirectVariable { span: assert.parameter.span, name: assert.parameter.value },
                self.phpdoc_name(&assert.method),
            ),
        }
    }

    pub(crate) fn lower_assert_property_annotation(
        &self,
        assert: &'arena AssertTagPropertyValue<'arena>,
    ) -> AssertAnnotation<'arena> {
        AssertAnnotation {
            span: assert.span(),
            negated: assert.is_negated(),
            equality: assert.is_equality(),
            r#type: self.lower_type_annotation(assert.r#type),
            target: AssertAnnotationTarget::Property(
                DirectVariable { span: assert.parameter.span, name: assert.parameter.value },
                self.phpdoc_name(&assert.property),
            ),
        }
    }

    pub(crate) fn lower_self_out_annotation(
        &self,
        self_out: &'arena SelfOutTagValue<'arena>,
    ) -> SelfOutAnnotation<'arena> {
        SelfOutAnnotation { span: self_out.span(), r#type: self.lower_type_annotation(self_out.r#type) }
    }
}
