//! Candidate scoring for completion.
//!
//! 1. exact match (case-insensitive)
//! 2. acronym match (`GATQH` -> `GetAllTransactionsQueryHandler`)
//! 3. prefix match (`Get` -> `GetAll...`)
//! 4. substring match (`All` -> `GetAll...`)
//! 5. fuzzy match (in-order subsequence)
//!
//! Matching is against a candidate's short name. The returned [`Score`] sorts so
//! a better tier always wins, with shorter and alphabetically-earlier names
//! breaking ties inside a tier.

/// The matched tier, best first. Field/discriminant order is the sort order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Tier {
    Exact = 0,
    Acronym = 1,
    Prefix = 2,
    Substring = 3,
    Fuzzy = 4,
}

/// A successful match. Field order is the sort order: better [`Tier`], then a
/// shorter candidate, then an alphabetically-earlier one.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Score {
    tier: Tier,
    length: u32,
    name: Vec<u8>,
}

impl Score {
    /// An LSP `sort_text` reproducing this ordering on the client. Tier and
    /// length are zero-padded so lexical comparison matches numeric comparison.
    #[must_use]
    pub(super) fn sort_text(&self) -> String {
        format!("{}-{:06}-{}", self.tier as u8, self.length.min(999_999), String::from_utf8_lossy(&self.name))
    }
}

/// Scores `candidate` (a short name) against `query`. Returns `None` when the
/// query matches at no tier. An empty query matches everything as a prefix, so
/// an opening completion list is ordered by length then name.
#[must_use]
pub(super) fn score(query: &[u8], candidate: &[u8]) -> Option<Score> {
    let make = |tier: Tier| Some(Score { tier, length: candidate.len() as u32, name: candidate.to_ascii_lowercase() });

    if query.is_empty() {
        return make(Tier::Prefix);
    }

    let q = query.to_ascii_lowercase();
    let c = candidate.to_ascii_lowercase();

    if q == c {
        return make(Tier::Exact);
    }

    // A single letter is better served by the prefix tier than a noisy acronym.
    if query.len() >= 2 && acronym_matches(query, candidate) {
        return make(Tier::Acronym);
    }

    if c.starts_with(&q) {
        return make(Tier::Prefix);
    }

    if c.windows(q.len()).any(|w| w == q.as_slice()) {
        return make(Tier::Substring);
    }

    if is_subsequence(&q, &c) {
        return make(Tier::Fuzzy);
    }

    None
}

/// `true` if the lowercased `query` is an in-order subsequence of the
/// candidate's word initials. Initials are the first character and every
/// character that starts a new word (an upper-case letter following a
/// lower-case one or a digit, or any character after `_` / `\`).
fn acronym_matches(query: &[u8], candidate: &[u8]) -> bool {
    let mut initials = Vec::new();
    let mut prev: Option<u8> = None;
    for &b in candidate {
        let boundary = match prev {
            None => true,
            Some(p) => matches!(p, b'_' | b'\\') || (!p.is_ascii_uppercase() && b.is_ascii_uppercase()),
        };
        if boundary && b.is_ascii_alphanumeric() {
            initials.push(b.to_ascii_lowercase());
        }
        prev = Some(b);
    }

    is_subsequence(&query.to_ascii_lowercase(), &initials)
}

/// `true` if every byte of `needle` appears in `haystack` in order.
fn is_subsequence(needle: &[u8], haystack: &[u8]) -> bool {
    let mut it = haystack.iter();
    needle.iter().all(|b| it.any(|h| h == b))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tier(query: &str, candidate: &str) -> Option<Tier> {
        score(query.as_bytes(), candidate.as_bytes()).map(|s| s.tier)
    }

    #[test]
    fn exact_match_wins() {
        assert_eq!(tier("Greeter", "Greeter"), Some(Tier::Exact));
        assert_eq!(tier("greeter", "Greeter"), Some(Tier::Exact));
    }

    #[test]
    fn acronym_matches_capitals() {
        assert_eq!(tier("GATQH", "GetAllTransactionsQueryHandler"), Some(Tier::Acronym));
        assert_eq!(tier("GAT", "GetAllTransactionsQueryHandler"), Some(Tier::Acronym));
        assert_eq!(tier("gatqh", "GetAllTransactionsQueryHandler"), Some(Tier::Acronym));
    }

    #[test]
    fn acronym_honours_underscore_boundaries() {
        assert_eq!(tier("MTC", "MAX_TILE_COUNT"), Some(Tier::Acronym));
    }

    #[test]
    fn prefix_beats_substring() {
        assert_eq!(tier("Get", "GetAllThings"), Some(Tier::Prefix));
        assert_eq!(tier("All", "GetAllThings"), Some(Tier::Substring));
    }

    #[test]
    fn fuzzy_is_last_resort() {
        assert_eq!(tier("gtt", "GetAllThings"), Some(Tier::Fuzzy));
        assert_eq!(tier("zzz", "GetAllThings"), None);
    }

    #[test]
    fn empty_query_matches_everything() {
        assert_eq!(tier("", "Anything"), Some(Tier::Prefix));
    }

    #[test]
    fn single_letter_prefers_prefix_over_acronym() {
        assert_eq!(tier("G", "GetAllThings"), Some(Tier::Prefix));
    }

    #[test]
    fn sort_text_orders_by_tier_then_length() {
        let exact = score(b"Foo", b"Foo").unwrap();
        let prefix = score(b"Fo", b"Foo").unwrap();
        let longer = score(b"Fo", b"FooBarBaz").unwrap();
        assert!(exact.sort_text() < prefix.sort_text());
        assert!(prefix.sort_text() < longer.sort_text());
    }
}
