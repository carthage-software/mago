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
        let mut var_annotation = None;
        let mut var_vendor: Option<TagVendor> = None;
        let mut param_annotation = None;
        let mut param_vendor: Option<TagVendor> = None;
        for element in document.elements.iter() {
            let Element::Tag(tag) = element else { continue };
            let tag = *tag;

            let (r#type, variable) = match &tag.value {
                TagValue::Var(value) => (value.r#type, value.variable),
                TagValue::Param(value) => (value.r#type, value.parameter),
                _ => continue,
            };

            let matches_parameter = match &variable {
                Some(variable) => variable.value == parameter_name,
                None => true,
            };

            if !matches_parameter {
                continue;
            }

            if matches!(tag.value, TagValue::Var(_)) {
                if var_annotation.is_none() || tag.vendor > var_vendor {
                    var_vendor = tag.vendor;
                    var_annotation = Some(self.lower_type_annotation(r#type));
                }
            } else if param_annotation.is_none() || tag.vendor > param_vendor {
                param_vendor = tag.vendor;
                param_annotation = Some(self.lower_type_annotation(r#type));
            }
        }

        var_annotation.or(param_annotation)
    }
}
