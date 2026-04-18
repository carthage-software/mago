//! GritQL surface-grammar front-end.
//!
//! Splits cleanly into three stages:
//!
//! 1. [`lexer::SurfaceLexer`] tokenises the pattern source into [`token::SurfaceToken`]s.
//! 2. [`parser::SurfaceParser`] walks those tokens and builds a
//!    [`grit_pattern_matcher::pattern::Pattern<MagoQueryContext>`].
//! 3. [`parse`] is the public façade that wires the two together and handles the
//!    bare-snippet fast path for patterns that contain no backticks.
//!
//! # Subset supported
//!
//! * Backtick-delimited code snippets: `` `eval(^x)` ``. Inside backticks is PHP source
//!   with `^name` metavariables, same as what [`crate::compiler::compile`] accepts when
//!   called with a bare snippet.
//! * Metavariable references: `^name`. As an expression-position pattern they resolve
//!   through the engine's [`Pattern::Variable`].
//! * Prefix operators: `not P`, `!P`, `contains P`, `within P`, `maybe P`, `bubble P`.
//! * Binary operators: `P <: Q` (P is a subtype of Q), `P => Q` (rewrite P to Q).
//! * Conjunction / disjunction blocks: `and { P, Q, R }`, `or { P, Q, R }`.
//! * `where { clauses… }`: a shorthand for `and { … }` attached to a pattern.
//! * Grouping parentheses.
//! * String / integer / float / boolean literals.
//!
//! # Bare-snippet fallback
//!
//! When the source contains no backticks, we skip the surface lexer and feed the whole
//! input straight to [`crate::compiler::lower_snippet_source`]. That preserves backwards
//! compatibility with legacy patterns like `eval(^x)`.

use bumpalo::Bump;
use grit_pattern_matcher::pattern::Pattern;

use crate::compiler::CompileError;
use crate::compiler::lower_snippet_source;
use crate::query_context::MagoQueryContext;

pub mod keyword;
pub mod lexer;
pub mod parser;
pub mod token;

pub use lexer::SurfaceLexer;
pub use parser::SurfaceParser;
pub use token::SurfaceToken;
pub use token::SurfaceTokenKind;

/// Output of compiling a surface-grammar source.
pub struct SurfacePattern {
    pub pattern: Pattern<MagoQueryContext>,
    /// Metavariable names referenced by the pattern, slot-ordered.
    pub variables: Vec<String>,
}

/// Parses `source` as a GritQL-subset surface pattern.
///
/// Patterns containing no backticks are routed directly to the snippet lowering pass, so
/// existing callers that write patterns like `eval(^x)` keep working without change.
/// Patterns containing a backtick go through the surface lexer + parser.
pub fn parse(arena: &Bump, source: &str) -> Result<SurfacePattern, CompileError> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Err(CompileError::Empty);
    }

    if !trimmed.contains('`') {
        let mut variables = Vec::new();
        let pattern = lower_snippet_source(arena, trimmed, &mut variables)?;
        return Ok(SurfacePattern { pattern, variables });
    }

    let tokens: Vec<SurfaceToken<'_>> = SurfaceLexer::new(trimmed).filter(|tok| !tok.kind.is_trivia()).collect();
    let mut parser = SurfaceParser::new(arena, tokens);
    let pattern = parser.parse_pattern()?;
    if let Some(tok) = parser.peek() {
        return Err(CompileError::SurfaceError(format!(
            "unexpected trailing token {:?} at offset {}",
            tok.kind, tok.start.offset
        )));
    }

    Ok(SurfacePattern { pattern, variables: parser.variables })
}
