//! Allocation-free name matching shared by completion and workspace symbols.
//!
//! Tiers follow the order an IDE user expects:
//!
//! 1. exact match
//! 2. prefix match (`Get` -> `GetAll...`)
//! 3. hump match anchored at the start (`GATQH`, `rtr` -> `redirectToRoute`)
//! 4. substring or hump match at a later word start (`Route` -> `redirectToRoute`)
//! 5. substring match anywhere (`oute` -> `redirectToRoute`)
//! 6. fuzzy match from a word start (in-order subsequence)
//!
//! Case never decides whether a name matches. Matches anchored at the start
//! (tiers 1-3) rank before the looser tiers; a query starting with an
//! upper-case letter anchors word-start matches (tier 4) too. Among anchored
//! matches, one whose first matched letter has the wrong case ranks after
//! every one whose first letter agrees.

const HUMP_BUDGET: u32 = 256;

/// What an empty query matches.
pub(crate) const ANY: Match = Match { loose: false, first_case_miss: false, tier: Tier::Prefix, case_miss: false };

/// The matched tier, best first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Tier {
    Exact,
    Prefix,
    Hump,
    WordStart,
    Substring,
    Fuzzy,
}

/// A successful match. Field order is the sort order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Match {
    pub(crate) loose: bool,
    pub(crate) first_case_miss: bool,
    pub(crate) tier: Tier,
    pub(crate) case_miss: bool,
}

impl Match {
    pub(crate) const fn is_exact(self) -> bool {
        matches!(self.tier, Tier::Exact)
    }

    pub(crate) const fn is_strict_exact(self) -> bool {
        self.is_exact() && !self.case_miss && !self.first_case_miss
    }

    pub(crate) const fn is_weak(self) -> bool {
        matches!(self.tier, Tier::Substring | Tier::Fuzzy)
    }
}

/// `false` when [`score`] cannot match, judged without reading case.
pub(crate) fn may_match(query: &[u8], candidate: &[u8]) -> bool {
    query.len() <= candidate.len() && is_subsequence(query, candidate)
}

/// Scores `candidate` against `query`, or `None` when no tier matches. An
/// empty query matches everything as a prefix.
#[must_use]
pub(crate) fn score(query: &[u8], candidate: &[u8]) -> Option<Match> {
    let Some(&first) = query.first() else { return Some(ANY) };
    if !may_match(query, candidate) {
        return None;
    }

    let found = |tier: Tier, start: usize, case_miss| {
        let loose = tier > Tier::WordStart || (tier == Tier::WordStart && !first.is_ascii_uppercase());
        let first_case_miss = !loose && first.is_ascii_alphabetic() && candidate[start] != first;
        Some(Match { loose, first_case_miss, tier, case_miss })
    };

    let head = &candidate[..query.len()];
    if head.eq_ignore_ascii_case(query) {
        let tier = if candidate.len() == query.len() { Tier::Exact } else { Tier::Prefix };
        return found(tier, 0, head != query);
    }

    if query.len() >= 2 && humps(query, candidate, 0) {
        return found(Tier::Hump, 0, false);
    }

    let mut substring = None;
    for start in 1..=candidate.len() - query.len() {
        if !candidate[start].eq_ignore_ascii_case(&first) {
            continue;
        }
        let window = &candidate[start..start + query.len()];
        let contiguous = window.eq_ignore_ascii_case(query);
        if is_word_start(candidate, start) && (contiguous || (query.len() >= 2 && humps(query, candidate, start))) {
            return found(Tier::WordStart, start, contiguous && window != query);
        }
        if contiguous && substring.is_none() {
            substring = Some((start, window != query));
        }
    }
    if let Some((start, case_miss)) = substring {
        return found(Tier::Substring, start, case_miss);
    }

    let start = (0..candidate.len())
        .find(|&index| candidate[index].eq_ignore_ascii_case(&first) && is_word_start(candidate, index))?;
    is_subsequence(&query[1..], &candidate[start + 1..]).then(|| found(Tier::Fuzzy, start, false)).flatten()
}

