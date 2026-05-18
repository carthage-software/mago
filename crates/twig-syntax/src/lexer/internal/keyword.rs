use crate::token::TwigTokenKind;

/// Continuation suffix + the resolved token kind it produces.
type MultiWordContinuation = (&'static [u8], TwigTokenKind);

/// An opener word + its set of valid continuations.
type MultiWordOpener = (&'static [u8], &'static [MultiWordContinuation]);

/// A first word that may open a multi-word operator, paired with its valid
/// continuations and the resolved token kind for each.
pub const MULTI_WORD_OPENERS: &[MultiWordOpener] = &[
    (b"not", &[(b"in", TwigTokenKind::NotIn)]),
    (b"starts", &[(b"with", TwigTokenKind::StartsWith)]),
    (b"ends", &[(b"with", TwigTokenKind::EndsWith)]),
    (b"has", &[(b"some", TwigTokenKind::HasSome), (b"every", TwigTokenKind::HasEvery)]),
    (b"same", &[(b"as", TwigTokenKind::SameAs)]),
    (b"divisible", &[(b"by", TwigTokenKind::DivisibleBy)]),
];

#[inline]
#[must_use]
pub fn continuation_words_for(first: &[u8]) -> Option<&'static [(&'static [u8], TwigTokenKind)]> {
    for (opener, cont) in MULTI_WORD_OPENERS {
        if *opener == first {
            return Some(cont);
        }
    }
    None
}

#[inline]
#[must_use]
pub fn single_word_operator_kind(name: &[u8]) -> Option<TwigTokenKind> {
    match name {
        b"and" => Some(TwigTokenKind::And),
        b"or" => Some(TwigTokenKind::Or),
        b"xor" => Some(TwigTokenKind::Xor),
        b"not" => Some(TwigTokenKind::Not),
        b"in" => Some(TwigTokenKind::In),
        b"is" => Some(TwigTokenKind::Is),
        b"matches" => Some(TwigTokenKind::Matches),
        _ => None,
    }
}
