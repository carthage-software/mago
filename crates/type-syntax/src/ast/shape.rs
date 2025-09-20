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
pub struct ShapeType<'input> {
    pub kind: ShapeTypeKind,
    pub keyword: Keyword<'input>,
    pub left_brace: Span,
    pub fields: Vec<ShapeField<'input>>,
    pub additional_fields: Option<ShapeAdditionalFields<'input>>,
    pub right_brace: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum ShapeKey<'input> {
    String { value: &'input str, span: Span },
    Integer { value: i64, span: Span },
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ShapeFieldKey<'input> {
    pub key: ShapeKey<'input>,
    pub question_mark: Option<Span>,
    pub colon: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ShapeField<'input> {
    pub key: Option<ShapeFieldKey<'input>>,
    pub value: Box<Type<'input>>,
    pub comma: Option<Span>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ShapeAdditionalFields<'input> {
    pub ellipsis: Span,
    pub parameters: Option<GenericParameters<'input>>,
}

impl ShapeTypeKind {
    #[inline]
    pub const fn is_array(&self) -> bool {
        matches!(self, ShapeTypeKind::Array | ShapeTypeKind::NonEmptyArray | ShapeTypeKind::AssociativeArray)
    }

    #[inline]
    pub const fn is_list(&self) -> bool {
        matches!(self, ShapeTypeKind::List | ShapeTypeKind::NonEmptyList)
    }

    #[inline]
    pub const fn is_non_empty(&self) -> bool {
        matches!(self, ShapeTypeKind::NonEmptyArray | ShapeTypeKind::NonEmptyList)
    }
}

impl ShapeField<'_> {
    #[inline]
    pub fn is_optional(&self) -> bool {
        if let Some(key) = self.key.as_ref() { key.question_mark.is_some() } else { false }
    }
}

impl ShapeType<'_> {
    #[inline]
    pub fn has_fields(&self) -> bool {
        !self.fields.is_empty()
    }

    #[inline]
    pub fn has_non_optional_fields(&self) -> bool {
        self.fields.iter().any(|field| !field.is_optional())
    }
}

impl HasSpan for ShapeType<'_> {
    fn span(&self) -> Span {
        self.keyword.span().join(self.right_brace)
    }
}

impl HasSpan for ShapeKey<'_> {
    fn span(&self) -> Span {
        match self {
            ShapeKey::String { span, .. } => *span,
            ShapeKey::Integer { span, .. } => *span,
        }
    }
}

impl HasSpan for ShapeFieldKey<'_> {
    fn span(&self) -> Span {
        self.key.span().join(self.colon)
    }
}

impl HasSpan for ShapeField<'_> {
    fn span(&self) -> Span {
        if let Some(key) = &self.key {
            if let Some(comma) = self.comma { key.span().join(comma) } else { key.span().join(self.value.span()) }
        } else if let Some(comma) = self.comma {
            self.value.span().join(comma)
        } else {
            self.value.span()
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

impl std::fmt::Display for ShapeKey<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShapeKey::String { value, .. } => write!(f, "{}", value),
            ShapeKey::Integer { value, .. } => write!(f, "{}", value),
        }
    }
}

impl std::fmt::Display for ShapeFieldKey<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}:", self.key, self.question_mark.as_ref().map_or("", |_| "?"))
    }
}

impl std::fmt::Display for ShapeField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(key) = self.key.as_ref() {
            write!(f, "{} {}", key, self.value)
        } else {
            write!(f, "{}", self.value)
        }
    }
}

impl std::fmt::Display for ShapeAdditionalFields<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "...")?;

        if let Some(generics) = &self.parameters { write!(f, "{generics}") } else { Ok(()) }
    }
}

impl std::fmt::Display for ShapeType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{{", self.keyword)?;

        for (i, field) in self.fields.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }

            write!(f, "{field}")?;
        }

        if let Some(additional_fields) = &self.additional_fields {
            if !self.fields.is_empty() {
                write!(f, ", ")?;
            }

            write!(f, "{additional_fields}")?;
        }

        write!(f, "}}")
    }
}