/// `true` when each query byte after the first continues at the next
/// candidate byte or jumps to a later word start, starting at `start`.
fn humps(query: &[u8], candidate: &[u8], start: usize) -> bool {
    let mut budget = HUMP_BUDGET;
    hump_from(query, candidate, start, &mut budget)
}

fn hump_from(query: &[u8], candidate: &[u8], at: usize, budget: &mut u32) -> bool {
    let Some((first, rest)) = query.split_first() else { return true };
    if candidate.len() - at < query.len() || !candidate[at].eq_ignore_ascii_case(first) {
        return false;
    }
    if rest.is_empty() {
        return true;
    }
    if *budget == 0 {
        return false;
    }
    *budget -= 1;

    let next = at + 1;
    if hump_from(rest, candidate, next, budget) {
        return true;
    }
    (next + 1..=candidate.len() - rest.len())
        .any(|index| is_word_start(candidate, index) && hump_from(rest, candidate, index, budget))
}

/// Word starts: the first byte, a byte after `_`, `\` or a digit, an upper-case
/// letter after a lower-case one, and the last capital of an acronym when a
/// lower-case letter follows (`HTTPClient` -> `C`).
fn is_word_start(name: &[u8], index: usize) -> bool {
    let Some(&byte) = name.get(index) else { return false };
    let Some(&previous) = index.checked_sub(1).and_then(|previous| name.get(previous)) else { return true };
    matches!(previous, b'_' | b'\\')
        || previous.is_ascii_digit()
        || (byte.is_ascii_uppercase()
            && (previous.is_ascii_lowercase()
                || (previous.is_ascii_uppercase() && name.get(index + 1).is_some_and(u8::is_ascii_lowercase))))
}

/// `true` when a namespace has a `Tests` or `Test` segment.
pub(crate) fn is_test_namespace(namespace: &[u8]) -> bool {
    namespace
        .split(|byte| *byte == b'\\')
        .any(|segment| segment.eq_ignore_ascii_case(b"tests") || segment.eq_ignore_ascii_case(b"test"))
}

