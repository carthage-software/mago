use mago_interner::ThreadedInterner;
use mago_span::Span;
use mago_syntax::ast::Trivia;
use mago_syntax::ast::TriviaKind;

use crate::document::Document;
use crate::error::ParseError;

mod internal;

pub mod document;
pub mod error;

#[inline]
pub fn parse_trivia(interner: &ThreadedInterner, trivia: &Trivia) -> Result<Document, ParseError> {
    if TriviaKind::DocBlockComment != trivia.kind {
        return Err(ParseError::InvalidTrivia(trivia.span));
    }

    parse_phpdoc_with_span(interner, interner.lookup(&trivia.value), trivia.span)
}

#[inline]
pub fn parse_phpdoc_with_span(interner: &ThreadedInterner, content: &str, span: Span) -> Result<Document, ParseError> {
    let tokens = internal::lexer::tokenize(content, span)?;

    internal::parser::parse_document(tokens.as_slice(), interner)
}

#[cfg(test)]
mod tests {
    use super::*;

    use mago_interner::ThreadedInterner;
    use mago_span::Position;
    use mago_span::Span;

    use crate::document::*;

    #[test]
    fn test_parse_all_elements() {
        let interner = ThreadedInterner::new();
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

        let span = Span::new(Position::dummy(0), Position::dummy(phpdoc.len()));
        let document = parse_phpdoc_with_span(&interner, phpdoc, span).expect("Failed to parse PHPDoc");
        assert_eq!(document.elements.len(), 12);

        let Element::Text(text) = &document.elements[0] else {
            panic!("Expected Element::Text, got {:?}", document.elements[0]);
        };

        assert_eq!(text.segments.len(), 1);

        let TextSegment::Paragraph { span, content } = &text.segments[0] else {
            panic!("Expected TextSegment::Paragraph, got {:?}", text.segments[0]);
        };

        let content = interner.lookup(content);
        assert_eq!(content, "This is a simple description.");
        assert_eq!(&phpdoc[span.start.offset..span.end.offset], "This is a simple description.");

        let Element::Line(_) = &document.elements[1] else {
            panic!("Expected Element::Line, got {:?}", document.elements[1]);
        };

        let Element::Text(text) = &document.elements[2] else {
            panic!("Expected Element::Text, got {:?}", document.elements[2]);
        };

        assert_eq!(text.segments.len(), 3);

        let TextSegment::Paragraph { span, content } = &text.segments[0] else {
            panic!("Expected TextSegment::Paragraph, got {:?}", text.segments[0]);
        };

        let content = interner.lookup(content);
        assert_eq!(content, "This text contains an inline code ");
        assert_eq!(&phpdoc[span.start.offset..span.end.offset], "This text contains an inline code ");

        let TextSegment::InlineCode(code) = &text.segments[1] else {
            panic!("Expected TextSegment::InlineCode, got {:?}", text.segments[1]);
        };

        let content = interner.lookup(&code.content);
        assert_eq!(content, "echo \"Hello, World!\";");
        assert_eq!(&phpdoc[code.span.start.offset..code.span.end.offset], "`echo \"Hello, World!\";`");

        let TextSegment::Paragraph { span, content } = &text.segments[2] else {
            panic!("Expected TextSegment::Paragraph, got {:?}", text.segments[2]);
        };

        let content = interner.lookup(content);
        assert_eq!(content, ".");
        assert_eq!(&phpdoc[span.start.offset..span.end.offset], ".");

        let Element::Line(_) = &document.elements[3] else {
            panic!("Expected Element::Line, got {:?}", document.elements[3]);
        };

        let Element::Text(text) = &document.elements[4] else {
            panic!("Expected Element::Text, got {:?}", document.elements[4]);
        };

        assert_eq!(text.segments.len(), 3);

        let TextSegment::Paragraph { span, content } = &text.segments[0] else {
            panic!("Expected TextSegment::Paragraph, got {:?}", text.segments[0]);
        };

        let content = interner.lookup(content);
        assert_eq!(content, "This text contains an inline tag ");
        assert_eq!(&phpdoc[span.start.offset..span.end.offset], "This text contains an inline tag ");

        let TextSegment::InlineTag(tag) = &text.segments[1] else {
            panic!("Expected TextSegment::InlineTag, got {:?}", text.segments[1]);
        };

        let name = interner.lookup(&tag.name);
        let description = interner.lookup(&tag.description);
        assert_eq!(name, "see");
        assert_eq!(description, "\\Some\\Class");
        assert_eq!(tag.kind, TagKind::See);
        assert_eq!(&phpdoc[tag.span.start.offset..tag.span.end.offset], "{@see \\Some\\Class}");

        let TextSegment::Paragraph { span, content } = &text.segments[2] else {
            panic!("Expected TextSegment::Paragraph, got {:?}", text.segments[2]);
        };

        let content = interner.lookup(content);
        assert_eq!(content, ".");
        assert_eq!(&phpdoc[span.start.offset..span.end.offset], ".");

        let Element::Line(_) = &document.elements[5] else {
            panic!("Expected Element::Line, got {:?}", document.elements[5]);
        };

        let Element::Code(code) = &document.elements[6] else {
            panic!("Expected Element::CodeBlock, got {:?}", document.elements[6]);
        };

        let content = interner.lookup(&code.content);
        let directives = code.directives.iter().map(|d| interner.lookup(d)).collect::<Vec<_>>();
        assert_eq!(directives, &["php"]);
        assert_eq!(content, "echo \"Hello, World!\";");
        assert_eq!(
            &phpdoc[code.span.start.offset..code.span.end.offset],
            "```php\n            * echo \"Hello, World!\";\n            * ```"
        );

        let Element::Line(_) = &document.elements[7] else {
            panic!("Expected Element::Line, got {:?}", document.elements[7]);
        };

        let Element::Code(code) = &document.elements[8] else {
            panic!("Expected Element::CodeBlock, got {:?}", document.elements[8]);
        };

        let content = interner.lookup(&code.content);
        assert!(code.directives.is_empty());
        assert_eq!(content, "$foo = \"bar\";\necho \"Hello, World!\";\n");
        assert_eq!(
            &phpdoc[code.span.start.offset..code.span.end.offset],
            "    $foo = \"bar\";\n            *     echo \"Hello, World!\";\n"
        );

        let Element::Tag(tag) = &document.elements[9] else {
            panic!("Expected Element::Tag, got {:?}", document.elements[9]);
        };

        let name = interner.lookup(&tag.name);
        let description = interner.lookup(&tag.description);
        assert_eq!(name, "param");
        assert_eq!(tag.kind, TagKind::Param);
        assert_eq!(description, "string $foo");
        assert_eq!(&phpdoc[tag.span.start.offset..tag.span.end.offset], "@param string $foo");

        let Element::Tag(tag) = &document.elements[10] else {
            panic!("Expected Element::Tag, got {:?}", document.elements[10]);
        };

        let name = interner.lookup(&tag.name);
        let description = interner.lookup(&tag.description);
        assert_eq!(name, "param");
        assert_eq!(tag.kind, TagKind::Param);
        assert_eq!(description, "array{\n  bar: string,\n  baz: int\n} $bar");
        assert_eq!(
            &phpdoc[tag.span.start.offset..tag.span.end.offset],
            "@param array{\n            *   bar: string,\n            *   baz: int\n            * } $bar"
        );

        let Element::Tag(tag) = &document.elements[11] else {
            panic!("Expected Element::Tag, got {:?}", document.elements[11]);
        };

        let name = interner.lookup(&tag.name);
        let description = interner.lookup(&tag.description);
        assert_eq!(name, "return");
        assert_eq!(tag.kind, TagKind::Return);
        assert_eq!(description, "void");
        assert_eq!(&phpdoc[tag.span.start.offset..tag.span.end.offset], "@return void");
    }

