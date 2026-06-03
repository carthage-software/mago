use mago_span::Span;
use serde::Serialize;

use crate::ir::identifier::Identifier;
use crate::ir::name::Name;

pub mod annotation;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum Variance {
    Invariant,
    Covariant,
    Contravariant,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum TypeParameterDefiningEntity<'arena> {
    ClassLike(Identifier<'arena>),
    Function(Identifier<'arena>),
    Method(Identifier<'arena>, Name<'arena>),
    Closure(Span),
}
