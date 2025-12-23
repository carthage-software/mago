use bumpalo::Bump;

use mago_span::Span;
use mago_syntax::ast::Trivia;
use mago_syntax::ast::TriviaKind;

use crate::document::Document;
use crate::error::ParseError;

mod internal;

pub mod document;
pub mod error;
pub mod tag;

/// Parses a docblock from a trivia token.
///
/// # Errors
///
/// Returns a [`ParseError`] if the trivia is not a docblock comment or parsing fails.
#[inline]
pub fn parse_trivia<'arena>(arena: &'arena Bump, trivia: &Trivia<'arena>) -> Result<Document<'arena>, ParseError> {
    if TriviaKind::DocBlockComment != trivia.kind {
        return Err(ParseError::InvalidTrivia(trivia.span));
    }

    parse_phpdoc_with_span(arena, trivia.value, trivia.span)
}

/// Parses a `PHPDoc` comment string with an associated span.
///
/// # Errors
///
/// Returns a [`ParseError`] if tokenization or parsing fails.
#[inline]
pub fn parse_phpdoc_with_span<'arena>(
    arena: &'arena Bump,
    content: &'arena str,
    span: Span,
) -> Result<Document<'arena>, ParseError> {
    let tokens = internal::lexer::tokenize(content, span)?;

    internal::parser::parse_document(span, tokens.as_slice(), arena)
}

#[cfg(test)]
mod tests {
    use super::*;

    use mago_database::file::FileId;
    use mago_span::Position;
    use mago_span::Span;

    use crate::document::*;