    #[test]
    fn test_unclosed_inline_tag() {
        // Test case for ParseError::UnclosedInlineTag
        let interner = ThreadedInterner::new();
        let phpdoc = "/** This is a doc block with an unclosed inline tag {@see Class */";
        let span = Span::new(Position::dummy(0), Position::dummy(phpdoc.len()));

        let result = parse_phpdoc_with_span(&interner, phpdoc, span);

        match result {
            Err(ParseError::UnclosedInlineTag(error_span)) => {
                let expected_start = phpdoc.find("{@see").unwrap();
                let expected_span = span.subspan(expected_start, phpdoc.len() - 3);
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
        let interner = ThreadedInterner::new();
        let phpdoc = "/** This is a doc block with unclosed inline code `code sample */";
        let span = Span::new(Position::dummy(0), Position::dummy(phpdoc.len()));

        let result = parse_phpdoc_with_span(&interner, phpdoc, span);

        match result {
            Err(ParseError::UnclosedInlineCode(error_span)) => {
                let expected_start = phpdoc.find('`').unwrap();
                let expected_span = span.subspan(expected_start, phpdoc.len() - 3);
                assert_eq!(error_span, expected_span);
            }
            _ => {
                panic!("Expected ParseError::UnclosedInlineCode");
            }
        }
    }

    #[test]
    fn test_unclosed_code_block() {
        let interner = ThreadedInterner::new();
        let phpdoc = r#"/**
            * This is a doc block with unclosed code block
            * ```
            * Some code here
            */"#;
        let span = Span::new(Position::dummy(0), Position::dummy(phpdoc.len()));

        let result = parse_phpdoc_with_span(&interner, phpdoc, span);

        match result {
            Err(ParseError::UnclosedCodeBlock(error_span)) => {
                let code_block_start = phpdoc.find("```").unwrap();
                let expected_span = span.subspan(code_block_start, 109);
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
        let interner = ThreadedInterner::new();
        let phpdoc = "/** @invalid_tag_name Description */";
        let span = Span::new(Position::dummy(0), Position::dummy(phpdoc.len()));

        let result = parse_phpdoc_with_span(&interner, phpdoc, span);

        match result {
            Err(ParseError::InvalidTagName(error_span)) => {
                let tag_start = phpdoc.find("@invalid_tag_name").unwrap();
                let tag_end = tag_start + "@invalid_tag_name".len();
                let expected_span = span.subspan(tag_start, tag_end);
                assert_eq!(error_span, expected_span);
            }
            _ => {
                panic!("Expected ParseError::InvalidTagName");
            }
        }
    }

    #[test]
    fn test_malformed_code_block() {
        let interner = ThreadedInterner::new();
        let phpdoc = r#"/**
            * ```
            * Some code here
            * Incorrect closing
            */"#;
        let span = Span::new(Position::dummy(0), Position::dummy(phpdoc.len()));

        let result = parse_phpdoc_with_span(&interner, phpdoc, span);

        match result {
            Ok(document) => {
                panic!("Expected the parser to return an error, got {:#?}", document);
            }
            Err(ParseError::UnclosedCodeBlock(error_span)) => {
                let code_block_start = phpdoc.find("```").unwrap();
                let expected_span = span.subspan(code_block_start, 82);
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
        let interner = ThreadedInterner::new();
        let phpdoc = "/* Not a valid doc block */";
        let span = Span::new(Position::dummy(0), Position::dummy(phpdoc.len()));

        let result = parse_phpdoc_with_span(&interner, phpdoc, span);

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
        let interner = ThreadedInterner::new();
        let phpdoc = r#"/**
    * This is a doc block
      * With inconsistent indentation
    */"#;
        let span = Span::new(Position::dummy(0), Position::dummy(phpdoc.len()));

        let result = parse_phpdoc_with_span(&interner, phpdoc, span);

        match result {
            Err(ParseError::InconsistentIndentation(error_span, expected, found)) => {
                // The expected and found indentation lengths
                assert_eq!(expected, 4); // Assuming 4 spaces
                assert_eq!(found, 6); // Assuming 6 spaces
                // The error_span should point to the line with inconsistent indentation
                let inconsistent_line = "      * With inconsistent indentation";
                let line_start = phpdoc.find(inconsistent_line).unwrap();
                let indent_length = inconsistent_line.chars().take_while(|c| c.is_whitespace()).count();
                let expected_span = span.subspan(line_start, line_start + indent_length);
                assert_eq!(error_span, expected_span);
            }
            _ => {
                panic!("Expected ParseError::InconsistentIndentation");
            }
        }
    }

    #[test]
    fn test_missing_asterisk() {
        // Test case for ParseError::MissingAsterisk
        let interner = ThreadedInterner::new();
        let phpdoc = r#"/**
     This line is missing an asterisk
     * This line is fine
     */"#;
        let span = Span::new(Position::dummy(0), Position::dummy(phpdoc.len()));

        let result = parse_phpdoc_with_span(&interner, phpdoc, span);

        match result {
            Err(ParseError::MissingAsterisk(error_span)) => {
                // The error_span should point to where the asterisk is missing
                let problematic_line = "     This line is missing an asterisk";
                let line_start = phpdoc.find(problematic_line).unwrap();
                let indent_length = problematic_line.chars().take_while(|c| c.is_whitespace()).count();
                let expected_span = span.subspan(line_start + indent_length, line_start + indent_length + 1);
                assert_eq!(error_span, expected_span);
            }
            _ => {
                panic!("Expected ParseError::MissingAsterisk");
            }
        }
    }

    #[test]
    fn test_missing_whitespace_after_asterisk() {
        // Test case for ParseError::MissingWhitespaceAfterAsterisk
        let interner = ThreadedInterner::new();
        let phpdoc = r#"/**
     *This line is missing a space after asterisk
     */"#;
        let span = Span::new(Position::dummy(0), Position::dummy(phpdoc.len()));

        let result = parse_phpdoc_with_span(&interner, phpdoc, span);

        match result {
            Err(ParseError::MissingWhitespaceAfterAsterisk(error_span)) => {
                // The error_span should point to the character after the asterisk
                let problematic_line = "*This line is missing a space after asterisk";
                let line_start = phpdoc.find(problematic_line).unwrap();
                let asterisk_pos = line_start;
                let expected_span = span.subspan(asterisk_pos + 1, asterisk_pos + 2);
                assert_eq!(error_span, expected_span);
            }
            _ => {
                panic!("Expected ParseError::MissingWhitespaceAfterAsterisk");
            }
        }
    }

    #[test]
    fn test_missing_whitespace_after_opening_asterisk() {
        // Test case for ParseError::MissingWhitespaceAfterOpeningAsterisk
        let interner = ThreadedInterner::new();
        let phpdoc = "/**This is a doc block without space after /** */";
        let span = Span::new(Position::dummy(0), Position::dummy(phpdoc.len()));

        let result = parse_phpdoc_with_span(&interner, phpdoc, span);

        match result {
            Err(ParseError::MissingWhitespaceAfterOpeningAsterisk(error_span)) => {
                // The error_span should point to the position after '/**'
                let expected_span = span.subspan(3, 4);
                assert_eq!(error_span, expected_span);
            }
            _ => {
                panic!("Expected ParseError::MissingWhitespaceAfterOpeningAsterisk");
            }
        }
    }

    #[test]
    fn test_missing_whitespace_before_closing_asterisk() {
        // Test case for ParseError::MissingWhitespaceBeforeClosingAsterisk
        let interner = ThreadedInterner::new();
        let phpdoc = "/** This is a doc block without space before */*/";
        let span = Span::new(Position::dummy(0), Position::dummy(phpdoc.len()));

        let result = parse_phpdoc_with_span(&interner, phpdoc, span);

        match result {
            Err(ParseError::MissingWhitespaceBeforeClosingAsterisk(error_span)) => {
                // The error_span should point to the position before '*/'
                let expected_span = span.subspan(phpdoc.len() - 3, phpdoc.len() - 2);
                assert_eq!(error_span, expected_span);
            }
            _ => {
                panic!("Expected ParseError::MissingWhitespaceBeforeClosingAsterisk");
            }
        }
    }

    #[test]
    fn test_utf8_characters() {
        let interner = ThreadedInterner::new();
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

        let span = Span::new(Position::dummy(0), Position::dummy(phpdoc.len()));
        let document = parse_phpdoc_with_span(&interner, phpdoc, span).expect("Failed to parse PHPDoc");

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

        let content_str = interner.lookup(content);
        assert_eq!(
            content_str,
            "هذا نص باللغة العربية.\n这是一段中文。\nHere are some mathematical symbols: ∑, ∆, π, θ."
        );

        assert_eq!(
            &phpdoc[span.start.offset..span.end.offset],
            "هذا نص باللغة العربية.\n    * 这是一段中文。\n    * Here are some mathematical symbols: ∑, ",
        );

        // Empty line
        let Element::Line(_) = &document.elements[1] else {
            panic!("Expected Element::Line, got {:?}", document.elements[3]);
        };

        // Code block
        let Element::Code(code) = &document.elements[2] else {
            panic!("Expected Element::Code, got {:?}", document.elements[2]);
        };

        let content_str = interner.lookup(&code.content);
        let expected_code = "// Arabic comment\necho \"مرحبا بالعالم\";\n// Chinese comment\necho \"你好，世界\";\n// Math symbols in code\n$sum = $a + $b; // ∑";
        assert_eq!(content_str, expected_code);
        assert_eq!(
            &phpdoc[code.span.start.offset..code.span.end.offset],
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

        let name = interner.lookup(&tag.name);
        let description = interner.lookup(&tag.description);
        assert_eq!(name, "param");
        assert_eq!(tag.kind, TagKind::Param);
        assert_eq!(description, "string $مثال A parameter with an Arabic variable name.");
        assert_eq!(
            &phpdoc[tag.span.start.offset..tag.span.end.offset],
            "@param string $مثال A parameter with an Arabic variable name."
        );

        // @return tag with Chinese description
        let Element::Tag(tag) = &document.elements[5] else {
            panic!("Expected Element::Tag, got {:?}", document.elements[5]);
        };

        let name = interner.lookup(&tag.name);
        let description = interner.lookup(&tag.description);
        assert_eq!(name, "return");
        assert_eq!(tag.kind, TagKind::Return);
        assert_eq!(description, "int 返回值是整数类型。");
        assert_eq!(&phpdoc[tag.span.start.offset..tag.span.end.offset], "@return int 返回值是整数类型。");
    }

    #[test]
    fn test_annotation_parsing() {
        let interner = ThreadedInterner::new();
        let phpdoc = r#"/**
         * @Event("Symfony\Component\Workflow\Event\CompletedEvent")
         * @AnotherAnnotation({
         *     "key": "value",
         *     "list": [1, 2, 3]
         * })
         * @SimpleAnnotation
         */"#;
        let span = Span::new(Position::dummy(0), Position::dummy(phpdoc.len()));
        let document = parse_phpdoc_with_span(&interner, phpdoc, span).expect("Failed to parse PHPDoc");

        // Verify that the document has the expected number of elements
        assert_eq!(document.elements.len(), 3);

        // First annotation
        let Element::Annotation(annotation) = &document.elements[0] else {
            panic!("Expected Element::Annotation, got {:?}", document.elements[0]);
        };

        let name = interner.lookup(&annotation.name);
        assert_eq!(name, "Event");
        let arguments = interner.lookup(&annotation.arguments.unwrap());
        assert_eq!(arguments, "(\"Symfony\\Component\\Workflow\\Event\\CompletedEvent\")");

        // Second annotation
        let Element::Annotation(annotation) = &document.elements[1] else {
            panic!("Expected Element::Annotation, got {:?}", document.elements[1]);
        };

        let name = interner.lookup(&annotation.name);
        assert_eq!(name, "AnotherAnnotation");
        let arguments = interner.lookup(&annotation.arguments.unwrap());
        let expected_arguments = "({\n    \"key\": \"value\",\n    \"list\": [1, 2, 3]\n})";
        assert_eq!(arguments, expected_arguments);

        // Third annotation
        let Element::Annotation(annotation) = &document.elements[2] else {
            panic!("Expected Element::Annotation, got {:?}", document.elements[2]);
        };

        let name = interner.lookup(&annotation.name);
        assert_eq!(name, "SimpleAnnotation");
        assert!(annotation.arguments.is_none());
    }
}
