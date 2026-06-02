use bumpalo::Bump;

use mago_database::file::FileId;
use mago_phpdoc_syntax::PHPDocParser;
use mago_phpdoc_syntax::cst::Document;
use mago_phpdoc_syntax::cst::Element;
use mago_phpdoc_syntax::cst::TriviaKind;
use mago_phpdoc_syntax::lexer::DocblockLexer;
use mago_span::HasSpan;
use mago_span::Position;
use mago_syntax_core::input::Input;

fn parse<'arena>(arena: &'arena Bump, source: &'arena [u8]) -> Document<'arena> {
    PHPDocParser::parse(arena, FileId::zero(), source)
}

fn assert_every_token_is_accounted_for(source: &[u8]) {
    let file_id = FileId::zero();

    let input = Input::anchored_at(file_id, source, Position::new(0));
    let mut lexer = DocblockLexer::new(input);
    let mut token_spans = Vec::new();
    while let Some(token) = lexer.advance() {
        token_spans.push(token.span_for(file_id));
    }

    let arena = Bump::new();
    let document = parse(&arena, source);

    for span in token_spans {
        let in_trivia = document
            .trivia
            .iter()
            .any(|trivia| trivia.span.start.offset <= span.start.offset && span.end.offset <= trivia.span.end.offset);
        let in_element = document.elements.iter().any(|element| {
            let element = element.span();
            element.start.offset <= span.start.offset && span.end.offset <= element.end.offset
        });

        assert!(
            in_trivia || in_element,
            "token at {}..{} is not part of the document (source: {:?})",
            span.start.offset,
            span.end.offset,
            String::from_utf8_lossy(source),
        );
    }
}

#[test]
fn every_source_token_is_accounted_for() {
    assert_every_token_is_accounted_for(b"/** @param int $x */");
    assert_every_token_is_accounted_for(b"/**\n * @return void\n */");
    assert_every_token_is_accounted_for(b"/**\n * Summary.\n *\n * @param string $a first\n * @return bool\n */");
    assert_every_token_is_accounted_for(b"/** @var array{a: int, b: string} $map the map */");
    assert_every_token_is_accounted_for(b"/** @method static T find<T>(int $id, ?string $name = null) by id */");
    assert_every_token_is_accounted_for(b"/** plain description with no tags */");
    assert_every_token_is_accounted_for(b"/** @var int // trailing line comment */");
    assert_every_token_is_accounted_for("/** @param string $مثال 中文描述 */".as_bytes());
    assert_every_token_is_accounted_for(b"/** */");
    assert_every_token_is_accounted_for(b"/** @var int */ trailing words after the marker");
    assert_every_token_is_accounted_for(b"/** x */\n");
}

#[test]
fn content_after_closing_marker_is_captured_as_trailing_trivia() {
    let arena = Bump::new();
    let source = b"/** @var int */ tail";
    let document = parse(&arena, source);

    assert_eq!(document.span.end.offset, source.len() as u32);

    let elements: Vec<&Element> = document.elements.iter().collect();
    assert_eq!(elements.len(), 1);
    assert!(matches!(elements[0], Element::Tag(_)));

    let Some(trailing) = document.trivia.iter().find(|trivia| trivia.kind == TriviaKind::Trailing) else {
        panic!("expected trailing trivia after the closing marker")
    };
    assert_eq!(trailing.value, b" tail");
}

fn trivia_pairs<'arena>(document: &'arena Document<'arena>) -> Vec<(TriviaKind, &'arena [u8])> {
    document.trivia.iter().map(|trivia| (trivia.kind, trivia.value)).collect()
}

#[test]
fn captures_opening_closing_markers_asterisks_and_whitespace() {
    let arena = Bump::new();
    let document = parse(&arena, b"/**\n * @return void\n */");

    assert_eq!(
        trivia_pairs(&document),
        vec![
            (TriviaKind::OpeningMarker, b"/**".as_slice()),
            (TriviaKind::Whitespace, b"\n ".as_slice()),
            (TriviaKind::Asterisk, b"*".as_slice()),
            (TriviaKind::Whitespace, b" ".as_slice()),
            (TriviaKind::Whitespace, b" ".as_slice()),
            (TriviaKind::Whitespace, b"\n ".as_slice()),
            (TriviaKind::ClosingMarker, b"*/".as_slice()),
        ]
    );
}

#[test]
fn captures_line_comments_as_trivia() {
    let arena = Bump::new();
    let document = parse(&arena, b"/** @var int // note */");

    assert_eq!(
        trivia_pairs(&document),
        vec![
            (TriviaKind::OpeningMarker, b"/** ".as_slice()),
            (TriviaKind::Whitespace, b" ".as_slice()),
            (TriviaKind::Whitespace, b" ".as_slice()),
            (TriviaKind::LineComment, b"// note ".as_slice()),
            (TriviaKind::ClosingMarker, b"*/".as_slice()),
        ]
    );
}

#[test]
fn missing_markers_are_not_faked() {
    let arena = Bump::new();
    let source = b"@var int $x";
    let document = parse(&arena, source);

    assert_eq!(document.span.start.offset, 0);
    assert_eq!(document.span.end.offset, source.len() as u32);

    assert_eq!(
        trivia_pairs(&document),
        vec![(TriviaKind::Whitespace, b" ".as_slice()), (TriviaKind::Whitespace, b" ".as_slice())]
    );
}
