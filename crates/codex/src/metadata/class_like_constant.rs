use serde::Deserialize;
use serde::Serialize;

use mago_interner::StringIdentifier;
use mago_span::Span;

use crate::metadata::attribute::AttributeMetadata;
use crate::metadata::ttype::TypeMetadata;
use crate::ttype::atomic::TAtomic;
use crate::visibility::Visibility;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClassLikeConstantMetadata {
    pub attributes: Vec<AttributeMetadata>,
    pub name: StringIdentifier,
    pub span: Span,
    pub visibility: Visibility,
    pub type_metadata: Option<TypeMetadata>,
    pub inferred_type: Option<TAtomic>,
    pub is_final: bool,
    pub is_deprecated: bool,
    pub is_internal: bool,
}

impl ClassLikeConstantMetadata {
    pub fn new(name: StringIdentifier, span: Span, visibility: Visibility) -> Self {
        Self {
            attributes: Vec::new(),
            name,
            span,
            visibility,
            type_metadata: None,
            inferred_type: None,
            is_final: false,
            is_deprecated: false,
            is_internal: false,
        }
    }

    #[inline]
    pub fn with_attributes(mut self, attributes: Vec<AttributeMetadata>) -> Self {
        self.attributes = attributes;
        self
    }

    #[inline]
    pub fn add_attribute(mut self, attribute: AttributeMetadata) -> Self {
        self.attributes.push(attribute);
        self
    }

    #[inline]
    pub fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }

    #[inline]
    pub fn with_type_signature(mut self, type_signature: Option<TypeMetadata>) -> Self {
        self.type_metadata = type_signature;
        self
    }

    #[inline]
    pub fn with_inferred_type(mut self, inferred_type: Option<TAtomic>) -> Self {
        self.inferred_type = inferred_type;
        self
    }

    #[inline]
    pub fn with_final(mut self, is_final: bool) -> Self {
        self.is_final = is_final;
        self
    }

    #[inline]
    pub fn with_deprecated(mut self, is_deprecated: bool) -> Self {
        self.is_deprecated = is_deprecated;
        self
    }

    #[inline]
    pub fn with_internal(mut self, is_internal: bool) -> Self {
        self.is_internal = is_internal;
        self
    }

    #[inline]
    pub fn get_name(&self) -> &StringIdentifier {
        &self.name
    }

    #[inline]
    pub const fn get_span(&self) -> Span {
        self.span
    }

    #[inline]
    pub const fn get_visibility(&self) -> Visibility {
        self.visibility
    }

    #[inline]
    pub fn get_attributes(&self) -> &[AttributeMetadata] {
        &self.attributes
    }

    #[inline]
    pub fn get_type_metadata(&self) -> Option<&TypeMetadata> {
        self.type_metadata.as_ref()
    }

    #[inline]
    pub fn get_type_signature_mut(&mut self) -> Option<&mut TypeMetadata> {
        self.type_metadata.as_mut()
    }

    #[inline]
    pub fn get_inferred_type(&self) -> Option<&TAtomic> {
        self.inferred_type.as_ref()
    }

    #[inline]
    pub fn get_inferred_type_mut(&mut self) -> Option<&mut TAtomic> {
        self.inferred_type.as_mut()
    }

    #[inline]
    pub const fn is_final(&self) -> bool {
        self.is_final
    }

    #[inline]
    pub const fn is_deprecated(&self) -> bool {
        self.is_deprecated
    }

    #[inline]
    pub const fn is_internal(&self) -> bool {
        self.is_internal
    }
}
