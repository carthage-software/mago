//! A high-performance, allocation-minimal pattern matcher for fully qualified names (FQNs).
//!
//! This module provides a replacement for regex-based matching to improve performance
//! by using a segment-based matching approach.

use mago_atom::starts_with_ignore_case;

/// The separator for namespace segments.
pub const SEPARATOR: char = '\\';

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
pub fn matches(fqcn: &str, pattern: &str, is_constant: bool, treat_as_namespace: bool) -> bool {
    // If pattern contains brace expansion, expand and check each variant.
    if pattern.contains('{') {
        return expand_braces(pattern).iter().any(|p| matches(fqcn, p, is_constant, treat_as_namespace));
    }

    if !pattern.contains('*') {
        let p = pattern.trim_matches(SEPARATOR);
        let f = fqcn.trim_matches(SEPARATOR);

        if p.is_empty() {
            return f.is_empty();
        }

        // if pattern ends with separator, it's a namespace match
        if treat_as_namespace || pattern.ends_with(SEPARATOR) {
            if !starts_with_ignore_case(f, p) {
                return false;
            }

            // Check that it's a full segment match
            return f.len() == p.len() || f.as_bytes().get(p.len()) == Some(&(SEPARATOR as u8));
        }

        // Exact match
        if is_constant {
            let f_last = f.rsplit_once(SEPARATOR);
            let p_last = p.rsplit_once(SEPARATOR);

            return match (f_last, p_last) {
                (Some((f_ns, f_name)), Some((p_ns, p_name))) => f_ns.eq_ignore_ascii_case(p_ns) && f_name == p_name,
                (None, None) => f == p,
                _ => false,
            };
        }
        return f.eq_ignore_ascii_case(p);
    }

    let fqcn = fqcn.trim_matches(SEPARATOR);
    let pattern = pattern.trim_matches(SEPARATOR);

    if pattern == "**" {
        return true;
    }

    if pattern == "*" {
        return !fqcn.contains(SEPARATOR);
    }

    if fqcn.is_empty() || pattern.is_empty() {
        return fqcn == pattern;
    }

    let fqcn_parts: Vec<&str> = fqcn.split(SEPARATOR).collect();
    let pattern_parts: Vec<&str> = pattern.split(SEPARATOR).collect();

    do_match(&fqcn_parts, &pattern_parts, is_constant)
}

/// The recursive matching engine.
fn do_match(fqcn_parts: &[&str], pattern_parts: &[&str], is_constant: bool) -> bool {
    match (fqcn_parts.first(), pattern_parts.first()) {
        (None, None) => true, // Both exhausted, it's a match.
        (_, Some(&"**")) => {
            // If `**` is the last pattern segment, it matches the rest of the FQN.
            if pattern_parts.len() == 1 {
                return true;
            }
            // If FQN is exhausted, `**` can match an empty sequence.
            if fqcn_parts.is_empty() {
                return do_match(fqcn_parts, &pattern_parts[1..], is_constant);
            }
            // `**` can match one or more segments. We try both possibilities:
            // 1. `**` matches nothing, so we match the rest of the pattern against the current FQN.
            // 2. `**` matches one segment, so we match the same pattern against the rest of the FQN.
            do_match(fqcn_parts, &pattern_parts[1..], is_constant)
                || do_match(&fqcn_parts[1..], pattern_parts, is_constant)
        }
        (Some(f_part), Some(p_part)) => {
            let is_last = fqcn_parts.len() == 1 && pattern_parts.len() == 1;
            // Case-sensitive check is only for the last segment.
            let case_sensitive = is_constant && is_last;

            if segment_matches(f_part, p_part, case_sensitive) {
                do_match(&fqcn_parts[1..], &pattern_parts[1..], is_constant)
            } else {
                false
            }
        }
        (None, Some(_)) => {
            // FQN is exhausted, but pattern is not.
            // This is a match only if the rest of the pattern is `**`.
            pattern_parts.len() == 1 && pattern_parts[0] == "**"
        }
        _ => false, // FQN has parts left, but pattern is exhausted.
    }
}