/// `true` if every byte of `needle` appears in `haystack` in order, ignoring ASCII case.
fn is_subsequence(needle: &[u8], haystack: &[u8]) -> bool {
    let mut rest = haystack.iter();
    needle.iter().all(|byte| rest.any(|candidate| candidate.eq_ignore_ascii_case(byte)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tier(query: &str, candidate: &str) -> Option<Tier> {
        score(query.as_bytes(), candidate.as_bytes()).map(|matched| matched.tier)
    }

    #[test]
    fn exact_match_wins() {
        assert_eq!(tier("Greeter", "Greeter"), Some(Tier::Exact));
        assert_eq!(tier("greeter", "Greeter"), Some(Tier::Exact));
    }

    #[test]
    fn first_letter_case_outranks_exactness() {
        assert!(score(b"strl", b"strlen") < score(b"strl", b"Strl"));
        assert!(score(b"Curren", b"Currency") < score(b"Curren", b"CURRENCY_SYMBOL"));
        assert!(score(b"Curren", b"CURRENCY_SYMBOL") < score(b"Curren", b"current"));
        assert!(score(b"_get", b"_GET").is_some_and(|matched| !matched.first_case_miss));
    }

    #[test]
    fn anchored_matches_outrank_loose_ones() {
        assert!(score(b"identity", b"Identity") < score(b"identity", b"InvalidEntityRepository"));
        assert!(score(b"req", b"Request") < score(b"req", b"stats_rand_gen_chisquare"));
        assert!(score(b"resp", b"Response") < score(b"resp", b"memory_reset_peak_usage"));
        assert!(score(b"route", b"redirectToRoute") < score(b"route", b"xroutex"));
    }

    #[test]
    fn upper_case_queries_anchor_word_starts() {
        assert!(score(b"Test", b"AbstractTestCase") < score(b"Test", b"testFoo"));
        assert!(score(b"Test", b"UserTest") < score(b"Test", b"testFoo"));
        assert!(score(b"Test", b"TestCase") < score(b"Test", b"UserTest"));
        assert!(score(b"ROUTE", b"redirectToRoute") < score(b"ROUTE", b"route"));
        assert!(score(b"Test", b"testFoo") < score(b"Test", b"CONTESTS"));
        assert!(score(b"test", b"testFoo") < score(b"test", b"AbstractTestCase"));
        assert!(score(b"test", b"TestCase") < score(b"test", b"UserTest"));
        assert!(score(b"_test", b"_TESTS") < score(b"_test", b"foo__test"));
    }

    #[test]
    fn humps_follow_word_starts() {
        assert_eq!(tier("GATQH", "GetAllTransactionsQueryHandler"), Some(Tier::Hump));
        assert_eq!(tier("GAT", "GetAllTransactionsQueryHandler"), Some(Tier::Hump));
        assert_eq!(tier("gatqh", "GetAllTransactionsQueryHandler"), Some(Tier::Hump));
        assert_eq!(tier("MTC", "MAX_TILE_COUNT"), Some(Tier::Hump));
        assert_eq!(tier("rtr", "redirectToRoute"), Some(Tier::Hump));
        assert_eq!(tier("redTR", "redirectToRoute"), Some(Tier::Hump));
        assert_eq!(tier("hc", "HTTPClient"), Some(Tier::Hump));
    }

    #[test]
    fn word_start_beats_substring() {
        assert_eq!(tier("Route", "redirectToRoute"), Some(Tier::WordStart));
        assert_eq!(tier("oute", "redirectToRoute"), Some(Tier::Substring));
        assert_eq!(tier("tr", "redirectToRoute"), Some(Tier::WordStart));
        assert!(score(b"Route", b"redirectToRoute") < score(b"oute", b"redirectToRoute"));
    }

    #[test]
    fn prefix_beats_substring() {
        assert_eq!(tier("Get", "GetAllThings"), Some(Tier::Prefix));
        assert_eq!(tier("All", "GetAllThings"), Some(Tier::WordStart));
        assert_eq!(tier("llT", "GetAllThings"), Some(Tier::Substring));
    }

    #[test]
    fn prefix_beats_loose_initials() {
        assert!(score(b"ac", b"act") < score(b"ac", b"ARRAY_COLUMN"));
    }

    #[test]
    fn substring_matches_member_partial() {
        assert_eq!(tier("a", "Draft"), Some(Tier::Substring));
        assert_eq!(tier("a", "cases"), Some(Tier::Substring));
        assert_eq!(tier("vIeWfLa", "ReviewFlag"), Some(Tier::Substring));
    }

    #[test]
    fn fuzzy_needs_a_word_start() {
        assert_eq!(tier("gtt", "GetAllThings"), Some(Tier::Fuzzy));
        assert_eq!(tier("dlc", "DeleteController"), Some(Tier::Fuzzy));
        assert_eq!(tier("dlc", "DashboardLinkCollector"), Some(Tier::Hump));
        assert_eq!(tier("elc", "DeleteController"), None);
        assert_eq!(tier("zzz", "GetAllThings"), None);
    }

    #[test]
    fn empty_query_matches_everything() {
        assert_eq!(tier("", "Anything"), Some(Tier::Prefix));
    }

    #[test]
    fn single_letter_prefers_prefix_over_hump() {
        assert_eq!(tier("G", "GetAllThings"), Some(Tier::Prefix));
    }

    #[test]
    fn case_misses_order_within_a_tier() {
        let exact = score(b"Qux", b"Qux");
        let folded = score(b"QUX", b"Qux");
        let prefix = score(b"Qu", b"QuxBarBaz");
        assert!(exact.is_some_and(Match::is_strict_exact));
        assert!(folded.is_some_and(|matched| matched.is_exact() && !matched.is_strict_exact()));
        assert!(exact < folded);
        assert!(folded < prefix);
    }

    #[test]
    fn repeated_letters_stay_bounded() {
        let candidate = format!("{}xb", "Aa".repeat(30));
        assert_eq!(tier(&format!("{}b", "a".repeat(25)), &candidate), Some(Tier::Fuzzy));
    }
}
