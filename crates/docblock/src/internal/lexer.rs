use mago_span::Span;

use crate::error::ParseError;
use crate::internal::token::Token;

#[inline]
pub fn tokenize<'a>(comment: &'a str, span: Span) -> Result<Vec<Token<'a>>, ParseError> {
    if comment.len() < 5 || !comment.starts_with("/**") || !comment.ends_with("*/") {
        return Err(ParseError::InvalidComment(span));
    }

    let content_start = 3;
    let content_end = comment.len() - 2;

    let content = &comment[content_start..content_end];

    if !content.contains('\n') {
        if content.is_empty() {
            return Ok(Vec::new());
        }

        if !content.starts_with(' ') {
            let error_span = span.subspan(content_start, content_start + 1);
            return Err(ParseError::MissingWhitespaceAfterOpeningAsterisk(error_span));
        }

        if !content.ends_with(' ') {
            let error_span = span.subspan(content_end - 1, content_end);
            return Err(ParseError::MissingWhitespaceBeforeClosingAsterisk(error_span));
        }

        let content_len = content.len();

        if content_len < 3 {
            return Ok(Vec::new());
        }

        Ok(vec![Token::Line {
            content: &content[1..content_len - 1],
            span: span.subspan(content_start + 1, content_end - 1),
        }])
    } else {
        let lines: Vec<&'a str> = content.lines().collect();

        let mut indent: Option<&str> = None;
        let mut lines_with_positions = Vec::new();
        let mut pos_in_content = 0;

        for line in lines {
            let line_len = line.len();
            lines_with_positions.push((line, pos_in_content));
            pos_in_content += line_len + 1;
        }

        let mut comment_lines = Vec::new();
        for (line, line_start_in_content) in lines_with_positions {
            let trimmed_line = line.trim_end();

            if trimmed_line.trim().is_empty() {
                continue;
            }

            // Find the byte length of the whitespace prefix.
            let line_indent_length = trimmed_line.find(|c: char| !c.is_whitespace()).unwrap_or(trimmed_line.len());
            let line_indent = &trimmed_line[..line_indent_length];

            match indent {
                Some(i) => {
                    if i != line_indent {
                        let expected = i.len();
                        let found = line_indent.len();
                        let error_span = span.subspan(
                            content_start + line_start_in_content,
                            content_start + line_start_in_content + line_indent_length,
                        );

                        return Err(ParseError::InconsistentIndentation(error_span, expected, found));
                    }
                }
                None => {
                    indent = Some(line_indent);
                }
            }

            let line_content_after_indent = &trimmed_line[line_indent_length..];

            if !line_content_after_indent.starts_with('*') {
                let error_span = span.subspan(
                    content_start + line_start_in_content + line_indent_length,
                    content_start + line_start_in_content + line_indent_length + 1,
                );
                return Err(ParseError::MissingAsterisk(error_span));
            }

            let line_after_asterisk = &line_content_after_indent[1..];

            if let Some(first_char) = line_after_asterisk.chars().next() {
                if !first_char.is_whitespace() {
                    let error_span = span.subspan(
                        content_start + line_start_in_content + line_indent_length + 1,
                        content_start + line_start_in_content + line_indent_length + 1 + first_char.len_utf8(),
                    );
                    return Err(ParseError::MissingWhitespaceAfterAsterisk(error_span));
                }

                let content_start_in_line = line_indent_length + 1 + first_char.len_utf8();
                let content_end_in_line = trimmed_line.len();

                let content_start_in_comment = content_start + line_start_in_content + content_start_in_line;
                let content_end_in_comment = content_start + line_start_in_content + content_end_in_line;

                let content_str = &comment[content_start_in_comment..content_end_in_comment];

                let content_span = span.subspan(content_start_in_comment, content_end_in_comment);

                comment_lines.push(Token::Line { content: content_str, span: content_span });
            } else {
                comment_lines.push(Token::EmptyLine {
                    span: span.subspan(content_start + line_start_in_content, content_start + line_start_in_content),
                });

                continue;
            }
        }

        Ok(comment_lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use mago_span::Position;

    #[test]
    fn test_lex_empty_single_line_comment() {
        let comment = "/***/";
        let span = Span::new(Position::dummy(0), Position::dummy(comment.len()));

        match tokenize(comment, span) {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 0);
            }
            Err(e) => {
                panic!("Error parsing comment: {e:?}");
            }
        }
    }

    #[test]
    fn test_lex_empty_multiline_line_comment() {
        let comment = "/**\n*/";
        let span = Span::new(Position::dummy(0), Position::dummy(comment.len()));

        match tokenize(comment, span) {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 0);
            }
            Err(e) => {
                panic!("Error parsing comment: {e:?}");
            }
        }
    }

    #[test]
    fn test_lex_single_line_comment() {
        let comment = "/** This is a single-line comment */";
        let span = Span::new(Position::dummy(0), Position::dummy(comment.len()));

        match tokenize(comment, span) {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 1);

                let Token::Line { content, span } = &tokens[0] else {
                    panic!("Expected a line, but got something else");
                };

                assert_eq!(*content, "This is a single-line comment");
                assert!(comment[span.start.offset..span.end.offset].eq(*content));
            }
            Err(e) => {
                panic!("Error parsing comment: {e:?}");
            }
        }
    }

    #[test]
    fn test_lex_single_line_comment_missing_whitespace_front() {
        let comment = "/**This is a single-line comment */";
        let span = Span::new(Position::dummy(0), Position::dummy(comment.len()));

        match tokenize(comment, span) {
            Ok(_) => {
                panic!("Expected an error, but got success");
            }
            Err(ParseError::MissingWhitespaceAfterOpeningAsterisk { .. }) => {
                // Expected
            }
            Err(e) => {
                panic!("Error parsing comment: {e:?}");
            }
        }
    }

    #[test]
    fn test_lex_single_line_comment_missing_whitespace_back() {
        let comment = "/** This is a single-line comment*/";
        let span = Span::new(Position::dummy(0), Position::dummy(comment.len()));

        match tokenize(comment, span) {
            Ok(_) => {
                panic!("Expected an error, but got success");
            }
            Err(ParseError::MissingWhitespaceBeforeClosingAsterisk { .. }) => {
                // Expected
            }
            Err(e) => {
                panic!("Error parsing comment: {e:?}");
            }
        }
    }

    #[test]
    fn test_lex_multi_line_comment() {
        let comment = r#"/**
                * This is a multi-line comment.
                * It has multiple lines.
                * Each line starts with an asterisk.
                */"#;

        let span = Span::new(Position::dummy(0), Position::dummy(comment.len()));

        match tokenize(comment, span) {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 3);

                let expected_contents =
                    ["This is a multi-line comment.", "It has multiple lines.", "Each line starts with an asterisk."];

                for (i, line) in tokens.iter().enumerate() {
                    let Token::Line { content, span } = line else {
                        panic!("Expected a line, but got something else");
                    };

                    assert_eq!(*content, expected_contents[i]);
                    assert!(comment[span.start.offset..span.end.offset].eq(*content));
                }
            }
            Err(e) => {
                panic!("Error parsing comment: {e:?}");
            }
        }
    }

    #[test]
    fn test_lex_multi_line_comment_indent() {
        let comment = r#"/**
                * This is a multi-line comment.
                * It has multiple lines.
                * Each line starts with an asterisk.
                *
                *     $foo = "bar";
                *     $bar = "baz";
                */"#;

        let span = Span::new(Position::dummy(0), Position::dummy(comment.len()));

        match tokenize(comment, span) {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 6);

                let expected_contents = [
                    "This is a multi-line comment.",
                    "It has multiple lines.",
                    "Each line starts with an asterisk.",
                    "",
                    "    $foo = \"bar\";",
                    "    $bar = \"baz\";",
                ];

                for (i, line) in tokens.iter().enumerate() {
                    let expected_content = expected_contents[i];
                    if expected_content.is_empty() {
                        match line {
                            Token::EmptyLine { span } => {
                                assert_eq!(&comment[span.start.offset..span.end.offset], "");
                            }
                            _ => {
                                panic!("Expected an empty line, but got something else");
                            }
                        }
                    } else {
                        let Token::Line { content, span } = line else {
                            panic!("Expected a line, but got something else");
                        };

                        assert_eq!(*content, expected_content);
                        assert!(comment[span.start.offset..span.end.offset].eq(*content));
                    }
                }
            }
            Err(e) => {
                panic!("Error parsing comment: {e:?}");
            }
        }
    }

    #[test]
    fn test_lex_multi_line_comment_inconsistent_indentation() {
        let comment = r#"/**
        * This is a multi-line comment.
            * It has multiple lines.
        * Each line starts with an asterisk.
        */"#;

        let span = Span::new(Position::dummy(0), Position::dummy(comment.len()));

        match tokenize(comment, span) {
            Ok(_) => {
                panic!("Parsing should have failed due to inconsistent indentation.");
            }
            Err(ParseError::InconsistentIndentation { .. }) => {
                // Correctly identified inconsistent indentation
            }
            Err(e) => {
                panic!("Unexpected error: {e:?}");
            }
        }
    }

    #[test]
    fn test_lex_multi_line_comment_missing_asterisk() {
        let comment = r#"/**
        * This is a multi-line comment.
        It has multiple lines.
        * Each line starts with an asterisk.
        */"#;

        let span = Span::new(Position::dummy(0), Position::dummy(comment.len()));

        match tokenize(comment, span) {
            Ok(_) => {
                panic!("Parsing should have failed due to missing asterisk.");
            }
            Err(ParseError::MissingAsterisk { .. }) => {
                // Correctly identified missing asterisk
            }
            Err(e) => {
                panic!("Unexpected error: {e:?}");
            }
        }
    }

    #[test]
    fn test_lex_multi_line_comment_missing_whitespace_after_asterisk() {
        let comment = r#"/**
        * This is a multi-line comment.
        *It has multiple lines.
        * Each line starts with an asterisk.
        */"#;

        let span = Span::new(Position::dummy(0), Position::dummy(comment.len()));

        match tokenize(comment, span) {
            Ok(_) => {
                panic!("Parsing should have failed due to missing whitespace after asterisk.");
            }
            Err(ParseError::MissingWhitespaceAfterAsterisk { .. }) => {
                // Correctly identified missing whitespace after asterisk
            }
            Err(e) => {
                panic!("Unexpected error: {e:?}");
            }
        }
    }
}
