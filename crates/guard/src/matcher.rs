//! A high-performance, allocation-minimal pattern matcher for fully qualified names (FQNs).
//!
//! This module provides a replacement for regex-based matching to improve performance
//! by using a segment-based matching approach.

use mago_word::starts_with_ignore_case;

/// The separator for namespace segments.
pub const SEPARATOR: u8 = b'\\';

/// Checks if a fully qualified name (`fqcn`) matches a given `pattern`.
///
/// # Arguments
///
/// * `fqcn`: The fully qualified name to check (e.g., `App\\Domain\\Model\\User`).
/// * `pattern`: The pattern to match against. It can contain wildcards:
///   - `*`: Matches any single segment.
///   - `**`: Matches any number of segments (including zero).
///   - Segments can also contain `*` for partial matches (e.g., `*Repository`).
/// * `is_constant`: If `true`, the last segment of the FQN is matched case-sensitively.
/// * `treat_as_namespace`: If `true`, the pattern is treated as a namespace, meaning it matches
///   any FQN that starts with the pattern.
///
/// # Returns
///
/// `true` if the `fqcn` matches the `pattern`, `false` otherwise.
pub fn matches(fqcn: &[u8], pattern: &[u8], is_constant: bool, treat_as_namespace: bool) -> bool {
    if pattern.contains(&b'{') {
        return expand_braces(pattern).iter().any(|p| matches(fqcn, p, is_constant, treat_as_namespace));
    }

    if !pattern.contains(&b'*') {
        let p = trim_separator(pattern);
        let f = trim_separator(fqcn);

        if p.is_empty() {
            return f.is_empty();
        }

        if treat_as_namespace || pattern.last() == Some(&SEPARATOR) {
            if !starts_with_ignore_case(f, p) {
                return false;
            }

            return f.len() == p.len() || f.get(p.len()) == Some(&SEPARATOR);
        }

        if is_constant {
            let f_last = rsplit_once_separator(f);
            let p_last = rsplit_once_separator(p);

            return match (f_last, p_last) {
                (Some((f_ns, f_name)), Some((p_ns, p_name))) => f_ns.eq_ignore_ascii_case(p_ns) && f_name == p_name,
                (None, None) => f == p,
                _ => false,
            };
        }
        return f.eq_ignore_ascii_case(p);
    }

    let fqcn = trim_separator(fqcn);
    let pattern = trim_separator(pattern);

    if pattern == b"**" {
        return true;
    }

    if pattern == b"*" {
        return !fqcn.contains(&SEPARATOR);
    }

    if fqcn.is_empty() || pattern.is_empty() {
        return fqcn == pattern;
    }

    do_match(Some(fqcn), Some(pattern), is_constant)
}

fn trim_separator(s: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = s.len();
    while start < end && s[start] == SEPARATOR {
        start += 1;
    }
    while end > start && s[end - 1] == SEPARATOR {
        end -= 1;
    }
    &s[start..end]
}

fn split_first_separator(s: &[u8]) -> (&[u8], Option<&[u8]>) {
    match s.iter().position(|&b| b == SEPARATOR) {
        Some(position) => (&s[..position], Some(&s[position + 1..])),
        None => (s, None),
    }
}

fn rsplit_once_separator(s: &[u8]) -> Option<(&[u8], &[u8])> {
    let pos = s.iter().rposition(|&b| b == SEPARATOR)?;
    Some((&s[..pos], &s[pos + 1..]))
}

