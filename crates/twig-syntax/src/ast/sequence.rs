//! Twig-facing view over the shared sequence primitives.
//!
//! Both [`Sequence`] and [`TokenSeparatedSequence`] live in
//! [`mago_syntax_core::ast`]; this module re-exports them and locks the
//! token type of `TokenSeparatedSequence` to [`TwigToken`] so call sites
//! can continue to write `TokenSeparatedSequence<'arena, T>` with a
//! single generic.

use crate::token::TwigToken;

pub use mago_syntax_core::ast::Sequence;

/// A Twig AST sequence whose separators are [`TwigToken`]s.
pub type TokenSeparatedSequence<'arena, T> =
    mago_syntax_core::ast::TokenSeparatedSequence<'arena, T, TwigToken<'arena>>;
