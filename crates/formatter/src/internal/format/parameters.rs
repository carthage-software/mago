use bumpalo::collections::Vec;
use bumpalo::vec;

use mago_span::HasSpan;
use mago_syntax::ast::FunctionLikeParameter;
use mago_syntax::ast::FunctionLikeParameterList;
use mago_syntax::ast::Modifier;
use mago_syntax::ast::Sequence;

use crate::document::BreakMode;
use crate::document::Document;
use crate::document::Group;
use crate::document::IfBreak;
use crate::document::Line;
use crate::internal::FormatterState;
use crate::internal::comment::CommentFlags;
use crate::internal::format::Format;
use crate::internal::format::misc;
use crate::internal::utils::string_width;

pub(super) fn print_function_like_parameters<'arena>(
    f: &mut FormatterState<'_, 'arena>,
    parameter_list: &'arena FunctionLikeParameterList<'arena>,
) -> Document<'arena> {
    if parameter_list.parameters.is_empty() {
        let mut contents = vec![in f.arena; Document::String("(")];
        if let Some(comments) = f.print_inner_comment(parameter_list.span(), true) {
            contents.push(comments);
        }

        contents.push(Document::String(")"));

        return Document::Array(contents);
    }

    let mut force_break = force_break_parameters(f, parameter_list);
    let preserve_break = preserve_break_parameters(f, parameter_list);
    let should_break = force_break || preserve_break;

    let list_id = f.next_id();
    let previous_list_group_id = f.parameter_state.list_group_id;
    f.parameter_state.list_group_id = Some(list_id);

    let should_hug_the_parameters = !should_break && should_hug_the_only_parameter(f, parameter_list);
    let parameter_variable_width = if should_align_parameters(f, parameter_list) {
        Some(
            get_max_parameter_prefix_width(parameter_list)
                + usize::from(needs_promoted_parameter_type_gutter(parameter_list)),
        )
    } else {
        None
    };

    let left_parenthesis = {
        let mut contents = vec![in f.arena; Document::String("(")];

        if let Some(trailing_comment) = f.print_trailing_comments(parameter_list.left_parenthesis) {
            contents.push(trailing_comment);
            force_break = true;
        }

        if let Some(parameter) = parameter_list.parameters.first()
            && let Some(trailing_comments) =
                f.print_dangling_comments_between_nodes(parameter_list.left_parenthesis, parameter.span())
        {
            contents.push(trailing_comments);
            force_break = true;
        }

        Document::Array(contents)
    };

    let mut parts = vec![in f.arena; left_parenthesis];

    let mut printed = vec![in f.arena; ];
    let len = parameter_list.parameters.len();
    let previous_variable_padding = f.parameter_state.variable_padding;
    for (i, parameter) in parameter_list.parameters.iter().enumerate() {
        f.parameter_state.variable_padding = match parameter_variable_width {
            Some(max_width) if can_align_parameter(f, parameter_list, i, parameter) => {
                let padding = max_width.saturating_sub(get_parameter_prefix_width(parameter));
                (padding > 0).then_some(padding)
            }
            _ => None,
        };

        printed.push(parameter.format(f));
        if i == len - 1 {
            break;
        }

        printed.push(Document::String(","));
        printed.push(Document::Line(Line::default()));

        if f.is_next_line_empty(parameter.span()) {
            printed.push(Document::BreakParent);
            printed.push(Document::Line(Line::hard()));
        }
    }
    f.parameter_state.variable_padding = previous_variable_padding;

    if should_hug_the_parameters {
        parts.extend(printed);
        parts.push(Document::String(")"));

        return Document::Array(parts);
    }

    if !parameter_list.parameters.is_empty() {
        let mut contents = vec![in f.arena; Document::Line(Line::soft())];
        contents.extend(printed);
        parts.push(Document::Indent(contents));

        if f.settings.trailing_comma {
            parts.push(Document::IfBreak(IfBreak::then(f.arena, Document::String(","))));
        }
    }

    if let Some(comments) =
        f.print_dangling_comments(parameter_list.left_parenthesis.join(parameter_list.right_parenthesis), true)
    {
        parts.push(comments);
    } else {
        parts.push(Document::Line(Line::soft()));
    }

    parts.push(Document::String(")"));

    f.parameter_state.list_group_id = previous_list_group_id;

    Document::Group(Group::new(parts).with_id(list_id).with_break_mode(if force_break {
        BreakMode::Force
    } else if preserve_break {
        BreakMode::Preserve
    } else {
        BreakMode::Auto
    }))
}

pub(super) fn force_break_parameters<'arena>(
    f: &FormatterState<'_, 'arena>,
    parameter_list: &'arena FunctionLikeParameterList<'arena>,
) -> bool {
    f.settings.break_promoted_properties_list
        && parameter_list.parameters.iter().any(FunctionLikeParameter::is_promoted_property)
}

pub(super) fn preserve_break_parameters<'arena>(
    f: &FormatterState<'_, 'arena>,
    parameter_list: &'arena FunctionLikeParameterList<'arena>,
) -> bool {
    f.settings.preserve_breaking_parameter_list
        && !parameter_list.parameters.is_empty()
        && misc::has_new_line_in_range(
            f.source_text,
            parameter_list.left_parenthesis.start.offset,
            parameter_list.parameters.as_slice()[0].span().start.offset,
        )
}

