use mago_allocator::Arena;
use mago_phpdoc_syntax::cst::Document;
use mago_phpdoc_syntax::cst::Element;
use mago_phpdoc_syntax::cst::tag::TagValue;
use mago_phpdoc_syntax::cst::tag::TagVendor;

use crate::ir::r#type::annotation::TypeAnnotation;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_parameter_var_annotation(
        &mut self,
        document: Option<&Document<'scratch>>,
        parameter_name: &[u8],
    ) -> Option<&'arena TypeAnnotation<'arena>> {
        let document = document?;
        let mut annotation = None;
        let mut vendor: Option<TagVendor> = None;
        for element in document.elements.iter() {
            let Element::Tag(tag) = element else { continue };
            let tag = *tag;
            let TagValue::Var(value) = &tag.value else { continue };

            let matches_parameter = match &value.variable {
                Some(variable) => variable.value == parameter_name,
                None => true,
            };

            if matches_parameter && (annotation.is_none() || tag.vendor > vendor) {
                vendor = tag.vendor;
                annotation = Some(self.lower_type_annotation(value.r#type));
            }
        }

        annotation
    }
}
