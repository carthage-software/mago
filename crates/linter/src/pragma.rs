use mago_interner::ThreadedInterner;
use mago_source::Source;
use mago_span::Span;
use mago_syntax::ast::Program;

use crate::utils::comment_lines;

/// Represents the kind of linter pragma.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum PragmaKind {
    /// A pragma that instructs the linter to ignore a specific rule.
    Ignore,
    /// A pragma that instructs the linter to expect a specific rule to be violated.
    Expect,
}

/// Represents a single pragma extracted from a comment.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Pragma<'a> {
    /// The kind of the pragma.
    pub kind: PragmaKind,
    /// The source span of the comment containing the pragma.
    pub span: Span,
    /// The starting line number of the comment.
    pub start_line: usize,
    /// The ending line number of the comment.
    pub end_line: usize,
    /// Indicates whether the comment appears on its own line (i.e., only whitespace precedes it).
    pub own_line: bool,
    /// The rule specification.
    pub rule: &'a str,
    /// An optional description explaining why this pragma is present.
    pub description: &'a str,
}

impl PragmaKind {
    /// Returns `true` if the pragma kind is `Ignore`.
    #[inline]
    pub const fn is_ignore(self) -> bool {
        matches!(self, PragmaKind::Ignore)
    }

    /// Returns `true` if the pragma kind is `Expect`.
    #[inline]
    pub const fn is_expect(self) -> bool {
        matches!(self, PragmaKind::Expect)
    }
}

/// Extracts and returns all linter pragmas from the given program's AST.
///
/// This function looks at every trivia (non-code) element in the AST, filtering for comments.
/// For each comment, it:
///
/// 1. Parses its content into individual lines via `comment_lines()`.
/// 2. Extracts pragmas from lines that start with `@mago-ignore` or `@mago-expect`.
///    - If a pragma's content contains a `/`, it is split on the first whitespace into a
///      rule and an optional description.
///    - If no `/` is present, the entire content is discarded.
/// 3. Determines whether the comment is on its own line. A comment is considered "own_line" if
///    all characters from the beginning of its source line (in the full source code) up to the
///    comment's start offset are whitespace.
///
/// # Parameters
///
/// - `source`: The source to inspect.
/// - `program`: The program containing the AST to inspect for comments.
/// - `interner`: The interner used to resolve the source code string.
///
/// # Returns
///
/// A vector of `Pragma` structs, each containing a parsed pragma and its location information.
#[inline]
pub fn get_pragmas<'s, 'a>(
    source: &'s Source,
    program: &'s Program,
    interner: &'a ThreadedInterner,
) -> Vec<Pragma<'a>> {
    // Get the full source code from the interner.
    let source_code: &'a str = interner.lookup(&source.content);

    program
        .trivia
        .iter()
        .filter(|trivia| trivia.kind.is_comment())
        .filter_map(|trivia| {
            // Parse pragmas from the comment's text.
            let pragma = parse_pragmas(comment_lines(trivia, interner));
            if pragma.is_empty() {
                return None;
            }

            let start_line = source.line_number(trivia.span.start.offset);
            let end_line = source.line_number(trivia.span.end.offset);
            let line_start = source.get_line_start_offset(start_line).unwrap_or(0);
            let prefix = &source_code[line_start..trivia.span.start.offset];
            let own_line = prefix.trim().is_empty();

            Some(pragma.into_iter().map(move |(kind, rule, description)| Pragma {
                kind,
                span: trivia.span,
                start_line,
                end_line,
                own_line,
                rule,
                description,
            }))
        })
        .flatten()
        .collect()
}

