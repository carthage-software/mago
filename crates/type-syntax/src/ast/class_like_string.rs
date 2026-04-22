use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::generics::SingleGenericParameter;
use crate::ast::keyword::Keyword;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ClassStringType<'arena> {
    pub keyword: Keyword<'arena>,
    pub parameter: Option<SingleGenericParameter<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct InterfaceStringType<'arena> {
    pub keyword: Keyword<'arena>,
    pub parameter: Option<SingleGenericParameter<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct EnumStringType<'arena> {
    pub keyword: Keyword<'arena>,
    pub parameter: Option<SingleGenericParameter<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TraitStringType<'arena> {
    pub keyword: Keyword<'arena>,
    pub parameter: Option<SingleGenericParameter<'arena>>,
}

impl HasSpan for ClassStringType<'_> {
    fn span(&self) -> Span {
        match &self.parameter {
            Some(parameter) => self.keyword.span.join(parameter.span()),
            None => self.keyword.span,
        }
    }
}

impl HasSpan for InterfaceStringType<'_> {
    fn span(&self) -> Span {
        match &self.parameter {
            Some(parameter) => self.keyword.span.join(parameter.span()),
            None => self.keyword.span,
        }
    }
}

impl HasSpan for EnumStringType<'_> {
    fn span(&self) -> Span {
        match &self.parameter {
            Some(parameter) => self.keyword.span.join(parameter.span()),
            None => self.keyword.span,
        }
    }
}

impl HasSpan for TraitStringType<'_> {
    fn span(&self) -> Span {
        match &self.parameter {
            Some(parameter) => self.keyword.span.join(parameter.span()),
            None => self.keyword.span,
        }
    }
}

impl std::fmt::Display for ClassStringType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(parameter) = &self.parameter {
            write!(f, "{}<{}>", self.keyword, parameter)
        } else {
            write!(f, "{}", self.keyword)
        }
    }
}

impl std::fmt::Display for InterfaceStringType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(parameter) = &self.parameter {
            write!(f, "{}<{}>", self.keyword, parameter)
        } else {
            write!(f, "{}", self.keyword)
        }
    }
}

impl std::fmt::Display for EnumStringType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(parameter) = &self.parameter {
            write!(f, "{}<{}>", self.keyword, parameter)
        } else {
            write!(f, "{}", self.keyword)
        }
    }
}

impl std::fmt::Display for TraitStringType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(parameter) = &self.parameter {
            write!(f, "{}<{}>", self.keyword, parameter)
        } else {
            write!(f, "{}", self.keyword)
        }
    }
}
