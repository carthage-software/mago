use mago_span::HasSpan;
use mago_span::Span;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ParseError {
    UnexpectedToken(Span),
    UnexpectedEndOfInput(Span),
    UnclosedInlineTag(Span),
    UnclosedInlineCode(Span),
    UnclosedCodeBlock(Span),
    MalformedCodeBlock(Span),
    UnclosedLiteralString(Span),
    RecursionLimitExceeded(Span),
}

impl ParseError {
    #[must_use]
    pub fn note(&self) -> &'static str {
        match self {
            ParseError::UnexpectedToken(_) => {
                "This token cannot appear here, so the type or tag stops making sense at this point."
            }
            ParseError::UnexpectedEndOfInput(_) => {
                "The comment ended while a type or tag was still expecting more to follow."
            }
            ParseError::UnclosedInlineTag(_) => {
                "An inline tag such as `{@see Foo}` was opened with `{` but never closed."
            }
            ParseError::UnclosedInlineCode(_) => {
                "Inline code is wrapped in backticks, and the closing backtick is missing."
            }
            ParseError::UnclosedCodeBlock(_) => "A fenced code block was opened with ``` but was never closed.",
            ParseError::MalformedCodeBlock(_) => {
                "A code fence has more backticks than can be represented; the maximum is 255."
            }
            ParseError::UnclosedLiteralString(_) => {
                "A quoted string inside the type was opened but has no closing quote."
            }
            ParseError::RecursionLimitExceeded(_) => {
                "This type nests deeper than the parser supports, so it cannot be analyzed."
            }
        }
    }

    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            ParseError::UnexpectedToken(_) => "Remove the stray token, or fix the type or tag syntax leading up to it.",
            ParseError::UnexpectedEndOfInput(_) => "Finish the type or tag, and close any open `<`, `(`, `[`, or `{`.",
            ParseError::UnclosedInlineTag(_) => "Add the closing `}` that ends the inline tag.",
            ParseError::UnclosedInlineCode(_) => "Add a backtick (`) to match the one that opened the inline code.",
            ParseError::UnclosedCodeBlock(_) => "Add a closing ``` fence to end the code block.",
            ParseError::MalformedCodeBlock(_) => "Shorten the run of backticks that opens or closes the code block.",
            ParseError::UnclosedLiteralString(_) => "Add the matching closing quote (`'` or `\"`).",
            ParseError::RecursionLimitExceeded(_) => {
                "Reduce the nesting, for example by extracting part of the type into a `@phpstan-type` alias."
            }
        }
    }
}

impl HasSpan for ParseError {
    fn span(&self) -> Span {
        match self {
            ParseError::UnexpectedToken(span)
            | ParseError::UnexpectedEndOfInput(span)
            | ParseError::UnclosedInlineTag(span)
            | ParseError::UnclosedInlineCode(span)
            | ParseError::UnclosedCodeBlock(span)
            | ParseError::MalformedCodeBlock(span)
            | ParseError::UnclosedLiteralString(span)
            | ParseError::RecursionLimitExceeded(span) => *span,
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken(_) => write!(f, "Unexpected token in PHPDoc type"),
            ParseError::UnexpectedEndOfInput(_) => write!(f, "PHPDoc comment ended unexpectedly"),
            ParseError::UnclosedInlineTag(_) => write!(f, "Unclosed inline tag"),
            ParseError::UnclosedInlineCode(_) => write!(f, "Unclosed inline code"),
            ParseError::UnclosedCodeBlock(_) => write!(f, "Unclosed code block"),
            ParseError::MalformedCodeBlock(_) => write!(f, "Malformed code block"),
            ParseError::UnclosedLiteralString(_) => write!(f, "Unclosed string literal in type"),
            ParseError::RecursionLimitExceeded(_) => write!(f, "Type nested too deeply"),
        }
    }
}

impl std::error::Error for ParseError {}
