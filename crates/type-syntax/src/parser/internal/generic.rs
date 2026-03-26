use mago_database::file::HasFileId;

use crate::ast::GenericParameterEntry;
use crate::ast::GenericParameters;
use crate::ast::SingleGenericParameter;
use crate::error::ParseError;
use crate::parser::internal::parse_type;
use crate::parser::internal::stream::TypeTokenStream;
use crate::token::TypeTokenKind;

#[inline]
pub fn parse_single_generic_parameter<'input>(
    stream: &mut TypeTokenStream<'input>,
) -> Result<SingleGenericParameter<'input>, ParseError> {
    Ok(SingleGenericParameter {
        less_than: stream.eat(TypeTokenKind::LessThan)?.span_for(stream.file_id()),
        entry: Box::new(GenericParameterEntry { inner: parse_type(stream)?, comma: None }),
        greater_than: stream.eat(TypeTokenKind::GreaterThan)?.span_for(stream.file_id()),
    })
}

#[inline]
pub fn parse_generic_parameters<'input>(
    stream: &mut TypeTokenStream<'input>,
) -> Result<GenericParameters<'input>, ParseError> {
    let less_than = stream.eat(TypeTokenKind::LessThan)?.span_for(stream.file_id());
    let mut entries = Vec::new();

    loop {
        let entry = GenericParameterEntry {
            inner: parse_type(stream)?,
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
        if stream.is_at(TypeTokenKind::GreaterThan)? {
            break;
        }
    }

    let greater_than = stream.eat(TypeTokenKind::GreaterThan)?.span_for(stream.file_id());

    Ok(GenericParameters { less_than, entries, greater_than })
}

#[inline]
pub fn parse_single_generic_parameter_or_none<'input>(
    stream: &mut TypeTokenStream<'input>,
) -> Result<Option<SingleGenericParameter<'input>>, ParseError> {
    if stream.is_at(TypeTokenKind::LessThan)? {
        let single_generic_parameter = parse_single_generic_parameter(stream)?;
        Ok(Some(single_generic_parameter))
    } else {
        Ok(None)
    }
}

#[inline]
pub fn parse_generic_parameters_or_none<'input>(
    stream: &mut TypeTokenStream<'input>,
) -> Result<Option<GenericParameters<'input>>, ParseError> {
    if stream.is_at(TypeTokenKind::LessThan)? {
        let generic_parameters = parse_generic_parameters(stream)?;
        Ok(Some(generic_parameters))
    } else {
        Ok(None)
    }
}
