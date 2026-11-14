use std::cmp::Reverse;

use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::BlockBody;
use crate::cst::SetBody;
use crate::cst::Statement;
use crate::cst::Template;

/// Indexes code-tag bounds and following statements for comment scopes.
#[derive(Debug)]
pub struct CommentScopes {
    statements: Vec<Span>,
    content: Vec<(u32, u32)>,
    tags: Vec<Span>,
}

impl CommentScopes {
    #[must_use]
    pub fn new(template: &Template<'_>) -> Self {
        let mut tags = Vec::new();
        let statements = statement_spans(template, &mut tags);
        tags.sort_unstable_by_key(|span| span.start);
        Self { statements, content: next_content(template), tags }
    }

    /// Bound an inline comment to the rest of its code tag, not the statement's body or a later tag.
    #[must_use]
    pub fn inline_scope(&self, comment: Span) -> Option<Span> {
        let index = self.tags.partition_point(|span| span.start < comment.start).checked_sub(1)?;
        let tag = self.tags[index];
        tag.contains(&comment).then_some(Span::new(comment.file_id, comment.end, tag.end))
    }

    /// Return the following statement's full span. Scoped pragmas should call this only for own-line comments.
    #[must_use]
    pub fn following_statement(&self, comment: Span) -> Option<Span> {
        let next = self.statements.partition_point(|span| span.start < comment.end);
        let scope = *self.statements.get(next)?;
        if scope.file_id != comment.file_id {
            return None;
        }
        let content = self.content.binary_search_by_key(&comment.end_offset(), |&(end, _)| end).ok()?;
        (self.content[content].1 == scope.start_offset()).then_some(scope)
    }
}

fn next_content(template: &Template<'_>) -> Vec<(u32, u32)> {
    let mut content = Vec::new();
    let mut end = template.source_text.len();
    let mut next = end as u32;
    for trivia in template.trivia.iter().rev().filter(|trivia| trivia.kind.is_comment()) {
        let start = trivia.end_offset() as usize;
        if let Some(offset) = template.source_text[start..end].iter().position(|byte| !byte.is_ascii_whitespace()) {
            next = (start + offset) as u32;
        }
        content.push((start as u32, next));
        end = trivia.start_offset() as usize;
    }
    content.reverse();
    content
}

fn statement_spans(template: &Template<'_>, tags: &mut Vec<Span>) -> Vec<Span> {
    let mut pending: Vec<_> = template.statements.iter().collect();
    let mut spans = Vec::new();
    while let Some(statement) = pending.pop() {
        super::tags::collect(statement, tags);
        if !matches!(statement, Statement::Text(_)) {
            spans.push(statement.span());
        }
        match statement {
            Statement::If(node) => {
                for branch in &node.branches {
                    pending.extend(branch.body.iter());
                }
                if let Some(branch) = &node.else_branch {
                    pending.extend(branch.body.iter());
                }
            }
            Statement::For(node) => {
                pending.extend(node.body.iter());
                if let Some(branch) = &node.else_branch {
                    pending.extend(branch.body.iter());
                }
            }
            Statement::Set(node) => {
                if let SetBody::Capture(body) = &node.body {
                    pending.extend(body.body.iter());
                }
            }
            Statement::Block(node) => {
                if let BlockBody::Long(body) = &node.body {
                    pending.extend(body.body.iter());
                }
            }
            Statement::Macro(node) => pending.extend(node.body.iter()),
            Statement::Embed(node) => pending.extend(node.body.iter()),
            Statement::With(node) => pending.extend(node.body.iter()),
            Statement::Apply(node) => pending.extend(node.body.iter()),
            Statement::Autoescape(node) => pending.extend(node.body.iter()),
            Statement::Sandbox(node) => pending.extend(node.body.iter()),
            Statement::Guard(node) => {
                pending.extend(node.body.iter());
                if let Some(branch) = &node.else_branch {
                    pending.extend(branch.body.iter());
                }
            }
            Statement::Cache(node) => pending.extend(node.body.iter()),
            _ => {}
        }
    }

    spans.sort_unstable_by_key(|span| (span.start, Reverse(span.end)));
    spans
}
