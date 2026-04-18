//! Recursive-descent parser for the GritQL-subset surface grammar.

use bumpalo::Bump;
use grit_pattern_matcher::pattern::And;
use grit_pattern_matcher::pattern::BooleanConstant;
use grit_pattern_matcher::pattern::Contains;
use grit_pattern_matcher::pattern::DynamicPattern;
use grit_pattern_matcher::pattern::DynamicSnippet;
use grit_pattern_matcher::pattern::DynamicSnippetPart;
use grit_pattern_matcher::pattern::FloatConstant;
use grit_pattern_matcher::pattern::IntConstant;
use grit_pattern_matcher::pattern::Maybe;
use grit_pattern_matcher::pattern::Not;
use grit_pattern_matcher::pattern::Or;
use grit_pattern_matcher::pattern::Pattern;
use grit_pattern_matcher::pattern::Rewrite;
use grit_pattern_matcher::pattern::StringConstant;
use grit_pattern_matcher::pattern::Variable;
use grit_pattern_matcher::pattern::Within;
use mago_syntax_core::part_of_identifier;
use mago_syntax_core::start_of_identifier;

use crate::compiler::CompileError;
use crate::compiler::lookup_meta_slot;
use crate::compiler::lower_snippet_source;
use crate::compiler::reserve_meta_slot;
use crate::query::RESERVED_SLOT_COUNT;
use crate::query_context::MagoQueryContext;
use crate::surface::token::SurfaceToken;
use crate::surface::token::SurfaceTokenKind;

/// Parses a slice of non-trivia tokens into a [`Pattern<MagoQueryContext>`], threading
/// metavariable names through `variables` for slot assignment.
pub struct SurfaceParser<'arena, 'input> {
    arena: &'arena Bump,
    tokens: Vec<SurfaceToken<'input>>,
    cursor: usize,
    pub variables: Vec<String>,
}

impl<'arena, 'input> SurfaceParser<'arena, 'input> {
    pub fn new(arena: &'arena Bump, tokens: Vec<SurfaceToken<'input>>) -> Self {
        Self { arena, tokens, cursor: 0, variables: Vec::new() }
    }

    pub fn parse_pattern(&mut self) -> Result<Pattern<MagoQueryContext>, CompileError> {
        self.parse_rewrite()
    }

