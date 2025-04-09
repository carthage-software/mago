use crate::ast::Type;
use crate::error::ParseError;
use crate::lexer::TypeLexer;
use crate::parser::internal::stream::TypeTokenStream;

mod internal;

pub fn construct(lexer: TypeLexer<'_>) -> Result<Type<'_>, ParseError> {
    let mut stream = TypeTokenStream::new(lexer);

    let ty = internal::parse_type(&mut stream)?;

    if let Some(next) = stream.lookahead(0)? {
        return Err(ParseError::UnexpectedToken(vec![], next.kind, next.span));
    }

    Ok(ty)
}
