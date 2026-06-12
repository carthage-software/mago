use mago_allocator::Arena;
use mago_phpdoc_syntax::cst::tag::AssertPattern;
use mago_phpdoc_syntax::cst::tag::AssertSubject;
use mago_phpdoc_syntax::cst::tag::AssertTagValue;
use mago_phpdoc_syntax::cst::tag::SelfOutTagValue;
use mago_phpdoc_syntax::cst::tag::ThrowsTagValue;
use mago_span::HasSpan;

use crate::ir::item::annotation::effect::AssertAnnotation;
use crate::ir::item::annotation::effect::AssertAnnotationPattern;
use crate::ir::item::annotation::effect::AssertAnnotationPatternKind;
use crate::ir::item::annotation::effect::AssertAnnotationTarget;
use crate::ir::item::annotation::effect::AssertAnnotationTargetKind;
use crate::ir::item::annotation::effect::SelfOutAnnotation;
use crate::ir::item::annotation::effect::ThrowsAnnotation;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_throws_annotation(
        &mut self,
        throws: &'scratch ThrowsTagValue<'scratch>,
    ) -> ThrowsAnnotation<'arena> {
        ThrowsAnnotation { span: throws.span(), r#type: self.lower_type_annotation(throws.r#type) }
    }

    pub(crate) fn lower_assert_annotation(
        &mut self,
        assert: &'scratch AssertTagValue<'scratch>,
    ) -> AssertAnnotation<'arena> {
        let target = match &assert.subject {
            AssertSubject::Parameter { variable } => AssertAnnotationTarget {
                span: variable.span,
                kind: AssertAnnotationTargetKind::Variable(self.phpdoc_variable(variable)),
            },
            AssertSubject::Method { parameter, method, .. } => AssertAnnotationTarget {
                span: parameter.span,
                kind: AssertAnnotationTargetKind::Method(self.phpdoc_variable(parameter), self.phpdoc_name(method)),
            },
            AssertSubject::Property { parameter, property, .. } => AssertAnnotationTarget {
                span: parameter.span,
                kind: AssertAnnotationTargetKind::Property(self.phpdoc_variable(parameter), self.phpdoc_name(property)),
            },
        };

        AssertAnnotation {
            span: assert.span(),
            negated: assert.is_negated(),
            equality: assert.is_equality(),
            pattern: self.lower_assert_pattern_annotation(&assert.pattern),
            target,
        }
    }

    fn lower_assert_pattern_annotation(
        &mut self,
        pattern: &AssertPattern<'scratch>,
    ) -> AssertAnnotationPattern<'arena> {
        AssertAnnotationPattern {
            span: pattern.span(),
            kind: match pattern {
                AssertPattern::Type(ty) => AssertAnnotationPatternKind::Type(self.lower_type_annotation(ty)),
                AssertPattern::Truthy(_) => AssertAnnotationPatternKind::Truthy,
                AssertPattern::Falsy(_) => AssertAnnotationPatternKind::Falsy,
                AssertPattern::NonEmpty(_) => AssertAnnotationPatternKind::NonEmpty,
            },
        }
    }

    pub(crate) fn lower_self_out_annotation(
        &mut self,
        self_out: &'scratch SelfOutTagValue<'scratch>,
    ) -> SelfOutAnnotation<'arena> {
        SelfOutAnnotation { span: self_out.span(), r#type: self.lower_type_annotation(self_out.r#type) }
    }
}
