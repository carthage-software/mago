use mago_phpdoc_syntax::cst::tag::ExtendsTagValue;
use mago_phpdoc_syntax::cst::tag::ImplementsTagValue;
use mago_phpdoc_syntax::cst::tag::InheritorsTagValue;
use mago_phpdoc_syntax::cst::tag::MixinTagValue;
use mago_phpdoc_syntax::cst::tag::RequireExtendsTagValue;
use mago_phpdoc_syntax::cst::tag::RequireImplementsTagValue;
use mago_phpdoc_syntax::cst::tag::SealedTagValue;
use mago_phpdoc_syntax::cst::tag::UsesTagValue;
use mago_span::HasSpan;

use crate::ir::inheritance::annotation::ExtendsAnnotation;
use crate::ir::inheritance::annotation::ImplementsAnnotation;
use crate::ir::inheritance::annotation::MixinAnnotation;
use crate::ir::inheritance::annotation::RequireExtendsAnnotation;
use crate::ir::inheritance::annotation::RequireImplementsAnnotation;
use crate::ir::inheritance::annotation::SealedAnnotation;
use crate::ir::inheritance::annotation::UseAnnotation;
use crate::lower::Lowering;

impl<'arena> Lowering<'_, 'arena> {
    pub(crate) fn lower_extends_annotation(
        &self,
        extends: &'arena ExtendsTagValue<'arena>,
    ) -> Option<ExtendsAnnotation<'arena>> {
        Some(ExtendsAnnotation { span: extends.span(), r#type: self.lower_named_type(extends.r#type)? })
    }

    pub(crate) fn lower_implements_annotation(
        &self,
        implements: &'arena ImplementsTagValue<'arena>,
    ) -> Option<ImplementsAnnotation<'arena>> {
        Some(ImplementsAnnotation { span: implements.span(), r#type: self.lower_named_type(implements.r#type)? })
    }

    pub(crate) fn lower_use_annotation(&self, uses: &'arena UsesTagValue<'arena>) -> Option<UseAnnotation<'arena>> {
        Some(UseAnnotation { span: uses.span(), r#type: self.lower_named_type(uses.r#type)? })
    }

    pub(crate) fn lower_require_extends_annotation(
        &self,
        require: &'arena RequireExtendsTagValue<'arena>,
    ) -> Option<RequireExtendsAnnotation<'arena>> {
        Some(RequireExtendsAnnotation { span: require.span(), r#type: self.lower_named_type(require.r#type)? })
    }

    pub(crate) fn lower_require_implements_annotation(
        &self,
        require: &'arena RequireImplementsTagValue<'arena>,
    ) -> Option<RequireImplementsAnnotation<'arena>> {
        Some(RequireImplementsAnnotation { span: require.span(), r#type: self.lower_named_type(require.r#type)? })
    }

    pub(crate) fn lower_mixin_annotation(&self, mixin: &'arena MixinTagValue<'arena>) -> MixinAnnotation<'arena> {
        MixinAnnotation { span: mixin.span(), r#type: self.lower_type_annotation_kind(mixin.r#type) }
    }

    pub(crate) fn lower_sealed_annotation(&self, sealed: &'arena SealedTagValue<'arena>) -> SealedAnnotation<'arena> {
        SealedAnnotation { span: sealed.span(), types: self.lower_named_types(sealed.r#type) }
    }

    pub(crate) fn lower_inheritors_annotation(
        &self,
        inheritors: &'arena InheritorsTagValue<'arena>,
    ) -> SealedAnnotation<'arena> {
        SealedAnnotation { span: inheritors.span(), types: self.lower_named_types(inheritors.r#type) }
    }
}
