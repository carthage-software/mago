use crate::ast::*;
use crate::error::ParseError;
use crate::parser::internal::parse_type;
use crate::parser::internal::stream::TypeTokenStream;
use crate::token::TypeTokenKind;

#[inline]
pub fn parse_generic_parameters<'input>(
    stream: &mut TypeTokenStream<'input>,
) -> Result<GenericParameters<'input>, ParseError> {
    let less_than = stream.eat(TypeTokenKind::LessThan)?.span;
    let mut entries = Vec::new();

    loop {
        let entry = GenericParameterEntry {
            inner: parse_type(stream)?,
            comma: if stream.is_at(TypeTokenKind::Comma)? { Some(stream.consume()?.span) } else { None },
        };

        if entry.comma.is_none() {
            entries.push(entry);
            break;
        }

        entries.push(entry);
        if stream.is_at(TypeTokenKind::GreaterThan)? {
            break;
        }
    }

    let greater_than = stream.eat(TypeTokenKind::GreaterThan)?.span;

    Ok(GenericParameters { less_than, entries, greater_than })
}

#[inline]
pub fn parse_optional_generic_parameters<'input>(
    stream: &mut TypeTokenStream<'input>,
) -> Result<Option<GenericParameters<'input>>, ParseError> {
    if stream.is_at(TypeTokenKind::LessThan)? {
        let generic_parameters = parse_generic_parameters(stream)?;
        Ok(Some(generic_parameters))
    } else {
        Ok(None)
    }
}
