//! Tag dispatcher. Each `{% name ... %}` tag has its own module under
//! `parser::internal::tag::*`; every parser is a method on `Parser`.

use crate::ast::Statement;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TwigToken;
use mago_allocator::prelude::*;

pub mod apply;
pub mod autoescape;
pub mod block;
pub mod cache;
pub mod deprecated;
pub mod do_;
pub mod embed;
pub mod extends;
pub mod flush;
pub mod r#for;
pub mod from;
pub mod guard;
pub mod r#if;
pub mod import;
pub mod include;
pub mod r#macro;
pub mod sandbox;
pub mod set;
pub mod types;
pub mod unknown;
pub mod r#use;
pub mod verbatim;
pub mod with;

impl<'arena, A> Parser<'_, 'arena, A>
where
    A: Arena,
{
    /// Dispatch a `{% keyword ...` tag to the matching per-tag parser.
    pub(crate) fn parse_tag(
        &mut self,
        open_tag: TwigToken<'arena>,
        keyword: TwigToken<'arena>,
    ) -> Result<Statement<'arena>, ParseError<'arena>> {
        match keyword.value {
            b"if" => self.parse_if(open_tag, keyword),
            b"for" => self.parse_for(open_tag, keyword),
            b"set" => self.parse_set(open_tag, keyword),
            b"block" => self.parse_block(open_tag, keyword),
            b"extends" => self.parse_extends(open_tag, keyword),
            b"use" => self.parse_use(open_tag, keyword),
            b"include" => self.parse_include(open_tag, keyword),
            b"embed" => self.parse_embed(open_tag, keyword),
            b"import" => self.parse_import(open_tag, keyword),
            b"from" => self.parse_from(open_tag, keyword),
            b"macro" => self.parse_macro(open_tag, keyword),
            b"with" => self.parse_with(open_tag, keyword),
            b"apply" => self.parse_apply(open_tag, keyword),
            b"autoescape" => self.parse_autoescape(open_tag, keyword),
            b"sandbox" => self.parse_sandbox(open_tag, keyword),
            b"deprecated" => self.parse_deprecated(open_tag, keyword),
            b"do" => self.parse_do(open_tag, keyword),
            b"flush" => self.parse_flush(open_tag, keyword),
            b"guard" => self.parse_guard(open_tag, keyword),
            b"cache" => self.parse_cache(open_tag, keyword),
            b"types" => self.parse_types(open_tag, keyword),
            b"verbatim" | b"raw" => self.parse_verbatim(open_tag, keyword),
            _ => self.parse_unknown_tag(open_tag, keyword),
        }
    }
}
