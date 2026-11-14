use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_span::{HasSpan, Span};
use mago_twig_syntax::cst::{BlockBody, ElseBranch, Expression, IfBranch, SetBody, Statement, Template};
use mago_twig_syntax::error::ParseError;
use mago_twig_syntax::walker::{Walker, walk_statement};

use crate::safety::{ExpressionSafety, ambiguous_name, preserves_cache_options, preserves_header_names};
use crate::tag::{CodeTag, Tag, TagContents};

pub(super) struct TagCollection<'arena, A: Arena> {
    pub tags: Vec<'arena, Tag<'arena>, A>,
    pub statements: Vec<'arena, Span, A>,
    pub has_legacy_raw: bool,
}

impl<'arena, A: Arena> TagCollection<'arena, A> {
    pub fn collect(
        arena: &'arena A,
        template: &'arena Template<'arena>,
        track_statements: bool,
    ) -> Result<Self, ParseError<'arena>> {
        let mut collector = Collector {
            result: Self { tags: Vec::new_in(arena), statements: Vec::new_in(arena), has_legacy_raw: false },
            safety: ExpressionSafety::new(arena),
            source: template.source_text,
            depth: 0,
            error: None,
            track_statements,
        };
        Visit.walk_template(template, &mut collector);
        if let Some(span) = collector.error {
            return Err(ParseError::RecursionLimitExceeded(span));
        }

        Ok(collector.result)
    }
}

struct Collector<'arena, A: Arena> {
    result: TagCollection<'arena, A>,
    safety: ExpressionSafety<'arena, A>,
    source: &'arena [u8],
    depth: u16,
    error: Option<Span>,
    track_statements: bool,
}

impl<'arena, A: Arena> Collector<'arena, A> {
    fn push(&mut self, open: Span, close: Span, kind: TagContents<'arena>) {
        let cache_is_complete = match kind {
            TagContents::Cache(cache) => preserves_cache_options(self.source, cache),
            _ => true,
        };
        let tag = if preserves_header_names(kind, self.source) && cache_is_complete {
            Tag::Code(CodeTag { open, close, contents: kind })
        } else {
            Tag::Preserved(open.join(close))
        };
        self.result.tags.push(tag);
    }

    fn keyword(&mut self, open: Span, close: Span, name: Option<&[u8]>) {
        if name.is_some_and(ambiguous_name) {
            self.result.tags.push(Tag::Preserved(open.join(close)));
        } else {
            self.push(open, close, TagContents::Keyword);
        }
    }
}

struct Visit;

impl<'arena, A: Arena> Walker<'arena, 'arena, Collector<'arena, A>> for Visit {
    fn walk_statement(&self, statement: &'arena Statement<'arena>, context: &mut Collector<'arena, A>) {
        if context.depth >= crate::NESTING_LIMIT {
            context.error = Some(statement.span());
            return;
        }

        if context.track_statements && !matches!(statement, Statement::Text(_)) {
            context.result.statements.push(statement.span());
        }

        context.depth += 1;
        walk_statement(self, statement, context);
        context.depth -= 1;
    }

    fn walk_in_statement(&self, statement: &'arena Statement<'arena>, context: &mut Collector<'arena, A>) {
        macro_rules! opening {
            ($node:expr, $kind:ident) => {
                context.push($node.open_tag, $node.close_tag, TagContents::$kind($node))
            };
        }

        match statement {
            Statement::Text(_) | Statement::If(_) => {}
            Statement::Print(node) => {
                context.push(node.open_variable, node.close_variable, TagContents::Print(&node.expression))
            }
            Statement::Set(node) => {
                let close = match &node.body {
                    SetBody::Inline(body) => body.close_tag,
                    SetBody::Capture(body) => body.close_tag,
                };
                context.push(node.open_tag, close, TagContents::Set(node));
            }
            Statement::Block(node) => {
                let close = match &node.body {
                    BlockBody::Short(body) => body.close_tag,
                    BlockBody::Long(body) => body.close_tag,
                };
                context.push(node.open_tag, close, TagContents::Block(node));
            }
            Statement::For(node) => opening!(node, For),
            Statement::Verbatim(node) => {
                context.result.has_legacy_raw |= node.keyword.value == b"raw";
                opening!(node, Verbatim);
            }
            Statement::Extends(node) => opening!(node, Extends),
            Statement::Use(node) => opening!(node, Use),
            Statement::Include(node) => opening!(node, Include),
            Statement::Embed(node) => opening!(node, Embed),
            Statement::Import(node) => opening!(node, Import),
            Statement::From(node) => opening!(node, From),
            Statement::Macro(node) => opening!(node, Macro),
            Statement::With(node) => opening!(node, With),
            Statement::Apply(node) => opening!(node, Apply),
            Statement::Autoescape(node) => opening!(node, Autoescape),
            Statement::Sandbox(node) => opening!(node, Sandbox),
            Statement::Deprecated(node) => opening!(node, Deprecated),
            Statement::Do(node) => opening!(node, Do),
            Statement::Flush(node) => opening!(node, Flush),
            Statement::Guard(node) => opening!(node, Guard),
            Statement::Cache(node) => opening!(node, Cache),
            Statement::Types(node) => opening!(node, Types),
            Statement::Unknown(node) => context.result.tags.push(Tag::Preserved(node.open_tag.join(node.close_tag))),
        }
    }

    fn walk_out_statement(&self, statement: &'arena Statement<'arena>, context: &mut Collector<'arena, A>) {
        macro_rules! closing {
            ($node:expr) => {
                context.keyword($node.end_open_tag, $node.end_close_tag, None)
            };
        }

        match statement {
            Statement::If(node) => closing!(node),
            Statement::For(node) => closing!(node),
            Statement::Verbatim(node) => closing!(node),
            Statement::Embed(node) => closing!(node),
            Statement::Macro(node) => {
                context.keyword(node.end_open_tag, node.end_close_tag, node.end_name.map(|name| name.value));
            }
            Statement::With(node) => closing!(node),
            Statement::Apply(node) => closing!(node),
            Statement::Autoescape(node) => closing!(node),
            Statement::Sandbox(node) => closing!(node),
            Statement::Guard(node) => closing!(node),
            Statement::Cache(node) => closing!(node),
            Statement::Set(node) => {
                if let SetBody::Capture(body) = &node.body {
                    closing!(body);
                }
            }
            Statement::Block(node) => {
                if let BlockBody::Long(body) = &node.body {
                    context.keyword(body.end_open_tag, body.end_close_tag, body.end_name.map(|name| name.value));
                }
            }
            _ => {}
        }
    }

    fn walk_in_if_branch(&self, branch: &'arena IfBranch<'arena>, context: &mut Collector<'arena, A>) {
        context.push(branch.open_tag, branch.close_tag, TagContents::If(branch));
    }

    fn walk_in_else_branch(&self, branch: &'arena ElseBranch<'arena>, context: &mut Collector<'arena, A>) {
        context.push(branch.open_tag, branch.close_tag, TagContents::Keyword);
    }

    fn walk_expression(&self, expression: &'arena Expression<'arena>, context: &mut Collector<'arena, A>) {
        if !context.safety.can_format(expression)
            && let Some(tag) = context.result.tags.last_mut()
        {
            let span = tag.span();
            debug_assert!(
                span.start <= expression.span().start && expression.span().end <= span.end,
                "Header expressions must be visited before the tag body"
            );
            *tag = Tag::Preserved(span);
        }
    }
}
