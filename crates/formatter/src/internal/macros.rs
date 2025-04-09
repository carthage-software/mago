#[macro_export]
macro_rules! wrap {
    ($f:ident, $self:expr, $node:ident, $block:block) => {{
        let node = mago_syntax::ast::Node::$node($self);
        $f.enter_node(node);
        let leading = $f.print_leading_comments(node.span());
        let doc = $block;
        let doc = $f.wrap_parens(doc, node);
        let trailing = $f.print_trailing_comments(node.span());
        let doc = $f.print_comments(leading, doc, trailing);
        $f.leave_node();
        doc
    }};
}
