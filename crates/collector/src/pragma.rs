use mago_allocator::prelude::*;

use mago_database::file::File;
use mago_span::Span;
use mago_syntax::ast::Trivia;
use mago_syntax::comments::comment_lines;

/// Allocate `bytes` as a UTF-8 `&str` in `arena` if valid; otherwise `None`.
///
/// Pragma codes, categories, and descriptions are tool-config strings that are ASCII in practice.
/// A non-UTF-8 byte means the pragma is malformed and can't match any rule name anyway, so
/// callers skip the whole pragma when this returns `None`.
fn alloc_utf8<'arena, A>(arena: &'arena A, bytes: &[u8]) -> Option<&'arena str>
where
    A: Arena,
{
    std::str::from_utf8(bytes).ok().map(|s| arena.alloc_str(s) as &str)
}

#[inline]
fn split_once_byte(s: &[u8], byte: u8) -> Option<(&[u8], &[u8])> {
    memchr::memchr(byte, s).map(|i| (&s[..i], &s[i + 1..]))
}

#[inline]
fn splitn_whitespace_2(s: &[u8]) -> (&[u8], &[u8]) {
    match s.iter().position(|b| b.is_ascii_whitespace()) {
        Some(i) => (&s[..i], &s[i + 1..]),
        None => (s, &[]),
    }
}

#[inline]
fn contains_ascii_whitespace(s: &[u8]) -> bool {
    s.iter().any(|b| b.is_ascii_whitespace())
}

/// Represents the kind of collector pragma.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u8)]
#[allow(clippy::exhaustive_enums)]
pub enum PragmaKind {
    /// A pragma that instructs the collector to ignore a specific code.
    Ignore,
    /// A pragma that instructs the collector to expect a specific code to be violated.
    Expect,
}

/// Represents a single pragma extracted from a comment.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Pragma<'arena> {
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
    pub category: &'arena str,
    /// The code specification.
    pub code: &'arena str,
    /// The span of the code (including any `(N)` count suffix) within the pragma.
    pub code_span: Span,
    /// The span of the parenthesized count suffix, if present (e.g., `(3)`).
    pub count_span: Option<Span>,
    /// The number of issues this pragma is expected to suppress.
    pub expected_matches: u16,
    /// The number of issues this pragma has actually matched.
    pub matches: u16,
    /// An optional description explaining why this pragma is present.
    pub description: &'arena str,
}

impl Pragma<'_> {
    /// Returns `true` if this pragma has matched at least as many issues as it expected.
    #[inline]
    #[must_use]
    pub fn is_fulfilled(&self) -> bool {
        self.matches >= self.expected_matches
    }

    /// Returns `true` if this pragma has absorbed its maximum allowed issues and should no
    /// longer match new ones.
    ///
    /// Line-level pragmas and scoped pragmas with an explicit `(N)` count are capped at
    /// `expected_matches`. Scoped pragmas without an explicit count retain the original
    /// "unlimited within scope" semantics so pre-existing code keeps working.
    #[inline]
    #[must_use]
    pub fn is_consumed(&self) -> bool {
        if self.scope_span.is_some() && self.count_span.is_none() {
            return false;
        }

        self.matches >= self.expected_matches
    }
}

impl PragmaKind {
    /// Returns `true` if the pragma kind is `Ignore`.
    #[inline]
    #[must_use]
    pub const fn is_ignore(self) -> bool {
        matches!(self, PragmaKind::Ignore)
    }

    /// Returns `true` if the pragma kind is `Expect`.
    #[inline]
    #[must_use]
    pub const fn is_expect(self) -> bool {
        matches!(self, PragmaKind::Expect)
    }

    /// Returns the string representation of the pragma kind.
    #[inline]
    #[must_use]
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
    /// - `categories`: An iterator of category strings to filter pragmas by.
    ///
    /// # Returns
    ///
    /// A vector of `Pragma` structs, each containing a parsed pragma and its precise location.
    #[inline]
    pub fn extract<A>(
        arena: &'arena A,
        file: &File,
        trivias: &[Trivia<'arena>],
        categories: &'static [&'static str],
    ) -> Vec<'arena, Pragma<'arena>, A>
    where
        A: Arena,
    {
        trivias
            .iter()
            .filter(|trivia| trivia.kind.is_comment())
            .flat_map(|trivia| parse_pragmas_in_trivia(arena, file, trivia, categories))
            .collect_in(arena)
    }
}