pub(super) fn should_hug_the_only_parameter<'arena>(
    f: &FormatterState<'_, 'arena>,
    parameter_list: &'arena FunctionLikeParameterList<'arena>,
) -> bool {
    if parameter_list.parameters.len() != 1 {
        return false;
    }

    let Some(parameter) = parameter_list.parameters.first() else {
        return false;
    };

    // Avoid hugging the parameter if it has a comment anywhere around it
    if f.has_comment(parameter.span(), CommentFlags::all()) {
        return false;
    }

    // Don't hug the parameter if it has an attribute, or if it has a
    // property hook list.
    //
    // TODO: maybe hug the parameter if it has a single attribute and no hooks?
    if !parameter.attribute_lists.is_empty() || parameter.hooks.is_some() {
        return false;
    }

    if !parameter.modifiers.is_empty() && f.settings.break_promoted_properties_list {
        return false;
    }

    true
}

fn should_align_parameters<'arena>(
    f: &FormatterState<'_, 'arena>,
    parameter_list: &'arena FunctionLikeParameterList<'arena>,
) -> bool {
    f.settings.align_parameters
        && parameter_list.parameters.len() >= 2
        && (force_break_parameters(f, parameter_list)
            || preserve_break_parameters(f, parameter_list)
            || parameter_list_exceeds_print_width(f, parameter_list))
        && parameter_list
            .parameters
            .iter()
            .enumerate()
            .all(|(index, parameter)| can_align_parameter(f, parameter_list, index, parameter))
}

fn parameter_list_exceeds_print_width<'arena>(
    f: &FormatterState<'_, 'arena>,
    parameter_list: &'arena FunctionLikeParameterList<'arena>,
) -> bool {
    let start = parameter_list.left_parenthesis.start.offset;
    let end = parameter_list.right_parenthesis.end.offset;

    let source = &f.source_text[start as usize..end as usize];
    let mut flattened = Vec::with_capacity_in(source.len(), f.arena);
    let mut previous_was_whitespace = false;
    for character in source.chars() {
        if character.is_whitespace() {
            if !previous_was_whitespace {
                flattened.push(b' ');
                previous_was_whitespace = true;
            }
        } else {
            let mut buffer = [0; 4];
            flattened.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
            previous_was_whitespace = false;
        }
    }

    let flattened = unsafe { std::str::from_utf8_unchecked(flattened.into_bump_slice()) };

    string_width(flattened.trim()) > f.settings.print_width
}

fn get_max_parameter_prefix_width<'arena>(parameter_list: &'arena FunctionLikeParameterList<'arena>) -> usize {
    parameter_list.parameters.iter().map(get_parameter_prefix_width).max().unwrap_or(0)
}

fn needs_promoted_parameter_type_gutter<'arena>(parameter_list: &'arena FunctionLikeParameterList<'arena>) -> bool {
    parameter_list.parameters.iter().any(|parameter| parameter.is_promoted_property() && parameter.hint.is_none())
}

fn get_parameter_prefix_width<'arena>(parameter: &'arena FunctionLikeParameter<'arena>) -> usize {
    let mut width = 0;

    let modifiers_width = get_modifier_sequence_width(&parameter.modifiers);
    if modifiers_width > 0 {
        width += modifiers_width;
        width += 1;
    }

    if let Some(hint) = &parameter.hint {
        width += hint.span().length() as usize;
        width += 1;
    }

    if parameter.ampersand.is_some() {
        width += string_width("&");
    }

    if parameter.ellipsis.is_some() {
        width += string_width("...");
    }

    width
}

fn can_align_parameter<'arena>(
    f: &FormatterState<'_, 'arena>,
    parameter_list: &'arena FunctionLikeParameterList<'arena>,
    index: usize,
    parameter: &'arena FunctionLikeParameter<'arena>,
) -> bool {
    let gap_start = if index == 0 {
        parameter_list.left_parenthesis.end.offset
    } else {
        parameter_list.parameters.as_slice()[index - 1].end_offset()
    };

    !has_comment_trivia_in_range(f, gap_start, parameter.start_offset())
        && !has_comment_trivia_in_range(f, parameter.start_offset(), parameter.variable.start_offset())
}

fn has_comment_trivia_in_range(f: &FormatterState<'_, '_>, start: u32, end: u32) -> bool {
    if start >= end {
        return false;
    }

    f.all_comments().iter().any(|comment| {
        let comment_span = comment.span;
        comment_span.start.offset < end && comment_span.end.offset > start
    })
}

fn get_modifier_sequence_width(modifiers: &Sequence<'_, Modifier<'_>>) -> usize {
    let ordered_modifiers = [
        modifiers.get_final(),
        modifiers.get_abstract(),
        modifiers.get_first_read_visibility(),
        modifiers.get_first_write_visibility(),
        modifiers.get_static(),
        modifiers.get_readonly(),
    ];

    let mut width = 0;
    for (printed_count, modifier) in ordered_modifiers.into_iter().flatten().enumerate() {
        if printed_count > 0 {
            width += 1;
        }

        width += string_width(modifier.get_keyword().value);
    }

    width
}
