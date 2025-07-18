use crate::ast::*;
use crate::error::ParseError;
use crate::parser::internal::parse_type;
use crate::parser::internal::stream::TypeTokenStream;
use crate::token::TypeTokenKind;

#[inline]
pub fn parse_callable_type_specifications<'input>(
    stream: &mut TypeTokenStream<'input>,
) -> Result<CallableTypeSpecification<'input>, ParseError> {
    Ok(CallableTypeSpecification {
        parameters: CallableTypeParameters {
            left_parenthesis: stream.eat(TypeTokenKind::LeftParenthesis)?.span,
            entries: {
                let mut entries = Vec::new();

                while !stream.is_at(TypeTokenKind::RightParenthesis)? {
                    let entry = CallableTypeParameter {
                        parameter_type: {
                            if stream.is_at(TypeTokenKind::Ellipsis)? { None } else { Some(parse_type(stream)?) }
                        },
                        equals: if stream.is_at(TypeTokenKind::Equals)? { Some(stream.consume()?.span) } else { None },
                        ellipsis: if stream.is_at(TypeTokenKind::Ellipsis)? {
                            Some(stream.consume()?.span)
                        } else {
                            None
                        },
                        variable: if stream.is_at(TypeTokenKind::Variable)? {
                            Some(VariableType::from(stream.consume()?))
                        } else {
                            None
                        },
                        comma: if stream.is_at(TypeTokenKind::Comma)? { Some(stream.consume()?.span) } else { None },
                    };

                    if entry.comma.is_none() {
                        entries.push(entry);
                        break;
                    }

                    entries.push(entry);
                }

                entries
            },
            right_parenthesis: stream.eat(TypeTokenKind::RightParenthesis)?.span,
        },
        return_type: if stream.is_at(TypeTokenKind::Colon)? {
            Some(CallableTypeReturnType { colon: stream.consume()?.span, return_type: Box::new(parse_type(stream)?) })
        } else {
            None
        },
    })
}

#[inline]
pub fn parse_optional_callable_type_specifications<'input>(
    stream: &mut TypeTokenStream<'input>,
) -> Result<Option<CallableTypeSpecification<'input>>, ParseError> {
    if stream.is_at(TypeTokenKind::LeftParenthesis)? {
        let specifications = parse_callable_type_specifications(stream)?;
        Ok(Some(specifications))
    } else {
        Ok(None)
    }
}
