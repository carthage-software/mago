use mago_ast::ast::*;
use mago_token::T;

use crate::error::ParseError;
use crate::internal::identifier;
use crate::internal::token_stream::TokenStream;
use crate::internal::utils;

#[inline]
pub fn is_at_type_hint(stream: &mut TokenStream<'_, '_>) -> Result<bool, ParseError> {
    Ok(matches!(
        utils::peek(stream)?.kind,
        T!["?"
            | "("
            | "array"
            | "callable"
            | "null"
            | "true"
            | "false"
            | "static"
            | "self"
            | "parent"
            | "enum"
            | "from"
            | Identifier
            | QualifiedIdentifier
            | FullyQualifiedIdentifier]
    ))
}

#[inline]
pub fn parse_optional_type_hint<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<Option<Hint<'i>>, ParseError> {
    if is_at_type_hint(stream)? {
        Ok(Some(parse_type_hint(stream)?))
    } else {
        Ok(None)
    }
}

#[inline]
pub fn parse_type_hint<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<Hint<'i>, ParseError> {
    let token = utils::peek(stream)?;

    let hint = match &token.kind {
        T!["?"] => Hint::Nullable(parse_nullable_type_hint(stream)?),
        T!["("] => Hint::Parenthesized(parse_parenthesized_type_hint(stream)?),
        T!["array"] => Hint::Array(utils::expect_any_keyword(stream)?),
        T!["callable"] => Hint::Callable(utils::expect_any_keyword(stream)?),
        T!["null"] => Hint::Null(utils::expect_any_keyword(stream)?),
        T!["true"] => Hint::True(utils::expect_any_keyword(stream)?),
        T!["false"] => Hint::False(utils::expect_any_keyword(stream)?),
        T!["static"] => Hint::Static(utils::expect_any_keyword(stream)?),
        T!["self"] => Hint::Self_(utils::expect_any_keyword(stream)?),
        T!["parent"] => Hint::Parent(utils::expect_any_keyword(stream)?),
        T!["enum" | "from" | QualifiedIdentifier | FullyQualifiedIdentifier] => {
            Hint::Identifier(identifier::parse_identifier(stream)?)
        }
        T![Identifier] => {
            let value = stream.interner().lookup(&token.value);

            match value.to_ascii_lowercase().as_str() {
                "void" => Hint::Void(identifier::parse_local_identifier(stream)?),
                "never" => Hint::Never(identifier::parse_local_identifier(stream)?),
                "float" => Hint::Float(identifier::parse_local_identifier(stream)?),
                "bool" => Hint::Bool(identifier::parse_local_identifier(stream)?),
                "int" => Hint::Integer(identifier::parse_local_identifier(stream)?),
                "string" => Hint::String(identifier::parse_local_identifier(stream)?),
                "object" => Hint::Object(identifier::parse_local_identifier(stream)?),
                "mixed" => Hint::Mixed(identifier::parse_local_identifier(stream)?),
                "iterable" => Hint::Iterable(identifier::parse_local_identifier(stream)?),
                _ => Hint::Identifier(identifier::parse_identifier(stream)?),
            }
        }
        _ => {
            return Err(utils::unexpected(
                stream,
                Some(token),
                T![
                    "?",
                    "(",
                    "array",
                    "callable",
                    "null",
                    "true",
                    "false",
                    "static",
                    "self",
                    "parent",
                    "enum",
                    "from",
                    Identifier,
                    QualifiedIdentifier,
                    FullyQualifiedIdentifier,
                ],
            ));
        }
    };

    Ok(match utils::peek(stream)?.kind {
        T!["|"] => {
            let left = hint;
            let pipe = utils::expect(stream, T!["|"])?.span;
            let right = parse_type_hint(stream)?;

            Hint::Union(UnionHint { left: stream.boxed(left), pipe, right: stream.boxed(right) })
        }
        T!["&"]
            if !matches!(
                utils::maybe_peek_nth(stream, 1)?.map(|t| t.kind),
                Some(T!["$variable"] | T!["..."] | T!["&"])
            ) =>
        {
            let left = hint;
            let ampersand = utils::expect(stream, T!["&"])?.span;
            let right = parse_type_hint(stream)?;

            Hint::Intersection(IntersectionHint { left: stream.boxed(left), ampersand, right: stream.boxed(right) })
        }
        _ => hint,
    })
}

#[inline]
pub fn parse_nullable_type_hint<'i>(stream: &mut TokenStream<'_, 'i>) -> Result<NullableHint<'i>, ParseError> {
    let question_mark = utils::expect(stream, T!["?"])?.span;
    let hint = parse_type_hint(stream)?;

    Ok(NullableHint { question_mark, hint: stream.boxed(hint) })
}

#[inline]
pub fn parse_parenthesized_type_hint<'i>(
    stream: &mut TokenStream<'_, 'i>,
) -> Result<ParenthesizedHint<'i>, ParseError> {
    let left_parenthesis = utils::expect(stream, T!["("])?.span;
    let hint = parse_type_hint(stream)?;
    let right_parenthesis = utils::expect(stream, T![")"])?.span;

    Ok(ParenthesizedHint { left_parenthesis, hint: stream.boxed(hint), right_parenthesis })
}
