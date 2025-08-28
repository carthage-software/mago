#[macro_export]
macro_rules! wrap {
    ($f:ident, $self:expr, $node:ident, $block:block) => {{
        let node = mago_syntax::ast::Node::$node($self);
        $f.enter_node(node);

        let was_wrapped_in_parens = $f.is_wrapped_in_parens;
        let needed_to_wrap_in_parens = $f.need_parens(node);
        $f.is_wrapped_in_parens |= needed_to_wrap_in_parens;
        let leading = $f.print_leading_comments(node.span());
        let doc = $block;
        let trailing = $f.print_trailing_comments_for_node(node);
        let has_leading_comments = leading.is_some();
        let doc = $f.print_comments(leading, doc, trailing);
        let doc = if needed_to_wrap_in_parens { $f.add_parens(doc, node, has_leading_comments) } else { doc };
        $f.leave_node();
        $f.is_wrapped_in_parens = was_wrapped_in_parens;
        doc
    }};
}
