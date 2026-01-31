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
use crate::parser::stream::TokenStream;

impl<'arena> Parser<'arena> {
    pub(crate) fn parse_use(&mut self, stream: &mut TokenStream<'_, 'arena>) -> Result<Use<'arena>, ParseError> {
        Ok(Use {
            r#use: self.expect_keyword(stream, T!["use"])?,
            items: self.parse_use_items(stream)?,
            terminator: self.parse_terminator(stream)?,
        })
    }

    pub(crate) fn parse_use_items(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<UseItems<'arena>, ParseError> {
        let next = stream.lookahead(0)?.map(|t| t.kind);

        Ok(match next {
            Some(T!["const" | "function"]) => match stream.lookahead(2)?.map(|t| t.kind) {
                Some(T!["\\"]) => UseItems::TypedList(self.parse_typed_use_item_list(stream)?),
                _ => UseItems::TypedSequence(self.parse_typed_use_item_sequence(stream)?),
            },
            _ => match stream.lookahead(1)?.map(|t| t.kind) {
                Some(T!["\\"]) => UseItems::MixedList(self.parse_mixed_use_item_list(stream)?),
                _ => UseItems::Sequence(self.parse_use_item_sequence(stream)?),
            },
        })
    }

    pub(crate) fn parse_use_item_sequence(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<UseItemSequence<'arena>, ParseError> {
        let start = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?.span.start;

        let mut items = self.new_vec();
        let mut commas = self.new_vec();
        loop {
            items.push(self.parse_use_item(stream)?);

            if let Some(T![","]) = stream.lookahead(0)?.map(|t| t.kind) {
                commas.push(stream.consume()?);
            } else {
                break;
            }
        }

        Ok(UseItemSequence { file_id: stream.file_id(), start, items: TokenSeparatedSequence::new(items, commas) })
    }

    pub(crate) fn parse_typed_use_item_sequence(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<TypedUseItemSequence<'arena>, ParseError> {
        let r#type = self.parse_use_type(stream)?;
        let mut items = self.new_vec();
        let mut commas = self.new_vec();
        loop {
            items.push(self.parse_use_item(stream)?);

            if let Some(T![","]) = stream.lookahead(0)?.map(|t| t.kind) {
                commas.push(stream.consume()?);
            } else {
                break;
            }
        }

        Ok(TypedUseItemSequence { r#type, items: TokenSeparatedSequence::new(items, commas) })
    }

    pub(crate) fn parse_typed_use_item_list(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<TypedUseItemList<'arena>, ParseError> {
        let r#type = self.parse_use_type(stream)?;
        let namespace = self.parse_identifier(stream)?;
        let namespace_separator = stream.eat(T!["\\"])?.span;
        let left_brace = stream.eat(T!["{"])?.span;
        let mut items = self.new_vec();
        let mut commas = self.new_vec();
        loop {
            if let Some(T!["}"]) = stream.lookahead(0)?.map(|t| t.kind) {
                break;
            }

            items.push(self.parse_use_item(stream)?);

            if let Some(T![","]) = stream.lookahead(0)?.map(|t| t.kind) {
                commas.push(stream.consume()?);
            } else {
                break;
            }
        }
        let right_brace = stream.eat(T!["}"])?.span;

        Ok(TypedUseItemList {
            r#type,
            namespace,
            namespace_separator,
            left_brace,
            items: TokenSeparatedSequence::new(items, commas),
            right_brace,
        })
    }

    pub(crate) fn parse_mixed_use_item_list(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<MixedUseItemList<'arena>, ParseError> {
        let namespace = self.parse_identifier(stream)?;
        let namespace_separator = stream.eat(T!["\\"])?.span;
        let left_brace = stream.eat(T!["{"])?.span;
        let mut items = self.new_vec();
        let mut commas = self.new_vec();
        loop {
            if let Some(T!["}"]) = stream.lookahead(0)?.map(|t| t.kind) {
                break;
            }

            items.push(self.parse_maybe_typed_use_item(stream)?);

            if let Some(T![","]) = stream.lookahead(0)?.map(|t| t.kind) {
                commas.push(stream.consume()?);
            } else {
                break;
            }
        }
        let right_brace = stream.eat(T!["}"])?.span;

        Ok(MixedUseItemList {
            namespace,
            namespace_separator,
            left_brace,
            items: TokenSeparatedSequence::new(items, commas),
            right_brace,
        })
    }

    pub(crate) fn parse_maybe_typed_use_item(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<MaybeTypedUseItem<'arena>, ParseError> {
        Ok(MaybeTypedUseItem { r#type: self.parse_optional_use_type(stream)?, item: self.parse_use_item(stream)? })
    }

    pub(crate) fn parse_optional_use_type(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Option<UseType<'arena>>, ParseError> {
        Ok(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["function"]) => Some(UseType::Function(self.expect_any_keyword(stream)?)),
            Some(T!["const"]) => Some(UseType::Const(self.expect_any_keyword(stream)?)),
            _ => None,
        })
    }

    pub(crate) fn parse_use_type(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<UseType<'arena>, ParseError> {
        let next = stream.lookahead(0)?.ok_or_else(|| stream.unexpected(None, &[]))?;

        Ok(match next.kind {
            T!["function"] => UseType::Function(self.expect_any_keyword(stream)?),
            T!["const"] => UseType::Const(self.expect_any_keyword(stream)?),
            _ => return Err(stream.unexpected(Some(next), T!["function", "const"])),
        })
    }

    pub(crate) fn parse_use_item(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<UseItem<'arena>, ParseError> {
        Ok(UseItem { name: self.parse_identifier(stream)?, alias: self.parse_optional_use_item_alias(stream)? })
    }

    pub(crate) fn parse_optional_use_item_alias(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<Option<UseItemAlias<'arena>>, ParseError> {
        Ok(match stream.lookahead(0)?.map(|t| t.kind) {
            Some(T!["as"]) => Some(self.parse_use_item_alias(stream)?),
            _ => None,
        })
    }

    pub(crate) fn parse_use_item_alias(
        &mut self,
        stream: &mut TokenStream<'_, 'arena>,
    ) -> Result<UseItemAlias<'arena>, ParseError> {
        let r#as = self.expect_keyword(stream, T!["as"])?;
        let id = self.parse_local_identifier(stream)?;

        Ok(UseItemAlias { r#as, identifier: id })
    }
}
