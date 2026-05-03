use foldhash::HashMap;

use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Conditional;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Node;
use mago_syntax::ast::Trivia;

use crate::internal::utils::unwrap_parenthesized;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentLinePosition {
    OwnLine,
    EndOfLine,
}

impl CommentLinePosition {
    pub fn at_offset(source_text: &str, offset: u32) -> Self {
        for &byte in source_text.as_bytes()[..offset as usize].iter().rev() {
            match byte {
                b'\n' | b'\r' => return Self::OwnLine,
                b' ' | b'\t' => {}
                _ => return Self::EndOfLine,
            }
        }

        Self::OwnLine
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    Leading,
    Trailing,
}

#[derive(Debug, Clone, Copy)]
pub struct PlacedComment {
    pub index: usize,
    pub placement: Placement,
    pub line_position: CommentLinePosition,
}

pub struct DecoratedComment<'ast, 'arena> {
    pub enclosing: Node<'ast, 'arena>,
    pub line_position: CommentLinePosition,
    pub comment_index: usize,
    pub trivia_start: u32,
}

pub enum CommentPlacement<'ast, 'arena> {
    Leading { node: Node<'ast, 'arena>, comment_index: usize },
    Trailing { node: Node<'ast, 'arena>, comment_index: usize },
    Default,
}

pub struct Comments {
    map: HashMap<Span, Vec<PlacedComment>>,
    placed: Vec<Option<Placement>>,
    consumed: Vec<bool>,
}

impl std::fmt::Debug for Comments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Comments").finish()
    }
}

impl Comments {
    pub fn new(total: usize) -> Self {
        Self { map: HashMap::default(), placed: vec![None; total], consumed: vec![false; total] }
    }

    pub fn empty() -> Self {
        Self { map: HashMap::default(), placed: Vec::new(), consumed: Vec::new() }
    }

    pub fn insert(&mut self, node_span: Span, comment: PlacedComment) {
        self.map.entry(node_span).or_default().push(comment);
    }

    pub fn get(&self, span: Span) -> &[PlacedComment] {
        self.map.get(&span).map_or(&[], |v| v.as_slice())
    }

    pub fn by_placement(&self, span: Span, placement: Placement) -> impl Iterator<Item = &PlacedComment> {
        self.get(span).iter().filter(move |c| c.placement == placement)
    }

    pub fn is_placed_leading(&self, index: usize) -> bool {
        self.placed.get(index).copied().flatten() == Some(Placement::Leading)
    }

    pub fn is_placed_trailing(&self, index: usize) -> bool {
        self.placed.get(index).copied().flatten() == Some(Placement::Trailing)
    }

    fn mark_placed(&mut self, index: usize, placement: Placement) {
        if let Some(v) = self.placed.get_mut(index) {
            *v = Some(placement);
        }
    }

    pub fn is_consumed(&self, index: usize) -> bool {
        self.consumed.get(index).copied().unwrap_or(false)
    }

    pub fn mark_consumed(&mut self, index: usize) {
        if let Some(consumed) = self.consumed.get_mut(index) {
            *consumed = true;
        }
    }
}

pub fn place_comments<'arena>(
    source_text: &str,
    root: Node<'_, 'arena>,
    all_comments: &'arena [Trivia<'arena>],
) -> Comments {
    let total = all_comments.len();
    if total == 0 {
        return Comments::empty();
    }

    let mut comments = Comments::new(total);
    let mut cursor: usize = 0;

    collect_node_comments(source_text, root, all_comments, &mut comments, &mut cursor);

    while cursor < total {
        let decorated = make_decorated(source_text, all_comments, cursor, root);
        place_and_insert(decorated, &mut comments);
        cursor += 1;
    }

    comments
}

