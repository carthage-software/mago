use mago_ast::*;
use mago_span::Span;

use crate::document::Document;
use crate::internal::FormatterState;
use crate::internal::format::block::print_block_of_nodes;
use crate::settings::BraceStyle;

pub fn print_class_like_body<'a>(
    f: &mut FormatterState<'a>,
    left_brace: &'a Span,
    class_like_members: &'a Sequence<ClassLikeMember>,
    right_brace: &'a Span,
) -> Document<'a> {
    let inline_empty = match f.settings.classlike_brace_style {
        BraceStyle::SameLine => true,
        BraceStyle::NextLine => false,
    };

    print_block_of_nodes(f, left_brace, class_like_members, right_brace, inline_empty)
}