/// Parses all pragmas within a single trivia (comment) node.
fn parse_pragmas_in_trivia<'arena, A>(
    arena: &'arena A,
    file: &File,
    trivia: &Trivia<'arena>,
    categories: &'static [&'static str],
) -> Vec<'arena, Pragma<'arena>, A>
where
    A: Arena,
{
    let mut pragmas: Vec<'arena, Pragma<'arena>, A> = Vec::new_in(arena);
    let base_offset = trivia.span.start;

    for (line_offset_in_trivia, line) in comment_lines(trivia) {
        let absolute_line_start = base_offset + line_offset_in_trivia;
        let trimmed = line.trim_ascii_start();
        let leading_whitespace = line.len() - trimmed.len();
        let pragma_start_offset = absolute_line_start + leading_whitespace as u32;

        let (kind, prefix) = if trimmed.starts_with(b"@mago-ignore") {
            (PragmaKind::Ignore, b"@mago-ignore".as_slice())
        } else if trimmed.starts_with(b"@mago-expect") {
            (PragmaKind::Expect, b"@mago-expect".as_slice())
        } else {
            continue;
        };

        let content_with_leading_space = &trimmed[prefix.len()..];
        let content = content_with_leading_space.trim_ascii_start();

        let Some((category_bytes, rest)) = split_once_byte(content, b':') else {
            // Handle `@mago-ignore all` / `@mago-expect all` without a category prefix.
            let (code_part, rest) = splitn_whitespace_2(content);
            if code_part != b"all" {
                continue;
            }

            let description_bytes = rest.trim_ascii();
            let Some(description) = alloc_utf8(arena, description_bytes) else {
                continue;
            };

            let Some(&default_category) = categories.first() else {
                continue;
            };

            let code = "all";
            let code_offset_in_line = content.as_ptr() as usize - line.as_ptr() as usize;
            let code_start_offset = absolute_line_start + code_offset_in_line as u32;
            let code_span = Span::new(file.id, code_start_offset, code_start_offset + code.len() as u32);

            let pragma_end_offset = pragma_start_offset + prefix.len() as u32 + content_with_leading_space.len() as u32;
            let span = Span::new(file.id, pragma_start_offset, pragma_end_offset);

            let start_line = file.line_number(trivia.span.start.offset);
            let end_line = file.line_number(trivia.span.end.offset);
            let line_start_offset = file.get_line_start_offset(start_line).unwrap_or(0);
            let line_end_offset = file.get_line_end_offset(end_line).unwrap_or(file.contents.len() as u32);
            let prefix_text = &file.contents[line_start_offset as usize..trivia.span.start.offset as usize];
            let postfix_text = &file.contents[trivia.span.end.offset as usize..line_end_offset as usize];
            let own_line = prefix_text.trim_ascii().is_empty() && postfix_text.trim_ascii().is_empty();

            pragmas.push(Pragma {
                kind,
                span,
                trivia_span: trivia.span,
                code_span,
                count_span: None,
                expected_matches: 1,
                matches: 0,
                start_line,
                end_line,
                own_line,
                category: default_category,
                code,
                description,
                scope_span: None,
            });

            continue;
        };

        if contains_ascii_whitespace(category_bytes) {
            continue; // Invalid category format.
        }

        let Some(category) = alloc_utf8(arena, category_bytes) else {
            continue;
        };

        if !categories.contains(&category) {
            continue; // Skip if category is not recognized.
        }

        let rest = rest.trim_ascii_start();

        let (codes_part, after_codes) = splitn_whitespace_2(rest);
        if codes_part.is_empty() {
            continue; // Malformed pragma, no code.
        }

        let description_bytes = after_codes.trim_ascii();
        let Some(description) = alloc_utf8(arena, description_bytes) else {
            continue;
        };

        // Split codes by comma and create a pragma for each code.
        // We iterate comma positions via memchr_iter, tracking each chunk's start in `chunk_start`,
        // and computing the trimmed code's start offset incrementally without re-scanning.
        let codes_start_offset = absolute_line_start + (codes_part.as_ptr() as u32) - (line.as_ptr() as u32);

        let mut chunk_start = 0usize;
        let mut comma_positions = memchr::memchr_iter(b',', codes_part);
        loop {
            let chunk_end = comma_positions.next().unwrap_or(codes_part.len());
            let raw_chunk = &codes_part[chunk_start..chunk_end];
            let code_bytes = raw_chunk.trim_ascii();
            let next_chunk_start = chunk_end + 1;

            if code_bytes.is_empty() {
                if chunk_end == codes_part.len() {
                    break;
                }
                chunk_start = next_chunk_start;
                continue;
            }

            let Some(code) = alloc_utf8(arena, code_bytes) else {
                if chunk_end == codes_part.len() {
                    break;
                }
                chunk_start = next_chunk_start;
                continue;
            };

            // Trimmed code's start offset inside codes_part = chunk start + leading whitespace count.
            let code_start_in_codes_part = chunk_start + (raw_chunk.len() - raw_chunk.trim_ascii_start().len());
            let code_start_offset = codes_start_offset + code_start_in_codes_part as u32;
            let code_span = Span::new(file.id, code_start_offset, code_start_offset + code.len() as u32);

            // Parse an optional `(N)` count suffix. `code(3)` means "suppress up to 3 issues".
            // A missing or malformed suffix defaults to 1.
            let (base_code, expected_matches, count_span) = match parse_count_suffix(code) {
                Some((base, count, count_start_in_code)) => {
                    let count_start = code_start_offset + count_start_in_code as u32;
                    let count_end = code_start_offset + code.len() as u32;
                    (base, count, Some(Span::new(file.id, count_start, count_end)))
                }
                None => (code, 1u16, None),
            };

            let pragma_end_offset = pragma_start_offset + prefix.len() as u32 + content_with_leading_space.len() as u32;
            let span = Span::new(file.id, pragma_start_offset, pragma_end_offset);

            let start_line = file.line_number(trivia.span.start.offset);
            let end_line = file.line_number(trivia.span.end.offset);
            let line_start_offset = file.get_line_start_offset(start_line).unwrap_or(0);
            let line_end_offset = file.get_line_end_offset(end_line).unwrap_or(file.contents.len() as u32);
            let prefix_text = &file.contents[line_start_offset as usize..trivia.span.start.offset as usize];
            let postfix_text = &file.contents[trivia.span.end.offset as usize..line_end_offset as usize];
            let own_line = prefix_text.trim_ascii().is_empty() && postfix_text.trim_ascii().is_empty();

            pragmas.push(Pragma {
                kind,
                span,
                trivia_span: trivia.span,
                code_span,
                count_span,
                expected_matches,
                matches: 0,
                start_line,
                end_line,
                own_line,
                category,
                code: base_code,
                description,
                scope_span: None,
            });

            if chunk_end == codes_part.len() {
                break;
            }
            chunk_start = next_chunk_start;
        }
    }

    pragmas
}

/// Parses the optional `(N)` count suffix from a pragma code.
///
/// Returns `Some((base_code, count, count_start_offset_in_code))` when the code ends with a
/// well-formed `(N)` suffix where `N` is a non-zero positive integer. Returns `None` otherwise,
/// meaning the caller should treat the entire string as a plain code with an implicit count of 1.
///
/// Malformed suffixes (`(0)`, `(abc)`, `(-1)`, unbalanced parentheses, etc.) are ignored so the
/// code is matched as-is — it will then fail the "is this a known code?" check downstream and
/// be reported as an unfulfilled pragma, which gives a clearer error than silently dropping it.
fn parse_count_suffix(code: &str) -> Option<(&str, u16, usize)> {
    if !code.ends_with(')') {
        return None;
    }

    let paren_start = code.rfind('(')?;
    if paren_start == 0 {
        return None;
    }

    let base = code[..paren_start].trim_end();
    if base.is_empty() {
        return None;
    }

    let count_str = code[paren_start + 1..code.len() - 1].trim();
    let count: u16 = count_str.parse().ok()?;
    if count == 0 {
        return None;
    }

    Some((base, count, paren_start))
}
