use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Type;
use crate::ast::VariableType;
use crate::ast::keyword::Keyword;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum CallableTypeKind {
    Callable,
    PureCallable,
    Closure,
    PureClosure,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct CallableType<'input> {
    pub kind: CallableTypeKind,
    pub keyword: Keyword<'input>,
    pub specification: Option<CallableTypeSpecification<'input>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct CallableTypeSpecification<'input> {
    pub parameters: CallableTypeParameters<'input>,
    pub return_type: Option<CallableTypeReturnType<'input>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct CallableTypeParameters<'input> {
    pub left_parenthesis: Span,
    pub entries: Vec<CallableTypeParameter<'input>>,
    pub right_parenthesis: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct CallableTypeParameter<'input> {
    pub parameter_type: Option<Type<'input>>,
    pub equals: Option<Span>,
    pub ellipsis: Option<Span>,
    pub variable: Option<VariableType<'input>>,
    pub comma: Option<Span>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct CallableTypeReturnType<'input> {
    pub colon: Span,
    pub return_type: Box<Type<'input>>,
}

impl CallableTypeKind {
    #[inline]
    pub fn is_pure(&self) -> bool {
        matches!(self, CallableTypeKind::PureCallable | CallableTypeKind::PureClosure)
    }

    #[inline]
    pub fn is_closure(&self) -> bool {
        matches!(self, CallableTypeKind::Closure | CallableTypeKind::PureClosure)
    }
}

impl CallableTypeParameter<'_> {
    #[inline]
    pub const fn is_variadic(&self) -> bool {
        self.ellipsis.is_some()
    }

    #[inline]
    pub const fn is_optional(&self) -> bool {
        self.equals.is_some()
    }
}

impl HasSpan for CallableType<'_> {
    fn span(&self) -> Span {
        match &self.specification {
            Some(specification) => self.keyword.span.join(specification.span()),
            None => self.keyword.span,
        }
    }
}

impl HasSpan for CallableTypeSpecification<'_> {
    fn span(&self) -> Span {
        match &self.return_type {
            Some(return_type) => self.parameters.span().join(return_type.span()),
            None => self.parameters.span(),
        }
    }
}

impl HasSpan for CallableTypeParameters<'_> {
    fn span(&self) -> Span {
        self.left_parenthesis.join(self.right_parenthesis)
    }
}

impl HasSpan for CallableTypeParameter<'_> {
    fn span(&self) -> Span {
        let start = match &self.parameter_type {
            Some(parameter_type) => parameter_type.span(),
            None => self.equals.or(self.ellipsis).or(self.variable.as_ref().map(|v| v.span())).or(self.comma).unwrap(),
        };

        let end =
            self.comma.or(self.variable.as_ref().map(|v| v.span())).or(self.ellipsis).or(self.equals).unwrap_or(start);

        start.join(end)
    }
}

impl HasSpan for CallableTypeReturnType<'_> {
    fn span(&self) -> Span {
        self.colon.join(self.return_type.span())
    }
}

impl std::fmt::Display for CallableTypeReturnType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ": {}", self.return_type)
    }
}

impl std::fmt::Display for CallableTypeParameter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(parameter_type) = &self.parameter_type {
            write!(f, "{}", parameter_type)?;
        }

        if self.equals.is_some() {
            write!(f, "=")?;
        } else if self.ellipsis.is_some() {
            write!(f, "...")?;
        }

        if let Some(variable) = &self.variable {
            write!(f, " {}", variable)?;
        }

        Ok(())
    }
}

impl std::fmt::Display for CallableTypeParameters<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        for (i, entry) in self.entries.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{entry}")?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl std::fmt::Display for CallableTypeSpecification<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.parameters)?;
        if let Some(return_type) = &self.return_type {
            write!(f, "{return_type}")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for CallableType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.keyword)?;
        if let Some(specification) = &self.specification {
            write!(f, "{specification}")?;
        }
        Ok(())
    }
}
