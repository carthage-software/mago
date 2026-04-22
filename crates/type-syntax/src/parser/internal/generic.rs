use crate::ast::GenericParameterEntry;
use crate::ast::GenericParameters;
use crate::ast::SingleGenericParameter;
use crate::error::ParseError;
use crate::parser::internal::parse_type;
use crate::parser::internal::stream::TypeTokenStream;
use crate::token::TypeTokenKind;

#[inline]
pub fn parse_single_generic_parameter<'arena>(
    stream: &mut TypeTokenStream<'arena>,
) -> Result<SingleGenericParameter<'arena>, ParseError> {
    let less_than = stream.eat_span(TypeTokenKind::LessThan)?;
    let inner = parse_type(stream)?;
    let entry = stream.alloc(GenericParameterEntry { inner, comma: None });
    let greater_than = stream.eat_span(TypeTokenKind::GreaterThan)?;
    Ok(SingleGenericParameter { less_than, entry, greater_than })
}

#[inline]
pub fn parse_generic_parameters<'arena>(
    stream: &mut TypeTokenStream<'arena>,
) -> Result<GenericParameters<'arena>, ParseError> {
    let less_than = stream.eat_span(TypeTokenKind::LessThan)?;
    let mut entries = stream.new_bvec::<GenericParameterEntry<'arena>>();

    loop {
        let entry = GenericParameterEntry {
            inner: parse_type(stream)?,
            comma: if stream.is_at(TypeTokenKind::Comma)? { Some(stream.consume_span()?) } else { None },
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

    let greater_than = stream.eat_span(TypeTokenKind::GreaterThan)?;

    Ok(GenericParameters { less_than, entries: mago_syntax_core::ast::Sequence::new(entries), greater_than })
}

#[inline]
pub fn parse_single_generic_parameter_or_none<'arena>(
    stream: &mut TypeTokenStream<'arena>,
) -> Result<Option<SingleGenericParameter<'arena>>, ParseError> {
    if stream.is_at(TypeTokenKind::LessThan)? {
        let single_generic_parameter = parse_single_generic_parameter(stream)?;
        Ok(Some(single_generic_parameter))
    } else {
        Ok(None)
    }
}

#[inline]
pub fn parse_generic_parameters_or_none<'arena>(
    stream: &mut TypeTokenStream<'arena>,
) -> Result<Option<GenericParameters<'arena>>, ParseError> {
    if stream.is_at(TypeTokenKind::LessThan)? {
        let generic_parameters = parse_generic_parameters(stream)?;
        Ok(Some(generic_parameters))
    } else {
        Ok(None)
    }
}
