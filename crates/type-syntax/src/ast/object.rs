use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Keyword;
use crate::ast::ShapeField;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ObjectType<'input> {
    pub keyword: Keyword<'input>,
    pub properties: Option<ObjectProperties<'input>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ObjectProperties<'input> {
    pub left_brace: Span,
    pub fields: Vec<ShapeField<'input>>,
    pub ellipsis: Option<Span>,
    pub right_brace: Span,
}

impl HasSpan for ObjectType<'_> {
    fn span(&self) -> Span {
        match &self.properties {
            Some(parameters) => self.keyword.span.join(parameters.span()),
            None => self.keyword.span,
        }
    }
}

impl std::fmt::Display for ObjectType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(parameters) = &self.properties {
            write!(f, "{}{}", self.keyword, parameters)
        } else {
            write!(f, "{}", self.keyword)
        }
    }
}

impl HasSpan for ObjectProperties<'_> {
    fn span(&self) -> Span {
        self.left_brace.join(self.right_brace)
    }
}

impl std::fmt::Display for ObjectProperties<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for (i, field) in self.fields.iter().enumerate() {
            if i > 0 {
                write!(f, ", {}", field)?;
            } else {
                write!(f, "{}", field)?;
            }
        }

        if self.ellipsis.is_some() {
            if !self.fields.is_empty() {
                write!(f, ", ")?;
            }

            write!(f, "...")?;
        }

        write!(f, "}}")
    }
}
