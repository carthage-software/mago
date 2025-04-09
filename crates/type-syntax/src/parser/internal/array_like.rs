use crate::ast::*;
use crate::error::ParseError;
use crate::parser::internal::generic::parse_optional_generic_parameters;
use crate::parser::internal::parse_type;
use crate::parser::internal::stream::TypeTokenStream;
use crate::token::TypeTokenKind;

#[inline]
pub fn parse_array_like_type<'input>(stream: &mut TypeTokenStream<'input>) -> Result<Type<'input>, ParseError> {
    let next = stream.peek()?;
    let (keyword, kind) = match next.kind {
        TypeTokenKind::Array => {
            let keyword = Keyword::from(stream.consume()?);
            if !stream.is_at(TypeTokenKind::LeftBrace)? {
                return Ok(Type::Array(ArrayType { keyword, parameters: parse_optional_generic_parameters(stream)? }));
            }

            (keyword, ShapeTypeKind::Array)
        }
        TypeTokenKind::NonEmptyArray => {
            let keyword = Keyword::from(stream.consume()?);
            if !stream.is_at(TypeTokenKind::LeftBrace)? {
                return Ok(Type::NonEmptyArray(NonEmptyArrayType {
                    keyword,
                    parameters: parse_optional_generic_parameters(stream)?,
                }));
            }

            (keyword, ShapeTypeKind::NonEmptyArray)
        }
        TypeTokenKind::AssociativeArray => {
            let keyword = Keyword::from(stream.consume()?);
            if !stream.is_at(TypeTokenKind::LeftBrace)? {
                return Ok(Type::AssociativeArray(AssociativeArrayType {
                    keyword,
                    parameters: parse_optional_generic_parameters(stream)?,
                }));
            }

            (keyword, ShapeTypeKind::AssociativeArray)
        }
        TypeTokenKind::List => {
            let keyword = Keyword::from(stream.consume()?);
            if !stream.is_at(TypeTokenKind::LeftBrace)? {
                return Ok(Type::List(ListType { keyword, parameters: parse_optional_generic_parameters(stream)? }));
            }

            (keyword, ShapeTypeKind::List)
        }
        TypeTokenKind::NonEmptyList => {
            let keyword = Keyword::from(stream.consume()?);
            if !stream.is_at(TypeTokenKind::LeftBrace)? {
                return Ok(Type::NonEmptyList(NonEmptyListType {
                    keyword,
                    parameters: parse_optional_generic_parameters(stream)?,
                }));
            }

            (keyword, ShapeTypeKind::NonEmptyList)
        }
        _ => {
            return Err(ParseError::UnexpectedToken(
                vec![
                    TypeTokenKind::Array,
                    TypeTokenKind::NonEmptyArray,
                    TypeTokenKind::AssociativeArray,
                    TypeTokenKind::List,
                    TypeTokenKind::NonEmptyList,
                ],
                next.kind,
                next.span,
            ));
        }
    };

    Ok(Type::Shape(ShapeType {
        kind,
        keyword,
        left_brace: stream.eat(TypeTokenKind::LeftBrace)?.span,
        fields: {
            let mut fields = Vec::new();
            while !stream.is_at(TypeTokenKind::RightBrace)? && !stream.is_at(TypeTokenKind::Ellipsis)? {
                let field = ShapeField {
                    key: Box::new(parse_type(stream)?),
                    question_mark: if stream.is_at(TypeTokenKind::Question)? {
                        Some(stream.consume()?.span)
                    } else {
                        None
                    },
                    colon: stream.eat(TypeTokenKind::Colon)?.span,
                    value: Box::new(parse_type(stream)?),
                    comma: if stream.is_at(TypeTokenKind::Comma)? { Some(stream.consume()?.span) } else { None },
                };

                if field.comma.is_none() {
                    fields.push(field);
                    break;
                }

                fields.push(field);
            }

            fields
        },
        additional_fields: {
            if !stream.is_at(TypeTokenKind::Ellipsis)? {
                None
            } else {
                Some(ShapeAdditionalFields {
                    ellipsis: stream.consume()?.span,
                    parameters: parse_optional_generic_parameters(stream)?,
                })
            }
        },
        right_brace: stream.eat(TypeTokenKind::RightBrace)?.span,
    }))
}
