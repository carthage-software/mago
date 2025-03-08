use unicode_width::UnicodeWidthStr;

use mago_ast::*;
use mago_span::*;

use crate::document::Document;
use crate::document::Group;
use crate::document::IfBreak;
use crate::document::Line;
use crate::internal::FormatterState;
use crate::internal::format::Format;
use crate::internal::format::misc;
use crate::internal::format::misc::is_string_word_type;
use crate::internal::format::misc::should_hug_expression;

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
    pub fn get_left_delimiter_span(&self) -> Span {
        match self {
            Self::Array(array) => array.left_bracket.span(),
            Self::List(list) => list.left_parenthesis.span(),
            Self::LegacyArray(array) => array.left_parenthesis.span(),
        }
    }

    #[inline]
    pub const fn get_left_delimiter(&self) -> &'static str {
        if matches!(self, Self::List(_) | Self::LegacyArray(_)) { "(" } else { "[" }
    }

    #[inline]
    pub fn get_right_delimiter_span(&self) -> Span {
        match self {
            Self::Array(array) => array.right_bracket.span(),
            Self::List(list) => list.right_parenthesis.span(),
            Self::LegacyArray(array) => array.right_parenthesis.span(),
        }
    }

    #[inline]
    pub const fn get_right_delimiter(&self) -> &'static str {
        if matches!(self, Self::List(_) | Self::LegacyArray(_)) { ")" } else { "]" }
    }

    fn prefix(&self, f: &mut FormatterState<'a>) -> Option<Document<'a>> {
        match self {
            Self::List(list) => Some(list.list.format(f)),
            Self::LegacyArray(array) => Some(array.array.format(f)),
            _ => None,
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

pub(super) fn print_array_like<'a>(f: &mut FormatterState<'a>, array_like: ArrayLike<'a>) -> Document<'a> {
    let left_delimiter = {
        let mut left_delimiter_content = vec![];
        if let Some(prefix) = array_like.prefix(f) {
            left_delimiter_content.push(prefix);
        }

        left_delimiter_content.push(Document::String(array_like.get_left_delimiter()));
        if let Some(s) = f.print_trailing_comments(array_like.get_left_delimiter_span()) {
            left_delimiter_content.push(s);
        }

        Document::Array(left_delimiter_content)
    };

    let get_right_delimiter = |f: &mut FormatterState<'a>, array_like: &ArrayLike<'a>| {
        let mut right_delimiter_content = vec![];
        right_delimiter_content.push(Document::String(array_like.get_right_delimiter()));
        if let Some(s) = f.print_trailing_comments(array_like.get_right_delimiter_span()) {
            right_delimiter_content.push(s);
        }

        Document::Array(right_delimiter_content)
    };

    let mut parts = vec![left_delimiter];

    if array_like.is_empty() {
        if let Some(dangling_comments) = f.print_dangling_comments(array_like.span(), true) {
            parts.push(dangling_comments);
            parts.push(Document::Line(Line::soft()));
        }

        parts.push(get_right_delimiter(f, &array_like));

        return Document::Group(Group::new(parts));
    }

    let must_break = f.settings.preserve_breaking_array_like
        && misc::has_new_line_in_range(
            f.source_text,
            array_like.span().start.offset,
            array_like.elements()[0].span().start.offset,
        );

    if !must_break {
        if let Some(element) = inline_single_element(f, &array_like) {
            parts.push(element);
            parts.push(get_right_delimiter(f, &array_like));

            return Document::Group(Group::new(parts));
        }
    }

    // Check if we should use table-style formatting
    let use_table_style = f.settings.array_table_style_alignment && is_table_style(f, &array_like);
    let column_widths = if use_table_style { calculate_column_widths(f, &array_like) } else { None };

    parts.push(Document::Indent({
        let len = array_like.len();
        let mut indent_parts = vec![];
        indent_parts.push(Document::Line(Line::soft()));

        if let Some(widths) = column_widths {
            for (i, element) in array_like.elements().iter().enumerate() {
                let formatted_element = element.format(f);
                if i == len - 1 {
                    indent_parts.push(format_row_with_alignment(f, formatted_element, &widths));
                    break;
                }

                indent_parts.push(format_row_with_alignment(f, formatted_element, &widths));
                indent_parts.push(Document::String(","));
                indent_parts.push(Document::Line(Line::hard()));
            }
        } else {
            // Standard formatting without alignment
            for (i, element) in array_like.elements().iter().enumerate() {
                indent_parts.push(element.format(f));
                if i == len - 1 {
                    break;
                }

                indent_parts.push(Document::String(","));
                indent_parts.push(Document::Line(Line::default()));
            }
        }

        indent_parts
    }));

    if f.settings.trailing_comma {
        parts.push(Document::IfBreak(IfBreak::then(Document::String(","))));
    }

    if let Some(dangling_comments) = f.print_dangling_comments(array_like.span(), true) {
        parts.push(dangling_comments);
    } else {
        parts.push(Document::Line(Line::soft()));
    }

    parts.push(get_right_delimiter(f, &array_like));

    Document::Group(Group::new(parts).with_break(use_table_style || must_break))
}

fn inline_single_element<'a>(f: &mut FormatterState<'a>, array_like: &ArrayLike<'a>) -> Option<Document<'a>> {
    if array_like.len() != 1 {
        return None;
    }

    let elements = array_like.elements();
    let first_element = elements.first()?;

    match first_element {
        ArrayElement::KeyValue(element) => {
            if (element.key.is_literal() || is_string_word_type(&element.key))
                && should_hug_expression(f, &element.value)
            {
                Some(first_element.format(f))
            } else {
                None
            }
        }
        ArrayElement::Value(element) => {
            if should_hug_expression(f, &element.value) {
                Some(first_element.format(f))
            } else {
                None
            }
        }
        ArrayElement::Variadic(element) => {
            if should_hug_expression(f, &element.value) {
                Some(first_element.format(f))
            } else {
                None
            }
        }
        ArrayElement::Missing(_) => None,
    }
}

fn format_row_with_alignment<'a>(
    f: &mut FormatterState<'a>,
    document: Document<'a>,
    column_widths: &[usize],
) -> Document<'a> {
    match document {
        Document::Array(mut elements) => {
            let Some(last_element) = elements.pop() else {
                return Document::Array(elements);
            };

            let formatted_row = format_row_with_alignment(f, last_element, column_widths);
            if elements.is_empty() {
                formatted_row
            } else {
                elements.push(formatted_row);

                Document::Array(elements)
            }
        }
        Document::Group(group) => {
            if let Some((opening_delimiter, elements, closing_delimiter)) = extract_array_elements(&group.contents) {
                let formatted_elements = format_elements_with_alignment(f, elements, column_widths);

                Document::Group(Group::new(vec![
                    opening_delimiter,
                    Document::Array(formatted_elements),
                    closing_delimiter,
                ]))
            } else {
                Document::Group(group)
            }
        }
        document => document,
    }
}

fn extract_array_elements<'a>(contents: &[Document<'a>]) -> Option<(Document<'a>, Vec<Document<'a>>, Document<'a>)> {
    let mut opening_delimiter = None;
    let mut closing_delimiter = None;
    let mut elements = Vec::new();
    let mut in_elements = false;

    for doc in contents {
        match doc {
            delimiter @ Document::Array(arr) => {
                // Check if this array contains the left delimiter
                for item in arr {
                    if let Document::String(s) = item {
                        if *s == "[" || *s == "(" {
                            opening_delimiter = Some(delimiter.clone());
                            in_elements = true;
                            break;
                        } else if !in_elements && *s == "]" || *s == ")" {
                            closing_delimiter = Some(delimiter.clone());
                            break;
                        }
                    }
                }
            }
            Document::Indent(indent_docs) if in_elements => {
                // Extract elements from the indented content
                let mut element_start = 1;
                for (i, item) in indent_docs.iter().enumerate() {
                    match item {
                        Document::String(s) if *s == "," => {
                            if i > element_start {
                                elements.push(indent_docs[element_start].clone());
                            }
                            element_start = i + 2; // Skip comma and Line
                        }
                        _ => {}
                    }
                }
                // Add last element
                if element_start < indent_docs.len() {
                    elements.push(indent_docs[element_start].clone());
                }

                in_elements = false;
            }
            _ => {}
        }
    }

    match (opening_delimiter, closing_delimiter) {
        (Some(opening_delimiter), Some(closing_delimiter)) => {
            if elements.is_empty() {
                None
            } else {
                Some((opening_delimiter, elements, closing_delimiter))
            }
        }
        _ => None,
    }
}