/// Parses a set of comment lines for linter pragmas (ignore and expect).
///
/// For each line in the provided vector:
/// - If the line starts with `@mago-ignore` or `@mago-expect`, the remainder of the line is trimmed and processed.
/// - If the content contains a `/`, it is split on the first whitespace into a rule and an optional description.
/// - If no `/` is present in the content, the entire content is discarded.
///
/// Lines that do not start with `@mago-ignore` or `@mago-expect` are ignored.
///
/// # Parameters
///
/// - `comment_lines`: A vector of string slices, each representing a line from a comment.
///
/// # Returns
///
/// A vector of `(PragmaKind, rule, description)` tuples, where `kind` is the pragma type,
/// `rule` is the rule name, and `description` is an optional description of why the pragma is present.
#[inline]
pub fn parse_pragmas(comment_lines: Vec<&str>) -> Vec<(PragmaKind, &str, &str)> {
    let mut pragmas = Vec::new();

    for line in comment_lines {
        let line = line.trim();
        let (kind, prefix_length) = if line.starts_with("@mago-ignore") {
            (PragmaKind::Ignore, "@mago-ignore".len())
        } else if line.starts_with("@mago-expect") {
            (PragmaKind::Expect, "@mago-expect".len())
        } else {
            continue;
        };

        let content = line[prefix_length..].trim();
        if content.is_empty() {
            continue;
        }

        if !content.contains('/') {
            continue;
        }

        let mut parts = content.splitn(2, char::is_whitespace);
        let rule = parts.next().unwrap();
        let description = parts.next().unwrap_or("").trim();

        pragmas.push((kind, rule, description));
    }

    pragmas
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_ignore() {
        let comment = "@mago-ignore security/no-literal-password - false positive";
        let result = parse_pragmas(comment.lines().collect());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, PragmaKind::Ignore);
        assert_eq!(result[0].1, "security/no-literal-password");
        assert_eq!(result[0].2, "- false positive");
    }

    #[test]
    fn test_parse_ignore_without_description() {
        let comment = "@mago-ignore laravel/no-request-all";
        let result = parse_pragmas(comment.lines().collect());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, PragmaKind::Ignore);
        assert_eq!(result[0].1, "laravel/no-request-all");
        assert_eq!(result[0].2, "");
    }

    #[test]
    fn test_parse_multiple_ignores() {
        let comment = r#"
            @mago-ignore security/no-literal-password we enjoy leaking passwords in code
            @mago-ignore safetly/no-eval
            @mago-ignore laravel/no-request-all Laravel specific ignore
        "#;
        let result = parse_pragmas(comment.lines().collect());
        assert_eq!(result.len(), 3);

        assert_eq!(result[0].0, PragmaKind::Ignore);
        assert_eq!(result[0].1, "security/no-literal-password");
        assert_eq!(result[0].2, "we enjoy leaking passwords in code");

        assert_eq!(result[1].0, PragmaKind::Ignore);
        assert_eq!(result[1].1, "safetly/no-eval");
        assert_eq!(result[1].2, "");

        assert_eq!(result[2].0, PragmaKind::Ignore);
        assert_eq!(result[2].1, "laravel/no-request-all");
        assert_eq!(result[2].2, "Laravel specific ignore");
    }

    #[test]
    fn test_ignore_invalid_lines() {
        let comment = r#"
            Some random text here
            @mago-ignore invalidformat
            @mago-ignore missing_slash no description
        "#;
        let result = parse_pragmas(comment.lines().collect());
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_parse_single_expect() {
        let comment = "@mago-expect security/no-literal-password - expected failure";
        let result = parse_pragmas(comment.lines().collect());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, PragmaKind::Expect);
        assert_eq!(result[0].1, "security/no-literal-password");
        assert_eq!(result[0].2, "- expected failure");
    }

    #[test]
    fn test_parse_multiple_mixed() {
        let comment = r#"
            @mago-ignore security/no-literal-password we enjoy leaking passwords in code
            @mago-expect safetly/no-eval expected failure
            @mago-ignore laravel/no-request-all Laravel specific ignore
        "#;
        let result = parse_pragmas(comment.lines().collect());
        assert_eq!(result.len(), 3);

        assert_eq!(result[0].0, PragmaKind::Ignore);
        assert_eq!(result[0].1, "security/no-literal-password");
        assert_eq!(result[0].2, "we enjoy leaking passwords in code");

        assert_eq!(result[1].0, PragmaKind::Expect);
        assert_eq!(result[1].1, "safetly/no-eval");
        assert_eq!(result[1].2, "expected failure");

        assert_eq!(result[2].0, PragmaKind::Ignore);
        assert_eq!(result[2].1, "laravel/no-request-all");
        assert_eq!(result[2].2, "Laravel specific ignore");
    }
}