fn collect_node_comments<'ast, 'arena>(
    source_text: &str,
    node: Node<'ast, 'arena>,
    all_comments: &'arena [Trivia<'arena>],
    comments: &mut Comments,
    cursor: &mut usize,
) {
    let total = all_comments.len();
    if *cursor >= total {
        return;
    }

    let node_span = node.span();
    if all_comments[*cursor].span.start.offset >= node_span.end.offset {
        return;
    }

    let children = node.children();

    if children.is_empty() {
        while *cursor < total {
            let comment_start = all_comments[*cursor].span.start.offset;

            if comment_start < node_span.start.offset || comment_start >= node_span.end.offset {
                break;
            }

            let decorated = make_decorated(source_text, all_comments, *cursor, node);
            place_and_insert(decorated, comments);
            *cursor += 1;
        }

        return;
    }

    let can_reattribute = !is_binary_like(node);
    let mut prev_child: Option<Node<'ast, 'arena>> = None;
    for child in children.iter() {
        let child = unwrap_parenthesized_node(*child);
        let child_start = child.span().start.offset;

        while *cursor < total {
            let comment_start = all_comments[*cursor].span.start.offset;

            if comment_start < node_span.start.offset || comment_start >= child_start {
                break;
            }

            let enclosing = if can_reattribute
                && let Some(prev) = prev_child.filter(|p| comment_start >= p.span().end.offset && is_binary_like(*p))
            {
                prev
            } else {
                node
            };
            let decorated = make_decorated(source_text, all_comments, *cursor, enclosing);
            place_and_insert(decorated, comments);
            *cursor += 1;
        }

        collect_node_comments(source_text, child, all_comments, comments, cursor);
        prev_child = Some(child);
    }

    while *cursor < total {
        let comment_start = all_comments[*cursor].span.start.offset;

        if comment_start < node_span.start.offset || comment_start >= node_span.end.offset {
            break;
        }

        let enclosing = if can_reattribute
            && let Some(prev) = prev_child.filter(|p| comment_start >= p.span().end.offset && is_binary_like(*p))
        {
            prev
        } else {
            node
        };

        let decorated = make_decorated(source_text, all_comments, *cursor, enclosing);
        place_and_insert(decorated, comments);
        *cursor += 1;
    }
}

fn is_binary_like(node: Node<'_, '_>) -> bool {
    match node {
        Node::Binary(_) | Node::Conditional(_) => true,
        Node::Expression(expression) => {
            matches!(unwrap_parenthesized(expression), Expression::Binary(_) | Expression::Conditional(_))
        }
        Node::Parenthesized(paren) => {
            matches!(unwrap_parenthesized(paren.expression), Expression::Binary(_) | Expression::Conditional(_))
        }
        _ => false,
    }
}

fn make_decorated<'ast, 'arena>(
    source_text: &str,
    all_comments: &'arena [Trivia<'arena>],
    cursor: usize,
    enclosing: Node<'ast, 'arena>,
) -> DecoratedComment<'ast, 'arena> {
    let trivia = &all_comments[cursor];

    DecoratedComment {
        enclosing,
        line_position: CommentLinePosition::at_offset(source_text, trivia.span.start.offset),
        comment_index: cursor,
        trivia_start: trivia.span.start.offset,
    }
}

fn place_and_insert(decorated: DecoratedComment<'_, '_>, comments: &mut Comments) {
    let line_position = decorated.line_position;

    let (node_span, placement, index) = match place_comment(decorated) {
        CommentPlacement::Leading { node, comment_index } => (node.span(), Placement::Leading, comment_index),
        CommentPlacement::Trailing { node, comment_index } => (node.span(), Placement::Trailing, comment_index),
        CommentPlacement::Default => return,
    };

    comments.insert(node_span, PlacedComment { index, placement, line_position });
    comments.mark_placed(index, placement);
}

/// Skip Parenthesized wrappers to match the formatter's strip behavior.
/// The formatter strips `Expression::Parenthesized` and re-adds parens via
/// `need_parens`. DFS must see the same structure as the formatted output
/// to keep placement stable across passes.
fn unwrap_parenthesized_node<'ast, 'arena>(node: Node<'ast, 'arena>) -> Node<'ast, 'arena> {
    match node {
        Node::Parenthesized(p) => unwrap_parenthesized_node(Node::Expression(p.expression)),
        Node::Expression(Expression::Parenthesized(p)) => unwrap_parenthesized_node(Node::Expression(p.expression)),
        _ => node,
    }
}

