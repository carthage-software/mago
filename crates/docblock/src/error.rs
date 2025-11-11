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
    InvalidTypeTag(Span, String),
    InvalidImportTypeTag(Span, String),
    InvalidTemplateTag(Span, String),
    InvalidParameterTag(Span, String),
    InvalidReturnTag(Span, String),
    InvalidPropertyTag(Span, String),
    InvalidMethodTag(Span, String),
    InvalidThrowsTag(Span, String),
    InvalidAssertionTag(Span, String),
    InvalidVarTag(Span, String),
    InvalidWhereTag(Span, String),
    InvalidParameterOutTag(Span, String),
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
            | ParseError::ExpectedLine(span)
            | ParseError::InvalidTypeTag(span, _)
            | ParseError::InvalidImportTypeTag(span, _)
            | ParseError::InvalidTemplateTag(span, _)
            | ParseError::InvalidParameterTag(span, _)
            | ParseError::InvalidReturnTag(span, _)
            | ParseError::InvalidPropertyTag(span, _)
            | ParseError::InvalidMethodTag(span, _)
            | ParseError::InvalidThrowsTag(span, _)
            | ParseError::InvalidAssertionTag(span, _)
            | ParseError::InvalidVarTag(span, _)
            | ParseError::InvalidWhereTag(span, _)
            | ParseError::InvalidParameterOutTag(span, _) => *span,
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
            ParseError::InvalidTypeTag(_, msg) => write!(f, "{}", msg),
            ParseError::InvalidImportTypeTag(_, msg) => write!(f, "{}", msg),
            ParseError::InvalidTemplateTag(_, msg) => write!(f, "{}", msg),
            ParseError::InvalidParameterTag(_, msg) => write!(f, "{}", msg),
            ParseError::InvalidReturnTag(_, msg) => write!(f, "{}", msg),
            ParseError::InvalidPropertyTag(_, msg) => write!(f, "{}", msg),
            ParseError::InvalidMethodTag(_, msg) => write!(f, "{}", msg),
            ParseError::InvalidThrowsTag(_, msg) => write!(f, "{}", msg),
            ParseError::InvalidAssertionTag(_, msg) => write!(f, "{}", msg),
            ParseError::InvalidVarTag(_, msg) => write!(f, "{}", msg),
            ParseError::InvalidWhereTag(_, msg) => write!(f, "{}", msg),
            ParseError::InvalidParameterOutTag(_, msg) => write!(f, "{}", msg),
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
            ParseError::InvalidTypeTag(_, _) => "Type alias must have name followed by type definition".to_string(),
            ParseError::InvalidImportTypeTag(_, _) => {
                "Import must have type name, `from` keyword, and class name".to_string()
            }
            ParseError::InvalidTemplateTag(_, _) => "Template must have parameter name".to_string(),
            ParseError::InvalidParameterTag(_, _) => "Parameter must have type followed by variable name".to_string(),
            ParseError::InvalidReturnTag(_, _) => "Return must have valid type".to_string(),
            ParseError::InvalidPropertyTag(_, _) => "Property must have type and/or variable name".to_string(),
            ParseError::InvalidMethodTag(_, _) => "Method must have return type, name, and parameter list".to_string(),
            ParseError::InvalidThrowsTag(_, _) => "Throws must have exception type".to_string(),
            ParseError::InvalidAssertionTag(_, _) => "Assertion must have type followed by variable name".to_string(),
            ParseError::InvalidVarTag(_, _) => "Variable must have type".to_string(),
            ParseError::InvalidWhereTag(_, _) => {
                "Template constraint must have parameter name, `is` or `:`, and type".to_string()
            }
            ParseError::InvalidParameterOutTag(_, _) => {
                "Output parameter must have type followed by variable name".to_string()
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
            ParseError::InvalidTypeTag(_, _) => {
                "Add type definition after alias name (can span multiple lines)".to_string()
            }
            ParseError::InvalidImportTypeTag(_, _) => {
                "Ensure type name is followed by `from` and a valid class name".to_string()
            }
            ParseError::InvalidTemplateTag(_, _) => "Provide a valid template parameter name".to_string(),
            ParseError::InvalidParameterTag(_, _) => {
                "Ensure type is followed by a valid parameter name (e.g., `$param`)".to_string()
            }
            ParseError::InvalidReturnTag(_, _) => "Provide a valid return type".to_string(),
            ParseError::InvalidPropertyTag(_, _) => {
                "Ensure property has valid type and/or variable name (e.g., `$prop`)".to_string()
            }
            ParseError::InvalidMethodTag(_, _) => "Provide return type, method name, and parameter list".to_string(),
            ParseError::InvalidThrowsTag(_, _) => "Provide a valid exception class name".to_string(),
            ParseError::InvalidAssertionTag(_, _) => {
                "Ensure type is followed by a valid variable name (e.g., `$var`)".to_string()
            }
            ParseError::InvalidVarTag(_, _) => "Provide a valid type for the variable".to_string(),
            ParseError::InvalidWhereTag(_, _) => {
                "Ensure template name is followed by `is` or `:` and a type".to_string()
            }
            ParseError::InvalidParameterOutTag(_, _) => {
                "Ensure type is followed by a valid parameter name (e.g., `$param`)".to_string()
            }
            _ => "Review the docblock syntax to ensure it is correctly formatted.".to_string(),
        }
    }
}