/// Checks if a single FQN segment matches a single pattern segment.
fn segment_matches(fqcn_part: &str, pattern_part: &str, case_sensitive: bool) -> bool {
    if pattern_part == "*" {
        return true;
    }

    if !pattern_part.contains('*') {
        return if case_sensitive { fqcn_part == pattern_part } else { fqcn_part.eq_ignore_ascii_case(pattern_part) };
    }

    // Handle partial wildcards like `*User`, `User*`, `*User*`.
    let p_chunks: Vec<&str> = pattern_part.split('*').collect();
    let mut remainder = fqcn_part;

    // Check first chunk (before the first `*`).
    if !p_chunks[0].is_empty() {
        if remainder.len() < p_chunks[0].len() {
            return false;
        }

        let prefix = &remainder[..p_chunks[0].len()];
        if !(if case_sensitive { prefix == p_chunks[0] } else { prefix.eq_ignore_ascii_case(p_chunks[0]) }) {
            return false;
        }

        remainder = &remainder[p_chunks[0].len()..];
    }

    let last_chunk = match p_chunks.last() {
        Some(chunk) => *chunk,
        None => return false,
    };

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

    // Check middle chunks.
    for chunk in &p_chunks[1..p_chunks.len() - 1] {
        if chunk.is_empty() {
            continue;
        }

        let found = if case_sensitive { remainder.find(chunk) } else { find_ignore_ascii_case(remainder, chunk) };
        if let Some(pos) = found {
            remainder = &remainder[pos + chunk.len()..];
        } else {
            return false;
        }
    }

    true
}

/// A helper to find a substring ignoring ASCII case, without allocation.
fn find_ignore_ascii_case(haystack: &str, needle: &str) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack.as_bytes().windows(needle.len()).position(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}