fn format_elements_with_alignment<'a>(
    f: &mut FormatterState<'a>,
    elements: Vec<Document<'a>>,
    column_widths: &[usize],
) -> Vec<Document<'a>> {
    let mut formatted_elements = Vec::new();

    let len = elements.len();
    for (i, element) in elements.into_iter().enumerate() {
        let formatted = if i < len - 1 {
            // For all elements except the last one, add padding
            let element_width = get_document_width(&element);
            let padding = column_widths[i].saturating_sub(element_width);

            if padding > 0 {
                // Create a padded document
                Document::Array(vec![
                    element,
                    Document::String(","),
                    Document::String(f.as_str(" ".repeat(padding + 1))), // +1 for standard space after comma
                ])
            } else {
                // No padding needed
                Document::Array(vec![element, Document::String(","), Document::space()])
            }
        } else {
            // Last element, no padding
            element
        };

        formatted_elements.push(formatted);
    }

    formatted_elements
}

fn is_table_style<'a>(f: &mut FormatterState<'a>, array_like: &ArrayLike<'a>) -> bool {
    // Arbitrary limit to prevent excessive column width
    const WIGGLE_ROOM: usize = 20;

    let elements = array_like.elements();
    if elements.len() < 2 {
        return false; // Need at least two rows for table style to make sense
    }

    let mut row_size = 0;
    let mut sizes = Vec::new();
    let mut maximum_width = 0;

    // Check if all elements are nested arrays with consistent row sizes
    for element in elements {
        if f.has_inner_comment(element.span()) {
            return false; // Do not format if there are inner comments
        }

        match element {
            ArrayElement::Value(element) => {
                if let Expression::Array(Array { elements, .. })
                | Expression::LegacyArray(LegacyArray { elements, .. }) = element.value.as_ref()
                {
                    let size = elements.len();

                    // Check if row size is consistent
                    row_size = row_size.max(size);
                    sizes.push(size);

                    // Check if all inner elements are simple (strings, numbers, etc.)
                    let mut elements_width = 0;
                    for inner_element in elements.iter() {
                        match inner_element {
                            ArrayElement::Value(inner_value) => {
                                match get_element_width(f, &inner_value.value) {
                                    Some(width) => elements_width += width,
                                    None => {
                                        return false; // Only support simple elements
                                    }
                                }
                            }
                            _ => {
                                return false; // Only support Value elements
                            }
                        }
                    }

                    let total_width = elements_width + ((size - 1) * 2);
                    if total_width > (f.settings.print_width - WIGGLE_ROOM) {
                        return false; // Exceeds column width limit
                    }

                    maximum_width = maximum_width.max(total_width);
                } else {
                    return false; // Not a nested array
                }
            }
            _ => return false, // Only support Value elements
        }
    }

    if maximum_width < WIGGLE_ROOM {
        return false; // Too narrow to be a table
    }

    // Check if row size is within reasonable bounds (3-10 columns)
    if !(3..=12).contains(&row_size) {
        println!("Row size: {}", row_size);

        return false;
    }

    // At least 60% of the rows should have the same size
    let is = (sizes.iter().filter(|size| **size == row_size).count() as f64) / (sizes.len() as f64) >= 0.6;

    println!("Row size: {}", row_size);
    println!("Sizes: {:?}", sizes);
    println!("Is: {}", is);

    is
}

