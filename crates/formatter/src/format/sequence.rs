use fennec_ast::sequence::Sequence;
use fennec_ast::sequence::TokenSeparatedSequence;
use fennec_span::HasSpan;

use crate::comment::CommentFlags;
use crate::document::Group;
use crate::document::IfBreak;
use crate::document::Line;
use crate::document::*;
use crate::format::delimited::Delimiter;
use crate::format::Format;
use crate::Formatter;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) struct TokenSeparatedSequenceFormatter<'a> {
    pub separator: &'a str,
    pub trailing_separator: bool,
    pub force_break: bool,
    pub force_inline: bool,
    pub break_parent: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) struct SequenceFormatter {
    pub force_break: bool,
}

impl<'a> TokenSeparatedSequenceFormatter<'a> {
    pub fn new(separator: &'a str) -> Self {
        Self { separator, trailing_separator: false, force_break: false, force_inline: false, break_parent: false }
    }

    pub fn with_trailing_separator(mut self, trailing_separator: bool) -> Self {
        self.trailing_separator = trailing_separator;
        self
    }

    pub fn format<T: Format<'a> + HasSpan>(
        self,
        f: &mut Formatter<'a>,
        nodes: &'a TokenSeparatedSequence<T>,
    ) -> Document<'a> {
        let mut contents = Vec::new();

        let mut must_break = self.force_break;
        let length = nodes.len();
        for (i, item, separator) in nodes.iter_with_tokens() {
            contents.push(item.format(f));
            if let Some(separator) = separator {
                // If a separator has a trailing single line comment, we must break
                // the sequence, otherwise a further node might get commented out.
                //
                // e.g.:
                //
                // ```
                // (
                //    $a, // comment
                //    $b,
                // )
                // ```
                //
                // If we don't break the sequence, the comment will be on the same line as the
                // rest of the sequence, which is not what we want.
                //
                // ```
                // ($a, // comment, $b)
                // ```
                //
                // resulting in `b` and the right parenthesis being commented out.
                must_break = must_break || f.has_comment(separator.span, CommentFlags::Trailing & CommentFlags::Line);

                let leading_comments = f.print_leading_comments(separator.span);
                let trailing_comments = f.print_trailing_comments(separator.span);
                let has_comments = leading_comments.is_some() || trailing_comments.is_some();
                let separator =
                    f.print_comments(leading_comments, Document::String(f.lookup(&separator.value)), trailing_comments);

                if (i < (length - 1)) || has_comments || must_break {
                    contents.push(separator);
                } else {
                    contents.push(Document::IfBreak(IfBreak::then(separator)));
                }
            } else if self.trailing_separator {
                if must_break {
                    contents.push(Document::String(self.separator));
                } else if !self.force_inline {
                    contents.push(Document::IfBreak(IfBreak::then(Document::String(self.separator))));
                }
            }

            if i < (length - 1) {
                if must_break {
                    contents.push(Document::Line(Line::hardline()));
                } else if self.force_inline {
                    contents.push(Document::space());
                } else {
                    contents.push(Document::Line(Line::default()));
                }
            }
        }

        Document::Array(contents)
    }

    pub fn format_with_delimiter<T: Format<'a> + HasSpan>(
        mut self,
        f: &mut Formatter<'a>,
        nodes: &'a TokenSeparatedSequence<T>,
        delimiter: Delimiter,
        preserve_breaks: bool,
    ) -> Document<'a> {
        let inner_content_is_empty = nodes.is_empty();
        if !self.force_break {
            self.force_break =
                if preserve_breaks { (!inner_content_is_empty) && delimiter.is_already_broken(f) } else { false };
        }

        // Format the left delimiter with any leading or trailing comments
        let (left_delimiter, has_left_trailing_comments) = delimiter.format_left(f);
        if has_left_trailing_comments {
            self.force_break = true;
        }

        // Format the inner content using the provided formatter function
        let inner_content = self.format(f, nodes);

        // Format the right delimiter with any leading or trailing comments
        let (right_delimiter, has_right_leading_comments) = delimiter.format_right(f);

        let delimiter_needs_space = delimiter.needs_space();

        // Construct the final document with proper grouping and indentation
        let documents = vec![
            left_delimiter,
            if inner_content_is_empty {
                Document::empty()
            } else {
                let mut contents = match inner_content {
                    Document::Array(contents) => contents,
                    _ => vec![inner_content],
                };

                contents.insert(
                    0,
                    if delimiter_needs_space {
                        Document::Line(Line::default())
                    } else {
                        Document::Line(Line::softline())
                    },
                );

                Document::Indent(contents)
            },
            if !inner_content_is_empty {
                if delimiter_needs_space {
                    Document::Line(Line::default())
                } else {
                    Document::Line(Line::softline())
                }
            } else {
                Document::empty()
            },
            right_delimiter,
        ];

        if self.force_inline {
            Document::Group(Group::new(documents))
        } else if self.force_break || has_right_leading_comments {
            Document::Group(Group::new(documents).with_break(true))
        } else {
            Document::Array(documents)
        }
    }
}

impl SequenceFormatter {
    pub fn new() -> Self {
        Self { force_break: false }
    }

    pub fn with_force_break(mut self, force_break: bool) -> Self {
        self.force_break = force_break;
        self
    }

    pub fn format<'a, T: Format<'a> + HasSpan>(self, f: &mut Formatter<'a>, nodes: &'a Sequence<T>) -> Document<'a> {
        let mut contents = Vec::new();

        let length = nodes.len();
        for (i, item) in nodes.iter().enumerate() {
            contents.push(item.format(f));

            if i < (length - 1) {
                if self.force_break {
                    contents.push(Document::Line(Line::hardline()));
                    if f.is_next_line_empty(item.span()) {
                        contents.push(Document::Line(Line::hardline()));
                    }
                } else {
                    contents.push(Document::Line(Line::default()));
                }
            }
        }

        Document::Group(Group::new(contents))
    }
}
