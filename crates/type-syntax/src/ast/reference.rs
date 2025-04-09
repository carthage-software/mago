use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::generics::GenericParameters;
use crate::ast::identifier::Identifier;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct ReferenceType<'input> {
    pub identifier: Identifier<'input>,
    pub parameters: Option<GenericParameters<'input>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(C)]
pub struct MemberReferenceType<'input> {
    pub class: Identifier<'input>,
    pub double_colon: Span,
    pub member: Identifier<'input>,
}

impl HasSpan for ReferenceType<'_> {
    fn span(&self) -> Span {
        match &self.parameters {
            Some(parameters) => self.identifier.span.join(parameters.span()),
            None => self.identifier.span,
        }
    }
}

impl HasSpan for MemberReferenceType<'_> {
    fn span(&self) -> Span {
        self.class.span.join(self.member.span)
    }
}
