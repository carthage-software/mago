use std::borrow::Cow;
use std::cmp::Ordering;

use crate::Issue;
use crate::IssueCollection;
use crate::Level;
use crate::formatter::FormatterConfig;

pub struct LazyFilteredIssues<'issues> {
    iter: std::slice::Iter<'issues, Issue>,
    min_level: Option<Level>,
    filter_fixable: bool,
}

pub enum FilteredIssues<'issues> {
    Lazy(LazyFilteredIssues<'issues>),
    Sorted(std::vec::IntoIter<&'issues Issue>),
}

impl<'issues> Iterator for LazyFilteredIssues<'issues> {
    type Item = &'issues Issue;

    #[inline]
    fn next(&mut self) -> Option<&'issues Issue> {
        for issue in self.iter.by_ref() {
            if let Some(min) = self.min_level
                && issue.level < min
            {
                continue;
            }

            if self.filter_fixable && issue.edits.is_empty() {
                continue;
            }

            return Some(issue);
        }

        None
    }
}

impl<'issues> Iterator for FilteredIssues<'issues> {
    type Item = &'issues Issue;

    #[inline]
    fn next(&mut self) -> Option<&'issues Issue> {
        match self {
            Self::Lazy(it) => it.next(),
            Self::Sorted(it) => it.next(),
        }
    }
}

/// Returns a borrowing iterator over the issues that pass the formatter's
/// minimum-level and fixable-only filters.
///
/// The `sortable` argument tells this helper whether sorting is *meaningful*
/// for the calling formatter:
///
/// * Pass `true` for human-readable formats (rich, ariadne, json, …) where
///   `--sort` should produce a stable, severity-ordered view of issues.
/// * Pass `false` for formats that already aggregate (`count`, `code-count`)
///   or whose consumers do their own ordering (`github`, `gitlab`, `sarif`,
///   `checkstyle`, `emacs`). For these, even if the user passed `--sort`,
///   sorting is wasted work.
#[inline]
pub fn filter_issues<'issues>(
    issues: &'issues IssueCollection,
    config: &FormatterConfig,
    sortable: bool,
) -> FilteredIssues<'issues> {
    let min_level = config.minimum_level;
    let filter_fixable = config.filter_fixable;

    let lazy = LazyFilteredIssues { iter: issues.issues.iter(), min_level, filter_fixable };

    if sortable && config.sort {
        let mut refs: Vec<&Issue> = lazy.collect();
        refs.sort_by(compare_issues);
        FilteredIssues::Sorted(refs.into_iter())
    } else {
        FilteredIssues::Lazy(lazy)
    }
}

pub(crate) fn utf8_preserving_byte_offsets(bytes: &[u8]) -> Cow<'_, str> {
    if let Ok(source) = std::str::from_utf8(bytes) {
        return Cow::Borrowed(source);
    }

    let mut source = String::with_capacity(bytes.len());
    let mut offset = 0;

    while offset < bytes.len() {
        match std::str::from_utf8(&bytes[offset..]) {
            Ok(suffix) => {
                source.push_str(suffix);
                break;
            }
            Err(error) => {
                let valid_end = offset + error.valid_up_to();
                source.push_str(String::from_utf8_lossy(&bytes[offset..valid_end]).as_ref());

                let invalid_length = error.error_len().unwrap_or(bytes.len() - valid_end);
                for _ in 0..invalid_length {
                    source.push('?');
                }

                offset = valid_end + invalid_length;
            }
        }
    }

    Cow::Owned(source)
}

#[inline]
fn compare_issues(a: &&Issue, b: &&Issue) -> Ordering {
    match a.level.cmp(&b.level) {
        Ordering::Less => Ordering::Less,
        Ordering::Greater => Ordering::Greater,
        Ordering::Equal => match a.code.as_deref().cmp(&b.code.as_deref()) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => match (a.primary_span(), b.primary_span()) {
                (Some(a_span), Some(b_span)) => a_span.cmp(&b_span),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => Ordering::Equal,
            },
        },
    }
}

