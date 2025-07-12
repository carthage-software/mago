use serde::Deserialize;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum ParseError {
    InvalidTrivia(Span),
    UnclosedInlineTag(Span),
    UnclosedInlineCode(Span),
    UnclosedCodeBlock(Span),
    InvalidTagName(Span),
    InvalidAnnotationName(Span),
    UnclosedAnnotationArguments(Span),
    MalformedCodeBlock(Span),
    InvalidComment(Span),
    ExpectedLine(Span),
}

impl HasSpan for ParseError {
    fn span(&self) -> Span {
        match self {
            ParseError::InvalidTrivia(span)
            | ParseError::UnclosedInlineTag(span)
            | ParseError::UnclosedInlineCode(span)
            | ParseError::UnclosedCodeBlock(span)
            | ParseError::InvalidTagName(span)
            | ParseError::InvalidAnnotationName(span)
            | ParseError::UnclosedAnnotationArguments(span)
            | ParseError::MalformedCodeBlock(span)
            | ParseError::InvalidComment(span)
            | ParseError::ExpectedLine(span) => *span,
        }
    }
}

impl std::error::Error for ParseError {}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidTrivia(_) | ParseError::InvalidComment(_) => {
                write!(f, "Invalid docblock format")
            }
            ParseError::UnclosedInlineTag(_) => write!(f, "Unclosed inline tag"),
            ParseError::UnclosedInlineCode(_) => write!(f, "Unclosed inline code"),
            ParseError::UnclosedCodeBlock(_) => write!(f, "Unclosed code block"),
            ParseError::InvalidTagName(_) => write!(f, "Invalid tag name"),
            ParseError::InvalidAnnotationName(_) => write!(f, "Invalid annotation name"),
            ParseError::UnclosedAnnotationArguments(_) => write!(f, "Unclosed annotation arguments"),
            ParseError::MalformedCodeBlock(_) => write!(f, "Malformed code block"),
            ParseError::ExpectedLine(_) => write!(f, "Unexpected end of docblock"),
        }
    }
}

impl ParseError {
    pub fn note(&self) -> String {
        match self {
            ParseError::InvalidTrivia(_) | ParseError::InvalidComment(_) => {
                "Docblocks must start with `/**` and end with `*/`.".to_string()
            }
            ParseError::UnclosedInlineTag(_) => {
                "Inline tags like `{@see}` must be closed with a matching `}`.".to_string()
            }
            ParseError::UnclosedInlineCode(_) => {
                "Inline code snippets must be enclosed in matching backticks (`).".to_string()
            }
            ParseError::UnclosedCodeBlock(_) => {
                "Multi-line code blocks must be terminated with a closing ```.".to_string()
            }
            ParseError::InvalidTagName(_) => {
                "Docblock tags like `@param` must contain only letters, numbers, hyphens, and colons.".to_string()
            }
            ParseError::InvalidAnnotationName(_) => {
                "Annotations must start with an uppercase letter, `_`, or `\\`.".to_string()
            }
            ParseError::UnclosedAnnotationArguments(_) => {
                "Arguments for an annotation must be enclosed in parentheses `()`.".to_string()
            }
            ParseError::MalformedCodeBlock(_) => {
                "A code block must start with ``` optionally followed by a language identifier.".to_string()
            }
            ParseError::ExpectedLine(_) => {
                "A tag or description was expected here, but the docblock ended prematurely.".to_string()
            }
        }
    }

    pub fn help(&self) -> String {
        match self {
            ParseError::UnclosedInlineTag(_) => "Add a closing `}` to complete the inline tag.".to_string(),
            ParseError::UnclosedInlineCode(_) => {
                "Add a closing backtick ` ` ` to terminate the inline code.".to_string()
            }
            ParseError::UnclosedCodeBlock(_) => "Add a closing ``` to terminate the code block.".to_string(),
            ParseError::InvalidTagName(_) => {
                "Correct the tag name to use only valid characters (e.g., `@my-custom-tag`).".to_string()
            }
            ParseError::InvalidAnnotationName(_) => {
                "Correct the annotation name to follow PSR-5 standards.".to_string()
            }
            ParseError::UnclosedAnnotationArguments(_) => {
                "Add a closing `)` to complete the annotation's argument list.".to_string()
            }
            _ => "Review the docblock syntax to ensure it is correctly formatted.".to_string(),
        }
    }
}