/// The recursive matching engine.
fn do_match(fqcn: Option<&[u8]>, pattern: Option<&[u8]>, is_constant: bool) -> bool {
    match (fqcn.map(split_first_separator), pattern.map(split_first_separator)) {
        (None, None) => true,
        (_, Some((b"**", pattern_rest))) => {
            if pattern_rest.is_none() {
                return true;
            }

            do_match(fqcn, pattern_rest, is_constant)
                || fqcn.is_some_and(|fqcn| do_match(split_first_separator(fqcn).1, pattern, is_constant))
        }
        (Some((f_part, fqcn_rest)), Some((p_part, pattern_rest))) => {
            let is_last = fqcn_rest.is_none() && pattern_rest.is_none();
            let case_sensitive = is_constant && is_last;

            if segment_matches(f_part, p_part, case_sensitive) {
                do_match(fqcn_rest, pattern_rest, is_constant)
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Checks if a single FQN segment matches a single pattern segment.
fn segment_matches(fqcn_part: &[u8], pattern_part: &[u8], case_sensitive: bool) -> bool {
    if pattern_part == b"*" {
        return true;
    }

    if !pattern_part.contains(&b'*') {
        return if case_sensitive { fqcn_part == pattern_part } else { fqcn_part.eq_ignore_ascii_case(pattern_part) };
    }

    let mut chunks = pattern_part.split(|&b| b == b'*');
    let first_chunk = chunks.next().unwrap_or_default();
    let last_chunk = chunks.next_back().unwrap_or_default();
    let mut remainder: &[u8] = fqcn_part;

    if !first_chunk.is_empty() {
        if remainder.len() < first_chunk.len() {
            return false;
        }

        let prefix = &remainder[..first_chunk.len()];
        if !(if case_sensitive { prefix == first_chunk } else { prefix.eq_ignore_ascii_case(first_chunk) }) {
            return false;
        }

        remainder = &remainder[first_chunk.len()..];
    }

    if !last_chunk.is_empty() {
        if remainder.len() < last_chunk.len() {
            return false;
        }

        let suffix = &remainder[remainder.len() - last_chunk.len()..];
        if !(if case_sensitive { suffix == last_chunk } else { suffix.eq_ignore_ascii_case(last_chunk) }) {
            return false;
        }

        remainder = &remainder[..remainder.len() - last_chunk.len()];
    }

    for chunk in chunks {
        if chunk.is_empty() {
            continue;
        }

        let found =
            if case_sensitive { find_subslice(remainder, chunk) } else { find_ignore_ascii_case(remainder, chunk) };
        if let Some(pos) = found {
            remainder = &remainder[pos + chunk.len()..];
        } else {
            return false;
        }
    }

    true
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack.windows(needle.len()).position(|window| window == needle)
}

fn find_ignore_ascii_case(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack.windows(needle.len()).position(|window| window.eq_ignore_ascii_case(needle))
}

/// Expands brace expressions in a pattern into all possible variants.
///
/// For example, `App\{Domain,Infrastructure}\*` expands to:
/// - `App\Domain\*`
/// - `App\Infrastructure\*`
///
/// Supports nested and multiple brace groups.
fn expand_braces(pattern: &[u8]) -> Vec<Vec<u8>> {
    let Some(open) = pattern.iter().position(|&b| b == b'{') else {
        return vec![pattern.to_vec()];
    };

    let mut depth = 0i32;
    let mut close = None;
    for (i, &b) in pattern[open..].iter().enumerate() {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(open + i);
                    break;
                }
            }
            _ => {}
        }
    }

    let Some(close) = close else {
        return vec![pattern.to_vec()];
    };

    let prefix = &pattern[..open];
    let suffix = &pattern[close + 1..];
    let alternatives = &pattern[open + 1..close];

    let mut parts: Vec<&[u8]> = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    for (i, &b) in alternatives.iter().enumerate() {
        match b {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            b',' if depth == 0 => {
                parts.push(&alternatives[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }

    parts.push(&alternatives[start..]);

    let mut results = Vec::with_capacity(parts.len());
    for part in parts {
        let mut expanded = Vec::with_capacity(prefix.len() + part.len() + suffix.len());
        expanded.extend_from_slice(prefix);
        expanded.extend_from_slice(part);
        expanded.extend_from_slice(suffix);
        results.extend(expand_braces(&expanded));
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        assert!(matches(b"App\\User", b"App\\User", false, false));
        assert!(matches(b"App\\User", b"app\\user", false, false));
        assert!(!matches(b"App\\User", b"App\\Role", false, false));
    }

    #[test]
    fn test_exact_match_constant() {
        assert!(matches(b"App\\USER", b"App\\USER", true, false));
        assert!(matches(b"App\\USER", b"app\\USER", true, false));
        assert!(!matches(b"App\\USER", b"App\\User", true, false));
        assert!(!matches(b"App\\User", b"App\\Role", true, false));
    }

    #[test]
    fn test_single_wildcard() {
        assert!(matches(b"App\\User", b"App\\*", false, false));
        assert!(matches(b"App\\Role", b"App\\*", false, false));
        assert!(!matches(b"App\\User\\Profile", b"App\\*", false, false));
        assert!(matches(b"App\\User\\Profile", b"App\\User\\*", false, false));
        assert!(matches(b"App\\User", b"*\\User", false, false));
        assert!(!matches(b"App\\User", b"*\\Role", false, false));
    }

    #[test]
    fn test_double_wildcard() {
        assert!(matches(b"App\\User", b"App\\**", false, false));
        assert!(matches(b"App\\User\\Role", b"App\\**", false, false));
        assert!(matches(b"App\\User\\Role\\Permission", b"App\\**", false, false));
        assert!(!matches(b"Domain\\User", b"App\\**", false, false));

        assert!(matches(b"User\\Role", b"**\\Role", false, false));
        assert!(matches(b"App\\User\\Role", b"**\\Role", false, false));
        assert!(!matches(b"App\\User\\Roles", b"**\\Role", false, false));

        assert!(matches(b"App\\Services\\Notifier", b"App\\**\\Notifier", false, false));
        assert!(matches(b"App\\Notifier", b"App\\**\\Notifier", false, false));
        assert!(matches(b"App\\Domain\\Services\\Notifier", b"App\\**\\Services\\**", false, false));
    }

    #[test]
    fn test_partial_wildcard_segment() {
        assert!(matches(b"UserRepository", b"*Repository", false, false));
        assert!(matches(b"DoctrineUserRepository", b"*Repository", false, false));
        assert!(!matches(b"UserRepository", b"*Repo", false, false));

        assert!(matches(b"UserModel", b"User*", false, false));
        assert!(matches(b"User", b"User*", false, false));
        assert!(!matches(b"RoleModel", b"User*", false, false));

        assert!(matches(b"JsonUserRepository", b"*User*", false, false));
        assert!(matches(b"UserRepository", b"*User*", false, false));
        assert!(matches(b"UserModel", b"*User*", false, false));

        assert!(matches(b"MyUserRepository", b"My*Repository", false, false));
        assert!(matches(b"MyOtherRepository", b"My*Repository", false, false));
    }

    #[test]
    fn test_partial_wildcard_with_case() {
        assert!(!matches(b"USER_REPOSITORY", b"*Repository", true, false));
        assert!(matches(b"USER_REPOSITORY", b"*REPOSITORY", true, false));
        assert!(!matches(b"USER_REPOSITORY", b"*repository", true, false));
        assert!(matches(b"USER_REPOSITORY", b"*repository", false, false));
    }

    #[test]
    fn test_constant_with_wildcard() {
        assert!(matches(b"App\\USER", b"App\\*", true, false));
        assert!(matches(b"App\\Services\\USER", b"App\\**\\*", true, false));
        assert!(matches(b"App\\Services\\USER", b"App\\**\\USER", true, false));
        assert!(!matches(b"App\\Services\\USER", b"App\\**\\User", true, false));
    }

    #[test]
    fn test_edge_cases() {
        assert!(matches(b"User", b"*", false, false));
        assert!(!matches(b"App\\User", b"*", false, false));
        assert!(matches(b"App\\User", b"**", false, false));
        assert!(matches(b"", b"", false, false));
        assert!(!matches(b"A", b"", false, false));
        assert!(!matches(b"", b"A", false, false));
        assert!(matches(b"A", b"A", false, false));
        assert!(matches(b"\\App\\User\\", b"App\\User", false, false));
        assert!(matches(b"App\\User", b"\\App\\User\\", false, false));
    }

    #[test]
    fn test_empty_segments_and_repeated_wildcards() {
        for (name, pattern, is_constant, expected) in [
            (b"A\\\\B".as_slice(), b"A\\*\\B".as_slice(), false, true),
            (b"A\\\\B", b"A\\B", false, false),
            (b"A\\B", b"A\\**\\**\\B", false, true),
            (b"A\\B\\C\\B", b"A\\**\\B", false, true),
            (b"A\\B\\C", b"A\\**\\B", false, false),
            (b"A\\USER", b"a\\**\\USER", true, true),
            (b"A\\USER", b"a\\**\\User", true, false),
            (b"PrefixMiddleSuffix", b"Prefix**Middle***Suffix", false, true),
            (b"PrefixSuffixMiddle", b"Prefix**Middle***Suffix", false, false),
            (b"PrefixSuffix", b"Prefix**Suffix", false, true),
            (b"Prefix", b"Prefix*Prefix", false, false),
        ] {
            assert_eq!(matches(name, pattern, is_constant, false), expected, "{name:?} against {pattern:?}");
        }
    }

    #[test]
    fn test_complex_middle_wildcard() {
        assert!(matches(b"A\\B\\C\\D", b"A\\**\\D", false, false));
        assert!(matches(b"A\\D", b"A\\**\\D", false, false));
        assert!(!matches(b"A\\B\\C\\E", b"A\\**\\D", false, false));
        assert!(matches(b"A\\B\\C\\D\\E", b"A\\**\\D\\**", false, false));
    }

    #[test]
    fn test_namespace_match() {
        assert!(matches(b"App\\Domain\\User", b"App\\", false, false));
        assert!(matches(b"App\\Domain\\User", b"App\\Domain\\", false, false));
        assert!(!matches(b"Apples\\Domain\\User", b"App\\", false, false));
        assert!(matches(b"App", b"App\\", false, false));
        assert!(matches(b"App\\", b"App\\", false, false));
        assert!(!matches(b"App", b"Application\\", false, false));
    }

    #[test]
    fn test_brace_expansion() {
        assert!(matches(b"App\\Domain\\User", b"App\\{Domain,Infrastructure}\\User", false, false));
        assert!(matches(b"App\\Infrastructure\\User", b"App\\{Domain,Infrastructure}\\User", false, false));
        assert!(!matches(b"App\\Application\\User", b"App\\{Domain,Infrastructure}\\User", false, false));
    }

    #[test]
    fn test_brace_expansion_namespace() {
        assert!(matches(b"App\\Gateway\\DTO\\Foo", b"App\\Gateway\\{DTO,Doctrine}\\", false, false));
        assert!(matches(b"App\\Gateway\\Doctrine\\Bar", b"App\\Gateway\\{DTO,Doctrine}\\", false, false));
        assert!(!matches(b"App\\Gateway\\Service\\Baz", b"App\\Gateway\\{DTO,Doctrine}\\", false, false));
    }

    #[test]
    fn test_brace_expansion_with_wildcards() {
        assert!(matches(b"App\\Domain\\UserRepository", b"App\\{Domain,Infrastructure}\\*Repository", false, false));
        assert!(matches(
            b"App\\Infrastructure\\OrderRepository",
            b"App\\{Domain,Infrastructure}\\*Repository",
            false,
            false
        ));
        assert!(!matches(
            b"App\\Application\\UserRepository",
            b"App\\{Domain,Infrastructure}\\*Repository",
            false,
            false
        ));
    }

    #[test]
    fn test_brace_expansion_multiple_groups() {
        assert!(matches(b"App\\Domain\\User\\Query", b"App\\{Domain,Infrastructure}\\{User,Order}\\*", false, false));
        assert!(matches(
            b"App\\Infrastructure\\Order\\Command",
            b"App\\{Domain,Infrastructure}\\{User,Order}\\*",
            false,
            false
        ));
        assert!(!matches(
            b"App\\Domain\\Product\\Query",
            b"App\\{Domain,Infrastructure}\\{User,Order}\\*",
            false,
            false
        ));
    }

    #[test]
    fn test_brace_expansion_single_alternative() {
        assert!(matches(b"App\\Domain\\User", b"App\\{Domain}\\User", false, false));
    }

    #[test]
    fn test_no_braces_unchanged() {
        assert!(matches(b"App\\User", b"App\\User", false, false));
        assert!(matches(b"App\\User", b"App\\*", false, false));
    }

    #[test]
    fn test_expand_braces_fn() {
        assert_eq!(expand_braces(b"A\\{B,C}\\D"), vec![b"A\\B\\D".to_vec(), b"A\\C\\D".to_vec()]);
        assert_eq!(expand_braces(b"no-braces"), vec![b"no-braces".to_vec()]);
        assert_eq!(expand_braces(b"{A,B}"), vec![b"A".to_vec(), b"B".to_vec()]);
        assert_eq!(
            expand_braces(b"{A,B}\\{C,D}"),
            vec![b"A\\C".to_vec(), b"A\\D".to_vec(), b"B\\C".to_vec(), b"B\\D".to_vec()]
        );
    }
}
