use crate::T;
use crate::ast::ast::TraitUse;
use crate::ast::ast::TraitUseAbsoluteMethodReference;
use crate::ast::ast::TraitUseAbstractSpecification;
use crate::ast::ast::TraitUseAdaptation;
use crate::ast::ast::TraitUseAliasAdaptation;
use crate::ast::ast::TraitUseConcreteSpecification;
use crate::ast::ast::TraitUseMethodReference;
use crate::ast::ast::TraitUsePrecedenceAdaptation;
use crate::ast::ast::TraitUseSpecification;
use crate::ast::sequence::Sequence;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_trait_use(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<TraitUse<'arena>, ParseError> {
        Ok(TraitUse {
            r#use: self.expect_keyword(stream, T!["use"])?,
            trait_names: {
                let mut traits = self.new_vec();
                let mut commas = self.new_vec();
                loop {
                    let next = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;
                    if matches!(next.kind, T!["{" | ";" | "?>"]) {
                        break;
                    }

                    traits.push(self.parse_identifier(stream)?);

                    match stream.lookahead(0)?.map(|t| t.kind) {
                        Some(T![","]) => {
                            commas.push(stream.consume()?);
                        }
                        _ => {
                            break;
                        }
                    }
                }

                TokenSeparatedSequence::new(traits, commas)
            },
            specification: self.parse_trait_use_specification(stream)?,
        })
    }

    fn parse_trait_use_specification(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<TraitUseSpecification<'arena>, ParseError> {
        let next = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;
        Ok(match next.kind {
            T![";" | "?>"] => {
                TraitUseSpecification::Abstract(TraitUseAbstractSpecification(self.parse_terminator(stream)?))
            }
            _ => TraitUseSpecification::Concrete(TraitUseConcreteSpecification {
                left_brace: stream.eat(T!["{"])?.span,
                adaptations: {
                    let mut adaptations = self.new_vec();
                    loop {
                        if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T!["}"])) {
                            break;
                        }

                        adaptations.push(self.parse_trait_use_adaptation(stream)?);
                    }
                    Sequence::new(adaptations)
                },
                right_brace: stream.eat(T!["}"])?.span,
            }),
        })
    }

    fn parse_trait_use_adaptation(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<TraitUseAdaptation<'arena>, ParseError> {
        Ok(match self.parse_trait_use_method_reference(stream)? {
            TraitUseMethodReference::Absolute(reference) => {
                let next = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;
                match next.kind {
                    T!["as"] => TraitUseAdaptation::Alias(TraitUseAliasAdaptation {
                        method_reference: TraitUseMethodReference::Absolute(reference),
                        r#as: self.expect_keyword(stream, T!["as"])?,
                        visibility: self.parse_optional_read_visibility_modifier(stream)?,
                        alias: match stream.lookahead(0)?.map(|t| t.kind) {
                            Some(T![";" | "?>"]) => None,
                            _ => Some(self.parse_local_identifier(stream)?),
                        },
                        terminator: self.parse_terminator(stream)?,
                    }),
                    T!["insteadof"] => TraitUseAdaptation::Precedence(TraitUsePrecedenceAdaptation {
                        method_reference: reference,
                        insteadof: self.expect_any_keyword(stream)?,
                        trait_names: {
                            let mut items = self.new_vec();
                            let mut commas = self.new_vec();
                            loop {
                                if matches!(stream.lookahead(0)?.map(|t| t.kind), Some(T![";" | "?>"])) {
                                    break;
                                }

                                items.push(self.parse_identifier(stream)?);

                                match stream.lookahead(0)?.map(|t| t.kind) {
                                    Some(T![","]) => {
                                        commas.push(stream.consume()?);
                                    }
                                    _ => {
                                        break;
                                    }
                                }
                            }

                            TokenSeparatedSequence::new(items, commas)
                        },
                        terminator: self.parse_terminator(stream)?,
                    }),
                    _ => return Err(stream.unexpected(Some(next), T!["as", "insteadof"])),
                }
            }
            method_reference @ TraitUseMethodReference::Identifier(_) => {
                TraitUseAdaptation::Alias(TraitUseAliasAdaptation {
                    method_reference,
                    r#as: self.expect_keyword(stream, T!["as"])?,
                    visibility: self.parse_optional_read_visibility_modifier(stream)?,
                    alias: match stream.lookahead(0)?.map(|t| t.kind) {
                        Some(T![";" | "?>"]) => None,
                        _ => Some(self.parse_local_identifier(stream)?),
                    },
                    terminator: self.parse_terminator(stream)?,
                })
            }
        })
    }

    fn parse_trait_use_method_reference(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<TraitUseMethodReference<'arena>, ParseError> {
        Ok(match stream.lookahead(1)?.map(|t| t.kind) {
            Some(T!["::"]) => {
                TraitUseMethodReference::Absolute(self.parse_trait_use_absolute_method_reference(stream)?)
            }
            _ => TraitUseMethodReference::Identifier(self.parse_local_identifier(stream)?),
        })
    }

    fn parse_trait_use_absolute_method_reference(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<TraitUseAbsoluteMethodReference<'arena>, ParseError> {
        Ok(TraitUseAbsoluteMethodReference {
            trait_name: self.parse_identifier(stream)?,
            double_colon: stream.eat(T!["::"])?.span,
            method_name: self.parse_local_identifier(stream)?,
        })
    }
}
