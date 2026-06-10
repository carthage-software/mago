use mago_allocator::LocalArena;

use mago_database::file::FileId;
use mago_phpdoc_syntax::PHPDocParser;
use mago_phpdoc_syntax::error::ParseError;
use mago_phpdoc_syntax::parse_type;
use mago_span::Position;
use mago_span::Span;

#[track_caller]
fn type_error(source: &[u8]) -> ParseError {
    let arena = LocalArena::new();
    let span = Span::new(FileId::zero(), Position::new(0), Position::new(source.len() as u32));

    match parse_type(&arena, source, span) {
        Ok(_) => panic!("expected a parse error for {:?}", String::from_utf8_lossy(source)),
        Err(error) => error,
    }
}

fn document_errors(source: &[u8]) -> Vec<ParseError> {
    let arena = LocalArena::new();

    PHPDocParser::parse(&arena, FileId::zero(), source).errors.to_vec()
}

#[track_caller]
fn assert_reports(source: &[u8], predicate: impl Fn(&ParseError) -> bool) {
    let errors = document_errors(source);

    assert!(
        errors.iter().any(predicate),
        "expected the error to be reported for {:?}, got {errors:?}",
        String::from_utf8_lossy(source)
    );
}

#[test]
fn unexpected_token_is_reported() {
    assert!(matches!(type_error(b"|int"), ParseError::UnexpectedToken(_)));
}

#[test]
fn unexpected_end_of_input_is_reported() {
    assert!(matches!(type_error(b"array<"), ParseError::UnexpectedEndOfInput(_)));
}

#[test]
fn unclosed_literal_string_is_reported() {
    assert!(matches!(type_error(b"'unterminated"), ParseError::UnclosedLiteralString(_)));
}

#[test]
fn unclosed_inline_tag_is_reported() {
    assert_reports(b"/** see {@see \\Some\\Class for details */", |error| {
        matches!(error, ParseError::UnclosedInlineTag(_))
    });
}

#[test]
fn unclosed_inline_code_is_reported() {
    assert_reports(b"/** the `code that never closes */", |error| matches!(error, ParseError::UnclosedInlineCode(_)));
}

#[test]
fn unclosed_code_block_is_reported() {
    assert_reports(b"/**\n * ```php\n * echo 1;\n */", |error| matches!(error, ParseError::UnclosedCodeBlock(_)));
}

#[test]
fn malformed_code_block_is_reported() {
    let mut source = Vec::new();
    source.extend_from_slice(b"/**\n * ");
    source.resize(source.len() + 300, b'`');
    source.extend_from_slice(b"\n * code\n */");

    let errors = document_errors(&source);

    assert!(
        errors.iter().any(|error| matches!(error, ParseError::MalformedCodeBlock(_))),
        "expected a malformed code-block error, got {errors:?}"
    );
}

#[test]
fn malformed_tag_surfaces_inner_error() {
    assert_reports(b"/** @param | */", |error| matches!(error, ParseError::UnexpectedToken(_)));
}

#[test]
fn malformed_tag_with_unclosed_string_surfaces_inner_error() {
    assert_reports(b"/** @var 'unterminated */", |error| matches!(error, ParseError::UnclosedLiteralString(_)));
}

#[test]
fn recursion_limit_exceeded_is_reported() {
    let spawned = std::thread::Builder::new().stack_size(64 * 1024 * 1024).spawn(|| {
        let nested = vec![b'('; 5000];
        let error = type_error(&nested);

        assert!(matches!(error, ParseError::RecursionLimitExceeded(_)), "got {error:?}");
    });

    let Ok(handle) = spawned else {
        panic!("failed to spawn parser thread");
    };

    if handle.join().is_err() {
        panic!("parser thread aborted (stack overflow)");
    }
}
