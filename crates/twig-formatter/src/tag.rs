use mago_span::Span;
use mago_twig_syntax::cst::{
    Apply, Autoescape, Block, Cache, Deprecated, Do, Embed, Expression, Extends, Flush, For, From, Guard, IfBranch,
    Import, Include, Macro, Sandbox, Set, Trivia, TriviaKind, Types, Use, Verbatim, With,
};

#[derive(Clone, Copy)]
pub(super) enum TagContents<'arena> {
    Print(&'arena Expression<'arena>),
    If(&'arena IfBranch<'arena>),
    For(&'arena For<'arena>),
    Set(&'arena Set<'arena>),
    Block(&'arena Block<'arena>),
    Extends(&'arena Extends<'arena>),
    Use(&'arena Use<'arena>),
    Include(&'arena Include<'arena>),
    Embed(&'arena Embed<'arena>),
    Import(&'arena Import<'arena>),
    From(&'arena From<'arena>),
    Macro(&'arena Macro<'arena>),
    With(&'arena With<'arena>),
    Apply(&'arena Apply<'arena>),
    Autoescape(&'arena Autoescape<'arena>),
    Sandbox(&'arena Sandbox<'arena>),
    Deprecated(&'arena Deprecated<'arena>),
    Do(&'arena Do<'arena>),
    Flush(&'arena Flush<'arena>),
    Guard(&'arena Guard<'arena>),
    Cache(&'arena Cache<'arena>),
    Types(&'arena Types<'arena>),
    Verbatim(&'arena Verbatim<'arena>),
    Keyword,
}

#[derive(Clone, Copy)]
pub(super) struct CodeTag<'arena> {
    pub open: Span,
    pub close: Span,
    pub contents: TagContents<'arena>,
}

impl CodeTag<'_> {
    pub fn span(self) -> Span {
        self.open.join(self.close)
    }
}

#[derive(Clone, Copy)]
pub(super) enum Tag<'arena> {
    Code(CodeTag<'arena>),
    Comment(Span),
    Preserved(Span),
}

impl Tag<'_> {
    pub fn span(self) -> Span {
        match self {
            Self::Code(tag) => tag.span(),
            Self::Comment(span) | Self::Preserved(span) => span,
        }
    }
}

pub(super) struct TagIterator<'ctx, 'arena> {
    code: &'ctx [Tag<'arena>],
    trivia: &'ctx [Trivia<'arena>],
}

impl<'ctx, 'arena> TagIterator<'ctx, 'arena> {
    pub fn new(code: &'ctx [Tag<'arena>], trivia: &'ctx [Trivia<'arena>]) -> Self {
        Self { code, trivia }
    }
}

impl<'arena> Iterator for TagIterator<'_, 'arena> {
    type Item = Tag<'arena>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((trivia, rest)) = self.trivia.split_first() {
            if self.code.first().is_some_and(|tag| trivia.span.start >= tag.span().start) {
                break;
            }

            self.trivia = rest;
            if trivia.kind == TriviaKind::Comment {
                return Some(Tag::Comment(trivia.span));
            }
        }

        let (tag, rest) = self.code.split_first()?;
        let mut tag = *tag;
        let span = tag.span();
        self.code = rest;
        while let Some((trivia, rest)) = self.trivia.split_first() {
            if trivia.span.start >= span.end {
                break;
            }

            if trivia.kind == TriviaKind::InlineComment {
                tag = Tag::Preserved(span);
            }
            self.trivia = rest;
        }

        Some(tag)
    }
}
