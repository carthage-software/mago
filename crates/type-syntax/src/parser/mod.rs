use bumpalo::Bump;

use mago_database::file::HasFileId;
use mago_span::Position;

use crate::ast::Type;
use crate::error::ParseError;
use crate::lexer::TypeLexer;
use crate::parser::internal::stream::TypeTokenStream;

mod internal;

/// Constructs a type AST from a lexer, allocating nodes in the given arena.
///
/// # Errors
///
/// Returns a [`ParseError`] if the type syntax is invalid.
pub fn construct<'arena>(arena: &'arena Bump, lexer: TypeLexer<'arena>) -> Result<Type<'arena>, ParseError> {
    let mut stream = TypeTokenStream::new(arena, lexer);

    let ty = internal::parse_type(&mut stream)?;

    if let Some(next) = stream.lookahead(0)? {
        return Err(ParseError::UnexpectedToken(vec![], next.kind, next.span_for(stream.file_id())));
    }

    Ok(ty)
}

/// Parse the longest type prefix and return the position past the
/// consumed bytes. Used by embedding callers that tokenise their own
/// trailing text after the type.
///
/// # Errors
///
/// Returns a [`ParseError`] if the input does not begin with a valid
/// type.
pub fn construct_prefix<'arena>(
    arena: &'arena Bump,
    lexer: TypeLexer<'arena>,
) -> Result<(Type<'arena>, Position), ParseError> {
    let mut stream = TypeTokenStream::new(arena, lexer);
    let ty = internal::parse_type(&mut stream)?;
    Ok((ty, stream.current_position()))
}
