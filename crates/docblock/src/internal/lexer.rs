use mago_span::Span;

use crate::error::ParseError;
use crate::internal::token::Token;

#[inline]
pub fn tokenize<'a>(comment: &'a str, span: Span) -> Result<Vec<Token<'a>>, ParseError> {
    if comment.len() < 5 || !comment.starts_with("/**") || !comment.ends_with("*/") {
        return Err(ParseError::InvalidComment(span));
    }

    let mut content_start = 3u32;
    let mut content_end = (comment.len() - 2) as u32;

    let content = &comment[3..(comment.len() - 2)];

    if content.contains('\n') {
        let lines_with_positions: Vec<(&'a str, u32)> = content
            .split('\n')
            .map(|line| {
                let cleaned_line = line.strip_suffix('\r').unwrap_or(line);

                let start_offset = (cleaned_line.as_ptr() as usize - content.as_ptr() as usize) as u32;

                (cleaned_line, start_offset)
            })
            .collect();

        let mut comment_lines = Vec::new();
        for (line, line_start_in_content) in lines_with_positions {
            let trimmed_line = line.trim_end();

            if trimmed_line.trim().is_empty() {
                continue;
            }

            let line_indent_length = trimmed_line.find(|c: char| !c.is_whitespace()).unwrap_or(trimmed_line.len());
            let line_content_after_indent = &trimmed_line[line_indent_length..];

            let mut content_start_in_line = line_indent_length as u32;
            let line_after_asterisk = if let Some(line_after_asterisk) = line_content_after_indent.strip_prefix('*') {
                content_start_in_line += 1;
                line_after_asterisk
            } else {
                line_content_after_indent
            };

            if let Some(first_char) = line_after_asterisk.chars().next() {
                if first_char.is_whitespace() {
                    content_start_in_line += first_char.len_utf8() as u32;
                }

                let content_end_in_line = trimmed_line.len() as u32;

                let content_start_in_comment = content_start + line_start_in_content + content_start_in_line;
                let content_end_in_comment = content_start + line_start_in_content + content_end_in_line;

                let content_str = &comment[content_start_in_comment as usize..content_end_in_comment as usize];
                let content_span = span.subspan(content_start_in_comment, content_end_in_comment);

                comment_lines.push(Token::Line { content: content_str, span: content_span });
            } else {
                comment_lines.push(Token::EmptyLine {
                    span: span.subspan(content_start + line_start_in_content, content_start + line_start_in_content),
                });
            }
        }

        Ok(comment_lines)
    } else {
        if content.is_empty() {
            return Ok(Vec::new());
        }

        let content = if let Some(content) = content.strip_prefix(' ') {
            content_start += 1; // Adjust start position to skip leading space
            content
        } else {
            content
        };

        let content = if let Some(content) = content.strip_suffix(' ') {
            content_end -= 1; // Adjust end position to skip trailing space
            content
        } else {
            content
        };

        if content.is_empty() {
            return Ok(Vec::new());
        }

        Ok(vec![Token::Line { content, span: span.subspan(content_start, content_end) }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use mago_database::file::FileId;
    use mago_span::Position;

    #[test]
    fn test_lex_empty_single_line_comment() {
        let comment = "/***/";
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(comment.len() as u32));

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
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(comment.len() as u32));

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
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(comment.len() as u32));

        match tokenize(comment, span) {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 1);

                let Token::Line { content, span } = &tokens[0] else {
                    panic!("Expected a line, but got something else");
                };

                assert_eq!(*content, "This is a single-line comment");
                assert!(comment[span.start.offset as usize..span.end.offset as usize].eq(*content));
            }
            Err(e) => {
                panic!("Error parsing comment: {e:?}");
            }
        }
    }

    #[test]
    fn test_lex_single_line_comment_missing_whitespace_front() {
        let comment = "/**This is a single-line comment */";
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(comment.len() as u32));

        match tokenize(comment, span) {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 1);

                let Token::Line { content, span } = &tokens[0] else {
                    panic!("Expected a line, but got something else");
                };

                assert_eq!(*content, "This is a single-line comment");
                assert!(comment[span.start.offset as usize..span.end.offset as usize].eq(*content));
            }
            Err(e) => {
                panic!("Error parsing comment: {e:?}");
            }
        }
    }

    #[test]
    fn test_lex_single_line_comment_missing_whitespace_back() {
        let comment = "/** This is a single-line comment*/";
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(comment.len() as u32));

        match tokenize(comment, span) {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 1);

                let Token::Line { content, span } = &tokens[0] else {
                    panic!("Expected a line, but got something else");
                };

                assert_eq!(*content, "This is a single-line comment");
                assert!(comment[span.start.offset as usize..span.end.offset as usize].eq(*content));
            }
            Err(e) => {
                panic!("Error parsing comment: {e:?}");
            }
        }
    }

    #[test]
    fn test_lex_multi_line_comment() {
        let comment = r"/**
                * This is a multi-line comment.
                * It has multiple lines.
                * Each line starts with an asterisk.
                */";

        let span = Span::new(FileId::zero(), Position::new(0), Position::new(comment.len() as u32));

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
                    assert!(comment[span.start.offset as usize..span.end.offset as usize].eq(*content));
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

        let span = Span::new(FileId::zero(), Position::new(0), Position::new(comment.len() as u32));

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
                                assert_eq!(&comment[span.start.offset as usize..span.end.offset as usize], "");
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
                        assert!(comment[span.start.offset as usize..span.end.offset as usize].eq(*content));
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
        let comment = r"/**
        * This is a multi-line comment.
            * It has multiple lines.
        * Each line starts with an asterisk.
        */";

        let span = Span::new(FileId::zero(), Position::new(0), Position::new(comment.len() as u32));

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
                    assert!(comment[span.start.offset as usize..span.end.offset as usize].eq(*content));
                }
            }
            Err(e) => {
                panic!("Unexpected error: {e:?}");
            }
        }
    }

    #[test]
    fn test_lex_multi_line_comment_missing_asterisk() {
        let comment = r"/**
        * This is a multi-line comment.
        It has multiple lines.
        * Each line starts with an asterisk.
        */";

        let span = Span::new(FileId::zero(), Position::new(0), Position::new(comment.len() as u32));

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
                    assert!(comment[span.start.offset as usize..span.end.offset as usize].eq(*content));
                }
            }
            Err(e) => {
                panic!("Unexpected error: {e:?}");
            }
        }
    }

    #[test]
    fn test_lex_multi_line_comment_missing_whitespace_after_asterisk() {
        let comment = r"/**
        * This is a multi-line comment.
        *It has multiple lines.
        * Each line starts with an asterisk.
        */";

        let span = Span::new(FileId::zero(), Position::new(0), Position::new(comment.len() as u32));

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
                    assert!(comment[span.start.offset as usize..span.end.offset as usize].eq(*content));
                }
            }
            Err(e) => {
                panic!("Unexpected error: {e:?}");
            }
        }
    }

    /// ref: <https://github.com/carthage-software/mago/issues/345>
    #[test]
    fn test_lex_multi_line_comment_crlf_with_multibyte_char() {
        let comment = "/**\r\n * blah blah ‰©\r\n */";
        let span = Span::new(FileId::zero(), Position::new(0), Position::new(comment.len() as u32));

        match tokenize(comment, span) {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 1, "Should have parsed exactly one line of content");

                let Token::Line { content, span: token_span } = &tokens[0] else {
                    panic!("Expected a Token::Line");
                };

                let expected_content = "blah blah ‰©";
                assert_eq!(*content, expected_content);

                let sliced = &comment[token_span.start.offset as usize..token_span.end.offset as usize];
                assert_eq!(sliced, expected_content);
            }
            Err(e) => {
                panic!("Failed to tokenize comment with CRLF endings: {e:?}");
            }
        }
    }
}
