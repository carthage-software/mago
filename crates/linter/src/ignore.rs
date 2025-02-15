use mago_ast::Program;
use mago_interner::ThreadedInterner;
use mago_project::module::Module;
use mago_span::Span;

use crate::utils::comment_lines;

/// Represents a single ignore directive extracted from a comment.
#[derive(Debug, PartialEq, Eq)]
pub struct IgnoreDirective<'a> {
    /// The source span of the comment containing the ignore directive.
    pub span: Span,
    /// The starting line number of the comment.
    pub start_line: usize,
    /// The ending line number of the comment.
    pub end_line: usize,
    /// Indicates whether the comment appears on its own line (i.e. only whitespace precedes it).
    pub own_line: bool,
    /// The rule specification.
    pub rule: &'a str,
    /// An optional description explaining why this ignore is present.
    pub description: &'a str,
}

/// Extracts and returns all ignore comments from the given module.
///
/// This function looks at every trivia (non-code) element in the AST, filtering for comments.
/// For each comment, it:
///
/// 1. Parses its content into individual lines via `comment_lines()`.
/// 2. Extracts ignore directives from lines that start with `@mago-ignore`.
///    - If a directive’s content contains a `/`, it is split on the first whitespace into a
///      rule and an optional description.
///    - If no `/` is present, the entire content is used as the rule, with an empty description.
/// 3. Determines whether the comment is on its own line. A comment is considered "own_line" if
///    all characters from the beginning of its source line (in the full source code) up to the
///    comment’s start offset are whitespace.
///
/// # Parameters
///
/// - `module`: The module to inspect (which contains the AST trivia and source content).
/// - `interner`: The interner used to resolve the source code string.
///
/// # Returns
///
/// A vector of `Ignore` comments, each containing its parsed directives and location information.
#[inline]
pub fn get_ignores<'s, 'a>(
    module: &'s Module,
    program: &'s Program,
    interner: &'a ThreadedInterner,
) -> Vec<IgnoreDirective<'a>> {
    // Get the full source code from the interner.
    let source_code: &'a str = interner.lookup(&module.source.content);

    program
        .trivia
        .iter()
        .filter(|trivia| trivia.kind.is_comment())
        .filter_map(|trivia| {
            // Parse ignore directives from the comment's text.
            let directives = parse_ignore_directives(comment_lines(trivia, interner));
            if directives.is_empty() {
                return None;
            }

            let start_line = module.source.line_number(trivia.span.start.offset);
            let end_line = module.source.line_number(trivia.span.end.offset);
            let line_start = module.source.get_line_start_offset(start_line).unwrap_or(0);
            let prefix = &source_code[line_start..trivia.span.start.offset];
            let own_line = prefix.trim().is_empty();

            Some(directives.into_iter().map(move |(rule, description)| IgnoreDirective {
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

/// Parses a set of comment lines for ignore directives.
///
/// For each line in the provided vector:
/// - If the line starts with `@mago-ignore`, the remainder of the line is trimmed and processed.
/// - If the content contains a `/`, it is split on the first whitespace into a rule and an optional description.
/// - If no `/` is present in the content, the entire content is used as the rule and the description is empty.
///
/// Lines that do not start with `@mago-ignore` are ignored.
///
/// # Parameters
///
/// - `comment_lines`: A vector of string slices, each representing a line from a comment.
///
/// # Returns
///
/// A vector of `(rule, description)` tuples, where `rule` is the rule name and `description` is an optional
/// description of why the ignore is present.
#[inline]
pub fn parse_ignore_directives(comment_lines: Vec<&str>) -> Vec<(&str, &str)> {
    let mut ignores = Vec::new();

    for line in comment_lines {
        let line = line.trim();
        if !line.starts_with("@mago-ignore") {
            continue;
        }

        // Extract the content following the keyword.
        let content = line["@mago-ignore".len()..].trim();
        if content.is_empty() {
            continue; // Nothing to parse after the keyword.
        }

        if !content.contains('/') {
            continue;
        }

        let mut parts = content.splitn(2, char::is_whitespace);
        let rule = parts.next().unwrap(); // Guaranteed non-empty.
        let description = parts.next().unwrap_or("").trim();

        ignores.push((rule, description));
    }

    ignores
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_ignore() {
        let comment = "@mago-ignore security/no-literal-password - false positive";
        let result = parse_ignore_directives(comment.lines().collect());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "security/no-literal-password");
        assert_eq!(result[0].1, "- false positive");
    }

    #[test]
    fn test_parse_ignore_without_description() {
        let comment = "@mago-ignore laravel/no-request-all";
        let result = parse_ignore_directives(comment.lines().collect());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "laravel/no-request-all");
        assert_eq!(result[0].1, "");
    }

    #[test]
    fn test_parse_multiple_ignores() {
        let comment = r#"
            @mago-ignore security/no-literal-password we enjoy leaking passwords in code
            @mago-ignore safetly/no-eval
            @mago-ignore laravel/no-request-all Laravel specific ignore
        "#;
        let result = parse_ignore_directives(comment.lines().collect());
        assert_eq!(result.len(), 3);

        // First directive.
        assert_eq!(result[0].0, "security/no-literal-password");
        assert_eq!(result[0].1, "we enjoy leaking passwords in code");

        // Second directive.
        assert_eq!(result[1].0, "safetly/no-eval");
        assert_eq!(result[1].1, "");

        // Third directive.
        assert_eq!(result[2].0, "laravel/no-request-all");
        assert_eq!(result[2].1, "Laravel specific ignore");
    }

    #[test]
    fn test_ignore_invalid_lines() {
        // These lines do not contain a '/' so they should still be kept as a single string.
        let comment = r#"
            Some random text here
            @mago-ignore invalidformat
            @mago-ignore missing_slash no description
        "#;
        let result = parse_ignore_directives(comment.lines().collect());
        assert_eq!(result.len(), 0);
    }
}
