use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::RwLock;

// A global, thread-safe cache for compiled regex patterns.
// This is the core of the optimization. `LazyLock` ensures it's initialized
// only once, and `RwLock` allows concurrent reads without blocking.
static REGEX_CACHE: LazyLock<RwLock<HashMap<String, Regex>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

/// Checks if a fully qualified name matches a given pattern.
pub fn matches_patterxn(fqn: &str, pattern: &str, is_namespace: bool) -> bool {
    if !pattern.contains('*') {
        if is_namespace {
            let ns_pattern = pattern.trim_end_matches('\\');
            if !fqn.to_ascii_lowercase().starts_with(&ns_pattern.to_ascii_lowercase()) {
                return false;
            }

            fqn.len() == ns_pattern.len() || fqn.as_bytes().get(ns_pattern.len()) == Some(&b'\\')
        } else {
            // This is an exact symbol match.
            fqn.eq_ignore_ascii_case(pattern)
        }
    } else {
        let cache = REGEX_CACHE.read().unwrap();
        if let Some(re) = cache.get(pattern) {
            return re.is_match(fqn);
        }

        drop(cache);

        let mut cache = REGEX_CACHE.write().unwrap();

        if let Some(re) = cache.get(pattern) {
            return re.is_match(fqn);
        }

        let regex_pattern = pattern_to_regex(pattern);
        if let Ok(re) = Regex::new(&regex_pattern) {
            cache.insert(pattern.to_string(), re.clone());
            re.is_match(fqn)
        } else {
            false
        }
    }
}

fn pattern_to_regex(pattern: &str) -> String {
    let mut regex = String::from("(?i)^");
    let mut chars = pattern.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '*' => {
                if chars.peek() == Some(&'*') {
                    chars.next();
                    regex.push_str(".*");
                } else {
                    regex.push_str("[^\\\\]+");
                }
            }
            '\\' => regex.push_str("\\\\"),
            '.' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '^' | '$' => {
                regex.push('\\');
                regex.push(ch);
            }
            _ => regex.push(ch),
        }
    }

    regex.push('$');
    regex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_pattern_exact() {
        assert!(matches_pattern("Foo\\Bar", "Foo\\Bar", false));
        assert!(!matches_pattern("Foo\\Bar", "Foo\\Baz", false));
    }

    #[test]
    fn test_matches_namespace_prefix() {
        assert!(matches_pattern("Foo\\Bar\\Baz", "Foo\\", true));
        assert!(matches_pattern("Foo\\Bar\\Baz", "Foo\\Bar\\", true));
        assert!(matches_pattern("Foo\\Bar", "Foo\\Bar\\", true));
        assert!(!matches_pattern("Foo\\BarBaz", "Foo\\Bar\\", true));
        assert!(!matches_pattern("Another\\Foo\\Bar", "Foo\\Bar\\", true));
    }

    #[test]
    fn test_matches_pattern_single_wildcard() {
        assert!(matches_pattern("Foo\\Bar", "Foo\\*", false));
        assert!(matches_pattern("Foo\\Baz", "Foo\\*", false));
        assert!(!matches_pattern("Foo\\Bar\\Baz", "Foo\\*", false));
    }

    #[test]
    fn test_matches_pattern_recursive_wildcard() {
        assert!(matches_pattern("Foo\\Bar", "Foo\\**", false));
        assert!(matches_pattern("Foo\\Bar\\Baz", "Foo\\**", false));
        assert!(matches_pattern("Foo\\Bar\\Baz\\Qux", "Foo\\**", false));
    }
}