    pub fn peek(&self) -> Option<&SurfaceToken<'input>> {
        self.tokens.get(self.cursor)
    }

    pub fn position(&self) -> usize {
        self.cursor
    }

    fn advance(&mut self) -> Option<SurfaceToken<'input>> {
        let tok = *self.tokens.get(self.cursor)?;
        self.cursor += 1;
        Some(tok)
    }

    fn peek_kind(&self) -> Option<SurfaceTokenKind> {
        self.peek().map(|t| t.kind)
    }

    fn eat(&mut self, kind: SurfaceTokenKind) -> bool {
        if self.peek_kind() == Some(kind) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: SurfaceTokenKind) -> Result<SurfaceToken<'input>, CompileError> {
        match self.advance() {
            Some(tok) if tok.kind == kind => Ok(tok),
            Some(tok) => Err(CompileError::SurfaceError(format!(
                "expected {kind:?} at offset {}, got {:?} ({:?})",
                tok.start.offset, tok.kind, tok.value
            ))),
            None => Err(CompileError::SurfaceError(format!("expected {kind:?}, got end of input"))),
        }
    }

    fn parse_rewrite(&mut self) -> Result<Pattern<MagoQueryContext>, CompileError> {
        let lhs = self.parse_subtype()?;
        if self.eat(SurfaceTokenKind::Rewrite) {
            // Every metavariable name introduced while parsing the LHS now lives in
            // `self.variables`. Snapshot the count so the RHS can only reference variables
            // that exist at-or-before this boundary; new names on the RHS would be
            // runtime-unbound and silently expand to errors.
            let lhs_variable_count = self.variables.len();
            let rhs = self.parse_rewrite_rhs(lhs_variable_count)?;
            return Ok(Pattern::Rewrite(Box::new(Rewrite::new(lhs, rhs, None))));
        }
        Ok(lhs)
    }

    /// Right-hand side of `=>`: a template rendered at rewrite time. Supports backtick
    /// snippets, bare `^name` variable references, and quoted string literals.
    ///
    /// `lhs_variable_count` is the length of `self.variables` before the RHS was parsed;
    /// any `^name` on the RHS whose slot index would exceed this count references a
    /// variable the LHS never bound, and is rejected.
    fn parse_rewrite_rhs(
        &mut self,
        lhs_variable_count: usize,
    ) -> Result<DynamicPattern<MagoQueryContext>, CompileError> {
        let tok = self
            .advance()
            .ok_or_else(|| CompileError::SurfaceError("expected right-hand side of `=>`, got end of input".into()))?;
        match tok.kind {
            SurfaceTokenKind::Backtick => {
                let body = strip_backticks(tok.value);
                let parts = self.build_template_parts(body, lhs_variable_count)?;
                Ok(DynamicPattern::Snippet(DynamicSnippet { parts }))
            }
            SurfaceTokenKind::Variable => {
                let name = variable_name(tok.value)?;
                let slot = lookup_meta_slot(&self.variables[..lhs_variable_count], name).ok_or_else(|| {
                    CompileError::SurfaceError(format!(
                        "right-hand side references `^{name}`, which is not bound by the left-hand side of the rewrite"
                    ))
                })?;
                Ok(DynamicPattern::Variable(Variable::new(0, RESERVED_SLOT_COUNT + slot)))
            }
            SurfaceTokenKind::String => {
                let s = unescape_double_quoted(tok.value);
                Ok(DynamicPattern::Snippet(DynamicSnippet { parts: vec![DynamicSnippetPart::String(s)] }))
            }
            _ => Err(CompileError::SurfaceError(format!(
                "right-hand side of `=>` must be a backtick snippet, variable, or string literal; got {:?}",
                tok.kind
            ))),
        }
    }

    fn build_template_parts(
        &mut self,
        body: &str,
        lhs_variable_count: usize,
    ) -> Result<Vec<DynamicSnippetPart>, CompileError> {
        let bytes = body.as_bytes();
        let mut parts: Vec<DynamicSnippetPart> = Vec::new();
        let mut current = String::new();
        let mut i = 0;
        while i < bytes.len() {
            // `^...name`: sequence splice. Same slot shape as `^name`, except the
            // compiler has bound the slot to a `MagoBinding::NodeList` whose `text()`
            // renders its elements comma-joined.
            if bytes[i] == b'^'
                && bytes.get(i + 1) == Some(&b'.')
                && bytes.get(i + 2) == Some(&b'.')
                && bytes.get(i + 3) == Some(&b'.')
                && i + 4 < bytes.len()
                && matches!(bytes[i + 4], start_of_identifier!())
            {
                if !current.is_empty() {
                    parts.push(DynamicSnippetPart::String(std::mem::take(&mut current)));
                }
                let start = i + 4;
                let mut j = start;
                while j < bytes.len() && matches!(bytes[j], part_of_identifier!()) {
                    j += 1;
                }
                // SAFETY: the scanned range passed the ASCII-ident macros; ASCII only.
                let name = unsafe { std::str::from_utf8_unchecked(&bytes[start..j]) };
                let slot = lookup_meta_slot(&self.variables[..lhs_variable_count], name).ok_or_else(|| {
                    CompileError::SurfaceError(format!(
                        "right-hand side references `^...{name}`, which is not bound by the left-hand side of the rewrite"
                    ))
                })?;
                parts.push(DynamicSnippetPart::Variable(Variable::new(0, RESERVED_SLOT_COUNT + slot)));
                i = j;
                continue;
            }

            if bytes[i] == b'^' && i + 1 < bytes.len() && matches!(bytes[i + 1], start_of_identifier!()) {
                if !current.is_empty() {
                    parts.push(DynamicSnippetPart::String(std::mem::take(&mut current)));
                }
                let start = i + 1;
                let mut j = start;
                while j < bytes.len() && matches!(bytes[j], part_of_identifier!()) {
                    j += 1;
                }
                // SAFETY: the scanned range passed the ASCII-ident macros, so it is ASCII
                // alphanumerics and `_` only, which is valid UTF-8.
                let name = unsafe { std::str::from_utf8_unchecked(&bytes[start..j]) };
                let slot = lookup_meta_slot(&self.variables[..lhs_variable_count], name).ok_or_else(|| {
                    CompileError::SurfaceError(format!(
                        "right-hand side references `^{name}`, which is not bound by the left-hand side of the rewrite"
                    ))
                })?;
                parts.push(DynamicSnippetPart::Variable(Variable::new(0, RESERVED_SLOT_COUNT + slot)));
                i = j;
            } else {
                let width = utf8_char_width(bytes[i]);
                // SAFETY: `body` came from a `&str`; `width` is derived from the leading
                // byte, so `bytes[i..i + width]` is a complete UTF-8 scalar.
                current.push_str(unsafe { std::str::from_utf8_unchecked(&bytes[i..i + width]) });
                i += width;
            }
        }
        if !current.is_empty() {
            parts.push(DynamicSnippetPart::String(current));
        }
        Ok(parts)
    }

    fn parse_subtype(&mut self) -> Result<Pattern<MagoQueryContext>, CompileError> {
        let lhs = self.parse_where()?;
        if self.eat(SurfaceTokenKind::Subtype) {
            let rhs = self.parse_where()?;
            return Ok(Pattern::And(Box::new(And::new(vec![lhs, rhs]))));
        }
        Ok(lhs)
    }

    fn parse_where(&mut self) -> Result<Pattern<MagoQueryContext>, CompileError> {
        let head = self.parse_prefix()?;
        if self.eat(SurfaceTokenKind::KwWhere) {
            self.expect(SurfaceTokenKind::LeftBrace)?;
            let clauses = self.parse_pattern_list_until(SurfaceTokenKind::RightBrace)?;
            let mut combined = Vec::with_capacity(1 + clauses.len());
            combined.push(head);
            combined.extend(clauses);
            return Ok(Pattern::And(Box::new(And::new(combined))));
        }
        Ok(head)
    }

    fn parse_prefix(&mut self) -> Result<Pattern<MagoQueryContext>, CompileError> {
        match self.peek_kind() {
            Some(SurfaceTokenKind::Bang) | Some(SurfaceTokenKind::KwNot) => {
                self.cursor += 1;
                let inner = self.parse_prefix()?;
                Ok(Pattern::Not(Box::new(Not::new(inner))))
            }
            Some(SurfaceTokenKind::KwContains) => {
                self.cursor += 1;
                let inner = self.parse_prefix()?;
                Ok(Pattern::Contains(Box::new(Contains::new(inner, None))))
            }
            Some(SurfaceTokenKind::KwWithin) => {
                self.cursor += 1;
                let inner = self.parse_prefix()?;
                Ok(Pattern::Within(Box::new(Within::new(inner, None))))
            }
            Some(SurfaceTokenKind::KwMaybe) => {
                self.cursor += 1;
                let inner = self.parse_prefix()?;
                Ok(Pattern::Maybe(Box::new(Maybe::new(inner))))
            }
            Some(SurfaceTokenKind::KwBubble) => {
                self.cursor += 1;
                self.parse_prefix()
            }
            _ => self.parse_atom(),
        }
    }

    fn parse_atom(&mut self) -> Result<Pattern<MagoQueryContext>, CompileError> {
        let tok =
            self.advance().ok_or_else(|| CompileError::SurfaceError("expected a pattern, got end of input".into()))?;
        match tok.kind {
            SurfaceTokenKind::Backtick => {
                let body = strip_backticks(tok.value);
                let pattern = lower_snippet_source(self.arena, body, &mut self.variables)?;
                Ok(pattern)
            }
            SurfaceTokenKind::Variable => {
                let name = variable_name(tok.value)?;
                let slot = reserve_meta_slot(&mut self.variables, name);
                Ok(Pattern::Variable(Variable::new(0, RESERVED_SLOT_COUNT + slot)))
            }
            SurfaceTokenKind::String => {
                let s = unescape_double_quoted(tok.value);
                Ok(Pattern::StringConstant(StringConstant::new(s)))
            }
            SurfaceTokenKind::Integer => {
                let n: i64 = tok.value.parse().map_err(|e: std::num::ParseIntError| {
                    CompileError::SurfaceError(format!("invalid integer literal `{}`: {e}", tok.value))
                })?;
                Ok(Pattern::IntConstant(IntConstant::new(n)))
            }
            SurfaceTokenKind::Float => {
                let f: f64 = tok.value.parse().map_err(|e: std::num::ParseFloatError| {
                    CompileError::SurfaceError(format!("invalid float literal `{}`: {e}", tok.value))
                })?;
                Ok(Pattern::FloatConstant(FloatConstant::new(f)))
            }
            SurfaceTokenKind::True => Ok(Pattern::BooleanConstant(BooleanConstant::new(true))),
            SurfaceTokenKind::False => Ok(Pattern::BooleanConstant(BooleanConstant::new(false))),
            SurfaceTokenKind::KwUndefined => Ok(Pattern::Undefined),
            SurfaceTokenKind::LeftParen => {
                let inner = self.parse_pattern()?;
                self.expect(SurfaceTokenKind::RightParen)?;
                Ok(inner)
            }
            SurfaceTokenKind::KwOr => {
                self.expect(SurfaceTokenKind::LeftBrace)?;
                let patterns = self.parse_pattern_list_until(SurfaceTokenKind::RightBrace)?;
                Ok(Pattern::Or(Box::new(Or::new(patterns))))
            }
            SurfaceTokenKind::KwAnd => {
                self.expect(SurfaceTokenKind::LeftBrace)?;
                let patterns = self.parse_pattern_list_until(SurfaceTokenKind::RightBrace)?;
                Ok(Pattern::And(Box::new(And::new(patterns))))
            }
            SurfaceTokenKind::UnterminatedBacktick => {
                Err(CompileError::SurfaceError(format!("unterminated backtick snippet at offset {}", tok.start.offset)))
            }
            SurfaceTokenKind::UnterminatedString => {
                Err(CompileError::SurfaceError(format!("unterminated string literal at offset {}", tok.start.offset)))
            }
            SurfaceTokenKind::Unknown => Err(CompileError::SurfaceError(format!(
                "unexpected byte `{}` at offset {}",
                tok.value, tok.start.offset
            ))),
            _ => Err(CompileError::SurfaceError(format!(
                "unexpected token {:?} in atom position at offset {}",
                tok.kind, tok.start.offset
            ))),
        }
    }

    fn parse_pattern_list_until(
        &mut self,
        terminator: SurfaceTokenKind,
    ) -> Result<Vec<Pattern<MagoQueryContext>>, CompileError> {
        let mut patterns = Vec::new();
        if self.eat(terminator) {
            return Ok(patterns);
        }
        loop {
            patterns.push(self.parse_pattern()?);
            match self.peek_kind() {
                Some(SurfaceTokenKind::Comma) => {
                    self.cursor += 1;
                    if self.eat(terminator) {
                        return Ok(patterns);
                    }
                }
                Some(kind) if kind == terminator => {
                    self.cursor += 1;
                    return Ok(patterns);
                }
                Some(other) => {
                    return Err(CompileError::SurfaceError(format!("expected `,` or {terminator:?}, got {other:?}")));
                }
                None => {
                    return Err(CompileError::SurfaceError("unexpected end of input inside block".into()));
                }
            }
        }
    }
}

