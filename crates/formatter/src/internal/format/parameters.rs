use bumpalo::vec;

use mago_span::HasSpan;
use mago_syntax::ast::FunctionLikeParameter;
use mago_syntax::ast::FunctionLikeParameterList;

use crate::document::BreakMode;
use crate::document::Document;
use crate::document::Group;
use crate::document::IfBreak;
use crate::document::Line;
use crate::internal::FormatterState;
use crate::internal::comment::CommentFlags;
use crate::internal::format::Format;
use crate::internal::format::misc;

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
    let preserve_break = f.settings.preserve_breaking_parameter_list
        && misc::has_new_line_in_range(
            f.source_text,
            parameter_list.left_parenthesis.start.offset,
            parameter_list.parameters.as_slice()[0].span().start.offset,
        );
    let should_break = force_break || preserve_break;

    let previous_break = f.parameter_state.force_break;
    if should_break {
        f.parameter_state.force_break = true;
    }

    let should_hug_the_parameters = !should_break && should_hug_the_only_parameter(f, parameter_list);

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
    for (i, parameter) in parameter_list.parameters.iter().enumerate() {
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

    f.parameter_state.force_break = previous_break;

    Document::Group(Group::new(parts).with_break_mode(if force_break {
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
