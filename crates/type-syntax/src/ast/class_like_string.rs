use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::generics::GenericParameters;
use crate::ast::keyword::Keyword;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct ClassStringType<'input> {
    pub keyword: Keyword<'input>,
    pub parameters: Option<GenericParameters<'input>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct InterfaceStringType<'input> {
    pub keyword: Keyword<'input>,
    pub parameters: Option<GenericParameters<'input>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct EnumStringType<'input> {
    pub keyword: Keyword<'input>,
    pub parameters: Option<GenericParameters<'input>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct TraitStringType<'input> {
    pub keyword: Keyword<'input>,
    pub parameters: Option<GenericParameters<'input>>,
}

impl HasSpan for ClassStringType<'_> {
    fn span(&self) -> Span {
        match &self.parameters {
            Some(parameters) => self.keyword.span.join(parameters.span()),
            None => self.keyword.span,
        }
    }
}

impl HasSpan for InterfaceStringType<'_> {
    fn span(&self) -> Span {
        match &self.parameters {
            Some(parameters) => self.keyword.span.join(parameters.span()),
            None => self.keyword.span,
        }
    }
}

impl HasSpan for EnumStringType<'_> {
    fn span(&self) -> Span {
        match &self.parameters {
            Some(parameters) => self.keyword.span.join(parameters.span()),
            None => self.keyword.span,
        }
    }
}

impl HasSpan for TraitStringType<'_> {
    fn span(&self) -> Span {
        match &self.parameters {
            Some(parameters) => self.keyword.span.join(parameters.span()),
            None => self.keyword.span,
        }
    }
}
