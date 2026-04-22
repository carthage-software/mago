use bumpalo::Bump;

use mago_database::file::HasFileId;

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
