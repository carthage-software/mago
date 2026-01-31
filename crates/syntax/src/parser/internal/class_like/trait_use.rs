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

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_trait_use(&mut self) -> Result<TraitUse<'arena>, ParseError> {
        Ok(TraitUse {
            r#use: self.expect_keyword(T!["use"])?,
            trait_names: {
                let mut traits = self.new_vec();
                let mut commas = self.new_vec();
                loop {
                    let next = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;
                    if matches!(next.kind, T!["{" | ";" | "?>"]) {
                        break;
                    }

                    traits.push(self.parse_identifier()?);

                    match self.stream.lookahead(0)?.map(|t| t.kind) {
                        Some(T![","]) => {
                            commas.push(self.stream.consume()?);
                        }
                        _ => {
                            break;
                        }
                    }
                }

                TokenSeparatedSequence::new(traits, commas)
            },
            specification: self.parse_trait_use_specification()?,
        })
    }

    fn parse_trait_use_specification(&mut self) -> Result<TraitUseSpecification<'arena>, ParseError> {
        let next = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;
        Ok(match next.kind {
            T![";" | "?>"] => TraitUseSpecification::Abstract(TraitUseAbstractSpecification(self.parse_terminator()?)),
            _ => TraitUseSpecification::Concrete(TraitUseConcreteSpecification {
                left_brace: self.stream.eat(T!["{"])?.span,
                adaptations: {
                    let mut adaptations = self.new_vec();
                    loop {
                        if matches!(self.stream.lookahead(0)?.map(|t| t.kind), Some(T!["}"])) {
                            break;
                        }

                        adaptations.push(self.parse_trait_use_adaptation()?);
                    }
                    Sequence::new(adaptations)
                },
                right_brace: self.stream.eat(T!["}"])?.span,
            }),
        })
    }

    fn parse_trait_use_adaptation(&mut self) -> Result<TraitUseAdaptation<'arena>, ParseError> {
        Ok(match self.parse_trait_use_method_reference()? {
            TraitUseMethodReference::Absolute(reference) => {
                let next = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;
                match next.kind {
                    T!["as"] => TraitUseAdaptation::Alias(TraitUseAliasAdaptation {
                        method_reference: TraitUseMethodReference::Absolute(reference),
                        r#as: self.expect_keyword(T!["as"])?,
                        visibility: self.parse_optional_read_visibility_modifier()?,
                        alias: match self.stream.lookahead(0)?.map(|t| t.kind) {
                            Some(T![";" | "?>"]) => None,
                            _ => Some(self.parse_local_identifier()?),
                        },
                        terminator: self.parse_terminator()?,
                    }),
                    T!["insteadof"] => TraitUseAdaptation::Precedence(TraitUsePrecedenceAdaptation {
                        method_reference: reference,
                        insteadof: self.expect_any_keyword()?,
                        trait_names: {
                            let mut items = self.new_vec();
                            let mut commas = self.new_vec();
                            loop {
                                if matches!(self.stream.lookahead(0)?.map(|t| t.kind), Some(T![";" | "?>"])) {
                                    break;
                                }

                                items.push(self.parse_identifier()?);

                                match self.stream.lookahead(0)?.map(|t| t.kind) {
                                    Some(T![","]) => {
                                        commas.push(self.stream.consume()?);
                                    }
                                    _ => {
                                        break;
                                    }
                                }
                            }

                            TokenSeparatedSequence::new(items, commas)
                        },
                        terminator: self.parse_terminator()?,
                    }),
                    _ => return Err(self.stream.unexpected(Some(next), T!["as", "insteadof"])),
                }
            }
            method_reference @ TraitUseMethodReference::Identifier(_) => {
                TraitUseAdaptation::Alias(TraitUseAliasAdaptation {
                    method_reference,
                    r#as: self.expect_keyword(T!["as"])?,
                    visibility: self.parse_optional_read_visibility_modifier()?,
                    alias: match self.stream.lookahead(0)?.map(|t| t.kind) {
                        Some(T![";" | "?>"]) => None,
                        _ => Some(self.parse_local_identifier()?),
                    },
                    terminator: self.parse_terminator()?,
                })
            }
        })
    }

    fn parse_trait_use_method_reference(&mut self) -> Result<TraitUseMethodReference<'arena>, ParseError> {
        Ok(match self.stream.lookahead(1)?.map(|t| t.kind) {
            Some(T!["::"]) => TraitUseMethodReference::Absolute(self.parse_trait_use_absolute_method_reference()?),
            _ => TraitUseMethodReference::Identifier(self.parse_local_identifier()?),
        })
    }

    fn parse_trait_use_absolute_method_reference(
        &mut self,
    ) -> Result<TraitUseAbsoluteMethodReference<'arena>, ParseError> {
        Ok(TraitUseAbsoluteMethodReference {
            trait_name: self.parse_identifier()?,
            double_colon: self.stream.eat(T!["::"])?.span,
            method_name: self.parse_local_identifier()?,
        })
    }
}
