//! Stateful Twig parser.
//!
//! The [`Parser`] struct owns the arena, a [`TokenStream`], parse [`State`],
//! and an error accumulator.
//!
//! Top-level parsing never bails on errors: every [`Template`] returned by
//! [`Parser::parse`] contains whatever statements were successfully parsed,
//! plus a list of errors for the parts that did not.

use mago_allocator::prelude::*;

use mago_database::file::File;
use mago_database::file::FileId;
use mago_database::file::HasFileId;
use mago_syntax_core::input::Input;

use crate::ast::Sequence;
use crate::ast::Template;
use crate::error::ParseError;
use crate::lexer::TwigLexer;
use crate::parser::stream::TokenStream;
use crate::settings::ParserSettings;

pub(crate) mod internal;

pub mod stream;

/// Maximum recursion depth for expression parsing. Prevents stack overflow
/// on pathologically nested expressions.
pub const MAX_RECURSION_DEPTH: u16 = 512;

/// Transient parser state. Carries information that cannot be derived
/// purely from the token stream - recursion depth, interpolation context,
/// etc.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct State {
    /// Whether the parser is currently inside a `"...#{ ... }..."`
    /// interpolation expression.
    pub within_string_interpolation: bool,
    /// Whether the parser is currently inside a `{% verbatim %}` block.
    pub within_verbatim: bool,
    /// Current expression recursion depth.
    pub recursion_depth: u16,
}

/// Stateful Twig parser.
#[derive(Debug)]
#[allow(dead_code)]
#[allow(clippy::field_scoped_visibility_modifiers)]
pub struct Parser<'input, 'arena, A>
where
    A: Arena,
{
    pub(crate) arena: &'arena A,
    pub(crate) settings: ParserSettings,
    pub(crate) state: State,
    pub(crate) stream: TokenStream<'arena, A>,
    pub(crate) errors: Vec<'arena, ParseError<'arena>, A>,
    _input: std::marker::PhantomData<&'input ()>,
}

impl<'arena, A> Parser<'_, 'arena, A>
where
    A: Arena,
{
    /// Build a parser for `content`, which must already live in the arena.
    #[inline]
    #[must_use]
    pub fn new(arena: &'arena A, file_id: FileId, content: &'arena [u8], settings: ParserSettings) -> Self {
        let input = Input::new(file_id, content);
        let lexer = TwigLexer::new(input, settings.lexer);
        let stream = TokenStream::new(arena, lexer);
        Self {
            arena,
            settings,
            state: State::default(),
            stream,
            errors: Vec::new_in(arena),
            _input: std::marker::PhantomData,
        }
    }

    /// Build a parser from a pre-constructed lexer, typically when the
    /// source has been anchored to a non-zero position via
    /// [`Input::anchored_at`].
    ///
    /// `source_text` must be the same `&str` that backs the lexer's
    /// [`Input`].
    #[inline]
    #[must_use]
    pub fn from_lexer(
        arena: &'arena A,
        _source_text: &'arena [u8],
        lexer: TwigLexer<'arena>,
        settings: ParserSettings,
    ) -> Self {
        let stream = TokenStream::new(arena, lexer);
        Self {
            arena,
            settings,
            state: State::default(),
            stream,
            errors: Vec::new_in(arena),
            _input: std::marker::PhantomData,
        }
    }

    /// Consume the parser and produce an arena-allocated [`Template`].
    #[must_use]
    pub fn parse(mut self, source_text: &'arena [u8], file_id: FileId) -> &'arena Template<'arena>
    where
        A: Arena,
    {
        let statements = match self.parse_statements(&internal::NoTerminator) {
            Ok(sequence) => sequence,
            Err(error) => {
                self.errors.push(error);
                while let Some(result) = self.stream.advance() {
                    if let Err(error) = result {
                        self.errors.push(error.into());
                        break;
                    }
                }
                Sequence::empty()
            }
        };

        if let Err(error) = self.stream.expect_eof() {
            self.errors.push(error);
        }

        let trivia = self.stream.get_trivia();

        self.arena.alloc(Template { file_id, source_text, trivia, statements, errors: self.errors.leak() })
    }

    #[inline]
    #[must_use]
    pub fn settings(&self) -> &ParserSettings {
        &self.settings
    }
}

/// Parse a Twig template file into an AST, with default settings.
#[inline]
pub fn parse_file<'arena, A>(arena: &'arena A, file: &File) -> &'arena Template<'arena>
where
    A: Arena,
{
    parse_file_content(arena, file.file_id(), file.contents.as_ref())
}

/// Parse a Twig template file into an AST with the supplied settings.
#[inline]
pub fn parse_file_with_settings<'arena, A>(
    arena: &'arena A,
    file: &File,
    settings: ParserSettings,
) -> &'arena Template<'arena>
where
    A: Arena,
{
    parse_file_content_with_settings(arena, file.file_id(), file.contents.as_ref(), settings)
}

/// Parse Twig source into an AST, associating every produced [`Span`] with
/// the supplied [`FileId`].  Uses default parser settings.
pub fn parse_file_content<'arena, A>(arena: &'arena A, file_id: FileId, content: &[u8]) -> &'arena Template<'arena>
where
    A: Arena,
{
    parse_file_content_with_settings(arena, file_id, content, ParserSettings::default())
}

/// Parse Twig source into an AST with the supplied [`FileId`] and
/// [`ParserSettings`].  The `content` is copied into the arena so that the
/// resulting [`Template`] owns its `source_text` slice.
pub fn parse_file_content_with_settings<'arena, A>(
    arena: &'arena A,
    file_id: FileId,
    content: &[u8],
    settings: ParserSettings,
) -> &'arena Template<'arena>
where
    A: Arena,
{
    let source_text = arena.alloc_slice_copy(content);
    Parser::new(arena, file_id, source_text, settings).parse(source_text, file_id)
}
