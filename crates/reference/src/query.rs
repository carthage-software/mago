use std::str::FromStr;

/// Represents different ways to match input text.
///
/// - `Exact(String, bool)` matches if the text is exactly equal to the query string.
/// - `StartsWith(String, bool)` matches if the text starts with the query string.
/// - `Contains(String, bool)` matches if the text contains the query string.
/// - `EndsWith(String, bool)` matches if the text ends with the query string.
///
/// The `bool` field indicates whether the match should be **case-sensitive** (`true`)
/// or **case-insensitive** (`false`). By default, it is **case-insensitive** unless the
/// query string is prefixed with `c:` (for “case-sensitive”) or `i:` (for “case-insensitive”).
#[derive(Debug, Clone, PartialEq)]
pub enum Query {
    /// Variant for an exact match.
    /// The second parameter specifies if it's case-sensitive.
    Exact(String, bool),
    /// Variant for matching if the text starts with the query.
    /// The second parameter specifies if it's case-sensitive.
    StartsWith(String, bool),
    /// Variant for matching if the text contains the query.
    /// The second parameter specifies if it's case-sensitive.
    Contains(String, bool),
    /// Variant for matching if the text ends with the query.
    /// The second parameter specifies if it's case-sensitive.
    EndsWith(String, bool),
}

impl Query {
    /// Checks whether the given `text` matches this query.
    ///
    /// By default, matches are **case-insensitive** unless specified otherwise in the query.
    /// If `case_sensitive` is `true`, the match must respect exact casing.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to check against the query.
    ///
    /// # Returns
    ///
    /// Returns `true` if the text matches according to the query variant (respecting case sensitivity),
    /// or `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::str::FromStr;
    /// # use mago_reference::query::Query;
    ///
    /// // Case-insensitive exact match example:
    /// let query = Query::from_str("=Hello").unwrap();
    /// assert!(query.matches("hello"));
    ///
    /// // Case-sensitive contains example:
    /// let query = Query::from_str("c:ell").unwrap();
    /// assert!(query.matches("Well")); // false
    /// assert!(query.matches("ell"));  // true if text is exactly "ell"
    /// ```
    pub fn matches(&self, text: &str) -> bool {
        match self {
            Query::Exact(q, case_sensitive) => {
                if *case_sensitive {
                    text == q
                } else {
                    text.eq_ignore_ascii_case(q)
                }
            }
            Query::StartsWith(q, case_sensitive) => {
                if *case_sensitive {
                    text.starts_with(q)
                } else {
                    text.to_ascii_lowercase().starts_with(&q.to_ascii_lowercase())
                }
            }
            Query::Contains(q, case_sensitive) => {
                if *case_sensitive {
                    text.contains(q)
                } else {
                    text.to_ascii_lowercase().contains(&q.to_ascii_lowercase())
                }
            }
            Query::EndsWith(q, case_sensitive) => {
                if *case_sensitive {
                    text.ends_with(q)
                } else {
                    text.to_ascii_lowercase().ends_with(&q.to_ascii_lowercase())
                }
            }
        }
    }
}

impl FromStr for Query {
    type Err = ();

