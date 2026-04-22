use mago_span::Position;

use crate::lexer::internal::mode::LexerMode;

/// Maximum nesting depth for lexer mode transitions (Data -> Block ->
/// DoubleQuoted -> Interpolation -> Block -> ...).  Realistic templates
/// stay well under a handful; anything beyond this is almost certainly a
/// malformed input and triggers a syntax error.
pub(crate) const MAX_MODE_DEPTH: usize = 64;

/// Maximum bracket nesting depth inside an expression.  Same rationale as
/// [`MAX_MODE_DEPTH`].
pub(crate) const MAX_BRACKET_DEPTH: usize = 128;

/// A single bracket opener tracked on the [`BracketStack`].
#[derive(Debug, Clone, Copy)]
pub(crate) struct Bracket {
    pub opener: u8,
    pub position: Position,
}

/// Inline stack of lexer modes - no heap allocation.
#[derive(Debug)]
pub(crate) struct ModeStack {
    buf: [LexerMode; MAX_MODE_DEPTH],
    len: usize,
}

impl ModeStack {
    #[inline]
    pub(crate) const fn new() -> Self {
        Self { buf: [LexerMode::Data; MAX_MODE_DEPTH], len: 0 }
    }

    #[inline]
    pub(crate) fn push(&mut self, mode: LexerMode) -> bool {
        if self.len == MAX_MODE_DEPTH {
            return false;
        }
        self.buf[self.len] = mode;
        self.len += 1;
        true
    }

    #[inline]
    pub(crate) fn pop(&mut self) -> Option<LexerMode> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        Some(self.buf[self.len])
    }

    #[inline]
    pub(crate) const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Inline stack of bracket openers - no heap allocation.
#[derive(Debug)]
pub(crate) struct BracketStack {
    buf: [Bracket; MAX_BRACKET_DEPTH],
    len: usize,
}

impl BracketStack {
    #[inline]
    pub(crate) const fn new() -> Self {
        Self { buf: [Bracket { opener: 0, position: Position::new(0) }; MAX_BRACKET_DEPTH], len: 0 }
    }

    #[inline]
    pub(crate) fn push(&mut self, bracket: Bracket) -> bool {
        if self.len == MAX_BRACKET_DEPTH {
            return false;
        }
        self.buf[self.len] = bracket;
        self.len += 1;
        true
    }

    #[inline]
    pub(crate) fn pop(&mut self) -> Option<Bracket> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        Some(self.buf[self.len])
    }

    #[inline]
    pub(crate) fn last(&self) -> Option<&Bracket> {
        if self.len == 0 { None } else { Some(&self.buf[self.len - 1]) }
    }

    #[inline]
    pub(crate) const fn is_empty(&self) -> bool {
        self.len == 0
    }
}