fn calculate_column_widths<'a>(f: &mut FormatterState<'a>, array_like: &ArrayLike<'a>) -> Option<Vec<usize>> {
    let elements = array_like.elements();
    let mut row_size = 0;

    // First pass: determine consistent row size and initialize column widths
    for element in elements {
        match element {
            ArrayElement::Value(element) => {
                if let Expression::Array(Array { elements, .. })
                | Expression::LegacyArray(LegacyArray { elements, .. }) = element.value.as_ref()
                {
                    let size = elements.len();

                    row_size = row_size.max(size);
                } else {
                    return None; // Not a nested array
                }
            }
            _ => return None, // Only support Value elements
        }
    }

    let mut column_maximum_widths = vec![0; row_size];

    // Second pass: calculate maximum width for each column
    for element in elements {
        if let ArrayElement::Value(element) = element {
            if let Expression::Array(Array { elements, .. }) | Expression::LegacyArray(LegacyArray { elements, .. }) =
                element.value.as_ref()
            {
                for (col_idx, col_element) in elements.iter().enumerate() {
                    if let ArrayElement::Value(value_element) = col_element {
                        if let Some(width) = get_element_width(f, &value_element.value) {
                            column_maximum_widths[col_idx] = column_maximum_widths[col_idx].max(width);
                        } else {
                            return None; // Cannot determine element width
                        }
                    } else {
                        return None; // Only support Value elements in inner arrays
                    }
                }
            }
        }
    }

    Some(column_maximum_widths)
}

