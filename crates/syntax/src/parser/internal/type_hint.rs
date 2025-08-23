use crate::T;
use crate::ast::ast::*;
use crate::error::ParseError;
use crate::parser::internal::identifier;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn is_at_type_hint<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<bool, ParseError> {
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

pub fn parse_optional_type_hint<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<Option<Hint<'arena>>, ParseError> {
    if is_at_type_hint(stream)? { Ok(Some(parse_type_hint(stream)?)) } else { Ok(None) }
}

pub fn parse_type_hint<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<Hint<'arena>, ParseError> {
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
        T![Identifier] => match token.value {
            val if val.eq_ignore_ascii_case("void") => Hint::Void(identifier::parse_local_identifier(stream)?),
            val if val.eq_ignore_ascii_case("never") => Hint::Never(identifier::parse_local_identifier(stream)?),
            val if val.eq_ignore_ascii_case("float") => Hint::Float(identifier::parse_local_identifier(stream)?),
            val if val.eq_ignore_ascii_case("bool") => Hint::Bool(identifier::parse_local_identifier(stream)?),
            val if val.eq_ignore_ascii_case("int") => Hint::Integer(identifier::parse_local_identifier(stream)?),
            val if val.eq_ignore_ascii_case("string") => Hint::String(identifier::parse_local_identifier(stream)?),
            val if val.eq_ignore_ascii_case("object") => Hint::Object(identifier::parse_local_identifier(stream)?),
            val if val.eq_ignore_ascii_case("mixed") => Hint::Mixed(identifier::parse_local_identifier(stream)?),
            val if val.eq_ignore_ascii_case("iterable") => Hint::Iterable(identifier::parse_local_identifier(stream)?),
            _ => Hint::Identifier(identifier::parse_identifier(stream)?),
        },
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

            Hint::Union(UnionHint { left: stream.alloc(left), pipe, right: stream.alloc(right) })
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

            Hint::Intersection(IntersectionHint { left: stream.alloc(left), ampersand, right: stream.alloc(right) })
        }
        _ => hint,
    })
}

pub fn parse_nullable_type_hint<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<NullableHint<'arena>, ParseError> {
    let question_mark = utils::expect(stream, T!["?"])?.span;
    let hint = parse_type_hint(stream)?;

    Ok(NullableHint { question_mark, hint: stream.alloc(hint) })
}

pub fn parse_parenthesized_type_hint<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<ParenthesizedHint<'arena>, ParseError> {
    let left_parenthesis = utils::expect(stream, T!["("])?.span;
    let hint = parse_type_hint(stream)?;
    let right_parenthesis = utils::expect(stream, T![")"])?.span;

    Ok(ParenthesizedHint { left_parenthesis, hint: stream.alloc(hint), right_parenthesis })
}
