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
    // Trim leading/trailing whitespace
    let mut maybe_numeric = input.trim();
    if maybe_numeric.is_empty() {
        return false;
    }

    // Check if the string starts with a sign (+ or -)
    if maybe_numeric.starts_with('+') || maybe_numeric.starts_with('-') {
        // Skip sign
        maybe_numeric = &maybe_numeric[1..];

        // Check if the remaining string is empty
        if maybe_numeric.is_empty() {
            return false;
        }
    }

    // Skip leading zeros
    maybe_numeric = maybe_numeric.trim_start_matches('0');
    if maybe_numeric.is_empty() {
        // Only zeros, so it's numeric
        return true;
    }

    // Check if the remaining string is numeric
    maybe_numeric.parse::<f64>().is_ok()
}
