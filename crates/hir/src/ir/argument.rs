use serde::Serialize;

use crate::ir::expression::Expression;
use crate::ir::name::Name;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum Argument<'arena, S, D, E> {
    Value(&'arena Expression<'arena, S, D, E>),
    Variadic(&'arena Expression<'arena, S, D, E>),
    Named(Name<'arena>, &'arena Expression<'arena, S, D, E>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum PartialArgument<'arena, S, D, E> {
    Value(&'arena Expression<'arena, S, D, E>),
    Variadic(&'arena Expression<'arena, S, D, E>),
    Named(Name<'arena>, &'arena Expression<'arena, S, D, E>),
    Placeholder,
    NamedPlaceholder(Name<'arena>),
    VariadicPlaceholder,
}
