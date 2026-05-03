//! Stateful lexer for Twig templates.
//!
//! The lexer is lossless: every byte of the input appears in the value of
//! exactly one non-EOF token. It drives a small state machine that mirrors
//! the one in upstream `Twig\Lexer`.
//!
//! # Safety conventions
//!
//! Every `unsafe { ... }` in this file calls into `memchr` /
//! `slice::get_unchecked` after bounds have been established by either:
//!
//! - the surrounding `i < bytes.len()` / `pos < total` guard, or
//! - a fresh `memchr` result whose returned offset is, by `memchr`'s
//!   contract, `< bytes.len()`.
//!
//! Adding individual `// SAFETY:` comments on every block would just
//! repeat that contract verbatim, so we suppress the lint for the file.

mod internal;

use core::hint::unreachable_unchecked;

use memchr::memchr;
use memchr::memmem;

use mago_database::file::FileId;
use mago_database::file::HasFileId;
use mago_span::Position;
use mago_syntax_core::input::Input;
use mago_syntax_core::part_of_identifier;

use crate::error::SyntaxError;
use crate::lexer::internal::consts::BYTE_CLASS;
use crate::lexer::internal::consts::ByteClass;
use crate::lexer::internal::consts::IDENT_PART;
use crate::lexer::internal::keyword::continuation_words_for;
use crate::lexer::internal::keyword::single_word_operator_kind;
use crate::lexer::internal::mode::LexerMode;
use crate::lexer::internal::mode::VerbatimKind;
use crate::lexer::internal::mode::VerbatimStage;
use crate::lexer::internal::stack::Bracket;
use crate::lexer::internal::stack::BracketStack;
use crate::lexer::internal::stack::ModeStack;
use crate::lexer::internal::utils::closer_kind;
use crate::lexer::internal::utils::is_whitespace_byte;
use crate::lexer::internal::utils::is_word_byte;
use crate::lexer::internal::utils::matching_closer;
use crate::lexer::internal::utils::opener_kind;
use crate::lexer::internal::utils::single_byte_symbol;
use crate::lexer::internal::utils::three_byte_operator;
use crate::lexer::internal::utils::two_byte_operator;
use crate::settings::LexerSettings;
use crate::token::TwigToken;
use crate::token::TwigTokenKind;

#[derive(Debug)]
pub struct TwigLexer<'input> {
    input: Input<'input>,
    settings: LexerSettings,
    mode: LexerMode,
    mode_stack: ModeStack,
    brackets: BracketStack,
    eof_emitted: bool,
    last_significant: Option<u8>,
}

#[allow(clippy::undocumented_unsafe_blocks)]
impl<'input> TwigLexer<'input> {
    #[inline]
    #[must_use]
    pub fn new(input: Input<'input>, settings: LexerSettings) -> TwigLexer<'input> {
        TwigLexer {
            input,
            settings,
            mode: LexerMode::Data,
            mode_stack: ModeStack::new(),
            brackets: BracketStack::new(),
            eof_emitted: false,
            last_significant: None,
        }
    }

    #[inline]
    #[must_use]
    pub fn settings(&self) -> &LexerSettings {
        &self.settings
    }

    #[inline]
    #[must_use]
    pub fn has_reached_eof(&self) -> bool {
        self.input.has_reached_eof()
    }

    #[inline]
    #[must_use]
    pub fn current_position(&self) -> Position {
        self.input.current_position()
    }

    #[inline]
    #[must_use]
    pub fn slice_in_range(&self, from: u32, to: u32) -> &'input str {
        let bytes_slice = self.input.slice_in_range(from, to);
        bytes_slice.utf8_chunks().next().map_or("", |chunk| chunk.valid())
    }

    #[inline]
    pub fn advance(&mut self) -> Option<Result<TwigToken<'input>, SyntaxError>> {
        if self.eof_emitted {
            return None;
        }
        if self.input.has_reached_eof() {
            self.eof_emitted = true;
            if let Some(b) = self.brackets.last() {
                return Some(Err(SyntaxError::UnclosedBracket(self.file_id(), b.opener, b.position)));
            }
            if !self.mode_stack.is_empty() || self.mode != LexerMode::Data {
                return Some(Err(SyntaxError::UnclosedTag(
                    self.file_id(),
                    self.mode.tag_name(),
                    self.current_position(),
                )));
            }
            return None;
        }

        let result = match self.mode {
            LexerMode::Data => self.lex_data(),
            LexerMode::Block => self.lex_inside_block(),
            LexerMode::Variable => self.lex_inside_variable(),
            LexerMode::DoubleQuoted => self.lex_inside_dq_string(),
            LexerMode::Interpolation => self.lex_inside_interpolation(),
            LexerMode::Verbatim(kind, stage) => self.lex_verbatim(kind, stage),
        };

