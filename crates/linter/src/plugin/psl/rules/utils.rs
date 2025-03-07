#[inline]
pub fn format_replacements(replacements: &'static [&'static str]) -> String {
    let count = replacements.len();
    if count == 0 {
        return String::new();
    }

    if count == 1 {
        return format!("`{}`", replacements[0]);
    }

    let mut result = String::new();
    for (i, replacement) in replacements.iter().enumerate() {
        if i == count - 1 {
            result.push_str("or ");
        } else if i > 0 {
            result.push_str(", ");
        }

        result.push('`');
        result.push_str(replacement);
        result.push('`');
    }

    result
}
