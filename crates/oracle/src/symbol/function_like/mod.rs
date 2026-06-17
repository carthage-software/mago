#[cfg(feature = "serde")]
use serde::Serialize;

use mago_span::HasSpan;

use crate::path::Path;
use crate::symbol::Symbol;
use crate::symbol::function_like::arrow_function::ArrowFunctionSymbol;
use crate::symbol::function_like::closure::ClosureSymbol;
use crate::symbol::function_like::function::FunctionSymbol;
use crate::symbol::part::attribute::AppliedAttribute;
use crate::symbol::part::constraint::SymbolConstraint;
use crate::symbol::part::origin::Origin;

pub mod arrow_function;
pub mod closure;
pub mod function;
pub mod part;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum FunctionLikeSymbol<'arena> {
    Function(&'arena FunctionSymbol<'arena>),
    Closure(&'arena ClosureSymbol<'arena>),
    ArrowFunction(&'arena ArrowFunctionSymbol<'arena>),
}

impl<'arena> Symbol<'arena> for FunctionLikeSymbol<'arena> {
    fn path(&self) -> Path<'arena> {
        match self {
            FunctionLikeSymbol::Function(f) => f.path(),
            FunctionLikeSymbol::Closure(c) => c.path(),
            FunctionLikeSymbol::ArrowFunction(a) => a.path(),
        }
    }

    fn origin(&self) -> Origin {
        match self {
            FunctionLikeSymbol::Function(f) => f.origin(),
            FunctionLikeSymbol::Closure(c) => c.origin(),
            FunctionLikeSymbol::ArrowFunction(a) => a.origin(),
        }
    }

    fn is_polyfill(&self) -> bool {
        match self {
            FunctionLikeSymbol::Function(f) => f.is_polyfill(),
            FunctionLikeSymbol::Closure(c) => c.is_polyfill(),
            FunctionLikeSymbol::ArrowFunction(a) => a.is_polyfill(),
        }
    }

    fn applied_attributes(&self) -> &'arena [AppliedAttribute<'arena>] {
        match self {
            FunctionLikeSymbol::Function(f) => f.applied_attributes(),
            FunctionLikeSymbol::Closure(c) => c.applied_attributes(),
            FunctionLikeSymbol::ArrowFunction(a) => a.applied_attributes(),
        }
    }

    fn constraint(&self) -> Option<SymbolConstraint<'arena>> {
        match self {
            FunctionLikeSymbol::Function(f) => f.constraint(),
            FunctionLikeSymbol::Closure(c) => c.constraint(),
            FunctionLikeSymbol::ArrowFunction(a) => a.constraint(),
        }
    }
}

impl HasSpan for FunctionLikeSymbol<'_> {
    fn span(&self) -> mago_span::Span {
        match self {
            FunctionLikeSymbol::Function(f) => f.span,
            FunctionLikeSymbol::Closure(c) => c.span,
            FunctionLikeSymbol::ArrowFunction(a) => a.span,
        }
    }
}
