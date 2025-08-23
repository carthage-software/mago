use crate::T;
use crate::ast::ast::*;
use crate::ast::sequence::Sequence;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::internal::identifier::parse_identifier;
use crate::parser::internal::identifier::parse_local_identifier;
use crate::parser::internal::modifier::parse_optional_read_visibility_modifier;
use crate::parser::internal::terminator::parse_terminator;
use crate::parser::internal::token_stream::TokenStream;
use crate::parser::internal::utils;

pub fn parse_trait_use<'arena>(stream: &mut TokenStream<'_, 'arena>) -> Result<TraitUse<'arena>, ParseError> {
    Ok(TraitUse {
        r#use: utils::expect_keyword(stream, T!["use"])?,
        trait_names: {
            let mut traits = stream.new_vec();
            let mut commas = stream.new_vec();
            loop {
                let next = utils::peek(stream)?;
                if matches!(next.kind, T!["{" | ";" | "?>"]) {
                    break;
                }

                traits.push(parse_identifier(stream)?);

                match utils::peek(stream)?.kind {
                    T![","] => {
                        commas.push(utils::expect_any(stream)?);
                    }
                    _ => {
                        break;
                    }
                }
            }

            TokenSeparatedSequence::new(traits, commas)
        },
        specification: parse_trait_use_specification(stream)?,
    })
}

pub fn parse_trait_use_specification<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<TraitUseSpecification<'arena>, ParseError> {
    let next = utils::peek(stream)?;
    Ok(match next.kind {
        T![";" | "?>"] => TraitUseSpecification::Abstract(TraitUseAbstractSpecification(parse_terminator(stream)?)),
        _ => TraitUseSpecification::Concrete(TraitUseConcreteSpecification {
            left_brace: utils::expect_span(stream, T!["{"])?,
            adaptations: {
                let mut adaptations = stream.new_vec();
                loop {
                    let next = utils::peek(stream)?;
                    if next.kind == T!["}"] {
                        break;
                    }

                    adaptations.push(parse_trait_use_adaptation(stream)?);
                }
                Sequence::new(adaptations)
            },
            right_brace: utils::expect_span(stream, T!["}"])?,
        }),
    })
}

pub fn parse_trait_use_adaptation<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<TraitUseAdaptation<'arena>, ParseError> {
    Ok(match parse_trait_use_method_reference(stream)? {
        TraitUseMethodReference::Absolute(reference) => {
            let next = utils::peek(stream)?;
            match next.kind {
                T!["as"] => TraitUseAdaptation::Alias(TraitUseAliasAdaptation {
                    method_reference: TraitUseMethodReference::Absolute(reference),
                    r#as: utils::expect_keyword(stream, T!["as"])?,
                    visibility: parse_optional_read_visibility_modifier(stream)?,
                    alias: match utils::maybe_peek(stream)?.map(|t| t.kind) {
                        Some(T![";" | "?>"]) => None,
                        _ => Some(parse_local_identifier(stream)?),
                    },
                    terminator: parse_terminator(stream)?,
                }),
                T!["insteadof"] => TraitUseAdaptation::Precedence(TraitUsePrecedenceAdaptation {
                    method_reference: reference,
                    insteadof: utils::expect_any_keyword(stream)?,
                    trait_names: {
                        let mut items = stream.new_vec();
                        let mut commas = stream.new_vec();
                        loop {
                            if matches!(utils::peek(stream)?.kind, T![";" | "?>"]) {
                                break;
                            }

                            items.push(parse_identifier(stream)?);

                            match utils::peek(stream)?.kind {
                                T![","] => {
                                    commas.push(utils::expect_any(stream)?);
                                }
                                _ => {
                                    break;
                                }
                            }
                        }

                        TokenSeparatedSequence::new(items, commas)
                    },
                    terminator: parse_terminator(stream)?,
                }),
                _ => return Err(utils::unexpected(stream, Some(next), T!["as", "insteadof"])),
            }
        }
        method_reference @ TraitUseMethodReference::Identifier(_) => {
            TraitUseAdaptation::Alias(TraitUseAliasAdaptation {
                method_reference,
                r#as: utils::expect_keyword(stream, T!["as"])?,
                visibility: parse_optional_read_visibility_modifier(stream)?,
                alias: match utils::maybe_peek(stream)?.map(|t| t.kind) {
                    Some(T![";" | "?>"]) => None,
                    _ => Some(parse_local_identifier(stream)?),
                },
                terminator: parse_terminator(stream)?,
            })
        }
    })
}

pub fn parse_trait_use_method_reference<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<TraitUseMethodReference<'arena>, ParseError> {
    Ok(match utils::maybe_peek_nth(stream, 1)?.map(|t| t.kind) {
        Some(T!["::"]) => TraitUseMethodReference::Absolute(parse_trait_use_absolute_method_reference(stream)?),
        _ => TraitUseMethodReference::Identifier(parse_local_identifier(stream)?),
    })
}

pub fn parse_trait_use_absolute_method_reference<'arena>(
    stream: &mut TokenStream<'_, 'arena>,
) -> Result<TraitUseAbsoluteMethodReference<'arena>, ParseError> {
    Ok(TraitUseAbsoluteMethodReference {
        trait_name: parse_identifier(stream)?,
        double_colon: utils::expect_span(stream, T!["::"])?,
        method_name: parse_local_identifier(stream)?,
    })
}
