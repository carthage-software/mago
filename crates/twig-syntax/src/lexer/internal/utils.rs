use crate::lexer::internal::consts::IDENT_PART;
use crate::token::TwigTokenKind;

/// The matching closing bracket byte for a given opener (`(` -> `)`,
/// `[` -> `]`, `{` -> `}`).  Returns `0` for non-bracket bytes.
#[inline]
pub(crate) fn matching_closer(opener: u8) -> u8 {
    match opener {
        b'(' => b')',
        b'[' => b']',
        b'{' => b'}',
        _ => 0,
    }
}

/// Whether `b` is an ASCII whitespace byte.
#[inline]
pub(crate) fn is_whitespace_byte(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\r' | b'\n' | 0x0B | 0x0C)
}

/// Whether `b` may continue an identifier (alphanumeric, `_`, or non-ASCII).
#[inline]
pub(crate) fn is_word_byte(b: u8) -> bool {
    IDENT_PART[b as usize]
}

/// Try to match a fixed three-byte operator at `(b0, b1, b2)`.
#[inline]
pub(crate) fn three_byte_operator(b0: u8, b1: Option<u8>, b2: Option<u8>) -> Option<(TwigTokenKind, usize)> {
    match (b0, b1, b2) {
        (b'<', Some(b'='), Some(b'>')) => Some((TwigTokenKind::Spaceship, 3)),
        (b'=', Some(b'='), Some(b'=')) => Some((TwigTokenKind::EqualEqualEqual, 3)),
        (b'!', Some(b'='), Some(b'=')) => Some((TwigTokenKind::BangEqualEqual, 3)),
        (b'.', Some(b'.'), Some(b'.')) => Some((TwigTokenKind::DotDotDot, 3)),
        _ => None,
    }
}

/// Try to match a fixed two-byte operator at `(b0, b1)`.
#[inline]
pub(crate) fn two_byte_operator(b0: u8, b1: Option<u8>) -> Option<(TwigTokenKind, usize)> {
    match (b0, b1) {
        (b'=', Some(b'=')) => Some((TwigTokenKind::EqualEqual, 2)),
        (b'!', Some(b'=')) => Some((TwigTokenKind::BangEqual, 2)),
        (b'<', Some(b'=')) => Some((TwigTokenKind::LessThanEqual, 2)),
        (b'>', Some(b'=')) => Some((TwigTokenKind::GreaterThanEqual, 2)),
        (b'*', Some(b'*')) => Some((TwigTokenKind::AsteriskAsterisk, 2)),
        (b'/', Some(b'/')) => Some((TwigTokenKind::SlashSlash, 2)),
        (b'?', Some(b'?')) => Some((TwigTokenKind::QuestionQuestion, 2)),
        (b'?', Some(b':')) => Some((TwigTokenKind::QuestionColon, 2)),
        (b'?', Some(b'.')) => Some((TwigTokenKind::QuestionDot, 2)),
        (b'.', Some(b'.')) => Some((TwigTokenKind::DotDot, 2)),
        (b'=', Some(b'>')) => Some((TwigTokenKind::FatArrow, 2)),
        _ => None,
    }
}

/// Token kind for an opening bracket byte.
#[inline]
pub(crate) fn opener_kind(b: u8) -> Option<TwigTokenKind> {
    match b {
        b'(' => Some(TwigTokenKind::LeftParen),
        b'[' => Some(TwigTokenKind::LeftBracket),
        b'{' => Some(TwigTokenKind::LeftBrace),
        _ => None,
    }
}

/// Token kind for a closing bracket byte.
#[inline]
pub(crate) fn closer_kind(b: u8) -> Option<TwigTokenKind> {
    match b {
        b')' => Some(TwigTokenKind::RightParen),
        b']' => Some(TwigTokenKind::RightBracket),
        b'}' => Some(TwigTokenKind::RightBrace),
        _ => None,
    }
}

/// Token kind for a single-byte operator or punctuation symbol.
#[inline]
pub(crate) fn single_byte_symbol(b: u8) -> Option<TwigTokenKind> {
    match b {
        b',' => Some(TwigTokenKind::Comma),
        b':' => Some(TwigTokenKind::Colon),
        b'.' => Some(TwigTokenKind::Dot),
        b'?' => Some(TwigTokenKind::Question),
        b'|' => Some(TwigTokenKind::Pipe),
        b'=' => Some(TwigTokenKind::Equal),
        b'+' => Some(TwigTokenKind::Plus),
        b'-' => Some(TwigTokenKind::Minus),
        b'*' => Some(TwigTokenKind::Asterisk),
        b'/' => Some(TwigTokenKind::Slash),
        b'%' => Some(TwigTokenKind::Percent),
        b'~' => Some(TwigTokenKind::Tilde),
        b'<' => Some(TwigTokenKind::LessThan),
        b'>' => Some(TwigTokenKind::GreaterThan),
        _ => None,
    }
}
