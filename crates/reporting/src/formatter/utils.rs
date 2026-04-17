use std::cmp::Ordering;

use crate::Issue;
use crate::IssueCollection;
use crate::Level;
use crate::formatter::FormatterConfig;

pub struct LazyFilteredIssues<'a> {
    iter: std::slice::Iter<'a, Issue>,
    min_level: Option<Level>,
    filter_fixable: bool,
}

pub enum FilteredIssues<'a> {
    Lazy(LazyFilteredIssues<'a>),
    Sorted(std::vec::IntoIter<&'a Issue>),
}

impl<'a> Iterator for LazyFilteredIssues<'a> {
    type Item = &'a Issue;

    #[inline]
    fn next(&mut self) -> Option<&'a Issue> {
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

impl<'a> Iterator for FilteredIssues<'a> {
    type Item = &'a Issue;

    #[inline]
    fn next(&mut self) -> Option<&'a Issue> {
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
pub fn filter_issues<'a>(issues: &'a IssueCollection, config: &FormatterConfig, sortable: bool) -> FilteredIssues<'a> {
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
pub fn osc8_hyperlink(template: &str, abs_path: &str, line: u32, column: u32, display_text: &str) -> String {
    let url = template
        .replace("%file%", &strip_windows_verbatim_prefix(abs_path))
        .replace("%line%", &line.to_string())
        .replace("%column%", &column.to_string());

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
    use super::strip_windows_verbatim_prefix;

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
}
