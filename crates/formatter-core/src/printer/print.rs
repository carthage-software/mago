use crate::document::Document;
use crate::printer::Printer;
use crate::printer::align::Mark;
use crate::printer::command::{Command, CommandDocument, Indentation, Mode};
use mago_allocator::Arena;

impl<'arena, A: Arena> Printer<'arena, A> {
    pub(super) fn print_commands(&mut self) {
        let mut should_remeasure = false;
        while let Some(Command { indentation, document, mode }) = self.commands.pop() {
            match document {
                CommandDocument::Array(docs) => {
                    docs.iter().rev().any(|doc| Self::propagate_breaks(&mut self.propagated_breaks, doc));
                    self.push_docs(indentation, mode, docs);
                }
                CommandDocument::Document(document) => {
                    if matches!(
                        document,
                        Document::Group(_)
                            | Document::Array(_)
                            | Document::Indent(_)
                            | Document::Align(_)
                            | Document::IndentIfBreak(_)
                            | Document::IfBreak(_)
                    ) {
                        Self::propagate_breaks(&mut self.propagated_breaks, document);
                    }

                    match document {
                        Document::String(s) => self.handle_bytes(s),
                        Document::RangeStart => self.range_start = Some(self.out.len()),
                        Document::RangeEnd => {
                            if self.line_suffix.is_empty() {
                                self.range_end = Some(self.out.len());
                            } else {
                                self.line_suffix.push(Command::new(indentation, mode, document));
                            }
                        }
                        Document::Space(space) => self.handle_space(*space),
                        Document::Column(column) => {
                            self.marks.push(Mark::new(*column, self.out.len(), self.position));
                        }
                        Document::Array(docs) => self.push_docs(indentation, mode, docs),
                        Document::Indent(docs) => self.push_docs(indentation.indented(), mode, docs),
                        Document::Align(align) => {
                            self.push_docs(Indentation::aligned(align.alignment), mode, &align.contents)
                        }
                        Document::Group(_) => {
                            should_remeasure = self.handle_group(&indentation, mode, document, should_remeasure);
                        }
                        Document::IndentIfBreak(docs) => self.handle_indent_if_break(&indentation, mode, docs),
                        Document::Line(line) => {
                            should_remeasure = self.handle_line(*line, &indentation, mode, document, should_remeasure);
                        }
                        Document::LineSuffix(docs) => {
                            self.line_suffix.push(Command::array(indentation, mode, docs));
                        }
                        Document::LineSuffixBoundary => {
                            should_remeasure = self.handle_line_suffix_boundary(indentation, mode, should_remeasure);
                        }
                        Document::IfBreak(if_break) => self.handle_if_break(if_break, indentation, mode),
                        Document::BreakParent => {}
                        Document::Trim(trim) => self.handle_trim(*trim),
                        Document::DoNotTrim => {
                            self.can_trim = false;
                        }
                    }
                }
            }

            if self.commands.is_empty() && !self.line_suffix.is_empty() {
                self.flush_line_suffix();
            }
        }
    }

    pub(super) fn flush_line_suffix(&mut self) {
        if !self.marks.is_empty() {
            self.suffix_offsets.push(self.out.len());
        }

        self.commands.extend(self.line_suffix.drain(..).rev());
    }

    pub(super) fn push_docs(
        &mut self,
        indentation: Indentation<'arena>,
        mode: Mode,
        docs: &'arena [Document<'arena, A>],
    ) {
        self.commands.extend(docs.iter().rev().map(|doc| Command::new(indentation, mode, doc)));
    }
}