fn place_comment<'ast, 'arena>(comment: DecoratedComment<'ast, 'arena>) -> CommentPlacement<'ast, 'arena> {
    let enclosing = match comment.enclosing {
        Node::Expression(expr) => match unwrap_parenthesized(expr) {
            Expression::Binary(binary) => Node::Binary(binary),
            Expression::Conditional(cond) => Node::Conditional(cond),
            _ => comment.enclosing,
        },
        other => other,
    };

    let comment = DecoratedComment { enclosing, ..comment };
    match comment.enclosing {
        Node::Binary(_) => place_binary(comment),
        Node::Conditional(_) => place_conditional(comment),
        _ => CommentPlacement::Default,
    }
}

fn place_binary<'ast, 'arena>(comment: DecoratedComment<'ast, 'arena>) -> CommentPlacement<'ast, 'arena> {
    let Node::Binary(binary) = comment.enclosing else {
        return CommentPlacement::Default;
    };

    let index = comment.comment_index;
    let cs = comment.trivia_start;
    let lhs = unwrap_parenthesized(binary.lhs);
    let rhs = unwrap_parenthesized(binary.rhs);
    let lhs_span = lhs.span();
    let rhs_span = rhs.span();
    let op_span = binary.operator.span();

    // Handles comments exposed before LHS by DFS parenthesized transparency.
    if cs < lhs_span.start.offset {
        return CommentPlacement::Leading { node: Node::Expression(lhs), comment_index: index };
    }

    if cs >= lhs_span.end.offset && cs < op_span.start.offset {
        return CommentPlacement::Trailing { node: Node::Expression(lhs), comment_index: index };
    }

    if cs >= op_span.end.offset && cs < rhs_span.start.offset {
        return CommentPlacement::Leading { node: Node::Expression(rhs), comment_index: index };
    }

    if cs >= rhs_span.end.offset {
        return CommentPlacement::Trailing { node: Node::Expression(rhs), comment_index: index };
    }

    CommentPlacement::Default
}

fn place_conditional<'ast, 'arena>(comment: DecoratedComment<'ast, 'arena>) -> CommentPlacement<'ast, 'arena> {
    let Node::Conditional(Conditional { condition, question_mark, then, colon, r#else }) = comment.enclosing else {
        return CommentPlacement::Default;
    };

    let index = comment.comment_index;
    let cs = comment.trivia_start;
    let condition = unwrap_parenthesized(condition);
    let condition_span = condition.span();
    let r#else = unwrap_parenthesized(r#else);
    let else_span = r#else.span();

    // Handles comments exposed before condition by DFS parenthesized transparency.
    if cs < condition_span.start.offset {
        return CommentPlacement::Leading { node: Node::Expression(condition), comment_index: index };
    }

    match then {
        Some(then) => {
            let then = unwrap_parenthesized(then);
            let then_span = then.span();

            if cs >= condition_span.end.offset && cs < question_mark.start.offset {
                return CommentPlacement::Trailing { node: Node::Expression(condition), comment_index: index };
            }

            if cs >= question_mark.end.offset && cs < then_span.start.offset {
                return CommentPlacement::Leading { node: Node::Expression(then), comment_index: index };
            }

            if cs >= then_span.end.offset && cs < colon.start.offset {
                return CommentPlacement::Trailing { node: Node::Expression(then), comment_index: index };
            }

            if cs >= colon.end.offset && cs < else_span.start.offset {
                return CommentPlacement::Leading { node: Node::Expression(r#else), comment_index: index };
            }

            if cs >= else_span.end.offset {
                return CommentPlacement::Trailing { node: Node::Expression(r#else), comment_index: index };
            }
        }
        None => {
            // Elvis (`?:`): condition and else only, no then branch.
            if cs >= condition_span.end.offset && cs < question_mark.start.offset {
                return CommentPlacement::Trailing { node: Node::Expression(condition), comment_index: index };
            }

            // Between `?` and `:` or after `:` but before else: treat as else leading,
            // matching binary's "after operator" convention.
            if cs >= question_mark.start.offset && cs < else_span.start.offset {
                return CommentPlacement::Leading { node: Node::Expression(r#else), comment_index: index };
            }

            if cs >= else_span.end.offset {
                return CommentPlacement::Trailing { node: Node::Expression(r#else), comment_index: index };
            }
        }
    }

    CommentPlacement::Default
}
