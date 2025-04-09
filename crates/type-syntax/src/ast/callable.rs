use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Type;
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
    pub parameter_type: Box<Type<'input>>,
    pub equals: Option<Span>,
    pub ellipsis: Option<Span>,
    pub comma: Option<Span>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct CallableTypeReturnType<'input> {
    pub colon: Span,
    pub return_type: Box<Type<'input>>,
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
        match &self.comma {
            Some(comma) => self.parameter_type.span().join(*comma),
            None => match &self.ellipsis {
                Some(ellipsis) => self.parameter_type.span().join(*ellipsis),
                None => match &self.equals {
                    Some(equals) => self.parameter_type.span().join(*equals),
                    None => self.parameter_type.span(),
                },
            },
        }
    }
}

impl HasSpan for CallableTypeReturnType<'_> {
    fn span(&self) -> Span {
        self.colon.join(self.return_type.span())
    }
}
