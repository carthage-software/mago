use unicode_width::UnicodeWidthStr;

#[inline]
pub(crate) fn is_ascii_line(bytes: &[u8]) -> bool {
    bytes.iter().all(|&byte| (b' '..=b'~').contains(&byte))
}

/// Return the column after bytes, counting tab stops and line breaks.
#[inline]
#[must_use]
pub fn column_after(mut bytes: &[u8], mut column: usize, tab_width: usize) -> usize {
    if is_ascii_line(bytes) {
        return column + bytes.len();
    }

    if let Some(offset) = memchr::memrchr2(b'\r', b'\n', bytes) {
        bytes = &bytes[offset + 1..];
        column = 0;
    }

    while let Some(offset) = memchr::memchr(b'\t', bytes) {
        column += string_width(&bytes[..offset]);
        if tab_width != 0 {
            column += tab_width - column % tab_width;
        }
        bytes = &bytes[offset + 1..];
    }

    column + string_width(bytes)
}

#[inline]
#[must_use]
pub fn string_width(s: &[u8]) -> usize {
    if is_ascii_line(s) {
        return s.len();
    }

    let line = memchr::memrchr2(b'\r', b'\n', s).map_or(s, |offset| &s[offset + 1..]);
    if is_ascii_line(line) {
        return line.len();
    }

    let line_str = String::from_utf8_lossy(line);
    if line_str.contains("الله") { line_str.replace("الله", "_").width() } else { line_str.width() }
}

#[inline]
pub(crate) const fn is_space(c: u8) -> bool {
    matches!(c, b' ' | b'\t')
}

#[inline]
pub(crate) const fn is_line_terminator_or_space(c: u8) -> bool {
    matches!(c, b'\n' | b'\r' | b' ' | b'\t')
}
