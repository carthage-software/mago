use mago_allocator::Arena;
use mago_formatter_core::printer::Printer;
use mago_twig_syntax::cst::Template;

use crate::alignment::Alignment;
use crate::collect::TagCollection;
use crate::comment;
use crate::directives::IgnoredRanges;
use crate::document::DocumentBuilder;
use crate::output::Output;
use crate::settings::FormatSettings;
use crate::tag::{Tag, TagIterator};

pub(super) struct TemplateFormatter<'arena, 'ctx, A: Arena> {
    template: &'arena Template<'arena>,
    tags: &'ctx [Tag<'arena>],
    ignored: IgnoredRanges<'arena, A>,
    builder: DocumentBuilder<'arena, A>,
    alignment: Alignment<'arena>,
    output: Output<'arena, A>,
    printer: Printer<'arena, A>,
    offset: usize,
}

impl<'arena, 'ctx, A: Arena> TemplateFormatter<'arena, 'ctx, A> {
    pub fn new(
        arena: &'arena A,
        template: &'arena Template<'arena>,
        collected: &'ctx TagCollection<'arena, A>,
        ignored: IgnoredRanges<'arena, A>,
        settings: FormatSettings,
    ) -> Self {
        Self {
            template,
            tags: &collected.tags,
            ignored,
            builder: DocumentBuilder { arena, source: template.source_text },
            alignment: Alignment::new(settings),
            output: Output::new(arena, template.source_text.len(), settings.tab_width),
            printer: Printer::new(arena, settings.into()),
            offset: 0,
        }
    }

    pub fn format(mut self) -> &'arena [u8] {
        for tag in TagIterator::new(self.tags, self.template.trivia.as_slice()) {
            self.format_tag(tag);
        }
        self.output.bytes.extend_from_slice(&self.template.source_text[self.offset..]);

        self.output.finish()
    }

    fn format_tag(&mut self, tag: Tag<'arena>) {
        let span = tag.span();
        let original = &self.template.source_text[span.start.offset as usize..span.end.offset as usize];
        self.output.bytes.extend_from_slice(&self.template.source_text[self.offset..span.start.offset as usize]);
        if self.ignored.advance_to(span.start.offset) {
            self.output.bytes.extend_from_slice(original);
        } else {
            match tag {
                Tag::Preserved(_) => self.output.bytes.extend_from_slice(original),
                Tag::Comment(_) => comment::write(&mut self.output.bytes, original),
                Tag::Code(tag) => {
                    let position = self.output.position();
                    let alignment = self.alignment.get(self.builder.arena, position.indentation);
                    let document = self.builder.tag(tag, alignment);
                    self.printer.print_into(document, position.column, &mut self.output.bytes);
                }
            }
        }
        self.offset = span.end.offset as usize;
    }
}
