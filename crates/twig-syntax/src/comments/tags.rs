use mago_span::Span;

use crate::cst::BlockBody;
use crate::cst::SetBody;
use crate::cst::Statement;

pub(super) fn collect(statement: &Statement<'_>, tags: &mut Vec<Span>) {
    macro_rules! paired {
        ($node:expr) => {{
            tags.push($node.end_open_tag.join($node.end_close_tag));
            ($node.open_tag, $node.close_tag)
        }};
    }

    let (open, close) = match statement {
        Statement::Text(_) => return,
        Statement::Print(node) => (node.open_variable, node.close_variable),
        Statement::If(node) => {
            for branch in &node.branches {
                tags.push(branch.open_tag.join(branch.close_tag));
            }
            if let Some(branch) = &node.else_branch {
                tags.push(branch.open_tag.join(branch.close_tag));
            }
            (node.end_open_tag, node.end_close_tag)
        }
        Statement::For(node) => {
            if let Some(branch) = &node.else_branch {
                tags.push(branch.open_tag.join(branch.close_tag));
            }
            paired!(node)
        }
        Statement::Set(node) => match &node.body {
            SetBody::Inline(body) => (node.open_tag, body.close_tag),
            SetBody::Capture(body) => {
                tags.push(body.end_open_tag.join(body.end_close_tag));
                (node.open_tag, body.close_tag)
            }
        },
        Statement::Block(node) => match &node.body {
            BlockBody::Short(body) => (node.open_tag, body.close_tag),
            BlockBody::Long(body) => {
                tags.push(body.end_open_tag.join(body.end_close_tag));
                (node.open_tag, body.close_tag)
            }
        },
        Statement::Guard(node) => {
            if let Some(branch) = &node.else_branch {
                tags.push(branch.open_tag.join(branch.close_tag));
            }
            paired!(node)
        }
        Statement::Verbatim(node) => paired!(node),
        Statement::Macro(node) => paired!(node),
        Statement::Embed(node) => paired!(node),
        Statement::With(node) => paired!(node),
        Statement::Apply(node) => paired!(node),
        Statement::Autoescape(node) => paired!(node),
        Statement::Sandbox(node) => paired!(node),
        Statement::Cache(node) => paired!(node),
        Statement::Extends(node) => (node.open_tag, node.close_tag),
        Statement::Use(node) => (node.open_tag, node.close_tag),
        Statement::Include(node) => (node.open_tag, node.close_tag),
        Statement::Import(node) => (node.open_tag, node.close_tag),
        Statement::From(node) => (node.open_tag, node.close_tag),
        Statement::Deprecated(node) => (node.open_tag, node.close_tag),
        Statement::Do(node) => (node.open_tag, node.close_tag),
        Statement::Flush(node) => (node.open_tag, node.close_tag),
        Statement::Types(node) => (node.open_tag, node.close_tag),
        Statement::Unknown(node) => (node.open_tag, node.close_tag),
    };
    tags.push(open.join(close));
}
