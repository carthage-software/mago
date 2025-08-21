use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::identifier::LocalIdentifier;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum MagicConstant<'arena> {
    Line(LocalIdentifier<'arena>),
    File(LocalIdentifier<'arena>),
    Directory(LocalIdentifier<'arena>),
    Trait(LocalIdentifier<'arena>),
    Method(LocalIdentifier<'arena>),
    Function(LocalIdentifier<'arena>),
    Property(LocalIdentifier<'arena>),
    Namespace(LocalIdentifier<'arena>),
    Class(LocalIdentifier<'arena>),
}

impl<'arena> MagicConstant<'arena> {
    pub fn value(&self) -> &LocalIdentifier<'arena> {
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

impl HasSpan for MagicConstant<'_> {
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
