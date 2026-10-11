#[macro_export]
macro_rules! wrap {
    ($f:ident, $self:expr, $node:ident, $block:block) => {{
        let node = mago_syntax::cst::Node::$node($self);
        $f.enter_node(node);
        let mut span = $f.needs_comment_span().then(|| node.span());

        let was_wrapped_in_parens = $f.is_wrapped_in_parens;
        let needed_to_wrap_in_parens = $f.need_parens(node);
        $f.is_wrapped_in_parens |= needed_to_wrap_in_parens;
        let leading = span.and_then(|span| $f.print_leading_comments_for_node(node, span));
        let doc = $block;
        if span.is_none() && $f.needs_comment_span() {
            span = Some(node.span());
        }
        let trailing = span.and_then(|span| $f.print_trailing_comments(span));
        let has_leading_comments = leading.is_some();
        let doc = if needed_to_wrap_in_parens {
            let doc = $f.print_comments(leading, doc, trailing);
            if let Some(span) = span {
                $f.add_parens_with_placed_leading(doc, node, span, has_leading_comments)
            } else {
                $f.add_parens(doc, node, has_leading_comments)
            }
        } else {
            let doc = if let Some(span) = span { $f.prepend_unclaimed_placed_leading(span, doc) } else { doc };
            $f.print_comments(leading, doc, trailing)
        };
        $f.leave_node();
        $f.is_wrapped_in_parens = was_wrapped_in_parens;
        doc
    }};
}