    /// Parses a string slice into a `Query` with optional case sensitivity.
    ///
    /// The parsing follows a simple syntax:
    ///
    /// - **Case Sensitivity**:
    ///   - If the string starts with `"c:"`, the query is **case-sensitive**.
    ///   - If it starts with `"i:"`, the query is explicitly case-insensitive.
    ///   - Otherwise, it defaults to **case-insensitive**.
    ///
    /// - **Match Variants**:
    ///   - If, after removing any `c:` or `i:` prefix, the string starts with `=`
    ///     => parsed as `Exact(...)`.
    ///   - If it starts with `^`
    ///     => parsed as `StartsWith(...)`.
    ///   - If it ends with `$`
    ///     => parsed as `EndsWith(...)`.
    ///   - Otherwise, it is parsed as `Contains(...)`.
    ///
    /// Whitespace is trimmed, and an empty input results in an error.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::str::FromStr;
    /// # use mago_reference::query::Query;
    ///
    /// // Case-insensitive exact match
    /// let query = Query::from_str("=Exact").unwrap();
    /// if let Query::Exact(ref s, false) = query {
    ///     assert_eq!(s, "Exact");
    /// } else {
    ///     panic!("Expected case-insensitive Exact variant");
    /// }
    ///
    /// // Case-sensitive starts-with match
    /// let query = Query::from_str("c:^Hello").unwrap();
    /// if let Query::StartsWith(ref s, true) = query {
    ///     assert_eq!(s, "Hello");
    /// } else {
    ///     panic!("Expected case-sensitive StartsWith variant");
    /// }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // 1) Trim whitespace
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(());
        }

        // 2) Determine case sensitivity
        let (remaining, case_sensitive) = if let Some(rest) = trimmed.strip_prefix("c:") {
            (rest, true)
        } else if let Some(rest) = trimmed.strip_prefix("i:") {
            (rest, false)
        } else {
            (trimmed, false) // default case-insensitive
        };

        // 3) Inspect the resulting string to see which variant we need
        // Lowercase not mandatory for case-sensitive scenario, but needed to check
        // special prefix/suffix markers only. We'll treat them consistently.
        let lower = remaining.to_ascii_lowercase();

        if lower.is_empty() {
            return Err(());
        }

        // Exact if starts with '='
        if let Some(rest) = remaining.strip_prefix('=') {
            Ok(Query::Exact(rest.to_string(), case_sensitive))
        }
        // StartsWith if starts with '^'
        else if let Some(rest) = remaining.strip_prefix('^') {
            Ok(Query::StartsWith(rest.to_string(), case_sensitive))
        }
        // EndsWith if ends with '$'
        else if lower.ends_with('$') {
            let query_str = &remaining[..remaining.len() - 1];
            Ok(Query::EndsWith(query_str.to_string(), case_sensitive))
        }
        // Otherwise, it's Contains
        else {
            Ok(Query::Contains(remaining.to_string(), case_sensitive))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_case_sensitive_exact() {
        let query = Query::from_str("c:=Hello").unwrap();
        match &query {
            Query::Exact(s, true) => assert_eq!(s, "Hello"),
            _ => panic!("Expected case-sensitive Exact variant"),
        }

        assert!(query.matches("Hello"));
        assert!(!query.matches("hello"));
    }

    #[test]
    fn test_case_insensitive_default() {
        // =Hello without c: => case-insensitive
        let query = Query::from_str("=Hello").unwrap();
        match &query {
            Query::Exact(s, false) => assert_eq!(s, "Hello"),
            _ => panic!("Expected case-insensitive Exact variant"),
        }

        assert!(query.matches("hello"));
        assert!(query.matches("HELLO"));
    }

    #[test]
    fn test_starts_with_case_sensitive() {
        let query = Query::from_str("c:^Hell").unwrap();
        match &query {
            Query::StartsWith(s, true) => assert_eq!(s, "Hell"),
            _ => panic!("Expected case-sensitive StartsWith variant"),
        }
        assert!(query.matches("Hellworld"));
        assert!(!query.matches("helloworld")); // different case
    }

    #[test]
    fn test_contains_case_insensitive() {
        let query = Query::from_str("CoNtEnT").unwrap();
        match &query {
            Query::Contains(s, false) => assert_eq!(s, "CoNtEnT"),
            _ => panic!("Expected case-insensitive Contains variant"),
        }
        assert!(query.matches("this content is here"));
        assert!(query.matches("CONTENT!"));
    }

    #[test]
    fn test_ends_with() {
        let query = Query::from_str("something$").unwrap();
        match &query {
            Query::EndsWith(s, false) => assert_eq!(s, "something"),
            _ => panic!("Expected ends-with, case-insensitive"),
        }

        assert!(query.matches("ANYTHING someThing"));
    }

    #[test]
    fn test_empty_string_error() {
        assert!(Query::from_str("  ").is_err());
    }
}
