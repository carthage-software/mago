/// Wrap a static string (ss)
#[macro_export]
macro_rules! static_str {
    ($s:expr) => {{
        $crate::document::Document::String($s)
    }};
}

#[macro_export]
macro_rules! token {
    ($f:ident, $s:expr, $v:expr) => {{
        let leading_comments = $f.print_leading_comments($s);
        let trailing_comments = $f.print_trailing_comments($s);
        $f.print_comments(leading_comments, $crate::static_str!($v), trailing_comments)
    }};
}

#[macro_export]
macro_rules! space {
    () => {{
        $crate::document::Document::String(" ")
    }};
}

#[macro_export]
macro_rules! empty_string {
    () => {{
        $crate::document::Document::String("")
    }};
}

#[macro_export]
macro_rules! string {
    ($p:ident, $s:expr) => {{
        $p.str($s)
    }};
}

#[macro_export]
macro_rules! indent {
    ($( $x:expr ),* $(,)?) => {
        {
            let mut v = vec![];
            $(
                v.push($x);
            )*
            $crate::document::Document::Indent(v)
        }
    };
    (@$contents:expr) => {
        $crate::document::Document::Indent($contents.into())
    };
}

#[macro_export]
macro_rules! indent_if_break {
    ($( $x:expr ),* $(,)?) => {
        {
            let mut v = vec![];
            $(
                v.push($x);
            )*

            $crate::indent_if_break!(@v)
        }
    };
    (@$contents:expr) => {
        $crate::indent_if_break!(@$contents, group_id: None)
    };
    (@$contents:expr, group_id: $group_id:expr) => {
        $crate::document::Document::IndentIfBreak($crate::document::IndentIfBreak {
            contents: $contents,
            group_id: $group_id,
        })
    };
}

#[macro_export]
macro_rules! default_line {
    () => {
        $crate::document::Document::Line($crate::document::Line::default())
    };
}

#[macro_export]
macro_rules! softline {
    () => {
        $crate::document::Document::Line($crate::document::Line::softline())
    };
}

#[macro_export]
macro_rules! hardline {
    () => {
        [$crate::document::Document::Line($crate::document::Line::hardline()), $crate::document::Document::BreakParent]
    };
}

#[macro_export]
macro_rules! array {
    ($( $x:expr ),* $(,)?) => {
        {
            let mut temp_vec = vec![];
            $(
                temp_vec.push($x);
            )*

            $crate::array!(@temp_vec)
        }
    };
    (@$content:expr) => {
        $crate::document::Document::Array($content.into())
    };
}

#[macro_export]
macro_rules! group {
    ($( $x:expr ),* $(,)?) => {
        {
            let mut v = vec![];
            $(
                v.push($x);
            )*
            $crate::document::Document::Group($crate::document::Group::new(v))
        }
    };
    (@$contents:expr) => {
        $crate::document::Document::Group($crate::document::Group::new($contents))
    };
    (@$contents:expr, with_break: $with_break:expr) => {
        $crate::document::Document::Group($crate::document::Group::new($contents).with_break(
            $with_break
        ))
    };
    (@$contents:expr, #$id:expr) => {
        $crate::document::Document::Group($crate::document::Group::new($contents).with_id(
            $id
        ))
    };
    ($f:ident, @$contents:expr) => {{
        let group_id = $f.next_id();
        let document = $crate::document::Document::Group($crate::document::Group::new($contents).with_id(group_id));

        (group_id, document)
    }};
}

#[macro_export]
macro_rules! conditional_group {
    ($p:ident, $c: expr, $( $x:expr ),* $(,)?) => {
        {
            let contents = vec![$c];
            let mut temp_vec = vec![];
                $(
                    temp_vec.push($x);
                )*

            $crate::document::Document::Group($crate::document::Group::new_conditional_group(contents, temp_vec))
        }
    };
}

#[macro_export]
macro_rules! group_break {
    ($p:ident, $( $x:expr ),* $(,)?) => {
        {
            let mut temp_vec = vec![];
            $(
                temp_vec.push($x);
            )*
            $crate::document::Document::Group($crate::document::Group::new(temp_vec).with_break(true))
        }
    };
    (@$content:expr) => {
        $crate::document::Document::Group($crate::document::Group::new($content.into()).with_break(true))
    };
}

#[macro_export]
macro_rules! if_break {
    ($s:expr, $flat:expr, $group_id:expr) => {{
        $crate::document::Document::IfBreak($crate::document::IfBreak {
            break_contents: Box::new($s),
            flat_content: Box::new($flat),
            group_id: $group_id,
        })
    }};
    ($s:expr, $flat:expr) => {{
        $crate::if_break!($s, $flat, None)
    }};
    ($s:expr) => {{
        $crate::if_break!($s, $crate::empty_string!(), None)
    }};
}

#[macro_export]
macro_rules! line_suffix {
    ($p:ident, $( $x:expr ),* $(,)?) => {
        {
            let mut temp_vec = $p.vec();
            $(
                temp_vec.push($x);
            )*
            $crate::document::Document::LineSuffix(temp_vec)
        }
    };
}

#[macro_export]
macro_rules! parenthesized {
    ($doc:expr) => {
        $crate::group!(
            // Add the opening token
            $crate::static_str!("("),
            $crate::indent_if_break!(
                // Insert a line break before contents if the group breaks
                $crate::if_break!(default_line!()),
                // Include the document
                $doc
            ),
            // Handle line break before the closing token
            $crate::if_break!($crate::default_line!()),
            // Add the closing token
            $crate::static_str!(")"),
        )
    };
}

#[macro_export]
macro_rules! bracketed {
    ($doc:expr) => {
        $crate::group!(
            // Add the opening token
            $crate::static_str!("["),
            $crate::indent_if_break!(
                // Insert a line break before contents if the group breaks
                $crate::if_break!(default_line!()),
                // Include the document
                $doc
            ),
            // Handle line break before the closing token
            $crate::if_break!($crate::default_line!()),
            // Add the closing token
            $crate::static_str!("]"),
        )
    };
}

#[macro_export]
macro_rules! braced {
    ($doc:expr) => {
        $crate::group!(
            // Add the opening token
            $crate::static_str!("{"),
            $crate::indent_if_break!(
                // Insert a line break before contents if the group breaks
                $crate::if_break!(default_line!()),
                // Include the document
                $doc
            ),
            // Handle line break before the closing token
            $crate::if_break!($crate::default_line!()),
            // Add the closing token
            $crate::static_str!("}"),
        )
    };
}

#[macro_export]
macro_rules! wrap {
    ($f:ident, $self:expr, $node:ident, $block:block) => {{
        let node = fennec_ast::Node::$node($self);
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