/// Expands brace expressions in a pattern into all possible variants.
///
/// For example, `App\{Domain,Infrastructure}\*` expands to:
/// - `App\Domain\*`
/// - `App\Infrastructure\*`
///
/// Supports nested and multiple brace groups.
fn expand_braces(pattern: &str) -> Vec<String> {
    let Some(open) = pattern.find('{') else {
        return vec![pattern.to_string()];
    };

    let mut depth = 0;
    let mut close = None;
    for (i, ch) in pattern[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
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
        return vec![pattern.to_string()];
    };

    let prefix = &pattern[..open];
    let suffix = &pattern[close + 1..];
    let alternatives = &pattern[open + 1..close];

    let mut parts = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    for (i, ch) in alternatives.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(&alternatives[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }

    parts.push(&alternatives[start..]);

    let mut results = Vec::with_capacity(parts.len());
    for part in parts {
        let expanded = format!("{prefix}{part}{suffix}");
        results.extend(expand_braces(&expanded));
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        assert!(matches("App\\User", "App\\User", false, false));
        assert!(matches("App\\User", "app\\user", false, false));
        assert!(!matches("App\\User", "App\\Role", false, false));
    }

    #[test]
    fn test_exact_match_constant() {
        assert!(matches("App\\USER", "App\\USER", true, false));
        assert!(matches("App\\USER", "app\\USER", true, false)); // NS is case-insensitive
        assert!(!matches("App\\USER", "App\\User", true, false)); // Name is case-sensitive
        assert!(!matches("App\\User", "App\\Role", true, false));
    }

    #[test]
    fn test_single_wildcard() {
        assert!(matches("App\\User", "App\\*", false, false));
        assert!(matches("App\\Role", "App\\*", false, false));
        assert!(!matches("App\\User\\Profile", "App\\*", false, false));
        assert!(matches("App\\User\\Profile", "App\\User\\*", false, false));
        assert!(matches("App\\User", "*\\User", false, false));
        assert!(!matches("App\\User", "*\\Role", false, false));
    }

    #[test]
    fn test_double_wildcard() {
        assert!(matches("App\\User", "App\\**", false, false));
        assert!(matches("App\\User\\Role", "App\\**", false, false));
        assert!(matches("App\\User\\Role\\Permission", "App\\**", false, false));
        assert!(!matches("Domain\\User", "App\\**", false, false));

        assert!(matches("User\\Role", "**\\Role", false, false));
        assert!(matches("App\\User\\Role", "**\\Role", false, false));
        assert!(!matches("App\\User\\Roles", "**\\Role", false, false));

        assert!(matches("App\\Services\\Notifier", "App\\**\\Notifier", false, false));
        assert!(matches("App\\Notifier", "App\\**\\Notifier", false, false));
        assert!(matches("App\\Domain\\Services\\Notifier", "App\\**\\Services\\**", false, false));
    }

    #[test]
    fn test_partial_wildcard_segment() {
        assert!(matches("UserRepository", "*Repository", false, false));
        assert!(matches("DoctrineUserRepository", "*Repository", false, false));
        assert!(!matches("UserRepository", "*Repo", false, false));

        assert!(matches("UserModel", "User*", false, false));
        assert!(matches("User", "User*", false, false));
        assert!(!matches("RoleModel", "User*", false, false));

        assert!(matches("JsonUserRepository", "*User*", false, false));
        assert!(matches("UserRepository", "*User*", false, false));
        assert!(matches("UserModel", "*User*", false, false));

        assert!(matches("MyUserRepository", "My*Repository", false, false));
        assert!(matches("MyOtherRepository", "My*Repository", false, false));
    }

    #[test]
    fn test_partial_wildcard_with_case() {
        assert!(!matches("USER_REPOSITORY", "*Repository", true, false));
        assert!(matches("USER_REPOSITORY", "*REPOSITORY", true, false));
        assert!(!matches("USER_REPOSITORY", "*repository", true, false));
        assert!(matches("USER_REPOSITORY", "*repository", false, false));
    }

    #[test]
    fn test_constant_with_wildcard() {
        // The case-sensitive check for constants only applies when the last segment is NOT a wildcard.
        assert!(matches("App\\USER", "App\\*", true, false));
        assert!(matches("App\\Services\\USER", "App\\**\\*", true, false));
        assert!(matches("App\\Services\\USER", "App\\**\\USER", true, false));
        assert!(!matches("App\\Services\\USER", "App\\**\\User", true, false));
    }

    #[test]
    fn test_edge_cases() {
        assert!(matches("User", "*", false, false));
        assert!(!matches("App\\User", "*", false, false));
        assert!(matches("App\\User", "**", false, false));
        assert!(matches("", "", false, false));
        assert!(!matches("A", "", false, false));
        assert!(!matches("", "A", false, false));
        assert!(matches("A", "A", false, false));
        assert!(matches("\\App\\User\\", "App\\User", false, false));
        assert!(matches("App\\User", "\\App\\User\\", false, false));
    }

    #[test]
    fn test_complex_middle_wildcard() {
        assert!(matches("A\\B\\C\\D", "A\\**\\D", false, false));
        assert!(matches("A\\D", "A\\**\\D", false, false));
        assert!(!matches("A\\B\\C\\E", "A\\**\\D", false, false));
        assert!(matches("A\\B\\C\\D\\E", "A\\**\\D\\**", false, false));
    }

    #[test]
    fn test_namespace_match() {
        assert!(matches("App\\Domain\\User", "App\\", false, false));
        assert!(matches("App\\Domain\\User", "App\\Domain\\", false, false));
        assert!(!matches("Apples\\Domain\\User", "App\\", false, false));
        assert!(matches("App", "App\\", false, false));
        assert!(matches("App\\", "App\\", false, false));
        assert!(!matches("App", "Application\\", false, false));
    }

    #[test]
    fn test_brace_expansion() {
        assert!(matches("App\\Domain\\User", "App\\{Domain,Infrastructure}\\User", false, false));
        assert!(matches("App\\Infrastructure\\User", "App\\{Domain,Infrastructure}\\User", false, false));
        assert!(!matches("App\\Application\\User", "App\\{Domain,Infrastructure}\\User", false, false));
    }

    #[test]
    fn test_brace_expansion_namespace() {
        assert!(matches("App\\Gateway\\DTO\\Foo", "App\\Gateway\\{DTO,Doctrine}\\", false, false));
        assert!(matches("App\\Gateway\\Doctrine\\Bar", "App\\Gateway\\{DTO,Doctrine}\\", false, false));
        assert!(!matches("App\\Gateway\\Service\\Baz", "App\\Gateway\\{DTO,Doctrine}\\", false, false));
    }

    #[test]
    fn test_brace_expansion_with_wildcards() {
        assert!(matches("App\\Domain\\UserRepository", "App\\{Domain,Infrastructure}\\*Repository", false, false));
        assert!(matches(
            "App\\Infrastructure\\OrderRepository",
            "App\\{Domain,Infrastructure}\\*Repository",
            false,
            false
        ));
        assert!(!matches(
            "App\\Application\\UserRepository",
            "App\\{Domain,Infrastructure}\\*Repository",
            false,
            false
        ));
    }

    #[test]
    fn test_brace_expansion_multiple_groups() {
        assert!(matches("App\\Domain\\User\\Query", "App\\{Domain,Infrastructure}\\{User,Order}\\*", false, false));
        assert!(matches(
            "App\\Infrastructure\\Order\\Command",
            "App\\{Domain,Infrastructure}\\{User,Order}\\*",
            false,
            false
        ));
        assert!(!matches("App\\Domain\\Product\\Query", "App\\{Domain,Infrastructure}\\{User,Order}\\*", false, false));
    }

    #[test]
    fn test_brace_expansion_single_alternative() {
        assert!(matches("App\\Domain\\User", "App\\{Domain}\\User", false, false));
    }

    #[test]
    fn test_no_braces_unchanged() {
        assert!(matches("App\\User", "App\\User", false, false));
        assert!(matches("App\\User", "App\\*", false, false));
    }

    #[test]
    fn test_expand_braces_fn() {
        assert_eq!(expand_braces("A\\{B,C}\\D"), vec!["A\\B\\D", "A\\C\\D"]);
        assert_eq!(expand_braces("no-braces"), vec!["no-braces"]);
        assert_eq!(expand_braces("{A,B}"), vec!["A", "B"]);
        assert_eq!(expand_braces("{A,B}\\{C,D}"), vec!["A\\C", "A\\D", "B\\C", "B\\D"]);
    }
}