/// Strips surrounding `` ` `` delimiters from a raw backtick-token slice.
fn strip_backticks(value: &str) -> &str {
    let bytes = value.as_bytes();
    debug_assert!(bytes.len() >= 2 && bytes[0] == b'`' && bytes[bytes.len() - 1] == b'`');
    &value[1..value.len() - 1]
}

/// Extracts the `name` from a `^name` variable token; returns an error for bare `^`.
fn variable_name(value: &str) -> Result<&str, CompileError> {
    let bytes = value.as_bytes();
    debug_assert!(!bytes.is_empty() && bytes[0] == b'^');
    if bytes.len() == 1 {
        return Err(CompileError::SurfaceError("expected identifier after `^`".into()));
    }
    Ok(&value[1..])
}

/// Expands `\n`, `\t`, `\r`, `\"`, `\\` escapes in a double-quoted string literal and
/// strips the surrounding quotes. Unknown `\X` sequences pass `X` through literally.
fn unescape_double_quoted(value: &str) -> String {
    let bytes = value.as_bytes();
    debug_assert!(bytes.len() >= 2 && bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"');
    let body = &value[1..value.len() - 1];
    let body_bytes = body.as_bytes();
    let mut out = String::with_capacity(body.len());
    let mut i = 0;
    while i < body_bytes.len() {
        if body_bytes[i] == b'\\' && i + 1 < body_bytes.len() {
            match body_bytes[i + 1] {
                b'n' => out.push('\n'),
                b't' => out.push('\t'),
                b'r' => out.push('\r'),
                b'\\' => out.push('\\'),
                b'"' => out.push('"'),
                other => out.push(other as char),
            }
            i += 2;
        } else {
            let width = utf8_char_width(body_bytes[i]);
            // SAFETY: `body` came from a `&str`; `width` is derived from the leading byte.
            out.push_str(unsafe { std::str::from_utf8_unchecked(&body_bytes[i..i + width]) });
            i += width;
        }
    }
    out
}

fn utf8_char_width(first_byte: u8) -> usize {
    match first_byte {
        0b0000_0000..=0b0111_1111 => 1,
        0b1100_0000..=0b1101_1111 => 2,
        0b1110_0000..=0b1110_1111 => 3,
        0b1111_0000..=0b1111_0111 => 4,
        _ => 1,
    }
}
