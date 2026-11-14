//! Formats Twig syntax without reformatting HTML or other template text.

use std::borrow::Cow;

use mago_allocator::Arena;
use mago_database::file::File;
use mago_twig_syntax::cst::Template;
use mago_twig_syntax::cst::TriviaKind;
use mago_twig_syntax::error::ParseError;
use mago_twig_syntax::parser::Parser;
use mago_twig_syntax::settings::ParserSettings;

use crate::collect::TagCollection;
use crate::directives::Directives;
use crate::settings::FormatSettings;
use crate::template::TemplateFormatter;

mod alignment;
mod collect;
mod comment;
mod directives;
mod document;
mod expression;
mod header;
mod output;
mod safety;
pub mod settings;
mod tag;
mod template;

const NESTING_LIMIT: u16 = 32;

#[derive(Debug)]
pub struct Formatter<'arena, A: Arena> {
    arena: &'arena A,
    settings: FormatSettings,
}

impl<'arena, A: Arena> Formatter<'arena, A> {
    #[must_use]
    pub fn new(arena: &'arena A, settings: FormatSettings) -> Self {
        Self { arena, settings }
    }

    /// Format a Twig source buffer.
    ///
    /// # Errors
    /// Returns the first syntax error without changing the input.
    pub fn format_code(
        &self,
        name: Cow<'static, [u8]>,
        code: Cow<'static, [u8]>,
    ) -> Result<&'arena [u8], ParseError<'arena>> {
        self.format_file(&File::ephemeral(name, code))
    }

    /// Format a Twig file, leaving template text and literal strings intact.
    ///
    /// # Errors
    /// Returns the first syntax error without changing the input.
    pub fn format_file(&self, file: &File) -> Result<&'arena [u8], ParseError<'arena>> {
        let source = self.arena.alloc_slice_copy(&file.contents);
        let template = Parser::new(self.arena, file.id, source, ParserSettings::default())
            .with_recursion_limit(NESTING_LIMIT)
            .parse(source, file.id);

        self.format(template)
    }

    /// Format a parsed template.
    ///
    /// # Errors
    /// Returns the first syntax error in the template.
    pub fn format(&self, template: &'arena Template<'arena>) -> Result<&'arena [u8], ParseError<'arena>> {
        if let Some(error) = template.errors.first() {
            return Err(error.clone());
        }

        let directives = Directives::collect(self.arena, template);
        if directives.ignores_file
            || !template.has_script() && !template.trivia.iter().any(|trivia| trivia.kind == TriviaKind::Comment)
        {
            return Ok(template.source_text);
        }

        let collected = TagCollection::collect(self.arena, template, directives.needs_statements())?;
        if collected.has_legacy_raw {
            return Ok(template.source_text);
        }

        let settings = self.settings.resolve(template, &collected.tags);
        let ignored = directives.resolve(&collected);

        Ok(TemplateFormatter::new(self.arena, template, &collected, ignored, settings).format())
    }
}
