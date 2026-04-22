use mago_database::file::HasFileId;

use crate::ast::CallableTypeParameter;
use crate::ast::CallableTypeParameters;
use crate::ast::CallableTypeReturnType;
use crate::ast::CallableTypeSpecification;
use crate::ast::VariableType;
use crate::error::ParseError;
use crate::parser::internal::parse_type;
use crate::parser::internal::parse_type_with_precedence;
use crate::parser::internal::stream::TypeTokenStream;
use crate::token::TypePrecedence;
use crate::token::TypeTokenKind;

#[inline]
pub fn parse_callable_type_specifications<'arena>(
    stream: &mut TypeTokenStream<'arena>,
) -> Result<CallableTypeSpecification<'arena>, ParseError> {
    Ok(CallableTypeSpecification {
        parameters: CallableTypeParameters {
            left_parenthesis: stream.eat(TypeTokenKind::LeftParenthesis)?.span_for(stream.file_id()),
            entries: {
                let mut entries = stream.new_bvec::<CallableTypeParameter<'arena>>();

                while !stream.is_at(TypeTokenKind::RightParenthesis)? {
                    let entry = CallableTypeParameter {
                        parameter_type: {
                            if stream.is_at(TypeTokenKind::Ellipsis)? { None } else { Some(parse_type(stream)?) }
                        },
                        equals: if stream.is_at(TypeTokenKind::Equals)? {
                            Some(stream.consume()?.span_for(stream.file_id()))
                        } else {
                            None
                        },
                        ellipsis: if stream.is_at(TypeTokenKind::Ellipsis)? {
                            Some(stream.consume()?.span_for(stream.file_id()))
                        } else {
                            None
                        },
                        variable: if stream.is_at(TypeTokenKind::Variable)? {
                            Some(VariableType::from_token(stream.consume()?, stream.file_id()))
                        } else {
                            None
                        },
                        comma: if stream.is_at(TypeTokenKind::Comma)? {
                            Some(stream.consume()?.span_for(stream.file_id()))
                        } else {
                            None
                        },
                    };

                    if entry.comma.is_none() {
                        entries.push(entry);
                        break;
                    }

                    entries.push(entry);
                }

                mago_syntax_core::ast::Sequence::new(entries)
            },
            right_parenthesis: stream.eat(TypeTokenKind::RightParenthesis)?.span_for(stream.file_id()),
        },
        return_type: if stream.is_at(TypeTokenKind::Colon)? {
            let colon = stream.consume()?.span_for(stream.file_id());
            let ret = parse_type_with_precedence(stream, TypePrecedence::Callable)?;
            Some(CallableTypeReturnType { colon, return_type: stream.alloc(ret) })
        } else {
            None
        },
    })
}

#[inline]
pub fn parse_optional_callable_type_specifications<'arena>(
    stream: &mut TypeTokenStream<'arena>,
) -> Result<Option<CallableTypeSpecification<'arena>>, ParseError> {
    if stream.is_at(TypeTokenKind::LeftParenthesis)? {
        let specifications = parse_callable_type_specifications(stream)?;
        Ok(Some(specifications))
    } else {
        Ok(None)
    }
}
