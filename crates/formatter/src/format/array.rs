use fennec_ast::*;
use fennec_span::*;

use crate::document::Document;
use crate::document::Group;
use crate::document::IfBreak;
use crate::document::Line;
use crate::format::misc;
use crate::format::Format;
use crate::Formatter;

use super::misc::is_string_word_type;
use super::misc::should_hug_expression;

#[allow(clippy::enum_variant_names)]
pub enum ArrayLike<'a> {
    Array(&'a Array),
    List(&'a List),
    LegacyArray(&'a LegacyArray),
}

impl<'a> ArrayLike<'a> {
    #[inline]
    fn len(&self) -> usize {
        match self {
            Self::Array(array) => array.elements.len(),
            Self::List(list) => list.elements.len(),
            Self::LegacyArray(array) => array.elements.len(),
        }
    }

    #[inline]
    fn is_empty(&self) -> bool {
        match self {
            Self::Array(array) => array.elements.is_empty(),
            Self::List(list) => list.elements.is_empty(),
            Self::LegacyArray(array) => array.elements.is_empty(),
        }
    }

    #[inline]
    fn elements(&self) -> &'a [ArrayElement] {
        match self {
            Self::Array(array) => array.elements.as_slice(),
            Self::LegacyArray(array) => array.elements.as_slice(),
            Self::List(list) => list.elements.as_slice(),
        }
    }

    #[inline]
    const fn uses_parenthesis(&self) -> bool {
        match self {
            Self::List(_) | Self::LegacyArray(_) => true,
            _ => false,
        }
    }

    fn prefix(&self, f: &mut Formatter<'a>) -> Option<Document<'a>> {
        match self {
            Self::List(list) => Some(list.list.format(f)),
            Self::LegacyArray(array) => Some(array.array.format(f)),
            _ => None,
        }
    }

    fn iter<'b>(&'b self, p: &'b mut Formatter<'a>) -> Box<dyn Iterator<Item = Document<'a>> + 'b> {
        match self {
            Self::Array(array) => Box::new(array.elements.iter().map(|element| element.format(p))),
            Self::List(list) => Box::new(list.elements.iter().map(|element| element.format(p))),
            Self::LegacyArray(array) => Box::new(array.elements.iter().map(|element| element.format(p))),
        }
    }
}

impl HasSpan for ArrayLike<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Array(array) => array.span(),
            Self::List(list) => list.span(),
            Self::LegacyArray(array) => array.span(),
        }
    }
}

pub(super) fn print_array_like<'a>(f: &mut Formatter<'a>, array_like: ArrayLike<'a>) -> Document<'a> {
    let left_delimiter = if let Some(prefix) = array_like.prefix(f) {
        Document::Array(vec![prefix, Document::String(if array_like.uses_parenthesis() { "(" } else { "[" })])
    } else {
        Document::String(if array_like.uses_parenthesis() { "(" } else { "[" })
    };

    let right_delimiter = Document::String(if array_like.uses_parenthesis() { ")" } else { "]" });

    if array_like.is_empty() {
        return Document::Group(Group::new(vec![
            left_delimiter,
            if let Some(dangling_comments) = f.print_dangling_comments(array_like.span(), true) {
                Document::Array(vec![dangling_comments, Document::Line(Line::softline())])
            } else {
                Document::empty()
            },
            right_delimiter,
        ]));
    }

    let mut parts = vec![left_delimiter];

    if let Some(element) = inline_single_element(f, &array_like) {
        parts.push(element);
        parts.push(right_delimiter);

        return Document::Group(Group::new(parts));
    }

    parts.push(Document::Indent({
        let len = array_like.len();
        let mut indent_parts = vec![];
        indent_parts.push(Document::Line(Line::softline()));
        for (i, doc) in array_like.iter(f).enumerate() {
            indent_parts.push(doc);
            if i == len - 1 {
                break;
            }

            indent_parts.push(Document::String(","));
            indent_parts.push(Document::Line(Line::default()));
        }

        indent_parts
    }));

    if f.settings.trailing_comma {
        parts.push(Document::IfBreak(IfBreak::then(Document::String(","))));
    }

    if let Some(dangling_comments) = f.print_dangling_comments(array_like.span(), true) {
        parts.push(dangling_comments);
    } else {
        parts.push(Document::Line(Line::softline()));
    }

    parts.push(right_delimiter);

    // preserve new lines between the opening delimiter ( e.g. `[` or `(` ) and the first element
    let should_break = misc::has_new_line_in_range(
        f.source_text,
        array_like.span().start.offset,
        array_like.elements()[0].span().start.offset,
    );

    Document::Group(Group::new(parts).with_break(should_break))
}

fn inline_single_element<'a>(f: &mut Formatter<'a>, array_like: &ArrayLike<'a>) -> Option<Document<'a>> {
    if array_like.len() != 1 {
        return None;
    }

    let elements = array_like.elements();
    let Some(first_element) = elements.first() else {
        return None;
    };

    match first_element {
        ArrayElement::KeyValue(element) => {
            if !element.key.is_literal() && !is_string_word_type(&element.key) {
                return None;
            }

            if should_hug_expression(f, &element.value) {
                return Some(first_element.format(f));
            } else {
                return None;
            }
        }
        ArrayElement::Value(element) => {
            if should_hug_expression(f, &element.value) {
                return Some(first_element.format(f));
            } else {
                return None;
            }
        }
        ArrayElement::Variadic(element) => {
            if should_hug_expression(f, &element.value) {
                return Some(first_element.format(f));
            } else {
                return None;
            }
        }
        ArrayElement::Missing(_) => {
            return None;
        }
    }
}
