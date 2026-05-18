/// Checks if a byte sequence represents a numeric value according to PHP's definition.
///
/// This function checks if the input is numeric by trimming leading/trailing whitespace,
/// removing leading zeros, and checking if the remaining bytes can be parsed as a number.
///
/// # Arguments
///
/// * `input` - The byte slice to check.
///
/// # Returns
///
/// * `true` - If the input is numeric.
/// * `false` - If the input is not numeric.
pub fn str_is_numeric(input: &[u8]) -> bool {
    // Trim leading/trailing whitespace
    let mut maybe_numeric = trim_ascii(input);
    if maybe_numeric.is_empty() {
        return false;
    }

    if maybe_numeric[0] == b'+' || maybe_numeric[0] == b'-' {
        maybe_numeric = &maybe_numeric[1..];
        if maybe_numeric.is_empty() {
            return false;
        }
    }

    while !maybe_numeric.is_empty() && maybe_numeric[0] == b'0' {
        maybe_numeric = &maybe_numeric[1..];
    }

    if maybe_numeric.is_empty() {
        return true;
    }

    std::str::from_utf8(maybe_numeric).ok().and_then(|s| s.parse::<f64>().ok()).is_some()
}

fn trim_ascii(input: &[u8]) -> &[u8] {
    let mut start = 0;
    while start < input.len() && input[start].is_ascii_whitespace() {
        start += 1;
    }
    let mut end = input.len();
    while end > start && input[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    &input[start..end]
}
