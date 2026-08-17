/// Checks if a string is numeric according to PHP's definition.
///
/// This function checks if a string is numeric by trimming leading/trailing whitespace,
/// removing leading zeros, and checking if the remaining string can be parsed as a number.
///
/// # Arguments
///
/// * `input` - The string to check.
///
/// # Returns
///
/// * `true` - If the string is numeric.
/// * `false` - If the string is not numeric.
pub fn str_is_numeric_bytes(input: &[u8]) -> bool {
    let Ok(s) = std::str::from_utf8(input) else { return false };
    str_is_numeric(s)
}

pub fn str_is_numeric(input: &str) -> bool {
    let mut maybe_numeric = input.trim();
    if maybe_numeric.is_empty() {
        return false;
    }

    if maybe_numeric.starts_with('+') || maybe_numeric.starts_with('-') {
        maybe_numeric = &maybe_numeric[1..];

        if maybe_numeric.is_empty() {
            return false;
        }
    }

    maybe_numeric = maybe_numeric.trim_start_matches('0');
    if maybe_numeric.is_empty() {
        return true;
    }

    maybe_numeric.parse::<f64>().is_ok()
}

/// Increments an alphanumeric string.
///
/// Rust implementation based on PHP's `str_increment` function from php-src:
/// <https://github.com/php/php-src/blob/1de16c7f15f3f927bf7e7c26b3a6b1bd5803b1cc/ext/standard/string.c#L1227>
///
/// # Arguments
///
/// * `input` - The string to increment
///
/// # Returns
///
/// * `Some(String)` - The incremented string on success
/// * `None` - If the input is empty or contains non-alphanumeric ASCII characters
pub fn str_increment_bytes(input: &[u8]) -> Option<String> {
    str_increment(std::str::from_utf8(input).ok()?)
}

pub fn str_increment(input: &str) -> Option<String> {
    if input.is_empty() {
        return None;
    }

    let input_bytes = input.as_bytes();
    let is_alnum = input_bytes.iter().all(|&b| b.is_ascii_alphanumeric());

    if !is_alnum {
        return None;
    }

    let mut bytes = input_bytes.to_vec();
    let len = bytes.len();
    let mut current_idx = len;

    loop {
        if current_idx == 0 {
            let first_char_of_original_input = input_bytes[0];

            let char_to_prepend = match first_char_of_original_input {
                b'z' => b'a',
                b'Z' => b'A',
                b'9' => b'1',
                _ => {
                    #[allow(clippy::unreachable)]
                    {
                        unreachable!("unexpected character for carry-over: {first_char_of_original_input}");
                    }
                }
            };

            let mut new_bytes = Vec::with_capacity(len + 1);
            new_bytes.push(char_to_prepend);
            new_bytes.extend_from_slice(&bytes);

            // Safety: All characters are known to be valid ASCII.
            return Some(unsafe { String::from_utf8_unchecked(new_bytes) });
        }

        current_idx -= 1;
        let current_byte = bytes[current_idx];

        match current_byte {
            b'a'..=b'y' | b'A'..=b'Y' | b'0'..=b'8' => {
                bytes[current_idx] = current_byte + 1;

                // Safety: All characters are known to be valid ASCII.
                return Some(unsafe { String::from_utf8_unchecked(bytes) });
            }
            b'z' => {
                bytes[current_idx] = b'a';
            }
            b'Z' => {
                bytes[current_idx] = b'A';
            }
            b'9' => {
                bytes[current_idx] = b'0';
            }
            _ => {
                #[allow(clippy::unreachable)]
                {
                    unreachable!("non-alphanumeric character found post-validation");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_numeric() {
        assert!(str_is_numeric("123"));
        assert!(str_is_numeric("0"));
        assert!(str_is_numeric("-123"));
        assert!(str_is_numeric("+123"));
        assert!(!str_is_numeric("abc"));
        assert!(str_is_numeric("12.34"));
        assert!(str_is_numeric("12e3"));
        assert!(!str_is_numeric(""));
        assert!(!str_is_numeric("  "));
    }

    #[test]
    fn test_increment_basic() {
        assert_eq!(str_increment("hello"), Some("hellp".to_string()));
        assert_eq!(str_increment("PHP"), Some("PHQ".to_string()));
        assert_eq!(str_increment("rust"), Some("rusu".to_string()));
        assert_eq!(str_increment("abc123"), Some("abc124".to_string()));
    }

    #[test]
    fn test_increment_with_carries() {
        assert_eq!(str_increment("hellz"), Some("helma".to_string()));
        assert_eq!(str_increment("TESTZ"), Some("TESUA".to_string()));
        assert_eq!(str_increment("xyz"), Some("xza".to_string()));
        assert_eq!(str_increment("zz"), Some("aaa".to_string()));
        assert_eq!(str_increment("ZZ"), Some("AAA".to_string()));
    }

    #[test]
    fn test_increment_with_numeric_carries() {
        assert_eq!(str_increment("9"), Some("10".to_string()));
        assert_eq!(str_increment("99"), Some("100".to_string()));
        assert_eq!(str_increment("999"), Some("1000".to_string()));
        assert_eq!(str_increment("abc9"), Some("abd0".to_string()));
        assert_eq!(str_increment("abc99"), Some("abd00".to_string()));
    }

    #[test]
    fn test_increment_mixed_alphanumeric() {
        assert_eq!(str_increment("a9"), Some("b0".to_string()));
        assert_eq!(str_increment("a99z"), Some("b00a".to_string()));
        assert_eq!(str_increment("Z9"), Some("AA0".to_string()));
        assert_eq!(str_increment("9z"), Some("10a".to_string()));
        assert_eq!(str_increment("9Z"), Some("10A".to_string()));
    }

    #[test]
    fn test_increment_at_boundaries() {
        assert_eq!(str_increment("z"), Some("aa".to_string()));
        assert_eq!(str_increment("Z"), Some("AA".to_string()));
    }

    #[test]
    fn test_increment_failure_cases() {
        assert_eq!(str_increment(""), None);
        assert_eq!(str_increment("hello!"), None);
        assert_eq!(str_increment("test-123"), None);
        assert_eq!(str_increment("user@example.com"), None);
        assert_eq!(str_increment("русский"), None);
    }
}
