use mago_allocator::Arena;
use mago_allocator::vec::Vec;

use mago_database::file::FileId;
use mago_database::file::HasFileId;
use mago_span::Position;
use mago_span::Span;
use mago_syntax_core::input::Input;

use crate::cst::Document;
use crate::cst::r#type::Type;
use crate::error::ParseError;
use crate::lexer::DocblockLexer;
use crate::parser::internal::stream::PHPDocTokenStream;

pub(crate) mod internal;

#[derive(Debug)]
#[allow(clippy::field_scoped_visibility_modifiers)]
pub struct PHPDocParser<'arena, A>
where
    A: Arena,
{
    pub(crate) arena: &'arena A,
    pub(crate) stream: PHPDocTokenStream<'arena, A>,
    pub(crate) errors: Vec<'arena, ParseError, A>,
}

impl<'arena, A> PHPDocParser<'arena, A>
where
    A: Arena,
{
    #[must_use]
    pub fn new(arena: &'arena A, content: &'arena [u8], span: Span) -> Self {
        let input = Input::anchored_at(span.file_id, content, span.start);
        let lexer = DocblockLexer::new(input);
        let stream = PHPDocTokenStream::new(arena, lexer, content, span.start);

        Self { arena, stream, errors: Vec::new_in(arena) }
    }

    #[must_use]
    pub fn parse_with_span(arena: &'arena A, content: &'arena [u8], span: Span) -> Document<'arena> {
        let mut parser = Self::new(arena, content, span);

        parser.parse_document(span)
    }

    #[must_use]
    pub fn parse(arena: &'arena A, file_id: FileId, content: &'arena [u8]) -> Document<'arena> {
        let span = Span::new(file_id, Position::new(0), Position::new(content.len() as u32));

        Self::parse_with_span(arena, content, span)
    }

    #[inline]
    pub(crate) fn file_id(&self) -> FileId {
        self.stream.file_id()
    }
}

/// Parses a standalone type expression (the PHPStan/Psalm type mini-language) from `content`.
///
/// # Errors
///
/// Returns a [`ParseError`] if the input is not a well-formed type.
pub fn parse_type<'arena, A>(arena: &'arena A, content: &'arena [u8], span: Span) -> Result<Type<'arena>, ParseError>
where
    A: Arena,
{
    let mut parser = PHPDocParser::new(arena, content, span);

    parser.parse_type()
}
