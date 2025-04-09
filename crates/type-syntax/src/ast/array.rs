use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::generics::GenericParameters;
use crate::ast::keyword::Keyword;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct ArrayType<'input> {
    pub keyword: Keyword<'input>,
    pub parameters: Option<GenericParameters<'input>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct NonEmptyArrayType<'input> {
    pub keyword: Keyword<'input>,
    pub parameters: Option<GenericParameters<'input>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct AssociativeArrayType<'input> {
    pub keyword: Keyword<'input>,
    pub parameters: Option<GenericParameters<'input>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct ListType<'input> {
    pub keyword: Keyword<'input>,
    pub parameters: Option<GenericParameters<'input>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct NonEmptyListType<'input> {
    pub keyword: Keyword<'input>,
    pub parameters: Option<GenericParameters<'input>>,
}

impl HasSpan for ArrayType<'_> {
    fn span(&self) -> Span {
        match &self.parameters {
            Some(parameters) => self.keyword.span.join(parameters.span()),
            None => self.keyword.span,
        }
    }
}

impl HasSpan for NonEmptyArrayType<'_> {
    fn span(&self) -> Span {
        match &self.parameters {
            Some(parameters) => self.keyword.span.join(parameters.span()),
            None => self.keyword.span,
        }
    }
}

impl HasSpan for AssociativeArrayType<'_> {
    fn span(&self) -> Span {
        match &self.parameters {
            Some(parameters) => self.keyword.span.join(parameters.span()),
            None => self.keyword.span,
        }
    }
}

impl HasSpan for ListType<'_> {
    fn span(&self) -> Span {
        match &self.parameters {
            Some(parameters) => self.keyword.span.join(parameters.span()),
            None => self.keyword.span,
        }
    }
}

impl HasSpan for NonEmptyListType<'_> {
    fn span(&self) -> Span {
        match &self.parameters {
            Some(parameters) => self.keyword.span.join(parameters.span()),
            None => self.keyword.span,
        }
    }
}
