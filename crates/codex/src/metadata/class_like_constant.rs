use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_span::HasSpan;
use mago_span::Span;

use crate::metadata::attribute::AttributeMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::ttype::TypeMetadata;
use crate::ttype::atomic::TAtomic;
use crate::visibility::Visibility;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClassLikeConstantMetadata {
    pub attributes: Vec<AttributeMetadata>,
    pub name: Atom,
    pub span: Span,
    pub visibility: Visibility,
    pub type_declaration: Option<TypeMetadata>,
    pub type_metadata: Option<TypeMetadata>,
    pub inferred_type: Option<TAtomic>,
    pub flags: MetadataFlags,
}

impl ClassLikeConstantMetadata {
    pub fn new(name: Atom, span: Span, visibility: Visibility, flags: MetadataFlags) -> Self {
        Self {
            attributes: Vec::new(),
            name,
            span,
            visibility,
            type_declaration: None,
            type_metadata: None,
            inferred_type: None,
            flags,
        }
    }

    pub fn set_type_declaration(&mut self, type_declaration: TypeMetadata) {
        if self.type_metadata.is_none() {
            self.type_metadata = Some(type_declaration.clone());
        }

        self.type_declaration = Some(type_declaration);
    }
}

impl HasSpan for ClassLikeConstantMetadata {
    fn span(&self) -> Span {
        self.span
    }
}
