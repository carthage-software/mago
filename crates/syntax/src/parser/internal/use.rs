use mago_database::file::HasFileId;

use crate::T;
use crate::ast::ast::MaybeTypedUseItem;
use crate::ast::ast::MixedUseItemList;
use crate::ast::ast::TypedUseItemList;
use crate::ast::ast::TypedUseItemSequence;
use crate::ast::ast::Use;
use crate::ast::ast::UseItem;
use crate::ast::ast::UseItemAlias;
use crate::ast::ast::UseItemSequence;
use crate::ast::ast::UseItems;
use crate::ast::ast::UseType;
use crate::ast::sequence::TokenSeparatedSequence;
use crate::error::ParseError;
use crate::parser::Parser;

impl<'input, 'arena> Parser<'input, 'arena> {
    pub(crate) fn parse_use(&mut self) -> Result<Use<'arena>, ParseError> {
        Ok(Use {
            r#use: self.expect_keyword(T!["use"])?,
            items: self.parse_use_items()?,
            terminator: self.parse_terminator()?,
        })
    }

    pub(crate) fn parse_use_items(&mut self) -> Result<UseItems<'arena>, ParseError> {
        let next = self.stream.peek_kind(0)?;

        Ok(match next {
            Some(T!["const" | "function"]) => match self.stream.lookahead(2)?.map(|t| t.kind) {
                Some(T!["\\"]) => UseItems::TypedList(self.parse_typed_use_item_list()?),
                _ => UseItems::TypedSequence(self.parse_typed_use_item_sequence()?),
            },
            _ => match self.stream.peek_kind(1)? {
                Some(T!["\\"]) => UseItems::MixedList(self.parse_mixed_use_item_list()?),
                _ => UseItems::Sequence(self.parse_use_item_sequence()?),
            },
        })
    }

    pub(crate) fn parse_use_item_sequence(&mut self) -> Result<UseItemSequence<'arena>, ParseError> {
        let start = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?.start;

        let mut items = self.new_vec();
        let mut commas = self.new_vec();
        loop {
            items.push(self.parse_use_item()?);

            if let Some(T![","]) = self.stream.peek_kind(0)? {
                commas.push(self.stream.consume()?);
            } else {
                break;
            }
        }

        Ok(UseItemSequence { file_id: self.stream.file_id(), start, items: TokenSeparatedSequence::new(items, commas) })
    }

    pub(crate) fn parse_typed_use_item_sequence(&mut self) -> Result<TypedUseItemSequence<'arena>, ParseError> {
        let r#type = self.parse_use_type()?;
        let mut items = self.new_vec();
        let mut commas = self.new_vec();
        loop {
            items.push(self.parse_use_item()?);

            if let Some(T![","]) = self.stream.peek_kind(0)? {
                commas.push(self.stream.consume()?);
            } else {
                break;
            }
        }

        Ok(TypedUseItemSequence { r#type, items: TokenSeparatedSequence::new(items, commas) })
    }

    pub(crate) fn parse_typed_use_item_list(&mut self) -> Result<TypedUseItemList<'arena>, ParseError> {
        let r#type = self.parse_use_type()?;
        let namespace = self.parse_identifier()?;
        let namespace_separator = self.stream.eat_span(T!["\\"])?;
        let left_brace = self.stream.eat_span(T!["{"])?;
        let mut items = self.new_vec();
        let mut commas = self.new_vec();
        loop {
            if let Some(T!["}"]) = self.stream.peek_kind(0)? {
                break;
            }

            items.push(self.parse_use_item()?);

            if let Some(T![","]) = self.stream.peek_kind(0)? {
                commas.push(self.stream.consume()?);
            } else {
                break;
            }
        }
        let right_brace = self.stream.eat_span(T!["}"])?;

        Ok(TypedUseItemList {
            r#type,
            namespace,
            namespace_separator,
            left_brace,
            items: TokenSeparatedSequence::new(items, commas),
            right_brace,
        })
    }

    pub(crate) fn parse_mixed_use_item_list(&mut self) -> Result<MixedUseItemList<'arena>, ParseError> {
        let namespace = self.parse_identifier()?;
        let namespace_separator = self.stream.eat_span(T!["\\"])?;
        let left_brace = self.stream.eat_span(T!["{"])?;
        let mut items = self.new_vec();
        let mut commas = self.new_vec();
        loop {
            if let Some(T!["}"]) = self.stream.peek_kind(0)? {
                break;
            }

            items.push(self.parse_maybe_typed_use_item()?);

            if let Some(T![","]) = self.stream.peek_kind(0)? {
                commas.push(self.stream.consume()?);
            } else {
                break;
            }
        }
        let right_brace = self.stream.eat_span(T!["}"])?;

        Ok(MixedUseItemList {
            namespace,
            namespace_separator,
            left_brace,
            items: TokenSeparatedSequence::new(items, commas),
            right_brace,
        })
    }

    pub(crate) fn parse_maybe_typed_use_item(&mut self) -> Result<MaybeTypedUseItem<'arena>, ParseError> {
        Ok(MaybeTypedUseItem { r#type: self.parse_optional_use_type()?, item: self.parse_use_item()? })
    }

    pub(crate) fn parse_optional_use_type(&mut self) -> Result<Option<UseType<'arena>>, ParseError> {
        Ok(match self.stream.peek_kind(0)? {
            Some(T!["function"]) => Some(UseType::Function(self.expect_any_keyword()?)),
            Some(T!["const"]) => Some(UseType::Const(self.expect_any_keyword()?)),
            _ => None,
        })
    }

    pub(crate) fn parse_use_type(&mut self) -> Result<UseType<'arena>, ParseError> {
        let next = self.stream.lookahead(0)?.ok_or_else(|| self.stream.unexpected(None, &[]))?;

        Ok(match next.kind {
            T!["function"] => UseType::Function(self.expect_any_keyword()?),
            T!["const"] => UseType::Const(self.expect_any_keyword()?),
            _ => return Err(self.stream.unexpected(Some(next), T!["function", "const"])),
        })
    }

    pub(crate) fn parse_use_item(&mut self) -> Result<UseItem<'arena>, ParseError> {
        Ok(UseItem { name: self.parse_identifier()?, alias: self.parse_optional_use_item_alias()? })
    }

    pub(crate) fn parse_optional_use_item_alias(&mut self) -> Result<Option<UseItemAlias<'arena>>, ParseError> {
        Ok(match self.stream.peek_kind(0)? {
            Some(T!["as"]) => Some(self.parse_use_item_alias()?),
            _ => None,
        })
    }

    pub(crate) fn parse_use_item_alias(&mut self) -> Result<UseItemAlias<'arena>, ParseError> {
        let r#as = self.expect_keyword(T!["as"])?;
        let id = self.parse_local_identifier()?;

        Ok(UseItemAlias { r#as, identifier: id })
    }
}
