//! Delimiter validation for literal `preg_match()` and `preg_match_all()` patterns.

use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;

use crate::code::IssueCode;
use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::function::FunctionReturnTypeProvider;
use crate::plugin::provider::function::FunctionTarget;

#[derive(Default)]
pub struct PregMatchPatternProvider;

impl Provider for PregMatchPatternProvider {
    fn meta() -> &'static ProviderMeta {
        static META: ProviderMeta = ProviderMeta::new(
            "php::pcre::pattern",
            "PCRE pattern delimiters",
            "Validates delimiters of literal preg_match() and preg_match_all() patterns.",
        );

        &META
    }
}

impl FunctionReturnTypeProvider for PregMatchPatternProvider {
    fn targets() -> FunctionTarget {
        FunctionTarget::ExactMultiple(&[b"preg_match", b"preg_match_all"])
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        // Provider dispatch identifies the resolved function, including imported names,
        // without mistaking a namespaced function for a builtin.
        let metadata = invocation.inner().target.get_function_like_metadata()?;
        if !metadata.flags.is_built_in() {
            return None;
        }

        let pattern = invocation.get_argument(0, &[b"pattern"])?;
        let pattern_type = context.get_expression_type(pattern)?;
        let pattern_value = pattern_type.get_single_literal_string_value()?;
        let error = find_delimiter_error(pattern_value)?;

        let message = match error {
            DelimiterError::EmptyPattern => "The regular expression is empty.".to_string(),
            DelimiterError::InvalidDelimiter => {
                "A delimiter cannot be alphanumeric, a backslash, or a NUL byte.".to_string()
            }
            DelimiterError::MissingClosingDelimiter(delimiter) => {
                format!("No matching closing delimiter `{}` was found.", char::from(delimiter).escape_default())
            }
        };

        context.report(
            IssueCode::InvalidArgument,
            Issue::error("Invalid regular expression delimiter.")
                .with_annotation(Annotation::primary(pattern.span()).with_message(message))
                .with_note("`preg_match()` and `preg_match_all()` require a pattern enclosed in delimiters.")
                .with_help("Enclose the pattern in matching delimiters and escape any literal delimiter inside it."),
        );

        // Keep the declared return type: valid delimiters do not rule out compilation
        // failures or runtime errors such as malformed UTF-8 and backtracking limits.
        None
    }
}

#[derive(Debug, PartialEq, Eq)]
enum DelimiterError {
    EmptyPattern,
    InvalidDelimiter,
    MissingClosingDelimiter(u8),
}

/// Detects definite delimiter errors without validating PCRE syntax or modifiers.
///
/// PHP scans delimiters before parsing the expression, so character classes do not
/// shield delimiters, and paired delimiters nest even within character classes.
fn find_delimiter_error(pattern: &[u8]) -> Option<DelimiterError> {
    let Some(start) = pattern.iter().position(|byte| !matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c))
    else {
        return Some(DelimiterError::EmptyPattern);
    };

    let opening = pattern[start];
    if !opening.is_ascii() {
        // PHP's classification of non-ASCII delimiter bytes depends on the locale.
        return None;
    }

    if opening.is_ascii_alphanumeric() || matches!(opening, b'\\' | b'\0') {
        return Some(DelimiterError::InvalidDelimiter);
    }

    let closing = match opening {
        b'(' => b')',
        b'[' => b']',
        b'{' => b'}',
        b'<' => b'>',
        _ => opening,
    };

    let mut depth = 1usize;
    let mut index = start + 1;
    while let Some(&byte) = pattern.get(index) {
        if byte == b'\\' {
            index += 2;
            continue;
        }

        if byte == closing {
            depth -= 1;
            if depth == 0 {
                return None;
            }
        } else if byte == opening {
            depth += 1;
        }

        index += 1;
    }

    Some(DelimiterError::MissingClosingDelimiter(closing))
}

#[cfg(test)]
mod tests {
    use super::DelimiterError;
    use super::find_delimiter_error;

    #[test]
    fn accepts_matching_delimiters() {
        for pattern in [
            b"/^[a-fA-F0-9]*$/".as_slice(),
            b"//",
            b"#foo#i",
            b"+foo+",
            b"{a{2}}",
            b"[a[b]c]",
            b"(a(b)c)",
            b"<a<b>c>",
            b"}foo}",
            b"]foo]",
            b")foo)",
            b">foo>",
            b" \t\n\r\x0b\x0c/foo/",
            br"/a\/b/",
            br"/a\\/i",
            br"{a\}b}",
            br"{a\{b}",
            b"/foo\0bar/",
        ] {
            assert_eq!(find_delimiter_error(pattern), None, "pattern: {pattern:?}");
        }
    }

    #[test]
    fn rejects_missing_closing_delimiters() {
        for (pattern, closing) in [
            (b"^[a-fA-F0-9]*$/".as_slice(), b'^'),
            (b"/foo", b'/'),
            (b"/", b'/'),
            (b"/foo\\", b'/'),
            (br"/foo\/", b'/'),
            (b"{foo]", b'}'),
            (b"{a{2}", b'}'),
            (b"[foo", b']'),
            (b"(foo", b')'),
            (b"<foo", b'>'),
            (b"{[{]}", b'}'),
        ] {
            assert_eq!(
                find_delimiter_error(pattern),
                Some(DelimiterError::MissingClosingDelimiter(closing)),
                "pattern: {pattern:?}",
            );
        }
    }

    #[test]
    fn rejects_empty_patterns_and_invalid_delimiters() {
        for pattern in [b"".as_slice(), b" \t\n\r\x0b\x0c"] {
            assert_eq!(find_delimiter_error(pattern), Some(DelimiterError::EmptyPattern));
        }

        for pattern in [b"foo".as_slice(), b"1foo1", b"\\foo\\", b"\0foo\0"] {
            assert_eq!(find_delimiter_error(pattern), Some(DelimiterError::InvalidDelimiter));
        }
    }

    #[test]
    fn leaves_pattern_syntax_modifiers_and_non_ascii_delimiters_unchecked() {
        for pattern in [b"/[a/".as_slice(), b"/foo/z", b"/foo/\t", b"{[}]}", b"\xfffoo"] {
            assert_eq!(find_delimiter_error(pattern), None, "pattern: {pattern:?}");
        }
    }
}