    #[test]
    fn test_parse_all_elements() {
        let arena = Bump::new();
        let phpdoc = r#"/**
            * This is a simple description.
            *
            * This text contains an inline code `echo "Hello, World!";`.
            *
            * This text contains an inline tag {@see \Some\Class}.
            *
            * ```php
            * echo "Hello, World!";
            * ```
            *
            *     $foo = "bar";
            *     echo "Hello, World!";
            *
            * @param string $foo
            * @param array{
            *   bar: string,
            *   baz: int
            * } $bar
            * @return void
            */"#;

        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));
        let document = parse_phpdoc_with_span(&arena, phpdoc, span).expect("Failed to parse PHPDoc");
        assert_eq!(document.elements.len(), 12);

        let Element::Text(text) = &document.elements[0] else {
            panic!("Expected Element::Text, got {:?}", document.elements[0]);
        };

        assert_eq!(text.segments.len(), 1);

        let TextSegment::Paragraph { span, content } = text.segments[0] else {
            panic!("Expected TextSegment::Paragraph, got {:?}", text.segments[0]);
        };

        assert_eq!(content, "This is a simple description.");
        assert_eq!(&phpdoc[span.start.offset as usize..span.end.offset as usize], "This is a simple description.");

        let Element::Line(_) = &document.elements[1] else {
            panic!("Expected Element::Line, got {:?}", document.elements[1]);
        };

        let Element::Text(text) = &document.elements[2] else {
            panic!("Expected Element::Text, got {:?}", document.elements[2]);
        };

        assert_eq!(text.segments.len(), 3);

        let TextSegment::Paragraph { content, .. } = text.segments[0] else {
            panic!("Expected TextSegment::Paragraph, got {:?}", text.segments[0]);
        };

        assert_eq!(content, "This text contains an inline code ");

        let TextSegment::InlineCode(code) = &text.segments[1] else {
            panic!("Expected TextSegment::InlineCode, got {:?}", text.segments[1]);
        };

        let content = code.content;
        assert_eq!(content, "echo \"Hello, World!\";");
        assert_eq!(
            &phpdoc[code.span.start.offset as usize..code.span.end.offset as usize],
            "`echo \"Hello, World!\";`"
        );

        let TextSegment::Paragraph { content, .. } = text.segments[2] else {
            panic!("Expected TextSegment::Paragraph, got {:?}", text.segments[2]);
        };

        assert_eq!(content, ".");

        let Element::Line(_) = &document.elements[3] else {
            panic!("Expected Element::Line, got {:?}", document.elements[3]);
        };

        let Element::Text(text) = &document.elements[4] else {
            panic!("Expected Element::Text, got {:?}", document.elements[4]);
        };

        assert_eq!(text.segments.len(), 3);

        let TextSegment::Paragraph { content, .. } = text.segments[0] else {
            panic!("Expected TextSegment::Paragraph, got {:?}", text.segments[0]);
        };

        assert_eq!(content, "This text contains an inline tag ");

        let TextSegment::InlineTag(tag) = &text.segments[1] else {
            panic!("Expected TextSegment::InlineTag, got {:?}", text.segments[1]);
        };

        let name = tag.name;
        let description = tag.description;
        assert_eq!(name, "see");
        assert_eq!(description, "\\Some\\Class");
        assert_eq!(tag.kind, TagKind::See);
        assert_eq!(&phpdoc[tag.span.start.offset as usize..tag.span.end.offset as usize], "{@see \\Some\\Class}");

        let TextSegment::Paragraph { content, .. } = text.segments[2] else {
            panic!("Expected TextSegment::Paragraph, got {:?}", text.segments[2]);
        };

        assert_eq!(content, ".");

        let Element::Line(_) = &document.elements[5] else {
            panic!("Expected Element::Line, got {:?}", document.elements[5]);
        };

        let Element::Code(code) = &document.elements[6] else {
            panic!("Expected Element::CodeBlock, got {:?}", document.elements[6]);
        };

        let content = code.content;
        assert_eq!(code.directives, &["php"]);
        assert_eq!(content, "echo \"Hello, World!\";");
        assert_eq!(
            &phpdoc[code.span.start.offset as usize..code.span.end.offset as usize],
            "```php\n            * echo \"Hello, World!\";\n            * ```"
        );

        let Element::Line(_) = &document.elements[7] else {
            panic!("Expected Element::Line, got {:?}", document.elements[7]);
        };

        let Element::Code(code) = &document.elements[8] else {
            panic!("Expected Element::CodeBlock, got {:?}", document.elements[8]);
        };

        let content = code.content;
        assert!(code.directives.is_empty());
        assert_eq!(content, "$foo = \"bar\";\necho \"Hello, World!\";\n");
        assert_eq!(
            &phpdoc[code.span.start.offset as usize..code.span.end.offset as usize],
            "    $foo = \"bar\";\n            *     echo \"Hello, World!\";\n"
        );

        let Element::Tag(tag) = &document.elements[9] else {
            panic!("Expected Element::Tag, got {:?}", document.elements[9]);
        };

        let name = tag.name;
        let description = tag.description;
        assert_eq!(name, "param");
        assert_eq!(tag.kind, TagKind::Param);
        assert_eq!(description, "string $foo");
        assert_eq!(&phpdoc[tag.span.start.offset as usize..tag.span.end.offset as usize], "@param string $foo");

        let Element::Tag(tag) = &document.elements[10] else {
            panic!("Expected Element::Tag, got {:?}", document.elements[10]);
        };

        let name = tag.name;
        let description = tag.description;
        assert_eq!(name, "param");
        assert_eq!(tag.kind, TagKind::Param);
        assert_eq!(description, "array{\n  bar: string,\n  baz: int\n} $bar");
        assert_eq!(
            &phpdoc[tag.span.start.offset as usize..tag.span.end.offset as usize],
            "@param array{\n            *   bar: string,\n            *   baz: int\n            * } $bar"
        );

        let Element::Tag(tag) = &document.elements[11] else {
            panic!("Expected Element::Tag, got {:?}", document.elements[11]);
        };

        let name = tag.name;
        let description = tag.description;
        assert_eq!(name, "return");
        assert_eq!(tag.kind, TagKind::Return);
        assert_eq!(description, "void");
        assert_eq!(&phpdoc[tag.span.start.offset as usize..tag.span.end.offset as usize], "@return void");
    }

    #[test]
    fn test_unclosed_inline_tag() {
        // Test case for ParseError::UnclosedInlineTag
        let arena = Bump::new();
        let phpdoc = "/** This is a doc block with an unclosed inline tag {@see Class */";
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));

        let result = parse_phpdoc_with_span(&arena, phpdoc, span);

        match result {
            Err(ParseError::UnclosedInlineTag(error_span)) => {
                let expected_start = phpdoc.find("{@see").unwrap();
                let expected_span = span.subspan(expected_start as u32, phpdoc.len() as u32 - 3);
                assert_eq!(error_span, expected_span);
            }
            _ => {
                panic!("Expected ParseError::UnclosedInlineTag");
            }
        }
    }

    #[test]
    fn test_unclosed_inline_code() {
        // Test case for ParseError::UnclosedInlineCode
        let arena = Bump::new();
        let phpdoc = "/** This is a doc block with unclosed inline code `code sample */";
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));

        let result = parse_phpdoc_with_span(&arena, phpdoc, span);

        match result {
            Err(ParseError::UnclosedInlineCode(error_span)) => {
                let expected_start = phpdoc.find('`').unwrap();
                let expected_span = span.subspan(expected_start as u32, phpdoc.len() as u32 - 3);
                assert_eq!(error_span, expected_span);
            }
            _ => {
                panic!("Expected ParseError::UnclosedInlineCode");
            }
        }
    }

    #[test]
    fn test_unclosed_code_block() {
        let arena = Bump::new();
        let phpdoc = r"/**
            * This is a doc block with unclosed code block
            * ```
            * Some code here
            */";
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));

        let result = parse_phpdoc_with_span(&arena, phpdoc, span);

        match result {
            Err(ParseError::UnclosedCodeBlock(error_span)) => {
                let code_block_start = phpdoc.find("```").unwrap();
                let expected_span = span.subspan(code_block_start as u32, 109);
                assert_eq!(error_span, expected_span);
            }
            _ => {
                panic!("Expected ParseError::UnclosedCodeBlock");
            }
        }
    }

    #[test]
    fn test_invalid_tag_name() {
        // Test case for ParseError::InvalidTagName
        let arena = Bump::new();
        let phpdoc = "/** @invalid_tag_name Description */";
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));

        let result = parse_phpdoc_with_span(&arena, phpdoc, span);

        match result {
            Err(ParseError::InvalidTagName(error_span)) => {
                let tag_start = phpdoc.find("@invalid_tag_name").unwrap();
                let tag_end = tag_start + "@invalid_tag_name".len();
                let expected_span = span.subspan(tag_start as u32, tag_end as u32);
                assert_eq!(error_span, expected_span);
            }
            _ => {
                panic!("Expected ParseError::InvalidTagName");
            }
        }
    }

    #[test]
    fn test_malformed_code_block() {
        let arena = Bump::new();
        let phpdoc = r"/**
            * ```
            * Some code here
            * Incorrect closing
            */";
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));

        let result = parse_phpdoc_with_span(&arena, phpdoc, span);

        match result {
            Ok(document) => {
                panic!("Expected the parser to return an error, got {document:#?}");
            }
            Err(ParseError::UnclosedCodeBlock(error_span)) => {
                let code_block_start = phpdoc.find("```").unwrap();
                let expected_span = span.subspan(code_block_start as u32, 82);
                assert_eq!(error_span, expected_span);
            }
            _ => {
                panic!("Expected ParseError::UnclosedCodeBlock");
            }
        }
    }

    #[test]
    fn test_invalid_comment() {
        // Test case for ParseError::InvalidComment
        let arena = Bump::new();
        let phpdoc = "/* Not a valid doc block */";
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));

        let result = parse_phpdoc_with_span(&arena, phpdoc, span);

        match result {
            Err(ParseError::InvalidComment(error_span)) => {
                assert_eq!(error_span, span);
            }
            _ => {
                panic!("Expected ParseError::InvalidComment");
            }
        }
    }

    #[test]
    fn test_inconsistent_indentation() {
        // Test case for ParseError::InconsistentIndentation
        let arena = Bump::new();
        let phpdoc = r"/**
    * This is a doc block
      * With inconsistent indentation
    */";
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));

        let result = parse_phpdoc_with_span(&arena, phpdoc, span);

        match result {
            Ok(document) => {
                assert_eq!(document.elements.len(), 1);
                let Element::Text(text) = &document.elements[0] else {
                    panic!("Expected Element::Text, got {:?}", document.elements[0]);
                };

                assert_eq!(text.segments.len(), 1);
                let TextSegment::Paragraph { span, content } = &text.segments[0] else {
                    panic!("Expected TextSegment::Paragraph, got {:?}", text.segments[0]);
                };

                assert_eq!(*content, "This is a doc block\nWith inconsistent indentation");
                assert_eq!(
                    &phpdoc[span.start.offset as usize..span.end.offset as usize],
                    "This is a doc block\n      * With inconsistent indentation"
                );
            }
            _ => {
                panic!("Expected ParseError::InconsistentIndentation");
            }
        }
    }

    #[test]
    fn test_missing_asterisk() {
        let arena = Bump::new();
        let phpdoc = r"/**
     This line is missing an asterisk
     * This line is fine
     */";
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));

        let result = parse_phpdoc_with_span(&arena, phpdoc, span);

        match result {
            Ok(document) => {
                assert_eq!(document.elements.len(), 1);
                let Element::Text(text) = &document.elements[0] else {
                    panic!("Expected Element::Text, got {:?}", document.elements[0]);
                };

                assert_eq!(text.segments.len(), 1);

                let TextSegment::Paragraph { span, content } = &text.segments[0] else {
                    panic!("Expected TextSegment::Paragraph, got {:?}", text.segments[0]);
                };

                assert_eq!(*content, "This line is missing an asterisk\nThis line is fine");
                assert_eq!(
                    &phpdoc[span.start.offset as usize..span.end.offset as usize],
                    "This line is missing an asterisk\n     * This line is fine"
                );
            }
            _ => {
                panic!("Expected ParseError::MissingAsterisk");
            }
        }
    }

    #[test]
    fn test_missing_whitespace_after_asterisk() {
        let arena = Bump::new();
        let phpdoc = r"/**
     *This line is missing a space after asterisk
     */";
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));

        let result = parse_phpdoc_with_span(&arena, phpdoc, span);

        match result {
            Ok(document) => {
                assert_eq!(document.elements.len(), 1);
                let Element::Text(text) = &document.elements[0] else {
                    panic!("Expected Element::Text, got {:?}", document.elements[0]);
                };

                assert_eq!(text.segments.len(), 1);
                let TextSegment::Paragraph { span, content } = &text.segments[0] else {
                    panic!("Expected TextSegment::Paragraph, got {:?}", text.segments[0]);
                };

                assert_eq!(*content, "This line is missing a space after asterisk");
                assert_eq!(
                    &phpdoc[span.start.offset as usize..span.end.offset as usize],
                    "This line is missing a space after asterisk"
                );
            }
            _ => {
                panic!("Expected ParseError::MissingWhitespaceAfterAsterisk");
            }
        }
    }

    #[test]
    fn test_missing_whitespace_after_opening_asterisk() {
        let arena = Bump::new();
        let phpdoc = "/**This is a doc block without space after /** */";
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));

        let result = parse_phpdoc_with_span(&arena, phpdoc, span);

        match result {
            Ok(document) => {
                assert_eq!(document.elements.len(), 1);
                let Element::Text(text) = &document.elements[0] else {
                    panic!("Expected Element::Text, got {:?}", document.elements[0]);
                };

                assert_eq!(text.segments.len(), 1);
                let TextSegment::Paragraph { span, content } = &text.segments[0] else {
                    panic!("Expected TextSegment::Paragraph, got {:?}", text.segments[0]);
                };

                assert_eq!(*content, "This is a doc block without space after /**");
                assert_eq!(
                    &phpdoc[span.start.offset as usize..span.end.offset as usize],
                    "This is a doc block without space after /**"
                );
            }
            _ => {
                panic!("Expected ParseError::MissingWhitespaceAfterOpeningAsterisk");
            }
        }
    }

    #[test]
    fn test_missing_whitespace_before_closing_asterisk() {
        let arena = Bump::new();
        let phpdoc = "/** This is a doc block without space before */*/";
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));

        let result = parse_phpdoc_with_span(&arena, phpdoc, span);

        match result {
            Ok(document) => {
                assert_eq!(document.elements.len(), 1);
                let Element::Text(text) = &document.elements[0] else {
                    panic!("Expected Element::Text, got {:?}", document.elements[0]);
                };

                assert_eq!(text.segments.len(), 1);
                let TextSegment::Paragraph { span, content } = &text.segments[0] else {
                    panic!("Expected TextSegment::Paragraph, got {:?}", text.segments[0]);
                };

                assert_eq!(*content, "This is a doc block without space before */");
                assert_eq!(
                    &phpdoc[span.start.offset as usize..span.end.offset as usize],
                    "This is a doc block without space before */"
                );
            }
            _ => {
                panic!("Expected ParseError::MissingWhitespaceBeforeClosingAsterisk");
            }
        }
    }

    #[test]
    fn test_utf8_characters() {
        let arena = Bump::new();
        let phpdoc = r#"/**
    * هذا نص باللغة العربية.
    * 这是一段中文。
    * Here are some mathematical symbols: ∑, ∆, π, θ.
    *
    * ```php
    * // Arabic comment
    * echo "مرحبا بالعالم";
    * // Chinese comment
    * echo "你好，世界";
    * // Math symbols in code
    * $sum = $a + $b; // ∑
    * ```
    *
    * @param string $مثال A parameter with an Arabic variable name.
    * @return int 返回值是整数类型。
    */"#;

        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));
        let document = parse_phpdoc_with_span(&arena, phpdoc, span).expect("Failed to parse PHPDoc");

        // Verify the number of elements parsed
        assert_eq!(document.elements.len(), 6);

        // First text element (Arabic text)
        let Element::Text(text) = &document.elements[0] else {
            panic!("Expected Element::Text, got {:?}", document.elements[0]);
        };

        assert_eq!(text.segments.len(), 1);

        let TextSegment::Paragraph { span, content } = &text.segments[0] else {
            panic!("Expected TextSegment::Paragraph, got {:?}", text.segments[0]);
        };

        assert_eq!(*content, "هذا نص باللغة العربية.\n这是一段中文。\nHere are some mathematical symbols: ∑, ∆, π, θ.");

        assert_eq!(
            &phpdoc[span.start.offset as usize..span.end.offset as usize],
            "هذا نص باللغة العربية.\n    * 这是一段中文。\n    * Here are some mathematical symbols: ∑, ∆, π, θ."
        );

        // Empty line
        let Element::Line(_) = &document.elements[1] else {
            panic!("Expected Element::Line, got {:?}", document.elements[3]);
        };

        // Code block
        let Element::Code(code) = &document.elements[2] else {
            panic!("Expected Element::Code, got {:?}", document.elements[2]);
        };

        let content_str = code.content;
        let expected_code = "// Arabic comment\necho \"مرحبا بالعالم\";\n// Chinese comment\necho \"你好，世界\";\n// Math symbols in code\n$sum = $a + $b; // ∑";
        assert_eq!(content_str, expected_code);
        assert_eq!(
            &phpdoc[code.span.start.offset as usize..code.span.end.offset as usize],
            "```php\n    * // Arabic comment\n    * echo \"مرحبا بالعالم\";\n    * // Chinese comment\n    * echo \"你好，世界\";\n    * // Math symbols in code\n    * $sum = $a + $b; // ∑\n    * ```"
        );

        // Empty line
        let Element::Line(_) = &document.elements[3] else {
            panic!("Expected Element::Line, got {:?}", document.elements[3]);
        };

        // @param tag with Arabic variable name
        let Element::Tag(tag) = &document.elements[4] else {
            panic!("Expected Element::Tag, got {:?}", document.elements[4]);
        };

        let name = tag.name;
        let description = tag.description;
        assert_eq!(name, "param");
        assert_eq!(tag.kind, TagKind::Param);
        assert_eq!(description, "string $مثال A parameter with an Arabic variable name.");
        assert_eq!(
            &phpdoc[tag.span.start.offset as usize..tag.span.end.offset as usize],
            "@param string $مثال A parameter with an Arabic variable name."
        );

        // @return tag with Chinese description
        let Element::Tag(tag) = &document.elements[5] else {
            panic!("Expected Element::Tag, got {:?}", document.elements[5]);
        };

        let name = tag.name;
        let description = tag.description;
        assert_eq!(name, "return");
        assert_eq!(tag.kind, TagKind::Return);
        assert_eq!(description, "int 返回值是整数类型。");
        assert_eq!(
            &phpdoc[tag.span.start.offset as usize..tag.span.end.offset as usize],
            "@return int 返回值是整数类型。"
        );
    }

    #[test]
    fn test_annotation_parsing() {
        let arena = Bump::new();
        let phpdoc = r#"/**
         * @Event("Symfony\Component\Workflow\Event\CompletedEvent")
         * @AnotherAnnotation({
         *     "key": "value",
         *     "list": [1, 2, 3]
         * })
         * @SimpleAnnotation
         */"#;
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));
        let document = parse_phpdoc_with_span(&arena, phpdoc, span).expect("Failed to parse PHPDoc");

        // Verify that the document has the expected number of elements
        assert_eq!(document.elements.len(), 3);

        // First annotation
        let Element::Annotation(annotation) = &document.elements[0] else {
            panic!("Expected Element::Annotation, got {:?}", document.elements[0]);
        };

        let name = annotation.name;
        assert_eq!(name, "Event");
        let arguments = annotation.arguments.unwrap();
        assert_eq!(arguments, "(\"Symfony\\Component\\Workflow\\Event\\CompletedEvent\")");

        // Second annotation
        let Element::Annotation(annotation) = &document.elements[1] else {
            panic!("Expected Element::Annotation, got {:?}", document.elements[1]);
        };

        let name = annotation.name;
        assert_eq!(name, "AnotherAnnotation");
        let arguments = annotation.arguments.unwrap();
        let expected_arguments = "({\n    \"key\": \"value\",\n    \"list\": [1, 2, 3]\n})";
        assert_eq!(arguments, expected_arguments);

        // Third annotation
        let Element::Annotation(annotation) = &document.elements[2] else {
            panic!("Expected Element::Annotation, got {:?}", document.elements[2]);
        };

        let name = annotation.name;
        assert_eq!(name, "SimpleAnnotation");
        assert!(annotation.arguments.is_none());
    }

    #[test]
    fn test_long_description_with_missing_asterisk() {
        let arena = Bump::new();
        let phpdoc = r"/** @var string[] this is a really long description
            that spans multiple lines, and demonstrates how the parser handles
            docblocks with multiple descriptions, and missing astricks*/";
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));
        let document = parse_phpdoc_with_span(&arena, phpdoc, span).expect("Failed to parse PHPDoc");

        assert_eq!(document.elements.len(), 1);
        let Element::Tag(tag) = &document.elements[0] else {
            panic!("Expected Element::Tag, got {:?}", document.elements[0]);
        };

        let name = tag.name;
        let description = tag.description;
        assert_eq!(name, "var");
        assert_eq!(tag.kind, TagKind::Var);
        assert_eq!(
            description,
            "string[] this is a really long description\nthat spans multiple lines, and demonstrates how the parser handles\ndocblocks with multiple descriptions, and missing astricks"
        );
        assert_eq!(
            &phpdoc[tag.span.start.offset as usize..tag.span.end.offset as usize],
            "@var string[] this is a really long description\n            that spans multiple lines, and demonstrates how the parser handles\n            docblocks with multiple descriptions, and missing astricks"
        );
    }

    #[test]
    fn test_code_indent_using_non_ascii_chars() {
        let arena = Bump::new();
        let phpdoc = r"/**
        *    └─ comment 2
        *       └─ comment 4
        *    └─ comment 3
        */";

        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));
        let document = parse_phpdoc_with_span(&arena, phpdoc, span).expect("Failed to parse PHPDoc");

        assert_eq!(document.elements.len(), 1);

        let Element::Code(code) = &document.elements[0] else {
            panic!("Expected Element::Code, got {:?}", document.elements[0]);
        };

        let content_str = code.content;
        assert_eq!(content_str, " └─ comment 2\n\u{a0}\u{a0} └─ comment 4\n └─ comment 3");
        assert_eq!(
            &phpdoc[code.span.start.offset as usize..code.span.end.offset as usize],
            " \u{a0} └─ comment 2\n        *    \u{a0}\u{a0} └─ comment 4\n        *  \u{a0} └─ comment 3"
        );
    }

    #[test]
    fn test_issue_456() {
        let arena = Bump::new();
        let phpdoc = "/**
             * \u{3000}(イベント日数をもとに計算)\u{3000}
             * @return\u{3000}int
             * @throws\u{3000}Exception
             */";

        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));
        let document = parse_phpdoc_with_span(&arena, phpdoc, span).expect("Failed to parse PHPDoc");

        assert_eq!(document.elements.len(), 3);

        let Element::Text(text) = &document.elements[0] else {
            panic!("Expected Element::Text, got {:?}", document.elements[0]);
        };

        assert_eq!(text.segments.len(), 1);
        let TextSegment::Paragraph { span, content } = &text.segments[0] else {
            panic!("Expected TextSegment::Paragraph, got {:?}", text.segments[0]);
        };

        assert_eq!(*content, "　(イベント日数をもとに計算)");
        assert_eq!(&phpdoc[span.start.offset as usize..span.end.offset as usize], "\u{3000}(イベント日数をもとに計算)");

        let Element::Tag(tag) = &document.elements[1] else {
            panic!("Expected Element::Tag, got {:?}", document.elements[1]);
        };

        let name = tag.name;
        let description = tag.description;
        assert_eq!(name, "return");
        assert_eq!(tag.kind, TagKind::Return);
        assert_eq!(description, "int");
        assert_eq!(&phpdoc[tag.span.start.offset as usize..tag.span.end.offset as usize], "@return\u{3000}int");

        let Element::Tag(tag) = &document.elements[2] else {
            panic!("Expected Element::Tag, got {:?}", document.elements[2]);
        };

        let name = tag.name;
        let description = tag.description;
        assert_eq!(name, "throws");
        assert_eq!(tag.kind, TagKind::Throws);
        assert_eq!(description, "Exception");
        assert_eq!(&phpdoc[tag.span.start.offset as usize..tag.span.end.offset as usize], "@throws\u{3000}Exception");
    }

    #[test]
    fn test_issue_808() {
        let arena = Bump::new();

        let phpdoc = "/** @param\u{3000}string $foo 中文描述 */";
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));
        let document = parse_phpdoc_with_span(&arena, phpdoc, span).expect("Failed to parse PHPDoc");

        assert_eq!(document.elements.len(), 1);
        let Element::Tag(tag) = &document.elements[0] else {
            panic!("Expected Element::Tag, got {:?}", document.elements[0]);
        };
        assert_eq!(tag.name, "param");
        assert_eq!(tag.description, "string $foo 中文描述");

        let phpdoc2 = "/** @return\u{3000}int 返回🎉值 */";
        let span2 = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc2.len() as u32));
        let document2 = parse_phpdoc_with_span(&arena, phpdoc2, span2).expect("Failed to parse PHPDoc");

        assert_eq!(document2.elements.len(), 1);
        let Element::Tag(tag2) = &document2.elements[0] else {
            panic!("Expected Element::Tag, got {:?}", document2.elements[0]);
        };
        assert_eq!(tag2.name, "return");
        assert_eq!(tag2.description, "int 返回🎉值");

        let phpdoc3 = "/** @see\u{3000}中文类::方法() 说明 */";
        let span3 = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc3.len() as u32));
        let document3 = parse_phpdoc_with_span(&arena, phpdoc3, span3).expect("Failed to parse PHPDoc");

        assert_eq!(document3.elements.len(), 1);
        let Element::Tag(tag3) = &document3.elements[0] else {
            panic!("Expected Element::Tag, got {:?}", document3.elements[0]);
        };
        assert_eq!(tag3.name, "see");
        assert_eq!(tag3.description, "中文类::方法() 说明");
    }
}