fn get_element_width<'a>(f: &mut FormatterState<'a>, element: &'a Expression) -> Option<usize> {
    Some(match element {
        Expression::Literal(literal) => match literal {
            Literal::String(literal_string) => width(f.interner.lookup(&literal_string.value)),
            Literal::Integer(literal_integer) => f.interner.lookup(&literal_integer.raw).width(),
            Literal::Float(literal_float) => f.interner.lookup(&literal_float.raw).width(),
            Literal::True(_) => 4,
            Literal::False(_) => 5,
            Literal::Null(_) => 4,
        },
        Expression::MagicConstant(magic_constant) => width(f.interner.lookup(&magic_constant.value().value)),
        Expression::ConstantAccess(ConstantAccess { name: Identifier::Local(local) })
        | Expression::Identifier(Identifier::Local(local)) => width(f.interner.lookup(&local.value)),
        Expression::Variable(Variable::Direct(variable)) => width(f.interner.lookup(&variable.name)),
        Expression::Call(Call::Function(FunctionCall { function, argument_list })) => {
            if !argument_list.arguments.is_empty() {
                return None;
            }

            return get_element_width(f, function).map(|width| width + 2);
        }
        Expression::Call(Call::StaticMethod(StaticMethodCall {
            class,
            method: ClassLikeMemberSelector::Identifier(method),
            argument_list,
            ..
        })) => {
            if !argument_list.arguments.is_empty() {
                return None;
            }

            return get_element_width(f, class).map(|class| class + 2 + width(f.interner.lookup(&method.value)) + 2);
        }
        Expression::Access(Access::ClassConstant(ClassConstantAccess {
            class,
            constant: ClassLikeConstantSelector::Identifier(constant),
            ..
        })) => {
            return get_element_width(f, class).map(|class| class + 2 + width(f.interner.lookup(&constant.value)) + 2);
        }
        _ => {
            return None;
        }
    })
}

fn get_document_width(doc: &Document<'_>) -> usize {
    match doc {
        Document::String(s) => width(s),
        Document::Array(docs) => docs.iter().map(get_document_width).sum(),
        Document::Group(group) => group.contents.iter().map(get_document_width).sum(),
        Document::Indent(docs) => docs.iter().map(get_document_width).sum(),
        // For other document types, provide reasonable estimates
        Document::Line(_) => 1,
        Document::IfBreak(if_break) => {
            get_document_width(&if_break.break_contents).max(get_document_width(&if_break.flat_content))
        }
        _ => 0,
    }
}

#[inline]
fn width(s: &str) -> usize {
    if s.contains("الله") {
        // The word "الله" is a special case, as it is usually rendered as a single glyph
        // while being 4 characters wide. This is a hack to handle this case.
        s.replace("الله", "_").width()
    } else {
        s.width()
    }
}
