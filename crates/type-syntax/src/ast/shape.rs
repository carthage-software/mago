use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Type;
use crate::ast::generics::GenericParameters;
use crate::ast::keyword::Keyword;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum ShapeTypeKind {
    Array,
    NonEmptyArray,
    AssociativeArray,
    List,
    NonEmptyList,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct ShapeType<'input> {
    pub kind: ShapeTypeKind,
    pub keyword: Keyword<'input>,
    pub left_brace: Span,
    pub fields: Vec<ShapeField<'input>>,
    pub additional_fields: Option<ShapeAdditionalFields<'input>>,
    pub right_brace: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct ShapeField<'input> {
    pub key: Box<Type<'input>>,
    pub question_mark: Option<Span>,
    pub colon: Span,
    pub value: Box<Type<'input>>,
    pub comma: Option<Span>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct ShapeAdditionalFields<'input> {
    pub ellipsis: Span,
    pub parameters: Option<GenericParameters<'input>>,
}

impl HasSpan for ShapeType<'_> {
    fn span(&self) -> Span {
        self.keyword.span().join(self.right_brace)
    }
}

impl HasSpan for ShapeField<'_> {
    fn span(&self) -> Span {
        match &self.comma {
            Some(comma) => self.key.span().join(*comma),
            None => self.key.span(),
        }
    }
}

impl HasSpan for ShapeAdditionalFields<'_> {
    fn span(&self) -> Span {
        match &self.parameters {
            Some(generics) => self.ellipsis.join(generics.span()),
            None => self.ellipsis,
        }
    }
}