        match result {
            Ok(tok) => {
                if !tok.kind.is_trivia()
                    && !tok.value.is_empty()
                    && let Some(b) = tok.value.as_bytes().last()
                {
                    self.last_significant = Some(*b);
                }
                Some(Ok(tok))
            }
            Err(err) => Some(Err(err)),
        }
    }

    #[inline]
    fn cursor(&self) -> usize {
        self.input.current_offset()
    }

    #[inline]
    fn total_len(&self) -> usize {
        self.input.len()
    }

    #[inline]
    fn jump_forward_to(&mut self, pos: usize) {
        let cur = self.cursor();
        debug_assert!(pos >= cur, "lexer cursor must not move backward");
        self.input.skip(pos - cur);
    }

    #[inline]
    fn position_at(&self, offset: usize) -> Position {
        Position::new(offset as u32)
    }

    #[inline]
    fn byte_at(&self, offset: usize) -> u8 {
        *self.input.read_at(offset)
    }

    #[inline]
    fn byte_opt(&self, offset: usize) -> Option<u8> {
        if offset < self.total_len() { Some(*self.input.read_at(offset)) } else { None }
    }

    #[inline]
    fn slice_bytes(&self, from: usize, to: usize) -> &'input [u8] {
        self.input.slice_in_range(from as u32, to as u32)
    }

    #[inline]
    fn slice_str(&self, from: usize, to: usize) -> &'input str {
        // SAFETY: every parser entry point takes a `&str`, so the underlying input is always
        // valid UTF-8; `slice_in_range` returns a sub-slice on byte boundaries that the lexer
        // chooses, so the resulting bytes still form a valid UTF-8 sequence.
        unsafe { std::str::from_utf8_unchecked(self.slice_bytes(from, to)) }
    }

    /// Find the next occurrence of any of `{%`, `{{`, `{#` starting at `from`.
    /// Returns `(offset, kind_byte)` where `kind_byte` is `b'%'`, `b'{'`, or
    /// `b'#'` identifying which introducer was matched.
    ///
    /// Uses a single SIMD-backed `memchr(b'{', ...)` scan and then a one-byte
    /// lookahead on each hit.  This is significantly faster than running three
    /// separate `memmem::find` passes over the haystack.
    fn find_next_introducer(&self, from: usize) -> Option<(usize, u8)> {
        let total = self.total_len();
        let bytes = self.slice_bytes(from, total);
        let mut offset = 0usize;
        while let Some(rel) = memchr(b'{', unsafe { bytes.get_unchecked(offset..) }) {
            let at = offset + rel;
            match bytes.get(at + 1).copied() {
                Some(b'%') => return Some((from + at, b'%')),
                Some(b'{') => return Some((from + at, b'{')),
                Some(b'#') => return Some((from + at, b'#')),
                _ => offset = at + 1,
            }
        }
        None
    }

    fn lex_data(&mut self) -> Result<TwigToken<'input>, SyntaxError> {
        let start = self.cursor();
        match self.find_next_introducer(start) {
            None => {
                let end = self.total_len();
                let value = self.slice_str(start, end);
                self.jump_forward_to(end);
                debug_assert!(!value.is_empty(), "lex_data called at EOF");
                Ok(TwigToken::new(TwigTokenKind::RawText, value, self.position_at(start)))
            }
            Some((at, kind_byte)) => {
                if at > start {
                    let value = self.slice_str(start, at);
                    self.jump_forward_to(at);
                    return Ok(TwigToken::new(TwigTokenKind::RawText, value, self.position_at(start)));
                }
                let introducer_start = self.cursor();
                let trim_byte = self.byte_opt(introducer_start + 2);
                let introducer_len = if matches!(trim_byte, Some(b'-') | Some(b'~')) { 3 } else { 2 };
                let end = introducer_start + introducer_len;
                let introducer_slice = self.slice_str(introducer_start, end);

                match kind_byte {
                    b'%' => {
                        let kind = match trim_byte {
                            Some(b'-') => TwigTokenKind::OpenBlockDash,
                            Some(b'~') => TwigTokenKind::OpenBlockTilde,
                            _ => TwigTokenKind::OpenBlock,
                        };
                        self.jump_forward_to(end);
                        // Lookahead: is this `{% verbatim %}` or `{% raw %}`?
                        // If so, switch into the dedicated verbatim mode -
                        // otherwise this is a normal block tag.
                        if let Some(verbatim_kind) = self.peek_verbatim_open() {
                            self.mode = LexerMode::Verbatim(verbatim_kind, VerbatimStage::Entering);
                            self.mode_stack.push(LexerMode::Data);
                        } else {
                            self.mode = LexerMode::Block;
                            self.mode_stack.push(LexerMode::Data);
                        }
                        Ok(TwigToken::new(kind, introducer_slice, self.position_at(introducer_start)))
                    }
                    b'{' => {
                        let kind = match trim_byte {
                            Some(b'-') => TwigTokenKind::OpenVariableDash,
                            Some(b'~') => TwigTokenKind::OpenVariableTilde,
                            _ => TwigTokenKind::OpenVariable,
                        };
                        self.jump_forward_to(end);
                        self.mode = LexerMode::Variable;
                        self.mode_stack.push(LexerMode::Data);
                        Ok(TwigToken::new(kind, introducer_slice, self.position_at(introducer_start)))
                    }
                    b'#' => {
                        self.jump_forward_to(end);
                        self.lex_comment_body(introducer_start, introducer_slice)
                    }
                    // SAFETY: `find_introducer` only returns `b'%'`, `b'{'`, or `b'#'` by
                    // construction; the three explicit arms above are exhaustive, so this branch
                    // is provably unreachable and we promise the optimizer can prune it.
                    _ => unsafe { unreachable_unchecked() },
                }
            }
        }
    }

    /// Lookahead from the current cursor (positioned just past the
    /// opening `{%` / `{%-` / `{%~`) to determine whether this tag is a
    /// `{% verbatim %}` or `{% raw %}`.  Does not consume any input.
    fn peek_verbatim_open(&self) -> Option<VerbatimKind> {
        let total = self.total_len();
        let mut i = self.cursor();
        while i < total && is_whitespace_byte(self.byte_at(i)) {
            i += 1;
        }
        if self.slice_bytes(i, (i + 8).min(total)) == b"verbatim"
            && (i + 8 >= total || !is_word_byte(self.byte_at(i + 8)))
        {
            return Some(VerbatimKind::Verbatim);
        }
        if self.slice_bytes(i, (i + 3).min(total)) == b"raw" && (i + 3 >= total || !is_word_byte(self.byte_at(i + 3))) {
            return Some(VerbatimKind::Raw);
        }
        None
    }

    /// Stage dispatcher for [`LexerMode::Verbatim`].  Each stage emits
    /// exactly one token (with optional preceding whitespace trivia) and
    /// transitions to the next.
    fn lex_verbatim(&mut self, kind: VerbatimKind, stage: VerbatimStage) -> Result<TwigToken<'input>, SyntaxError> {
        match stage {
            VerbatimStage::Entering => self.lex_verbatim_keyword(kind, kind.open_keyword(), VerbatimStage::Emitted),
            VerbatimStage::Emitted => self.lex_verbatim_close_marker(kind, VerbatimStage::Body),
            VerbatimStage::Body => self.lex_verbatim_body(kind),
            VerbatimStage::EndingOpened => self.lex_verbatim_open_marker(kind, VerbatimStage::EndOpened),
            VerbatimStage::EndOpened => self.lex_verbatim_keyword(kind, kind.end_keyword(), VerbatimStage::Ended),
            VerbatimStage::Ended => self.lex_verbatim_final_close(kind),
        }
    }

    /// Emit either preceding whitespace (as trivia) or the expected
    /// keyword `Name` token, then transition to `next_stage`.
    fn lex_verbatim_keyword(
        &mut self,
        kind: VerbatimKind,
        expected: &'static [u8],
        next_stage: VerbatimStage,
    ) -> Result<TwigToken<'input>, SyntaxError> {
        let start = self.cursor();
        if start < self.total_len() && is_whitespace_byte(self.byte_at(start)) {
            let consumed = self.input.consume_whitespaces();
            let slice = unsafe { std::str::from_utf8_unchecked(consumed) };
            return Ok(TwigToken::new(TwigTokenKind::Whitespace, slice, self.position_at(start)));
        }
        let total = self.total_len();
        let end = start + expected.len();
        if end > total || self.slice_bytes(start, end) != expected || (end < total && is_word_byte(self.byte_at(end))) {
            return Err(SyntaxError::UnexpectedCharacter(
                self.file_id(),
                self.byte_opt(start).unwrap_or(0),
                self.position_at(start),
            ));
        }
        let slice = self.slice_str(start, end);
        self.jump_forward_to(end);
        self.mode = LexerMode::Verbatim(kind, next_stage);
        Ok(TwigToken::new(TwigTokenKind::Name, slice, self.position_at(start)))
    }

    /// Emit either preceding whitespace or the closing `%}` (with optional
    /// `-`/`~` trim marker) of the opening tag, then transition.
    fn lex_verbatim_close_marker(
        &mut self,
        kind: VerbatimKind,
        next_stage: VerbatimStage,
    ) -> Result<TwigToken<'input>, SyntaxError> {
        let start = self.cursor();
        if start < self.total_len() && is_whitespace_byte(self.byte_at(start)) {
            let consumed = self.input.consume_whitespaces();
            let slice = unsafe { std::str::from_utf8_unchecked(consumed) };
            return Ok(TwigToken::new(TwigTokenKind::Whitespace, slice, self.position_at(start)));
        }
        let total = self.total_len();
        let trim_byte = self.byte_opt(start);
        let (close_kind, len) = match trim_byte {
            Some(b'-') if start + 3 <= total && self.slice_bytes(start + 1, start + 3) == b"%}" => {
                (TwigTokenKind::CloseBlockDash, 3)
            }
            Some(b'~') if start + 3 <= total && self.slice_bytes(start + 1, start + 3) == b"%}" => {
                (TwigTokenKind::CloseBlockTilde, 3)
            }
            _ if start + 2 <= total && self.slice_bytes(start, start + 2) == b"%}" => (TwigTokenKind::CloseBlock, 2),
            _ => {
                return Err(SyntaxError::UnclosedTag(self.file_id(), "verbatim tag", self.position_at(start)));
            }
        };
        let slice = self.slice_str(start, start + len);
        self.jump_forward_to(start + len);
        self.mode = LexerMode::Verbatim(kind, next_stage);
        Ok(TwigToken::new(close_kind, slice, self.position_at(start)))
    }

    /// Scan forward for the closing `{%` + optional trim + optional
    /// whitespace + `endverbatim`/`endraw` (word-boundary).  Emit a
    /// `VerbatimText` token covering the body and transition to
    /// `EndingOpened`.  If the body is empty, skip emission and fall
    /// through to emitting the closing `{%` directly.
    fn lex_verbatim_body(&mut self, kind: VerbatimKind) -> Result<TwigToken<'input>, SyntaxError> {
        let start = self.cursor();
        let close_at = self.find_verbatim_close(start, kind.end_keyword())?;
        if close_at > start {
            let slice = self.slice_str(start, close_at);
            self.jump_forward_to(close_at);
            self.mode = LexerMode::Verbatim(kind, VerbatimStage::EndingOpened);
            return Ok(TwigToken::new(TwigTokenKind::VerbatimText, slice, self.position_at(start)));
        }
        // Empty body: hand off directly to the closing-`{%` emitter.
        self.lex_verbatim_open_marker(kind, VerbatimStage::EndOpened)
    }

    /// Emit the closing tag's opening `{%` (with optional `-`/`~` trim
    /// marker), then transition.  Cursor is expected to sit on the `{`.
    fn lex_verbatim_open_marker(
        &mut self,
        kind: VerbatimKind,
        next_stage: VerbatimStage,
    ) -> Result<TwigToken<'input>, SyntaxError> {
        let start = self.cursor();
        let total = self.total_len();
        if start + 2 > total || self.slice_bytes(start, start + 2) != b"{%" {
            return Err(SyntaxError::UnclosedVerbatim(self.file_id(), self.position_at(start)));
        }
        let trim_byte = self.byte_opt(start + 2);
        let (open_kind, len) = match trim_byte {
            Some(b'-') => (TwigTokenKind::OpenBlockDash, 3),
            Some(b'~') => (TwigTokenKind::OpenBlockTilde, 3),
            _ => (TwigTokenKind::OpenBlock, 2),
        };
        let slice = self.slice_str(start, start + len);
        self.jump_forward_to(start + len);
        self.mode = LexerMode::Verbatim(kind, next_stage);
        Ok(TwigToken::new(open_kind, slice, self.position_at(start)))
    }

    /// Final stage: emit either preceding whitespace or the closing tag's
    /// `%}` (with optional trim marker), pop back to the previous mode.
    fn lex_verbatim_final_close(&mut self, kind: VerbatimKind) -> Result<TwigToken<'input>, SyntaxError> {
        let start = self.cursor();
        if start < self.total_len() && is_whitespace_byte(self.byte_at(start)) {
            let consumed = self.input.consume_whitespaces();
            let slice = unsafe { std::str::from_utf8_unchecked(consumed) };
            return Ok(TwigToken::new(TwigTokenKind::Whitespace, slice, self.position_at(start)));
        }
        let total = self.total_len();
        let trim_byte = self.byte_opt(start);
        let (close_kind, len) = match trim_byte {
            Some(b'-') if start + 3 <= total && self.slice_bytes(start + 1, start + 3) == b"%}" => {
                (TwigTokenKind::CloseBlockDash, 3)
            }
            Some(b'~') if start + 3 <= total && self.slice_bytes(start + 1, start + 3) == b"%}" => {
                (TwigTokenKind::CloseBlockTilde, 3)
            }
            _ if start + 2 <= total && self.slice_bytes(start, start + 2) == b"%}" => (TwigTokenKind::CloseBlock, 2),
            _ => {
                return Err(SyntaxError::UnclosedTag(self.file_id(), "verbatim tag", self.position_at(start)));
            }
        };
        let slice = self.slice_str(start, start + len);
        self.jump_forward_to(start + len);
        self.mode = self.mode_stack.pop().unwrap_or(LexerMode::Data);
        let _ = kind;
        Ok(TwigToken::new(close_kind, slice, self.position_at(start)))
    }

    /// Scan from `from` for the next `{%` followed (after optional `-`/`~`
    /// trim and optional whitespace) by `end_keyword` on a word boundary.
    /// Returns the offset of that `{%`.  Embedded `{%` that don't match
    /// the closer are skipped - everything inside verbatim is literal.
    fn find_verbatim_close(&self, from: usize, end_keyword: &[u8]) -> Result<usize, SyntaxError> {
        let total = self.total_len();
        let bytes = self.slice_bytes(0, total);
        let mut offset = from;
        while let Some(rel) = memchr(b'{', unsafe { bytes.get_unchecked(offset..) }) {
            let at = offset + rel;
            if at + 2 > total || unsafe { *bytes.get_unchecked(at + 1) } != b'%' {
                offset = at + 1;
                continue;
            }
            let mut k = at + 2;
            if k < total && matches!(unsafe { *bytes.get_unchecked(k) }, b'-' | b'~') {
                k += 1;
            }
            while k < total && is_whitespace_byte(unsafe { *bytes.get_unchecked(k) }) {
                k += 1;
            }
            let kw_end = k + end_keyword.len();
            if kw_end <= total
                && unsafe { bytes.get_unchecked(k..kw_end) } == end_keyword
                && (kw_end >= total || !is_word_byte(unsafe { *bytes.get_unchecked(kw_end) }))
            {
                return Ok(at);
            }
            offset = at + 2;
        }
        Err(SyntaxError::UnclosedVerbatim(self.file_id(), self.position_at(from)))
    }

    fn lex_comment_body(
        &mut self,
        introducer_start: usize,
        _introducer_slice: &'input str,
    ) -> Result<TwigToken<'input>, SyntaxError> {
        let search_from = self.cursor();
        let total = self.total_len();
        let haystack = self.slice_bytes(search_from, total);
        let Some(rel) = memmem::find(haystack, b"#}") else {
            return Err(SyntaxError::UnclosedComment(self.file_id(), self.position_at(introducer_start)));
        };
        let mut close_start = search_from + rel;
        let mut trim_len = 0usize;
        if close_start > search_from && matches!(self.byte_at(close_start - 1), b'-' | b'~') {
            close_start -= 1;
            trim_len = 1;
        }
        let closer_end = close_start + 2 + trim_len;
        let full_slice = self.slice_str(introducer_start, closer_end);

        self.jump_forward_to(closer_end);
        Ok(TwigToken::new(TwigTokenKind::Comment, full_slice, self.position_at(introducer_start)))
    }

    fn lex_inside_block(&mut self) -> Result<TwigToken<'input>, SyntaxError> {
        if self.brackets.is_empty()
            && let Some(tok) = self.try_close_tag(b'%')?
        {
            return Ok(tok);
        }
        self.lex_expression()
    }

    fn lex_inside_variable(&mut self) -> Result<TwigToken<'input>, SyntaxError> {
        if self.brackets.is_empty()
            && let Some(tok) = self.try_close_tag(b'}')?
        {
            return Ok(tok);
        }
        self.lex_expression()
    }

    /// Try to consume a tag closer that matches the current state.
    fn try_close_tag(&mut self, closer_byte: u8) -> Result<Option<TwigToken<'input>>, SyntaxError> {
        let start = self.cursor();
        let c0 = self.byte_opt(start);
        let c1 = self.byte_opt(start + 1);
        let c2 = self.byte_opt(start + 2);
        let (matched_len, kind) = match (c0, c1, c2) {
            (Some(b), Some(c), _) if b == closer_byte && c == b'}' => (
                Some(2usize),
                if closer_byte == b'%' { TwigTokenKind::CloseBlock } else { TwigTokenKind::CloseVariable },
            ),
            (Some(b'-'), Some(b), Some(b'}')) if b == closer_byte => (
                Some(3),
                if closer_byte == b'%' { TwigTokenKind::CloseBlockDash } else { TwigTokenKind::CloseVariableDash },
            ),
            (Some(b'~'), Some(b), Some(b'}')) if b == closer_byte => (
                Some(3),
                if closer_byte == b'%' { TwigTokenKind::CloseBlockTilde } else { TwigTokenKind::CloseVariableTilde },
            ),
            _ => (None, TwigTokenKind::CloseBlock),
        };
        if let Some(len) = matched_len {
            let slice = self.slice_str(start, start + len);
            self.jump_forward_to(start + len);
            self.mode = self.mode_stack.pop().unwrap_or(LexerMode::Data);
            return Ok(Some(TwigToken::new(kind, slice, self.position_at(start))));
        }
        Ok(None)
    }

    fn lex_expression(&mut self) -> Result<TwigToken<'input>, SyntaxError> {
        let start = self.cursor();
        let b = self.byte_at(start);

        match BYTE_CLASS[b as usize] {
            ByteClass::Whitespace => {
                let consumed = self.input.consume_whitespaces();
                let slice = unsafe { std::str::from_utf8_unchecked(consumed) };
                Ok(TwigToken::new(TwigTokenKind::Whitespace, slice, self.position_at(start)))
            }
            ByteClass::Hash => {
                let bytes = self.input.read_remaining();
                let end = start + memchr(b'\n', bytes).unwrap_or(bytes.len());
                let slice = self.slice_str(start, end);
                self.jump_forward_to(end);
                Ok(TwigToken::new(TwigTokenKind::InlineComment, slice, self.position_at(start)))
            }
            ByteClass::SingleQuote => self.read_single_quoted_string(),
            ByteClass::DoubleQuote => self.read_double_quoted_string_start(),
            ByteClass::Digit => self.read_number(),
            ByteClass::IdentifierStart => self.read_name_or_word_operator(),
            ByteClass::Other => self.read_operator_or_punctuation(),
        }
    }

    fn read_number(&mut self) -> Result<TwigToken<'input>, SyntaxError> {
        let start = self.cursor();
        let total = self.total_len();
        let bytes = self.slice_bytes(0, total);
        let at = |i: usize| unsafe { *bytes.get_unchecked(i) };

        let mut end = start;
        let mut last_was_digit = false;
        while end < total {
            let c = at(end);
            if c.is_ascii_digit() {
                end += 1;
                last_was_digit = true;
            } else if c == b'_' && last_was_digit && end + 1 < total && at(end + 1).is_ascii_digit() {
                end += 1;
                last_was_digit = false;
            } else {
                break;
            }
        }
        if end + 1 < total && at(end) == b'.' && at(end + 1).is_ascii_digit() {
            end += 1;
            last_was_digit = false;
            while end < total {
                let c = at(end);
                if c.is_ascii_digit() {
                    end += 1;
                    last_was_digit = true;
                } else if c == b'_' && last_was_digit && end + 1 < total && at(end + 1).is_ascii_digit() {
                    end += 1;
                    last_was_digit = false;
                } else {
                    break;
                }
            }
        }
        if end < total && matches!(at(end), b'e' | b'E') {
            let mut p = end + 1;
            if p < total && matches!(at(p), b'+' | b'-') {
                p += 1;
            }
            if p < total && at(p).is_ascii_digit() {
                end = p + 1;
                while end < total && at(end).is_ascii_digit() {
                    end += 1;
                }
            }
        }
        let slice = self.slice_str(start, end);
        self.jump_forward_to(end);
        Ok(TwigToken::new(TwigTokenKind::Number, slice, self.position_at(start)))
    }

    fn read_single_quoted_string(&mut self) -> Result<TwigToken<'input>, SyntaxError> {
        let start = self.cursor();
        let total = self.total_len();
        let bytes = self.slice_bytes(0, total);
        let mut i = start + 1;
        while i < total {
            // SIMD-scan for the next interesting byte (`\` or `'`).
            let Some(rel) = memchr::memchr2(b'\\', b'\'', unsafe { bytes.get_unchecked(i..) }) else {
                return Err(SyntaxError::UnclosedString(self.file_id(), self.position_at(start)));
            };

            i += rel;
            let c = unsafe { *bytes.get_unchecked(i) };
            if c == b'\\' {
                if i + 1 >= total {
                    return Err(SyntaxError::UnclosedString(self.file_id(), self.position_at(start)));
                }
                i += 2;
                continue;
            }

            // Must be `'`: close the string.
            i += 1;
            let slice = self.slice_str(start, i);
            self.jump_forward_to(i);
            return Ok(TwigToken::new(TwigTokenKind::StringSingleQuoted, slice, self.position_at(start)));
        }

        Err(SyntaxError::UnclosedString(self.file_id(), self.position_at(start)))
    }

    fn read_double_quoted_string_start(&mut self) -> Result<TwigToken<'input>, SyntaxError> {
        let start = self.cursor();
        let total = self.total_len();
        let bytes = self.slice_bytes(0, total);
        let mut i = start + 1;
        let mut has_interp = false;
        while i < total {
            // SIMD-scan for the next interesting byte (`\`, `#`, or `"`).
            let Some(rel) = memchr::memchr3(b'\\', b'#', b'"', unsafe { bytes.get_unchecked(i..) }) else {
                return Err(SyntaxError::UnclosedString(self.file_id(), self.position_at(start)));
            };
            i += rel;
            let c = unsafe { *bytes.get_unchecked(i) };
            if c == b'\\' {
                if i + 1 >= total {
                    return Err(SyntaxError::UnclosedString(self.file_id(), self.position_at(start)));
                }
                i += 2;
                continue;
            }
            if c == b'#' {
                if i + 1 < total && unsafe { *bytes.get_unchecked(i + 1) } == b'{' {
                    has_interp = true;
                    break;
                }
                i += 1;
                continue;
            }
            // Must be `"`: close the string.
            i += 1;
            let slice = self.slice_str(start, i);
            self.jump_forward_to(i);
            return Ok(TwigToken::new(TwigTokenKind::StringDoubleQuoted, slice, self.position_at(start)));
        }
        if !has_interp {
            return Err(SyntaxError::UnclosedString(self.file_id(), self.position_at(start)));
        }
        let slice = self.slice_str(start, start + 1);
        self.jump_forward_to(start + 1);
        self.brackets.push(Bracket { opener: b'"', position: self.position_at(start) });
        self.mode_stack.push(self.mode);
        self.mode = LexerMode::DoubleQuoted;
        Ok(TwigToken::new(TwigTokenKind::DoubleQuoteStart, slice, self.position_at(start)))
    }

    fn lex_inside_dq_string(&mut self) -> Result<TwigToken<'input>, SyntaxError> {
        let start = self.cursor();
        let total = self.total_len();
        if self.slice_bytes(start, (start + 2).min(total)) == b"#{" {
            let slice = self.slice_str(start, start + 2);
            self.jump_forward_to(start + 2);
            self.brackets.push(Bracket { opener: b'#', position: self.position_at(start) });
            self.mode_stack.push(self.mode);
            self.mode = LexerMode::Interpolation;
            return Ok(TwigToken::new(TwigTokenKind::InterpolationStart, slice, self.position_at(start)));
        }
        if self.byte_at(start) == b'"' {
            let slice = self.slice_str(start, start + 1);
            self.jump_forward_to(start + 1);
            if !matches!(self.brackets.last(), Some(b) if b.opener == b'"') {
                return Err(SyntaxError::UnmatchedBracket(self.file_id(), b'"', self.position_at(start)));
            }
            self.brackets.pop();
            self.mode = self.mode_stack.pop().unwrap_or(LexerMode::Data);
            return Ok(TwigToken::new(TwigTokenKind::DoubleQuoteEnd, slice, self.position_at(start)));
        }
        let bytes = self.slice_bytes(0, total);
        let mut i = start;
        while i < total {
            // SIMD-scan for the next interesting byte (`\`, `#`, or `"`).
            let Some(rel) = memchr::memchr3(b'\\', b'#', b'"', unsafe { bytes.get_unchecked(i..) }) else {
                return Err(SyntaxError::UnclosedString(self.file_id(), self.position_at(start)));
            };

            i += rel;
            let c = unsafe { *bytes.get_unchecked(i) };
            if c == b'\\' {
                if i + 1 >= total {
                    return Err(SyntaxError::UnclosedString(self.file_id(), self.position_at(start)));
                }
                i += 2;
                continue;
            }
            if c == b'#' {
                if i + 1 < total && unsafe { *bytes.get_unchecked(i + 1) } == b'{' {
                    break;
                }
                i += 1;
                continue;
            }

            // Must be `"`.
            break;
        }

        if i == start {
            return Err(SyntaxError::UnclosedString(self.file_id(), self.position_at(start)));
        }

        let slice = self.slice_str(start, i);
        self.jump_forward_to(i);
        Ok(TwigToken::new(TwigTokenKind::StringPart, slice, self.position_at(start)))
    }

    fn lex_inside_interpolation(&mut self) -> Result<TwigToken<'input>, SyntaxError> {
        let start = self.cursor();
        if matches!(self.brackets.last(), Some(b) if b.opener == b'#') && self.byte_opt(start) == Some(b'}') {
            let slice = self.slice_str(start, start + 1);
            self.jump_forward_to(start + 1);
            self.brackets.pop();
            self.mode = self.mode_stack.pop().unwrap_or(LexerMode::Data);
            return Ok(TwigToken::new(TwigTokenKind::InterpolationEnd, slice, self.position_at(start)));
        }
        self.lex_expression()
    }

    fn read_name_or_word_operator(&mut self) -> Result<TwigToken<'input>, SyntaxError> {
        let start = self.cursor();
        let total = self.total_len();
        let bytes = self.slice_bytes(0, total);
        let mut end = start + 1;
        while end < total && IDENT_PART[unsafe { *bytes.get_unchecked(end) } as usize] {
            end += 1;
        }
        // Recognise `b-and` / `b-or` / `b-xor` as single operator tokens when
        // `b` is followed by `-and` / `-or` / `-xor` on a word boundary.
        if end == start + 1 && self.byte_at(start) == b'b' && self.byte_opt(end) == Some(b'-') {
            for (suffix, kind) in [
                (b"-and".as_slice(), TwigTokenKind::BAnd),
                (b"-or".as_slice(), TwigTokenKind::BOr),
                (b"-xor".as_slice(), TwigTokenKind::BXor),
            ] {
                if end + suffix.len() <= total && self.slice_bytes(end, end + suffix.len()) == suffix {
                    let after = end + suffix.len();
                    if after >= total || !matches!(self.byte_at(after), part_of_identifier!()) {
                        let slice = self.slice_str(start, after);
                        self.jump_forward_to(after);
                        return Ok(TwigToken::new(kind, slice, self.position_at(start)));
                    }
                }
            }
        }

        let name = self.slice_str(start, end);

        // Context: if the previous significant byte was `.` or `|`, the word
        // is NOT a word operator regardless of its spelling.
        let may_be_word_op = !matches!(self.last_significant, Some(b'.') | Some(b'|'));

        if may_be_word_op
            && let Some(conts) = continuation_words_for(name)
            && let Some((new_end, collapsed, kind)) = self.try_extend_word_multi(start, end, conts)
        {
            self.jump_forward_to(new_end);
            return Ok(TwigToken::new(kind, collapsed, self.position_at(start)));
        }

        if may_be_word_op && let Some(kind) = single_word_operator_kind(name) {
            self.jump_forward_to(end);
            return Ok(TwigToken::new(kind, name, self.position_at(start)));
        }

        self.jump_forward_to(end);
        Ok(TwigToken::new(TwigTokenKind::Name, name, self.position_at(start)))
    }

    /// Attempt to extend the token starting at `after_first` with any of the
    /// given continuation words separated by whitespace.  Returns the end
    /// offset, the source slice covering the full compound operator, and the
    /// resolved token kind.
    fn try_extend_word_multi(
        &self,
        first_start: usize,
        after_first: usize,
        continuations: &[(&str, TwigTokenKind)],
    ) -> Option<(usize, &'input str, TwigTokenKind)> {
        for (cont, kind) in continuations {
            if let Some(end) = self.try_extend_word(after_first, cont) {
                let slice = self.slice_str(first_start, end);
                return Some((end, slice, *kind));
            }
        }
        None
    }

    /// If the current position is followed by whitespace and then `word` (on a
    /// word boundary), return the new end offset.  Otherwise `None`.
    fn try_extend_word(&self, after_first: usize, word: &str) -> Option<usize> {
        let total = self.total_len();
        let mut i = after_first;
        let ws_start = i;
        while i < total && is_whitespace_byte(self.byte_at(i)) {
            i += 1;
        }
        if i == ws_start {
            return None;
        }
        let word_bytes = word.as_bytes();
        if i + word_bytes.len() > total {
            return None;
        }
        if self.slice_bytes(i, i + word_bytes.len()) != word_bytes {
            return None;
        }
        let after_word = i + word_bytes.len();
        if after_word < total && matches!(self.byte_at(after_word), part_of_identifier!()) {
            return None;
        }
        Some(after_word)
    }

    fn read_operator_or_punctuation(&mut self) -> Result<TwigToken<'input>, SyntaxError> {
        let start = self.cursor();
        let b0 = self.byte_at(start);
        let b1 = self.byte_opt(start + 1);
        let b2 = self.byte_opt(start + 2);

        if let Some((kind, len)) = three_byte_operator(b0, b1, b2) {
            return self.emit_fixed(start, len, kind);
        }
        if let Some((kind, len)) = two_byte_operator(b0, b1) {
            return self.emit_fixed(start, len, kind);
        }

        if let Some(kind) = opener_kind(b0) {
            self.brackets.push(Bracket { opener: b0, position: self.position_at(start) });
            self.jump_forward_to(start + 1);
            let slice = self.slice_str(start, start + 1);
            return Ok(TwigToken::new(kind, slice, self.position_at(start)));
        }
        if let Some(kind) = closer_kind(b0) {
            if !matches!(self.brackets.last(), Some(b) if matching_closer(b.opener) == b0) {
                return Err(SyntaxError::UnmatchedBracket(self.file_id(), b0, self.position_at(start)));
            }
            self.brackets.pop();
            self.jump_forward_to(start + 1);
            let slice = self.slice_str(start, start + 1);
            return Ok(TwigToken::new(kind, slice, self.position_at(start)));
        }
        if let Some(kind) = single_byte_symbol(b0) {
            self.jump_forward_to(start + 1);
            let slice = self.slice_str(start, start + 1);
            return Ok(TwigToken::new(kind, slice, self.position_at(start)));
        }

        Err(SyntaxError::UnexpectedCharacter(self.file_id(), b0, self.position_at(start)))
    }

    #[inline]
    fn emit_fixed(&mut self, start: usize, len: usize, kind: TwigTokenKind) -> Result<TwigToken<'input>, SyntaxError> {
        self.jump_forward_to(start + len);
        let slice = self.slice_str(start, start + len);
        Ok(TwigToken::new(kind, slice, self.position_at(start)))
    }
}

impl HasFileId for TwigLexer<'_> {
    #[inline]
    fn file_id(&self) -> FileId {
        self.input.file_id()
    }
}
