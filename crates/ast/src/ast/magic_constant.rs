use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::identifier::LocalIdentifier;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum MagicConstant {
    Line(LocalIdentifier),
    File(LocalIdentifier),
    Directory(LocalIdentifier),
    Trait(LocalIdentifier),
    Method(LocalIdentifier),
    Function(LocalIdentifier),
    Property(LocalIdentifier),
    Namespace(LocalIdentifier),
    Class(LocalIdentifier),
}

impl MagicConstant {
    pub fn value(&self) -> &LocalIdentifier {
        match self {
            MagicConstant::Line(value) => value,
            MagicConstant::File(value) => value,
            MagicConstant::Directory(value) => value,
            MagicConstant::Trait(value) => value,
            MagicConstant::Method(value) => value,
            MagicConstant::Function(value) => value,
            MagicConstant::Property(value) => value,
            MagicConstant::Namespace(value) => value,
            MagicConstant::Class(value) => value,
        }
    }
}

impl HasSpan for MagicConstant {
    fn span(&self) -> Span {
        match self {
            MagicConstant::Line(value) => value.span(),
            MagicConstant::File(value) => value.span(),
            MagicConstant::Directory(value) => value.span(),
            MagicConstant::Trait(value) => value.span(),
            MagicConstant::Method(value) => value.span(),
            MagicConstant::Function(value) => value.span(),
            MagicConstant::Property(value) => value.span(),
            MagicConstant::Namespace(value) => value.span(),
            MagicConstant::Class(value) => value.span(),
        }
    }
}
