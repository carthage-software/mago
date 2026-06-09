use mago_allocator::prelude::*;

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
pub fn parse_trivia<'arena, A>(arena: &'arena A, trivia: &Trivia<'arena>) -> Result<Document<'arena>, ParseError>
where
    A: Arena,
{
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
pub fn parse_phpdoc_with_span<'arena, A>(
    arena: &'arena A,
    content: &'arena [u8],
    span: Span,
) -> Result<Document<'arena>, ParseError>
where
    A: Arena,
{
    let tokens = internal::lexer::tokenize(content, span)?;

    internal::parser::parse_document(span, tokens.as_slice(), arena)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    use mago_database::file::FileId;
    use mago_span::HasSpan;
    use mago_span::Position;
    use mago_span::Span;

    use crate::document::*;

    #[test]
    fn test_parse_all_elements() {
        let arena = LocalArena::new();
        let phpdoc = br#"/**
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

        assert_eq!(content, b"This is a simple description." as &[u8]);
        assert_eq!(&phpdoc[span.start_offset() as usize..span.end_offset() as usize], b"This is a simple description.");

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

        assert_eq!(content, b"This text contains an inline code " as &[u8]);

        let TextSegment::InlineCode(code) = &text.segments[1] else {
            panic!("Expected TextSegment::InlineCode, got {:?}", text.segments[1]);
        };

        let content = code.content;
        assert_eq!(content, b"echo \"Hello, World!\";" as &[u8]);
        assert_eq!(
            &phpdoc[code.span.start_offset() as usize..code.span.end_offset() as usize],
            b"`echo \"Hello, World!\";`"
        );

        let TextSegment::Paragraph { content, .. } = text.segments[2] else {
            panic!("Expected TextSegment::Paragraph, got {:?}", text.segments[2]);
        };

        assert_eq!(content, b"." as &[u8]);

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

        assert_eq!(content, b"This text contains an inline tag " as &[u8]);

        let TextSegment::InlineTag(tag) = &text.segments[1] else {
            panic!("Expected TextSegment::InlineTag, got {:?}", text.segments[1]);
        };

        let name = tag.name;
        let description = tag.description;
        assert_eq!(name, b"see" as &[u8]);
        assert_eq!(description, b"\\Some\\Class" as &[u8]);
        assert_eq!(tag.kind, TagKind::See);
        assert_eq!(&phpdoc[tag.span.start_offset() as usize..tag.span.end_offset() as usize], b"{@see \\Some\\Class}");

        let TextSegment::Paragraph { content, .. } = text.segments[2] else {
            panic!("Expected TextSegment::Paragraph, got {:?}", text.segments[2]);
        };

        assert_eq!(content, b"." as &[u8]);

        let Element::Line(_) = &document.elements[5] else {
            panic!("Expected Element::Line, got {:?}", document.elements[5]);
        };

        let Element::Code(code) = &document.elements[6] else {
            panic!("Expected Element::CodeBlock, got {:?}", document.elements[6]);
        };

        let content = code.content;
        assert_eq!(code.directives, &[b"php" as &[u8]]);
        assert_eq!(content, b"echo \"Hello, World!\";" as &[u8]);
        assert_eq!(
            &phpdoc[code.span.start_offset() as usize..code.span.end_offset() as usize],
            "```php\n            * echo \"Hello, World!\";\n            * ```".as_bytes()
        );

        let Element::Line(_) = &document.elements[7] else {
            panic!("Expected Element::Line, got {:?}", document.elements[7]);
        };

        let Element::Code(code) = &document.elements[8] else {
            panic!("Expected Element::CodeBlock, got {:?}", document.elements[8]);
        };

        let content = code.content;
        assert!(code.directives.is_empty());
        assert_eq!(content, b"$foo = \"bar\";\necho \"Hello, World!\";\n" as &[u8]);
        assert_eq!(
            &phpdoc[code.span.start_offset() as usize..code.span.end_offset() as usize],
            "    $foo = \"bar\";\n            *     echo \"Hello, World!\";\n".as_bytes()
        );

        let Element::Tag(tag) = &document.elements[9] else {
            panic!("Expected Element::Tag, got {:?}", document.elements[9]);
        };

        let name = tag.name;
        let description = tag.description;
        assert_eq!(name, b"param" as &[u8]);
        assert_eq!(tag.kind, TagKind::Param);
        assert_eq!(description, b"string $foo" as &[u8]);
        assert_eq!(&phpdoc[tag.span.start_offset() as usize..tag.span.end_offset() as usize], b"@param string $foo");

        let Element::Tag(tag) = &document.elements[10] else {
            panic!("Expected Element::Tag, got {:?}", document.elements[10]);
        };

        let name = tag.name;
        let description = tag.description;
        assert_eq!(name, b"param" as &[u8]);
        assert_eq!(tag.kind, TagKind::Param);
        assert_eq!(description, b"array{\n  bar: string,\n  baz: int\n} $bar" as &[u8]);
        assert_eq!(
            &phpdoc[tag.span.start_offset() as usize..tag.span.end_offset() as usize],
            "@param array{\n            *   bar: string,\n            *   baz: int\n            * } $bar".as_bytes()
        );

        let Element::Tag(tag) = &document.elements[11] else {
            panic!("Expected Element::Tag, got {:?}", document.elements[11]);
        };

        let name = tag.name;
        let description = tag.description;
        assert_eq!(name, b"return" as &[u8]);
        assert_eq!(tag.kind, TagKind::Return);
        assert_eq!(description, b"void" as &[u8]);
        assert_eq!(&phpdoc[tag.span.start_offset() as usize..tag.span.end_offset() as usize], b"@return void");
    }

    #[test]
    fn test_unclosed_inline_tag() {
        // Test case for ParseError::UnclosedInlineTag
        let arena = LocalArena::new();
        let phpdoc = b"/** This is a doc block with an unclosed inline tag {@see Class */";
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));

        let result = parse_phpdoc_with_span(&arena, phpdoc, span);

        match result {
            Err(ParseError::UnclosedInlineTag(error_span)) => {
                let expected_start = memchr::memmem::find(phpdoc, b"{@see").unwrap();
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
        let arena = LocalArena::new();
        let phpdoc = b"/** This is a doc block with unclosed inline code `code sample */";
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));

        let result = parse_phpdoc_with_span(&arena, phpdoc, span);

        match result {
            Err(ParseError::UnclosedInlineCode(error_span)) => {
                let expected_start = memchr::memchr(b'`', phpdoc).unwrap();
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
        let arena = LocalArena::new();
        let phpdoc = b"/**
            * This is a doc block with unclosed code block
            * ```
            * Some code here
            */";
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));

        let result = parse_phpdoc_with_span(&arena, phpdoc, span);

        match result {
            Err(ParseError::UnclosedCodeBlock(error_span)) => {
                let code_block_start = memchr::memmem::find(phpdoc, b"```").unwrap();
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
        // Test case for ParseError::InvalidTagName — use a character not valid in identifiers
        let arena = LocalArena::new();
        let phpdoc = b"/** @invalid!tag Description */";
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));

        let result = parse_phpdoc_with_span(&arena, phpdoc, span);

        assert!(
            matches!(result, Err(ParseError::InvalidTagName(_))),
            "Expected ParseError::InvalidTagName, got {result:?}"
        );
    }

    #[test]
    fn test_underscore_tag_name_is_valid() {
        let arena = LocalArena::new();
        let phpdoc = b"/** @some_tag Description */";
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));

        let document = parse_phpdoc_with_span(&arena, phpdoc, span).expect("Failed to parse PHPDoc");
        let Element::Tag(tag) = &document.elements[0] else {
            panic!("Expected Element::Tag");
        };
        assert_eq!(tag.name, b"some_tag" as &[u8]);
    }

    #[test]
    fn test_malformed_code_block() {
        let arena = LocalArena::new();
        let phpdoc = b"/**
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
                let code_block_start = memchr::memmem::find(phpdoc, b"```").unwrap();
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
        let arena = LocalArena::new();
        let phpdoc = b"/* Not a valid doc block */";
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
        let arena = LocalArena::new();
        let phpdoc = b"/**
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

                assert_eq!(*content, b"This is a doc block\nWith inconsistent indentation" as &[u8]);
                assert_eq!(
                    &phpdoc[span.start_offset() as usize..span.end_offset() as usize],
                    b"This is a doc block\n      * With inconsistent indentation"
                );
            }
            _ => {
                panic!("Expected ParseError::InconsistentIndentation");
            }
        }
    }

    #[test]
    fn test_missing_asterisk() {
        let arena = LocalArena::new();
        let phpdoc = b"/**
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

                assert_eq!(*content, b"This line is missing an asterisk\nThis line is fine" as &[u8]);
                assert_eq!(
                    &phpdoc[span.start_offset() as usize..span.end_offset() as usize],
                    b"This line is missing an asterisk\n     * This line is fine"
                );
            }
            _ => {
                panic!("Expected ParseError::MissingAsterisk");
            }
        }
    }

    #[test]
    fn test_missing_whitespace_after_asterisk() {
        let arena = LocalArena::new();
        let phpdoc = b"/**
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

                assert_eq!(*content, b"This line is missing a space after asterisk" as &[u8]);
                assert_eq!(
                    &phpdoc[span.start_offset() as usize..span.end_offset() as usize],
                    b"This line is missing a space after asterisk"
                );
            }
            _ => {
                panic!("Expected ParseError::MissingWhitespaceAfterAsterisk");
            }
        }
    }

    #[test]
    fn test_missing_whitespace_after_opening_asterisk() {
        let arena = LocalArena::new();
        let phpdoc = b"/**This is a doc block without space after /** */";
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

                assert_eq!(*content, b"This is a doc block without space after /**" as &[u8]);
                assert_eq!(
                    &phpdoc[span.start_offset() as usize..span.end_offset() as usize],
                    b"This is a doc block without space after /**"
                );
            }
            _ => {
                panic!("Expected ParseError::MissingWhitespaceAfterOpeningAsterisk");
            }
        }
    }

    #[test]
    fn test_missing_whitespace_before_closing_asterisk() {
        let arena = LocalArena::new();
        let phpdoc = b"/** This is a doc block without space before */*/";
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

                assert_eq!(*content, b"This is a doc block without space before */" as &[u8]);
                assert_eq!(
                    &phpdoc[span.start_offset() as usize..span.end_offset() as usize],
                    b"This is a doc block without space before */"
                );
            }
            _ => {
                panic!("Expected ParseError::MissingWhitespaceBeforeClosingAsterisk");
            }
        }
    }

    #[test]
    fn test_utf8_characters() {
        let arena = LocalArena::new();
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
    */"#
        .as_bytes();

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

        assert_eq!(
            *content,
            "هذا نص باللغة العربية.\n这是一段中文。\nHere are some mathematical symbols: ∑, ∆, π, θ.".as_bytes()
        );

        assert_eq!(
            &phpdoc[span.start_offset() as usize..span.end_offset() as usize],
            "هذا نص باللغة العربية.\n    * 这是一段中文。\n    * Here are some mathematical symbols: ∑, ∆, π, θ."
                .as_bytes()
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
        let expected_code = "// Arabic comment\necho \"مرحبا بالعالم\";\n// Chinese comment\necho \"你好，世界\";\n// Math symbols in code\n$sum = $a + $b; // ∑".as_bytes();
        assert_eq!(content_str, expected_code);
        assert_eq!(
            &phpdoc[code.span.start_offset() as usize..code.span.end_offset() as usize],
            "```php\n    * // Arabic comment\n    * echo \"مرحبا بالعالم\";\n    * // Chinese comment\n    * echo \"你好，世界\";\n    * // Math symbols in code\n    * $sum = $a + $b; // ∑\n    * ```".as_bytes()
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
        assert_eq!(name, b"param" as &[u8]);
        assert_eq!(tag.kind, TagKind::Param);
        assert_eq!(description, "string $مثال A parameter with an Arabic variable name.".as_bytes());
        assert_eq!(
            &phpdoc[tag.span.start_offset() as usize..tag.span.end_offset() as usize],
            "@param string $مثال A parameter with an Arabic variable name.".as_bytes()
        );

        // @return tag with Chinese description
        let Element::Tag(tag) = &document.elements[5] else {
            panic!("Expected Element::Tag, got {:?}", document.elements[5]);
        };

        let name = tag.name;
        let description = tag.description;
        assert_eq!(name, b"return" as &[u8]);
        assert_eq!(tag.kind, TagKind::Return);
        assert_eq!(description, "int 返回值是整数类型。".as_bytes());
        assert_eq!(
            &phpdoc[tag.span.start_offset() as usize..tag.span.end_offset() as usize],
            "@return int 返回值是整数类型。".as_bytes()
        );
    }

    #[test]
    #[allow(clippy::missing_asserts_for_indexing)]
    fn test_annotation_parsing() {
        let arena = LocalArena::new();
        let phpdoc = br#"/**
         * @Event("Symfony\Component\Workflow\Event\CompletedEvent")
         * @AnotherAnnotation({
         *     "key": "value",
         *     "list": [1, 2, 3]
         * })
         * @SimpleAnnotation
         */"#;
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));
        let document = parse_phpdoc_with_span(&arena, phpdoc, span).expect("Failed to parse PHPDoc");

        let Element::Tag(tag) = &document.elements[0] else {
            panic!("Expected Element::Tag, got {:?}", document.elements[0]);
        };

        assert_eq!(tag.name, b"Event" as &[u8]);
        assert_eq!(tag.metadata.unwrap(), b"(\"Symfony\\Component\\Workflow\\Event\\CompletedEvent\")" as &[u8]);

        let Element::Tag(tag) = &document.elements[1] else {
            panic!("Expected Element::Tag, got {:?}", document.elements[1]);
        };
        assert_eq!(tag.name, b"AnotherAnnotation" as &[u8]);

        let last_idx = document.elements.len() - 1;
        let Element::Tag(tag) = &document.elements[last_idx] else {
            panic!("Expected Element::Tag, got {:?}", document.elements[last_idx]);
        };
        assert_eq!(tag.name, b"SimpleAnnotation" as &[u8]);
        assert!(tag.metadata.is_none());
    }

    #[test]
    fn test_long_description_with_missing_asterisk() {
        let arena = LocalArena::new();
        let phpdoc = b"/** @var string[] this is a really long description
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
        assert_eq!(name, b"var" as &[u8]);
        assert_eq!(tag.kind, TagKind::Var);
        assert_eq!(
            description,
            b"string[] this is a really long description\nthat spans multiple lines, and demonstrates how the parser handles\ndocblocks with multiple descriptions, and missing astricks" as &[u8]
        );
        assert_eq!(
            &phpdoc[tag.span.start_offset() as usize..tag.span.end_offset() as usize],
            b"@var string[] this is a really long description\n            that spans multiple lines, and demonstrates how the parser handles\n            docblocks with multiple descriptions, and missing astricks"
        );
    }

    #[test]
    fn test_code_indent_using_non_ascii_chars() {
        let arena = LocalArena::new();
        let phpdoc = "/**
        *    └─ comment 2
        *       └─ comment 4
        *    └─ comment 3
        */"
        .as_bytes();

        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));
        let document = parse_phpdoc_with_span(&arena, phpdoc, span).expect("Failed to parse PHPDoc");

        assert_eq!(document.elements.len(), 1);

        let Element::Code(code) = &document.elements[0] else {
            panic!("Expected Element::Code, got {:?}", document.elements[0]);
        };

        let content_str = code.content;
        assert_eq!(content_str, "\u{a0} └─ comment 2\n  \u{a0}\u{a0} └─ comment 4\n\u{a0} └─ comment 3".as_bytes());
        assert_eq!(
            &phpdoc[code.span.start_offset() as usize..code.span.end_offset() as usize],
            " \u{a0} └─ comment 2\n        *    \u{a0}\u{a0} └─ comment 4\n        *  \u{a0} └─ comment 3".as_bytes()
        );
    }

    #[test]
    fn test_issue_456() {
        let arena = LocalArena::new();
        let phpdoc = "/**
             * \u{3000}(イベント日数をもとに計算)\u{3000}
             * @return\u{3000}int
             * @throws\u{3000}Exception
             */"
        .as_bytes();

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

        assert_eq!(*content, "\u{3000}(イベント日数をもとに計算)\u{3000}".as_bytes());
        assert_eq!(
            &phpdoc[span.start_offset() as usize..span.end_offset() as usize],
            "\u{3000}(イベント日数をもとに計算)\u{3000}".as_bytes()
        );

        let Element::Tag(tag) = &document.elements[1] else {
            panic!("Expected Element::Tag, got {:?}", document.elements[1]);
        };

        let name = tag.name;
        let description = tag.description;
        assert_eq!(name, "return\u{3000}int".as_bytes());
        assert_eq!(tag.kind, TagKind::Other);
        assert_eq!(description, b"" as &[u8]);
        assert_eq!(
            &phpdoc[tag.span.start_offset() as usize..tag.span.end_offset() as usize],
            "@return\u{3000}int".as_bytes()
        );

        let Element::Tag(tag) = &document.elements[2] else {
            panic!("Expected Element::Tag, got {:?}", document.elements[2]);
        };

        let name = tag.name;
        let description = tag.description;
        assert_eq!(name, "throws\u{3000}Exception".as_bytes());
        assert_eq!(tag.kind, TagKind::Other);
        assert_eq!(description, b"" as &[u8]);
        assert_eq!(
            &phpdoc[tag.span.start_offset() as usize..tag.span.end_offset() as usize],
            "@throws\u{3000}Exception".as_bytes()
        );
    }

    #[test]
    fn test_issue_808() {
        let arena = LocalArena::new();

        let phpdoc = "/** @param\u{3000}string $foo 中文描述 */".as_bytes();
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));
        let document = parse_phpdoc_with_span(&arena, phpdoc, span).expect("Failed to parse PHPDoc");

        assert_eq!(document.elements.len(), 1);
        let Element::Tag(tag) = &document.elements[0] else {
            panic!("Expected Element::Tag, got {:?}", document.elements[0]);
        };
        assert_eq!(tag.name, "param\u{3000}string".as_bytes());
        assert_eq!(tag.description, "$foo 中文描述".as_bytes());

        let phpdoc2 = "/** @return\u{3000}int 返回🎉值 */".as_bytes();
        let span2 = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc2.len() as u32));
        let document2 = parse_phpdoc_with_span(&arena, phpdoc2, span2).expect("Failed to parse PHPDoc");

        assert_eq!(document2.elements.len(), 1);
        let Element::Tag(tag2) = &document2.elements[0] else {
            panic!("Expected Element::Tag, got {:?}", document2.elements[0]);
        };
        assert_eq!(tag2.name, "return\u{3000}int".as_bytes());
        assert_eq!(tag2.description, "返回🎉值".as_bytes());

        let phpdoc3 = "/** @see\u{3000}中文类::方法() 说明 */".as_bytes();
        let span3 = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc3.len() as u32));
        let document3 = parse_phpdoc_with_span(&arena, phpdoc3, span3).expect("Failed to parse PHPDoc");

        assert_eq!(document3.elements.len(), 1);
        let Element::Tag(tag3) = &document3.elements[0] else {
            panic!("Expected Element::Tag, got {:?}", document3.elements[0]);
        };
        assert_eq!(tag3.name, "see\u{3000}中文类::方法".as_bytes());
        assert_eq!(tag3.description, "说明".as_bytes());
    }

    #[test]
    fn test_indented_code_with_fullwidth_space_in_indent() {
        // Test case for multi-byte whitespace in indented code (Issue #967)
        // parse_indented_code is only called when line starts with ASCII space/tab
        // The bug occurs when indent contains full-width space after ASCII spaces
        //
        // After lexer processing, content becomes "  \u{3000}code"
        // is_indented_line returns true (starts with ASCII space)
        // indent_len = 3 (2 ASCII spaces + 1 full-width space char)
        // But byte offset should be 2 + 3 = 5
        let arena = LocalArena::new();
        // Format: " * " (asterisk + space) + "  " (2 ASCII spaces) + "\u{3000}" (full-width) + "code"
        let phpdoc = "/**\n *   \u{3000}code\n */".as_bytes();
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));

        let result = parse_phpdoc_with_span(&arena, phpdoc, span);
        assert!(result.is_ok(), "Parsing should succeed without panic");

        let document = result.unwrap();
        assert_eq!(document.elements.len(), 1);
        let Element::Code(code) = &document.elements[0] else {
            panic!("Expected Element::Code, got {:?}", document.elements[0]);
        };
        assert_eq!(code.content, "\u{3000}code".as_bytes());
    }

    #[test]
    fn test_indented_code_with_mixed_multibyte_whitespace() {
        // Multiple lines with mixed ASCII and full-width whitespace
        let arena = LocalArena::new();
        let phpdoc = "/**\n *  \u{3000}first line\n *  \u{3000}second line\n */".as_bytes();
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));

        let result = parse_phpdoc_with_span(&arena, phpdoc, span);
        assert!(result.is_ok(), "Parsing should succeed without panic");

        let document = result.unwrap();
        assert_eq!(document.elements.len(), 1);
        let Element::Code(code) = &document.elements[0] else {
            panic!("Expected Element::Code, got {:?}", document.elements[0]);
        };
        assert_eq!(code.content, "\u{3000}first line\n\u{3000}second line".as_bytes());
    }

    #[test]
    fn test_indented_code_with_tab_and_fullwidth_space() {
        // Tab + full-width space: is_indented_line checks for '\t' as well
        let arena = LocalArena::new();
        // After "* " there is a tab followed by full-width space
        let phpdoc = "/**\n * \t\u{3000}code\n */".as_bytes();
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));

        let result = parse_phpdoc_with_span(&arena, phpdoc, span);
        assert!(result.is_ok(), "Parsing should succeed without panic");

        let document = result.unwrap();
        assert_eq!(document.elements.len(), 1);
        let Element::Code(code) = &document.elements[0] else {
            panic!("Expected Element::Code, got {:?}", document.elements[0]);
        };
        assert_eq!(code.content, "\u{3000}code".as_bytes());
    }

    #[test]
    fn test_issue_967_original_pattern() {
        // Original Issue #967 reproduction case
        // Error: byte index 3 is not a char boundary; it is inside '\u{3000}' (bytes 1..4) of ` 　 メールクリックがない`
        // After lexer processing: " " + "\u{3000}" + " " + Japanese text
        // This triggers parse_indented_code because line starts with ASCII space
        let arena = LocalArena::new();
        // Format: " * " + " " (1 ASCII space) + "\u{3000}" (full-width) + " " (1 ASCII space) + text
        let phpdoc = "/**\n *  \u{3000} メールクリックがない\n */".as_bytes();
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));

        let result = parse_phpdoc_with_span(&arena, phpdoc, span);
        assert!(result.is_ok(), "Parsing should succeed without panic");

        let document = result.unwrap();
        assert_eq!(document.elements.len(), 1);
        let Element::Code(code) = &document.elements[0] else {
            panic!("Expected Element::Code, got {:?}", document.elements[0]);
        };
        assert_eq!(code.content, "\u{3000} メールクリックがない".as_bytes());
    }

    #[test]
    fn test_multiline_inline_tag() {
        let arena = LocalArena::new();
        let phpdoc = b"/**
            * This method gets a count of the Foo.
            * {@internal Developers should note that it silently
            *            adds one extra Foo.}
            *
            * @return int
            */";

        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));
        let document = parse_phpdoc_with_span(&arena, phpdoc, span).expect("Failed to parse PHPDoc");

        let Element::Text(text) = &document.elements[0] else {
            panic!("Expected Element::Text, got {:?}", document.elements[0]);
        };

        assert!(text.segments.len() >= 2, "Expected at least 2 segments, got {:?}", text.segments);

        let has_inline_tag = text
            .segments
            .iter()
            .any(|seg| matches!(seg, TextSegment::InlineTag(tag) if tag.name == b"internal" as &[u8]));

        assert!(has_inline_tag, "Expected an InlineTag with name 'internal', got segments: {:?}", text.segments);
    }

    #[test]
    fn test_multiline_inline_tag_with_nested() {
        let arena = LocalArena::new();
        let phpdoc = b"/**
            * {@internal Developers should note that it silently
            *            adds one extra Foo (see {@link http://example.com}).}
            */";

        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));
        let document = parse_phpdoc_with_span(&arena, phpdoc, span).expect("Failed to parse PHPDoc");

        let Element::Text(text) = &document.elements[0] else {
            panic!("Expected Element::Text, got {:?}", document.elements[0]);
        };

        let has_internal_tag = text
            .segments
            .iter()
            .any(|seg| matches!(seg, TextSegment::InlineTag(tag) if tag.name == b"internal" as &[u8]));

        assert!(has_internal_tag, "Expected an InlineTag with name 'internal', got segments: {:?}", text.segments);
    }

    #[test]
    fn test_single_line_inline_tag_still_works() {
        let arena = LocalArena::new();
        let phpdoc = br#"/**
            * See {@see \Some\Class} for details.
            */"#;

        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));
        let document = parse_phpdoc_with_span(&arena, phpdoc, span).expect("Failed to parse PHPDoc");

        let Element::Text(text) = &document.elements[0] else {
            panic!("Expected Element::Text, got {:?}", document.elements[0]);
        };

        let has_see_tag =
            text.segments.iter().any(|seg| matches!(seg, TextSegment::InlineTag(tag) if tag.name == b"see" as &[u8]));

        assert!(has_see_tag, "Expected an InlineTag with name 'see', got segments: {:?}", text.segments);
    }

    #[test]
    fn test_multiline_inline_tag_chinese() {
        let arena = LocalArena::new();
        let phpdoc = "/**
            * 获取用户数量的方法。
            * {@internal 开发者请注意，此方法会静默地
            *            添加一个额外的用户。}
            *
            * @return int
            */"
        .as_bytes();

        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));
        let document = parse_phpdoc_with_span(&arena, phpdoc, span).expect("Failed to parse Chinese PHPDoc");

        let Element::Text(text) = &document.elements[0] else {
            panic!("Expected Element::Text, got {:?}", document.elements[0]);
        };

        let has_internal = text
            .segments
            .iter()
            .any(|seg| matches!(seg, TextSegment::InlineTag(tag) if tag.name == b"internal" as &[u8]));
        assert!(has_internal, "Expected InlineTag 'internal' with Chinese content, got: {:?}", text.segments);
    }

    #[test]
    fn test_multiline_inline_tag_japanese() {
        let arena = LocalArena::new();
        let phpdoc = r#"/**
            * ユーザー数を取得するメソッド。
            * {@see \App\Service\UserCounter このクラスは
            *       ユーザーの数を数えます。}
            */"#
        .as_bytes();

        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));
        let document = parse_phpdoc_with_span(&arena, phpdoc, span).expect("Failed to parse Japanese PHPDoc");

        let Element::Text(text) = &document.elements[0] else {
            panic!("Expected Element::Text, got {:?}", document.elements[0]);
        };

        let has_see =
            text.segments.iter().any(|seg| matches!(seg, TextSegment::InlineTag(tag) if tag.name == b"see" as &[u8]));
        assert!(has_see, "Expected InlineTag 'see' with Japanese content, got: {:?}", text.segments);
    }

    #[test]
    fn test_multiline_inline_tag_arabic() {
        let arena = LocalArena::new();
        let phpdoc = "/**
            * طريقة للحصول على عدد المستخدمين.
            * {@internal يجب على المطورين ملاحظة أن هذه الطريقة
            *            تضيف مستخدمًا إضافيًا بصمت.}
            *
            * @return int
            */"
        .as_bytes();

        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));
        let document = parse_phpdoc_with_span(&arena, phpdoc, span).expect("Failed to parse Arabic PHPDoc");

        let Element::Text(text) = &document.elements[0] else {
            panic!("Expected Element::Text, got {:?}", document.elements[0]);
        };

        let has_internal = text
            .segments
            .iter()
            .any(|seg| matches!(seg, TextSegment::InlineTag(tag) if tag.name == b"internal" as &[u8]));
        assert!(has_internal, "Expected InlineTag 'internal' with Arabic content, got: {:?}", text.segments);
    }

    #[test]
    fn test_multiline_inline_tag_mixed_scripts() {
        let arena = LocalArena::new();
        let phpdoc = "/**
            * Documentation with mixed scripts.
            * {@internal 注意: This method は静かに adds один
            *            дополнительный элемент 요소를 추가합니다.}
            */"
        .as_bytes();

        let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));
        let document = parse_phpdoc_with_span(&arena, phpdoc, span).expect("Failed to parse mixed-script PHPDoc");

        let Element::Text(text) = &document.elements[0] else {
            panic!("Expected Element::Text, got {:?}", document.elements[0]);
        };

        let has_internal = text
            .segments
            .iter()
            .any(|seg| matches!(seg, TextSegment::InlineTag(tag) if tag.name == b"internal" as &[u8]));
        assert!(has_internal, "Expected InlineTag 'internal' with mixed-script content, got: {:?}", text.segments);
    }
}
