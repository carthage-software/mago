use mago_hir::ir::attribute::Attribute;
use mago_word::word;

use crate::metadata::attribute::AttributeMetadata;

#[must_use]
pub fn scan_attributes(attributes: &[Attribute<'_, (), (), ()>]) -> Vec<AttributeMetadata> {
    attributes
        .iter()
        .map(|attribute| AttributeMetadata { name: word(attribute.class.value), span: attribute.span })
        .collect()
}
