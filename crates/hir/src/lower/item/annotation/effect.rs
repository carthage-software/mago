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
        AssertAnnotation {
            span: assert.span(),
            negated: assert.is_negated(),
            equality: assert.is_equality(),
            pattern: self.lower_assert_pattern_annotation(&assert.pattern),
            target: self.lower_assert_target_annotation(&assert.subject),
        }
    }

    fn lower_assert_target_annotation(&mut self, subject: &AssertSubject<'scratch>) -> AssertAnnotationTarget<'arena> {
        let kind = match subject {
            AssertSubject::Parameter { variable } => {
                AssertAnnotationTargetKind::Variable(self.phpdoc_variable(variable))
            }
            AssertSubject::Method { object, method, .. } => {
                let object = self.lower_assert_target_annotation(object);

                AssertAnnotationTargetKind::Method(self.arena.alloc(object), self.phpdoc_name(method))
            }
            AssertSubject::Property { object, property, .. } => {
                let object = self.lower_assert_target_annotation(object);

                AssertAnnotationTargetKind::Property(self.arena.alloc(object), self.phpdoc_name(property))
            }
        };

        AssertAnnotationTarget { span: subject.span(), kind }
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
