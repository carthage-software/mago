use bumpalo::Bump;
use bumpalo::collections::CollectIn;
use bumpalo::collections::Vec;

use mago_database::file::File;
use mago_span::Span;
use mago_syntax::ast::Trivia;
use mago_syntax::comments::comment_lines;

/// Represents the kind of collector pragma.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum PragmaKind {
    /// A pragma that instructs the collector to ignore a specific code.
    Ignore,
    /// A pragma that instructs the collector to expect a specific code to be violated.
    Expect,
}

/// Represents a single pragma extracted from a comment.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Pragma<'a> {
    /// The kind of the pragma.
    pub kind: PragmaKind,
    /// The source span of the pragma.
    pub span: Span,
    /// The span of the trivia (comment) that contains the pragma.
    pub trivia_span: Span,
    /// The scope span where the pragma applies, if applicable.
    pub scope_span: Option<Span>,
    /// The starting line number of the comment.
    pub start_line: u32,
    /// The ending line number of the comment.
    pub end_line: u32,
    /// Indicates whether the comment appears on its own line (i.e., only whitespace precedes it).
    pub own_line: bool,
    /// The category of the pragma, e.g., "lint" or "analysis".
    pub category: &'a str,
    /// The code specification.
    pub code: &'a str,
    /// The span of the code within the pragma.
    pub code_span: Span,
    /// An optional description explaining why this pragma is present.
    pub description: &'a str,
    /// Indicates whether the pragma is used to suppress an issue.
    pub used: bool,
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

    /// Returns the string representation of the pragma kind.
    pub const fn as_str(self) -> &'static str {
        match self {
            PragmaKind::Ignore => "ignore",
            PragmaKind::Expect => "expect",
        }
    }
}

impl<'arena> Pragma<'arena> {
    /// Extracts and returns all pragmas from a slice of trivia that match the specified category.
    ///
    /// This function scans all comments in the trivia, calculates the precise span for each
    /// pragma found, and filters them based on the provided category.
    ///
    /// # Parameters
    ///
    /// - `file`: The source file being analyzed.
    /// - `trivias`: The slice of trivia (comments and whitespace) to scan.
    /// - `category`: The category of pragmas to collect (e.g., "lint", "analysis"), or
    ///   `None` to collect all pragmas regardless of category.
    ///
    /// # Returns
    ///
    /// A vector of `Pragma` structs, each containing a parsed pragma and its precise location.
    pub fn extract(
        arena: &'arena Bump,
        file: &File,
        trivias: &[Trivia<'arena>],
        category: Option<&'static str>,
    ) -> Vec<'arena, Pragma<'arena>> {
        trivias
            .iter()
            .filter(|trivia| trivia.kind.is_comment())
            .flat_map(|trivia| parse_pragmas_in_trivia(arena, file, trivia, category))
            .collect_in(arena)
    }
}

/// Parses all pragmas within a single trivia (comment) node.
fn parse_pragmas_in_trivia<'arena>(
    arena: &'arena Bump,
    file: &File,
    trivia: &Trivia<'arena>,
    category_filter: Option<&'static str>,
) -> Vec<'arena, Pragma<'arena>> {
    let mut pragmas: Vec<'arena, Pragma<'arena>> = Vec::new_in(arena);
    let base_offset = trivia.span.start;

    for (line_offset_in_trivia, line) in comment_lines(trivia) {
        let absolute_line_start = base_offset + line_offset_in_trivia;
        let trimmed = line.trim_start();
        let leading_whitespace = line.len() - trimmed.len();
        let pragma_start_offset = absolute_line_start + leading_whitespace as u32;

        let (kind, prefix) = if trimmed.starts_with("@mago-ignore") {
            (PragmaKind::Ignore, "@mago-ignore")
        } else if trimmed.starts_with("@mago-expect") {
            (PragmaKind::Expect, "@mago-expect")
        } else {
            continue;
        };

        let content_with_leading_space = &trimmed[prefix.len()..];
        let content = content_with_leading_space.trim_start();

        let Some((category, rest)) = content.split_once(':') else {
            continue; // Malformed pragma, no category or code.
        };

        if category.contains(char::is_whitespace) {
            continue; // Invalid category format.
        }

        if category_filter.is_some_and(|category_filter| category_filter != category) {
            continue; // Skip if category does not match the filter.
        }

        let rest = rest.trim_start();

        let mut parts = rest.splitn(2, char::is_whitespace);
        let Some(code) = parts.next() else {
            continue; // Malformed pragma, no code.
        };

        let description = parts.next().unwrap_or("").trim();

        // Calculate the precise span for the code part of the pragma.
        let code_start_offset = absolute_line_start + (code.as_ptr() as u32) - (line.as_ptr() as u32);
        let code_span = Span::new(file.id, code_start_offset, code_start_offset + code.len() as u32);

        let pragma_end_offset = pragma_start_offset + prefix.len() as u32 + content_with_leading_space.len() as u32;
        let span = Span::new(file.id, pragma_start_offset, pragma_end_offset);

        let start_line = file.line_number(trivia.span.start.offset);
        let end_line = file.line_number(trivia.span.end.offset);
        let line_start_offset = file.get_line_start_offset(start_line).unwrap_or(0);
        let prefix_text = &file.contents[line_start_offset as usize..trivia.span.start.offset as usize];
        let own_line = start_line == end_line && prefix_text.trim().is_empty();

        pragmas.push(Pragma {
            kind,
            span,
            trivia_span: trivia.span,
            code_span,
            start_line,
            end_line,
            own_line,
            category,
            code,
            description,
            scope_span: None,
            used: false,
        });
    }

    pragmas
}
