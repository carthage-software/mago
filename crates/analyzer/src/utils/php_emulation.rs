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
/// Rust implementation based on PHP's str_increment function from php-src:
/// https://github.com/php/php-src/blob/1de16c7f15f3f927bf7e7c26b3a6b1bd5803b1cc/ext/standard/string.c#L1227
///
/// # Arguments
///
/// * `input` - The string to increment
///
/// # Returns
///
/// * `Some(String)` - The incremented string on success
/// * `None` - If the input is empty or contains non-alphanumeric ASCII characters
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
                    unreachable!("unexpected character for carry-over: {first_char_of_original_input}");
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
                unreachable!("non-alphanumeric character found post-validation");
            }
        }
    }
}

/// Decrements an alphanumeric string.
///
/// Rust implementation based on PHP's str_decrement function from php-src:
/// https://github.com/php/php-src/blob/1de16c7f15f3f927bf7e7c26b3a6b1bd5803b1cc/ext/standard/string.c#L1283
///
/// # Arguments
///
/// * `input` - The string to decrement
///
/// # Returns
///
/// * `Some(String)` - The decremented string on success
/// * `None` - If the input is empty, contains non-alphanumeric ASCII characters,
///   or is out of decrement range (like "0" or "a")
#[allow(dead_code)]
pub fn str_decrement(input: &str) -> Option<String> {
    if input.is_empty() {
        return None;
    }

    let input_bytes = input.as_bytes();
    let is_alnum = input_bytes.iter().all(|&b| b.is_ascii_alphanumeric());

    if !is_alnum {
        return None;
    }

    let input_bytes_length = input_bytes.len();

    if (input_bytes_length >= 1 && input_bytes[0] == b'0')
        || (input_bytes_length == 1 && (b'A' == input_bytes[0] || b'a' == input_bytes[0]))
    {
        return None;
    }

    let mut bytes = input_bytes.to_vec();
    let len = bytes.len();
    let mut borrow = true;

    for i in (0..len).rev() {
        if !borrow {
            break;
        }

        let current_byte = bytes[i];
        match current_byte {
            b'b'..=b'z' | b'B'..=b'Z' | b'1'..=b'9' => {
                bytes[i] = current_byte - 1;
                borrow = false;
            }
            b'a' => {
                bytes[i] = b'z';

                if i == 0 {
                    return Some(unsafe { String::from_utf8_unchecked(bytes[1..].to_vec()) });
                }
            }
            b'A' => {
                bytes[i] = b'Z';
                if i == 0 {
                    return Some(unsafe { String::from_utf8_unchecked(bytes[1..].to_vec()) });
                }
            }
            b'0' => {
                bytes[i] = b'9';
            }
            _ => {
                unreachable!("non-alphanumeric character found post-validation during decrement");
            }
        }
    }

    debug_assert!(!bytes.is_empty(), "bytes became empty unexpectedly");

    if bytes[0] == b'0' {
        // Safety: All characters are known to be valid ASCII.
        return Some(unsafe { String::from_utf8_unchecked(bytes[1..].to_vec()) });
    }

    // Safety: All characters are known to be valid ASCII.
    Some(unsafe { String::from_utf8_unchecked(bytes) })
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

    #[test]
    fn test_decrement_basic() {
        assert_eq!(str_decrement("hellp"), Some("hello".to_string()));
        assert_eq!(str_decrement("PHQ"), Some("PHP".to_string()));
        assert_eq!(str_decrement("rusu"), Some("rust".to_string()));
        assert_eq!(str_decrement("abc124"), Some("abc123".to_string()));
    }

    #[test]
    fn test_decrement_with_carries() {
        assert_eq!(str_decrement("helma"), Some("hellz".to_string()));
        assert_eq!(str_decrement("TESTAA"), Some("TESSZZ".to_string()));
        assert_eq!(str_decrement("xza"), Some("xyz".to_string()));
        assert_eq!(str_decrement("aaa"), Some("zz".to_string()));
        assert_eq!(str_decrement("AAA"), Some("ZZ".to_string()));
    }

    #[test]
    fn test_decrement_with_numeric_carries() {
        assert_eq!(str_decrement("10"), Some("9".to_string()));
        assert_eq!(str_decrement("100"), Some("99".to_string()));
        assert_eq!(str_decrement("1000"), Some("999".to_string()));
        assert_eq!(str_decrement("abc10"), Some("abc09".to_string()));
        assert_eq!(str_decrement("abc100"), Some("abc099".to_string()));
    }

    #[test]
    fn test_decrement_mixed_alphanumeric() {
        assert_eq!(str_decrement("b0"), Some("a9".to_string()));
        assert_eq!(str_decrement("b00a"), Some("a99z".to_string()));
        assert_eq!(str_decrement("AA0"), Some("Z9".to_string()));
        assert_eq!(str_decrement("10a"), Some("9z".to_string()));
        assert_eq!(str_decrement("10A"), Some("9Z".to_string()));
    }

    #[test]
    fn test_decrement_at_boundaries() {
        assert_eq!(str_decrement("aa"), Some("z".to_string()));
        assert_eq!(str_decrement("a"), None);
        assert_eq!(str_decrement("AA"), Some("Z".to_string()));
        assert_eq!(str_decrement("A"), None);
    }

    #[test]
    fn test_decrement_leading_zeros() {
        assert_eq!(str_decrement("01"), None);
        assert_eq!(str_decrement("001"), None);
        assert_eq!(str_decrement("0abc"), None);
    }

    #[test]
    fn test_decrement_failure_cases() {
        assert_eq!(str_decrement(""), None);
        assert_eq!(str_decrement("0"), None);
        assert_eq!(str_decrement("hello!"), None);
        assert_eq!(str_decrement("test-123"), None);
        assert_eq!(str_decrement("user@example.com"), None);
        assert_eq!(str_decrement("русский"), None);
    }
}
