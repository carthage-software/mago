use mago_allocator::Arena;
use mago_phpdoc_syntax::cst::tag::ExtendsTagValue;
use mago_phpdoc_syntax::cst::tag::ImplementsTagValue;
use mago_phpdoc_syntax::cst::tag::InheritorsTagValue;
use mago_phpdoc_syntax::cst::tag::MixinTagValue;
use mago_phpdoc_syntax::cst::tag::RequireExtendsTagValue;
use mago_phpdoc_syntax::cst::tag::RequireImplementsTagValue;
use mago_phpdoc_syntax::cst::tag::SealedTagValue;
use mago_phpdoc_syntax::cst::tag::UseTagValue;
use mago_span::HasSpan;

use crate::ir::item::annotation::inheritance::ExtendsAnnotation;
use crate::ir::item::annotation::inheritance::ImplementsAnnotation;
use crate::ir::item::annotation::inheritance::MixinAnnotation;
use crate::ir::item::annotation::inheritance::RequireExtendsAnnotation;
use crate::ir::item::annotation::inheritance::RequireImplementsAnnotation;
use crate::ir::item::annotation::inheritance::SealedAnnotation;
use crate::ir::item::annotation::inheritance::UseAnnotation;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_extends_annotation(
        &mut self,
        extends: &'scratch ExtendsTagValue<'scratch>,
    ) -> Option<ExtendsAnnotation<'arena>> {
        Some(ExtendsAnnotation { span: extends.span(), r#type: self.lower_named_type(extends.r#type)? })
    }

    pub(crate) fn lower_implements_annotation(
        &mut self,
        implements: &'scratch ImplementsTagValue<'scratch>,
    ) -> Option<ImplementsAnnotation<'arena>> {
        Some(ImplementsAnnotation { span: implements.span(), r#type: self.lower_named_type(implements.r#type)? })
    }

    pub(crate) fn lower_use_annotation(
        &mut self,
        uses: &'scratch UseTagValue<'scratch>,
    ) -> Option<UseAnnotation<'arena>> {
        Some(UseAnnotation { span: uses.span(), r#type: self.lower_named_type(uses.r#type)? })
    }

    pub(crate) fn lower_require_extends_annotation(
        &mut self,
        require: &'scratch RequireExtendsTagValue<'scratch>,
    ) -> Option<RequireExtendsAnnotation<'arena>> {
        Some(RequireExtendsAnnotation { span: require.span(), r#type: self.lower_named_type(require.r#type)? })
    }

    pub(crate) fn lower_require_implements_annotation(
        &mut self,
        require: &'scratch RequireImplementsTagValue<'scratch>,
    ) -> Option<RequireImplementsAnnotation<'arena>> {
        Some(RequireImplementsAnnotation { span: require.span(), r#type: self.lower_named_type(require.r#type)? })
    }

    pub(crate) fn lower_mixin_annotation(
        &mut self,
        mixin: &'scratch MixinTagValue<'scratch>,
    ) -> MixinAnnotation<'arena> {
        MixinAnnotation { span: mixin.span(), r#type: self.lower_type_annotation_kind(mixin.r#type) }
    }

    pub(crate) fn lower_sealed_annotation(
        &mut self,
        sealed: &'scratch SealedTagValue<'scratch>,
    ) -> SealedAnnotation<'arena> {
        SealedAnnotation { span: sealed.span(), types: self.lower_named_types(sealed.r#type) }
    }

    pub(crate) fn lower_inheritors_annotation(
        &mut self,
        inheritors: &'scratch InheritorsTagValue<'scratch>,
    ) -> SealedAnnotation<'arena> {
        SealedAnnotation { span: inheritors.span(), types: self.lower_named_types(inheritors.r#type) }
    }
}