/// XML-encode a string by escaping special characters.
pub fn xml_encode(input: impl AsRef<str>) -> String {
    let input = input.as_ref();
    // the result will never be smaller than the input,
    // so we can preallocate the result with the same capacity.
    let mut result = String::with_capacity(input.len());

    for c in input.chars() {
        if !is_xml_1_0_character(c) {
            continue;
        }

        let next = match c {
            '&' => "&amp;",
            '<' => "&lt;",
            '>' => "&gt;",
            '"' => "&quot;",
            '\'' => "&apos;",
            '\n' => "&#10;",
            '\r' => "&#13;",
            _ => {
                result.push(c);

                continue;
            }
        };

        result.push_str(next);
    }

    result
}

#[inline]
fn is_xml_1_0_character(character: char) -> bool {
    matches!(
        character,
        '\u{9}' | '\u{A}' | '\u{D}' | '\u{20}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}' | '\u{10000}'..='\u{10FFFF}'
    )
}

/// Build a long message from an issue including notes, help, and links.
pub fn long_message(issue: &Issue, include_annotations: bool) -> String {
    let mut message = issue.message.clone();

    if include_annotations {
        for annotation in &issue.annotations {
            if let Some(annotation_msg) = annotation.message.as_ref() {
                message.push('\n');
                message.push('>');
                message.push_str(annotation_msg.as_str());
            }
        }
    }

    if !issue.notes.is_empty() {
        message.push('\n');

        for note in &issue.notes {
            message.push('\n');
            message.push_str(note.as_str());
        }
    }

    if let Some(help) = issue.help.as_ref() {
        message.push_str("\n\nHelp: ");
        message.push_str(help.as_str());
    }

    if let Some(link) = issue.link.as_ref() {
        message.push_str("\n\nMore information: ");
        message.push_str(link.as_str());
    }

    message
}

/// Build an OSC 8 hyperlink wrapping `display_text`.
///
/// The URL is constructed by replacing `%file%`, `%line%`, and `%column%` placeholders
/// in `template` with the provided values.
#[must_use]
pub fn osc8_hyperlink(template: &str, abs_path: &str, line: u32, column: u32, display_text: &str) -> String {
    osc8_file_hyperlink(template, abs_path, abs_path, line, column, display_text)
}

/// Build an OSC 8 hyperlink wrapping `display_text` with absolute and relative file paths.
///
/// The URL is constructed by replacing `%file%`, `%rel_file%`, `%line%`, and `%column%`
/// placeholders in `template` with the provided values.
#[must_use]
pub fn osc8_file_hyperlink(
    template: &str,
    abs_path: &str,
    relative_path: &str,
    line: u32,
    column: u32,
    display_text: &str,
) -> String {
    let absolute_path = strip_windows_verbatim_prefix(abs_path);
    let line = line.to_string();
    let column = column.to_string();
    let replacements = [
        ("%file%", absolute_path.as_ref()),
        ("%rel_file%", relative_path),
        ("%line%", line.as_str()),
        ("%column%", column.as_str()),
    ];
    let mut url = String::with_capacity(template.len());
    let mut remaining = template;

    while let Some(position) = remaining.find('%') {
        url.push_str(&remaining[..position]);
        remaining = &remaining[position..];

        let mut replaced = false;
        for &(placeholder, value) in &replacements {
            if let Some(rest) = remaining.strip_prefix(placeholder) {
                url.push_str(value);
                remaining = rest;
                replaced = true;
                break;
            }
        }

        if !replaced {
            url.push('%');
            remaining = &remaining[1..];
        }
    }

    url.push_str(remaining);

    format!("\x1b]8;;{url}\x1b\\{display_text}\x1b]8;;\x1b\\")
}

