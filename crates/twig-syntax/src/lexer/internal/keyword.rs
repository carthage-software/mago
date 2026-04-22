use crate::token::TwigTokenKind;

/// A first word that may open a multi-word operator, paired with its valid
/// continuations and the resolved token kind for each.
pub const MULTI_WORD_OPENERS: &[(&str, &[(&str, TwigTokenKind)])] = &[
    ("not", &[("in", TwigTokenKind::NotIn)]),
    ("starts", &[("with", TwigTokenKind::StartsWith)]),
    ("ends", &[("with", TwigTokenKind::EndsWith)]),
    ("has", &[("some", TwigTokenKind::HasSome), ("every", TwigTokenKind::HasEvery)]),
    ("same", &[("as", TwigTokenKind::SameAs)]),
    ("divisible", &[("by", TwigTokenKind::DivisibleBy)]),
];

#[inline]
#[must_use]
pub fn continuation_words_for(first: &str) -> Option<&'static [(&'static str, TwigTokenKind)]> {
    for (opener, cont) in MULTI_WORD_OPENERS {
        if *opener == first {
            return Some(cont);
        }
    }
    None
}

#[inline]
#[must_use]
pub fn single_word_operator_kind(name: &str) -> Option<TwigTokenKind> {
    match name {
        "and" => Some(TwigTokenKind::And),
        "or" => Some(TwigTokenKind::Or),
        "xor" => Some(TwigTokenKind::Xor),
        "not" => Some(TwigTokenKind::Not),
        "in" => Some(TwigTokenKind::In),
        "is" => Some(TwigTokenKind::Is),
        "matches" => Some(TwigTokenKind::Matches),
        _ => None,
    }
}
