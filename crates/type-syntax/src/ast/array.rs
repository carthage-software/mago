use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::generics::GenericParameters;
use crate::ast::keyword::Keyword;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ArrayType<'arena> {
    pub keyword: Keyword<'arena>,
    pub parameters: Option<GenericParameters<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct NonEmptyArrayType<'arena> {
    pub keyword: Keyword<'arena>,
    pub parameters: Option<GenericParameters<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct AssociativeArrayType<'arena> {
    pub keyword: Keyword<'arena>,
    pub parameters: Option<GenericParameters<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ListType<'arena> {
    pub keyword: Keyword<'arena>,
    pub parameters: Option<GenericParameters<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct NonEmptyListType<'arena> {
    pub keyword: Keyword<'arena>,
    pub parameters: Option<GenericParameters<'arena>>,
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

impl std::fmt::Display for ArrayType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(parameters) = &self.parameters {
            write!(f, "{}{}", self.keyword, parameters)
        } else {
            write!(f, "{}", self.keyword)
        }
    }
}

impl std::fmt::Display for NonEmptyArrayType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(parameters) = &self.parameters {
            write!(f, "{}{}", self.keyword, parameters)
        } else {
            write!(f, "{}", self.keyword)
        }
    }
}

impl std::fmt::Display for AssociativeArrayType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(parameters) = &self.parameters {
            write!(f, "{}{}", self.keyword, parameters)
        } else {
            write!(f, "{}", self.keyword)
        }
    }
}

impl std::fmt::Display for ListType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(parameters) = &self.parameters {
            write!(f, "{}{}", self.keyword, parameters)
        } else {
            write!(f, "{}", self.keyword)
        }
    }
}

impl std::fmt::Display for NonEmptyListType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(parameters) = &self.parameters {
            write!(f, "{}{}", self.keyword, parameters)
        } else {
            write!(f, "{}", self.keyword)
        }
    }
}
