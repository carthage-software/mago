use crate::T;
use crate::ast::ast::*;
use crate::error::ParseError;
use crate::parser::internal::identifier::parse_local_identifier;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_magic_constant<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<MagicConstant<'arena>, ParseError> {
    let token = utils::peek(stream)?;

    Ok(match token.kind {
        T!["__CLASS__"] => MagicConstant::Class(parse_local_identifier(stream)?),
        T!["__DIR__"] => MagicConstant::Directory(parse_local_identifier(stream)?),
        T!["__FILE__"] => MagicConstant::File(parse_local_identifier(stream)?),
        T!["__FUNCTION__"] => MagicConstant::Function(parse_local_identifier(stream)?),
        T!["__LINE__"] => MagicConstant::Line(parse_local_identifier(stream)?),
        T!["__METHOD__"] => MagicConstant::Method(parse_local_identifier(stream)?),
        T!["__NAMESPACE__"] => MagicConstant::Namespace(parse_local_identifier(stream)?),
        T!["__TRAIT__"] => MagicConstant::Trait(parse_local_identifier(stream)?),
        _ => {
            return Err(utils::unexpected(
                stream,
                Some(token),
                T![
                    "__CLASS__",
                    "__DIR__",
                    "__FILE__",
                    "__FUNCTION__",
                    "__LINE__",
                    "__METHOD__",
                    "__NAMESPACE__",
                    "__TRAIT__"
                ],
            ));
        }
    })
}
