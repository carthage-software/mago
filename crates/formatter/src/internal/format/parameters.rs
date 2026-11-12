use mago_allocator::Arena;
use mago_allocator::vec_in;

use mago_php_version::feature::Feature;
use mago_span::HasSpan;
use mago_syntax::cst::FunctionLikeParameter;
use mago_syntax::cst::FunctionLikeParameterList;

use crate::document::BreakMode;
use crate::document::Column;
use crate::document::Document;
use crate::document::Group;
use crate::document::IfBreak;
use crate::document::IndentIfBreak;
use crate::document::Line;
use crate::internal::FormatterState;
use crate::internal::comment::CommentFlags;
use crate::internal::format::Format;
use crate::internal::format::misc;

pub(super) fn print_function_like_parameters<'arena, A>(
    f: &mut FormatterState<'_, 'arena, A>,
    parameter_list: &'arena FunctionLikeParameterList<'arena>,
) -> Document<'arena, A>
where
    A: Arena,
{
    f.parameter_state.hugged_list_group_id = None;

    if parameter_list.parameters.is_empty() {
        let mut contents = vec_in![f.arena; Document::String(b"(")];
        if let Some(comments) = f.print_inner_comment(parameter_list.span()) {
            contents.push(comments);
        }

        contents.push(Document::String(b")"));

        return Document::Array(contents);
    }

    let mut force_break = force_break_parameters(f, parameter_list);
    let preserve_break = preserve_break_parameters(f, parameter_list);
    let should_break = force_break || preserve_break;

    let list_id = f.next_id();
    let previous_list_group_id = f.parameter_state.list_group_id;
    f.parameter_state.list_group_id = Some(list_id);

    let should_hug_the_parameters = !force_break && should_hug_the_only_parameter(f, parameter_list);
    let variable_column = should_align_parameters(f, parameter_list).then(|| Column {
        table: f.next_id(),
        index: 1,
        shift: 0,
        gutter: u8::from(needs_promoted_parameter_type_gutter(parameter_list)),
        adjacent: false,
    });

    let left_parenthesis = {
        let mut contents = vec_in![f.arena; Document::String(b"(")];

        if !should_break
            && parameter_list.parameters.len() == 1
            && let Some(parameter) = parameter_list.parameters.first()
            && let Some(inline_doc) =
                f.collect_inline_block_comments_between(parameter_list.left_parenthesis, parameter.span())
        {
            contents.push(inline_doc);
        }

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

    let mut parts = vec_in![f.arena; left_parenthesis];

    let mut printed = vec_in![f.arena; ];
    let len = parameter_list.parameters.len();
    for (i, parameter) in parameter_list.parameters.iter().enumerate() {
        f.parameter_state.variable_column = variable_column;

        printed.push(parameter.format(f));
        if i == len - 1 {
            break;
        }

        printed.push(Document::String(b","));
        printed.push(Document::Line(Line::default()));

        if f.is_next_line_empty(parameter.span()) {
            printed.push(Document::BreakParent);
            printed.push(Document::Line(Line::hard()));
        }
    }
    f.parameter_state.variable_column = None;
    f.parameter_state.list_group_id = previous_list_group_id;
    f.parameter_state.hugged_list_group_id = should_hug_the_parameters.then_some(list_id);

    if should_hug_the_parameters {
        let line_if_broken =
            || Document::IfBreak(IfBreak::then(f.arena, Document::Line(Line::soft())).with_id(list_id));

        let mut contents = vec_in![f.arena; line_if_broken()];
        contents.extend(printed);

        let mut hugged = vec_in![f.arena;
            Document::Group(Group::new(parts).with_id(list_id).with_break_mode(if preserve_break {
                BreakMode::Preserve
            } else {
                BreakMode::Auto
            })),
            Document::IndentIfBreak(IndentIfBreak::new(list_id, contents)),
        ];

        if f.trailing_comma_for(Feature::TrailingCommaInParameterList) {
            hugged.push(Document::IfBreak(IfBreak::then(f.arena, Document::String(b",")).with_id(list_id)));
        }

        hugged.push(line_if_broken());
        hugged.push(Document::String(b")"));

        return Document::Array(hugged);
    }

    if !parameter_list.parameters.is_empty() {
        let mut contents = vec_in![f.arena; Document::Line(Line::soft())];
        contents.extend(printed);
        parts.push(Document::Indent(contents));

        if f.trailing_comma_for(Feature::TrailingCommaInParameterList) {
            parts.push(Document::IfBreak(IfBreak::then(f.arena, Document::String(b","))));
        }
    }

    if let Some(comments) =
        f.print_dangling_comments(parameter_list.left_parenthesis.join(parameter_list.right_parenthesis), true)
    {
        parts.push(comments);
    } else {
        parts.push(Document::Line(Line::soft()));
    }

    parts.push(Document::String(b")"));

    Document::Group(Group::new(parts).with_id(list_id).with_break_mode(if force_break {
        BreakMode::Force
    } else if preserve_break {
        BreakMode::Preserve
    } else {
        BreakMode::Auto
    }))
}

pub(super) fn force_break_parameters<'arena, A>(
    f: &FormatterState<'_, 'arena, A>,
    parameter_list: &'arena FunctionLikeParameterList<'arena>,
) -> bool
where
    A: Arena,
{
    f.settings.break_promoted_properties_list
        && parameter_list.parameters.iter().any(FunctionLikeParameter::is_promoted_property)
}

