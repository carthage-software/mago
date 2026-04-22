//! Tag dispatcher. Each `{% name ... %}` tag has its own module under
//! `parser::internal::tag::*`; every parser is a method on `Parser`.

use crate::ast::Statement;
use crate::error::ParseError;
use crate::parser::Parser;
use crate::token::TwigToken;

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

impl<'input, 'arena> Parser<'input, 'arena> {
    /// Dispatch a `{% keyword ...` tag to the matching per-tag parser.
    pub(crate) fn parse_tag(
        &mut self,
        open_tag: TwigToken<'arena>,
        keyword: TwigToken<'arena>,
    ) -> Result<Statement<'arena>, ParseError> {
        match keyword.value {
            "if" => self.parse_if(open_tag, keyword),
            "for" => self.parse_for(open_tag, keyword),
            "set" => self.parse_set(open_tag, keyword),
            "block" => self.parse_block(open_tag, keyword),
            "extends" => self.parse_extends(open_tag, keyword),
            "use" => self.parse_use(open_tag, keyword),
            "include" => self.parse_include(open_tag, keyword),
            "embed" => self.parse_embed(open_tag, keyword),
            "import" => self.parse_import(open_tag, keyword),
            "from" => self.parse_from(open_tag, keyword),
            "macro" => self.parse_macro(open_tag, keyword),
            "with" => self.parse_with(open_tag, keyword),
            "apply" => self.parse_apply(open_tag, keyword),
            "autoescape" => self.parse_autoescape(open_tag, keyword),
            "sandbox" => self.parse_sandbox(open_tag, keyword),
            "deprecated" => self.parse_deprecated(open_tag, keyword),
            "do" => self.parse_do(open_tag, keyword),
            "flush" => self.parse_flush(open_tag, keyword),
            "guard" => self.parse_guard(open_tag, keyword),
            "cache" => self.parse_cache(open_tag, keyword),
            "types" => self.parse_types(open_tag, keyword),
            "verbatim" | "raw" => self.parse_verbatim(open_tag, keyword),
            _ => self.parse_unknown_tag(open_tag, keyword),
        }
    }
}
