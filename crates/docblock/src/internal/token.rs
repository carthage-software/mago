use mago_span::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum Token<'arena> {
    Line { content: &'arena str, span: Span },
    EmptyLine { span: Span },
}

impl Token<'_> {
    pub fn span(&self) -> Span {
        match self {
            Token::Line { span, .. } => *span,
            Token::EmptyLine { span } => *span,
        }
    }
}