pub(super) fn preserve_break_parameters<'arena, A>(
    f: &FormatterState<'_, 'arena, A>,
    parameter_list: &'arena FunctionLikeParameterList<'arena>,
) -> bool
where
    A: Arena,
{
    f.settings.preserve_breaking_parameter_list
        && !parameter_list.parameters.is_empty()
        && misc::has_new_line_in_range(
            f.source_text,
            parameter_list.left_parenthesis.start.offset,
            parameter_list.parameters.as_slice()[0].span().start.offset,
        )
}

pub(super) fn should_hug_the_only_parameter<'arena, A>(
    f: &FormatterState<'_, 'arena, A>,
    parameter_list: &'arena FunctionLikeParameterList<'arena>,
) -> bool
where
    A: Arena,
{
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

fn should_align_parameters<'arena, A>(
    f: &FormatterState<'_, 'arena, A>,
    parameter_list: &'arena FunctionLikeParameterList<'arena>,
) -> bool
where
    A: Arena,
{
    f.settings.align_parameters
        && parameter_list.parameters.len() >= 2
        && parameter_list
            .parameters
            .iter()
            .enumerate()
            .all(|(index, parameter)| can_align_parameter(f, parameter_list, index, parameter))
}

fn needs_promoted_parameter_type_gutter<'arena>(parameter_list: &'arena FunctionLikeParameterList<'arena>) -> bool {
    parameter_list.parameters.iter().any(|parameter| parameter.is_promoted_property() && parameter.hint.is_none())
}

fn can_align_parameter<'arena, A>(
    f: &FormatterState<'_, 'arena, A>,
    parameter_list: &'arena FunctionLikeParameterList<'arena>,
    index: usize,
    parameter: &'arena FunctionLikeParameter<'arena>,
) -> bool
where
    A: Arena,
{
    let gap_start = if index == 0 {
        parameter_list.left_parenthesis.end.offset
    } else {
        parameter_list.parameters.as_slice()[index - 1].end_offset()
    };

    !has_comment_trivia_in_range(f, gap_start, parameter.start_offset())
        && !has_comment_trivia_in_range(f, parameter.start_offset(), parameter.variable.start_offset())
}

fn has_comment_trivia_in_range<A>(f: &FormatterState<'_, '_, A>, start: u32, end: u32) -> bool
where
    A: Arena,
{
    if start >= end {
        return false;
    }

    f.all_comments().iter().any(|comment| {
        let comment_span = comment.span;
        comment_span.start.offset < end && comment_span.end.offset > start
    })
}