/// Strips the Win32 verbatim (`\\?\`) prefix that `std::fs::canonicalize` adds to absolute
/// paths on Windows. The prefix is required by some low-level Win32 APIs but isn't accepted
/// by editors, shells, or `file://` URL handlers, so paths surfaced to users (OSC 8 hyperlinks,
/// editor-url templates) must have it removed.
///
/// * `\\?\C:\dir\file` -> `C:\dir\file`
/// * `\\?\UNC\server\share` -> `\\server\share`
/// * any other path is returned unchanged.
fn strip_windows_verbatim_prefix(path: &str) -> std::borrow::Cow<'_, str> {
    if let Some(rest) = path.strip_prefix(r"\\?\UNC\") {
        std::borrow::Cow::Owned(format!(r"\\{rest}"))
    } else if let Some(rest) = path.strip_prefix(r"\\?\") {
        std::borrow::Cow::Borrowed(rest)
    } else {
        std::borrow::Cow::Borrowed(path)
    }
}

#[cfg(test)]
mod tests {
    use super::osc8_file_hyperlink;
    use super::strip_windows_verbatim_prefix;
    use super::utf8_preserving_byte_offsets;
    use super::xml_encode;

    #[test]
    fn preserves_byte_offsets_when_replacing_invalid_utf8() {
        let bytes = b"valid \xc2\xa9 invalid \xa9 truncated \xf0\x9f\nnext";
        let source = utf8_preserving_byte_offsets(bytes);

        assert_eq!(source.len(), bytes.len());
        assert_eq!(source.as_bytes(), b"valid \xc2\xa9 invalid ? truncated ??\nnext");
    }

    #[test]
    fn xml_encoding_escapes_markup_and_line_boundaries() {
        assert_eq!(xml_encode("<&>\"'\r\n"), "&lt;&amp;&gt;&quot;&apos;&#13;&#10;");
    }

    #[test]
    fn xml_encoding_removes_characters_forbidden_by_xml_1_0() {
        assert_eq!(xml_encode("a\0\u{1}\u{B}\u{C}\u{1F}\u{FFFE}\u{FFFF}b\t"), "ab\t");
    }

    #[test]
    fn strips_verbatim_drive_prefix() {
        assert_eq!(strip_windows_verbatim_prefix(r"\\?\C:\Users\foo\bar.php"), r"C:\Users\foo\bar.php");
    }

    #[test]
    fn strips_verbatim_unc_prefix() {
        assert_eq!(strip_windows_verbatim_prefix(r"\\?\UNC\server\share\file.php"), r"\\server\share\file.php");
    }

    #[test]
    fn leaves_plain_windows_paths_unchanged() {
        assert_eq!(strip_windows_verbatim_prefix(r"C:\Users\foo\bar.php"), r"C:\Users\foo\bar.php");
    }

    #[test]
    fn leaves_unix_paths_unchanged() {
        assert_eq!(strip_windows_verbatim_prefix("/home/foo/bar.php"), "/home/foo/bar.php");
    }

    #[test]
    fn leaves_unc_without_verbatim_unchanged() {
        assert_eq!(strip_windows_verbatim_prefix(r"\\server\share\file.php"), r"\\server\share\file.php");
    }

    #[test]
    fn editor_url_replaces_absolute_and_relative_file_placeholders() {
        assert_eq!(
            osc8_file_hyperlink(
                "editor://%file%?relative=%rel_file%&line=%line%&column=%column%",
                "/workspace/src/Foo.php",
                "src/Foo.php",
                12,
                34,
                "src/Foo.php",
            ),
            "\x1b]8;;editor:///workspace/src/Foo.php?relative=src/Foo.php&line=12&column=34\x1b\\src/Foo.php\x1b]8;;\x1b\\",
        );
    }

    #[test]
    fn editor_url_does_not_expand_placeholders_inside_file_paths() {
        assert_eq!(
            osc8_file_hyperlink(
                "editor://%file%?relative=%rel_file%",
                "/workspace/%line%/Foo.php",
                "src/%column%/Foo.php",
                12,
                34,
                "src/Foo.php",
            ),
            "\x1b]8;;editor:///workspace/%line%/Foo.php?relative=src/%column%/Foo.php\x1b\\src/Foo.php\x1b]8;;\x1b\\",
        );
    }
}
