//! Keyword lookup for the surface-grammar lexer.

use crate::surface::token::SurfaceTokenKind;

/// Maps a raw identifier slice to its keyword kind, or `None` if the identifier is not
/// a keyword. Matching is case-sensitive, mirroring GritQL convention.
#[inline]
#[must_use]
pub fn lookup(bytes: &[u8]) -> Option<SurfaceTokenKind> {
    Some(match bytes {
        b"not" => SurfaceTokenKind::KwNot,
        b"contains" => SurfaceTokenKind::KwContains,
        b"within" => SurfaceTokenKind::KwWithin,
        b"maybe" => SurfaceTokenKind::KwMaybe,
        b"bubble" => SurfaceTokenKind::KwBubble,
        b"where" => SurfaceTokenKind::KwWhere,
        b"or" => SurfaceTokenKind::KwOr,
        b"and" => SurfaceTokenKind::KwAnd,
        b"as" => SurfaceTokenKind::KwAs,
        b"undefined" => SurfaceTokenKind::KwUndefined,
        b"true" => SurfaceTokenKind::True,
        b"false" => SurfaceTokenKind::False,
        _ => return None,
    })
}
